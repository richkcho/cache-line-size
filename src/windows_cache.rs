use crate::{CacheInfoError, CacheLevel, CacheType};
use std::mem::{offset_of, size_of};
use windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
use windows::Win32::System::SystemInformation::{
    CACHE_RELATIONSHIP, CacheData as CACHE_DATA, CacheInstruction as CACHE_INSTRUCTION,
    CacheUnified as CACHE_UNIFIED, GetLogicalProcessorInformationEx, PROCESSOR_CACHE_TYPE,
    RelationCache, SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
};

const SIZE_OFFSET: usize = offset_of!(SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX, Size);
const CACHE_OFFSET: usize = offset_of!(SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX, Anonymous);
const LEVEL_OFFSET: usize = CACHE_OFFSET + offset_of!(CACHE_RELATIONSHIP, Level);
const LINE_SIZE_OFFSET: usize = CACHE_OFFSET + offset_of!(CACHE_RELATIONSHIP, LineSize);
const TYPE_OFFSET: usize = CACHE_OFFSET + offset_of!(CACHE_RELATIONSHIP, Type);
const RECORD_PREFIX_LEN: usize = TYPE_OFFSET + size_of::<i32>();

fn with_cache_bytes<T>(
    parse: impl FnOnce(&[u8]) -> Result<T, CacheInfoError>,
) -> Result<T, CacheInfoError> {
    // Use aligned, initialized storage for the Win32 output buffer.
    let mut buffer = Vec::<u64>::new();
    let mut len = 0;
    loop {
        let capacity = len;
        let ptr = (!buffer.is_empty()).then_some(buffer.as_mut_ptr().cast());
        // A null buffer requests the size. Otherwise the buffer holds at least `capacity` bytes.
        match unsafe { GetLogicalProcessorInformationEx(RelationCache, ptr, &mut len) } {
            Ok(()) if len > 0 && len <= capacity => {
                let bytes =
                    unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast(), len as usize) };
                return parse(bytes);
            }
            Ok(()) => return Err(CacheInfoError::InvalidValue),
            Err(error) if error.code() == ERROR_INSUFFICIENT_BUFFER.to_hresult() => {
                if len <= capacity {
                    return Err(CacheInfoError::InvalidValue);
                }
                buffer.resize((len as usize).div_ceil(size_of::<u64>()), 0);
            }
            Err(_) => return Err(CacheInfoError::Unsupported),
        }
    }
}

fn visit_cache_records(
    mut bytes: &[u8],
    mut visit: impl FnMut(u8, CacheType, u16),
) -> Result<(), CacheInfoError> {
    while !bytes.is_empty() {
        // Cache records have a fixed prefix and a variable affinity tail. Read only the prefix.
        let header = bytes.get(..CACHE_OFFSET).ok_or(CacheInfoError::InvalidValue)?;
        let relationship = i32::from_ne_bytes(header[..4].try_into().unwrap());
        let size = u32::from_ne_bytes(header[SIZE_OFFSET..SIZE_OFFSET + 4].try_into().unwrap());
        let (record, rest) = bytes
            .split_at_checked(size as usize)
            .ok_or(CacheInfoError::InvalidValue)?;
        if relationship != RelationCache.0 || record.len() < RECORD_PREFIX_LEN {
            return Err(CacheInfoError::InvalidValue);
        }

        bytes = rest;
        let kind = match PROCESSOR_CACHE_TYPE(i32::from_ne_bytes(
            record[TYPE_OFFSET..RECORD_PREFIX_LEN].try_into().unwrap(),
        )) {
            CACHE_DATA => CacheType::Data,
            CACHE_INSTRUCTION => CacheType::Instruction,
            CACHE_UNIFIED => CacheType::Unified,
            _ => continue,
        };
        let size = u16::from_ne_bytes(
            record[LINE_SIZE_OFFSET..LINE_SIZE_OFFSET + 2].try_into().unwrap(),
        );
        visit(record[LEVEL_OFFSET], kind, size);
    }
    Ok(())
}

fn find_cache_line_size(
    bytes: &[u8],
    level: CacheLevel,
    cache_type: CacheType,
) -> Result<usize, CacheInfoError> {
    let mut minimum: Option<u16> = None;
    let mut missing = CacheInfoError::NotPresent;
    visit_cache_records(bytes, |record_level, kind, size| {
        if record_level == level.as_u8() && kind == cache_type {
            if size > 0 {
                minimum = Some(minimum.map_or(size, |min| min.min(size)));
            } else {
                missing = CacheInfoError::InvalidValue;
            }
        }
    })?;
    minimum.map(usize::from).ok_or(missing)
}

/// Returns the smallest nonzero line size in bytes reported by Windows for the
/// requested cache level and type.
///
/// Returns [`CacheInfoError::Unsupported`] if the Windows query fails,
/// [`CacheInfoError::NotPresent`] if no cache matches, or
/// [`CacheInfoError::InvalidValue`] if the returned records are malformed or no
/// matching record reports a nonzero line size.
#[cfg(target_arch = "aarch64")]
pub fn get_cache_line_size(level: CacheLevel, cache_type: CacheType) -> Result<usize, CacheInfoError> {
    with_cache_bytes(|bytes| find_cache_line_size(bytes, level, cache_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(level: u8, kind: PROCESSOR_CACHE_TYPE, line_size: u16, len: usize) -> Vec<u8> {
        let mut bytes = vec![0; len];
        bytes[..4].copy_from_slice(&RelationCache.0.to_ne_bytes());
        bytes[4..8].copy_from_slice(&(len as u32).to_ne_bytes());
        bytes[8] = level;
        bytes[10..12].copy_from_slice(&line_size.to_ne_bytes());
        bytes[16..20].copy_from_slice(&kind.0.to_ne_bytes());
        bytes
    }

    #[test]
    fn picks_smallest_nonzero_line_size_for_the_requested_level_and_type() {
        let bytes = [
            record(1, CACHE_DATA, 128, 72),
            record(2, CACHE_DATA, 16, 56),
            record(1, CACHE_INSTRUCTION, 32, 56),
            record(1, CACHE_DATA, 0, 56),
            record(1, CACHE_DATA, 64, 56),
            record(1, CACHE_DATA, 256, 56),
        ]
        .concat();
        assert_eq!(
            find_cache_line_size(&bytes, CacheLevel::L1, CacheType::Data),
            Ok(64)
        );
    }

    #[test]
    fn distinguishes_missing_caches_from_zero_line_sizes() {
        let bytes = record(3, CACHE_UNIFIED, 0, 56);
        assert_eq!(
            find_cache_line_size(&bytes, CacheLevel::L2, CacheType::Unified),
            Err(CacheInfoError::NotPresent)
        );
        assert_eq!(
            find_cache_line_size(&bytes, CacheLevel::L3, CacheType::Unified),
            Err(CacheInfoError::InvalidValue)
        );
    }

    #[test]
    fn rejects_bad_record_sizes_even_after_a_valid_match() {
        let valid = record(1, CACHE_DATA, 64, 56);
        for size in [0u32, 19, 57] {
            let mut bad = valid.clone();
            bad[4..8].copy_from_slice(&size.to_ne_bytes());
            assert_eq!(
                find_cache_line_size(
                    &[valid.clone(), bad].concat(),
                    CacheLevel::L1,
                    CacheType::Data
                ),
                Err(CacheInfoError::InvalidValue)
            );
        }
        assert_eq!(
            find_cache_line_size(&valid[..7], CacheLevel::L1, CacheType::Data),
            Err(CacheInfoError::InvalidValue)
        );
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn windows_x86_cache_line_sizes_match_win32() {
        let records = with_cache_bytes(|bytes| {
            let mut records = Vec::new();
            visit_cache_records(bytes, |level, kind, size| records.push((level, kind, size)))?;
            Ok(records)
        })
        .expect("Windows should report cache metadata");
        assert!(
            !records.is_empty(),
            "Windows did not report known cache types"
        );

        for &(level, kind, _) in &records {
            if let Ok(size) = crate::get_cache_line_size(level.into(), kind) {
                assert!(
                    records
                        .iter()
                        .any(|&(l, k, s)| (l, k, usize::from(s)) == (level, kind, size)),
                    "No Windows record matches the level {level} {kind:?} line size of {size}"
                );
            }
        }
    }
}
