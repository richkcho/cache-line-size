use core::arch::asm;

use crate::{CacheInfoError, CacheLevel, CacheType};

#[cfg(target_arch = "aarch64")]
#[inline]
fn read_cache_type_register() -> u64 {
    let mut value: u64;
    unsafe {
        asm!("mrs {value}, ctr_el0", value = out(reg) value);
    }
    value
}

#[cfg(target_arch = "arm")]
#[inline]
fn read_cache_type_register() -> u64 {
    let mut value: u32;
    unsafe {
        asm!("mrc p15, 0, {value}, c0, c0, 1", value = out(reg) value);
    }
    value as u64
}

const WORD_SIZE: usize = 4;
const LINE_SIZE_MASK: u64 = 0xF;
const DMINLINE_SHIFT: u64 = 16;

#[inline]
fn decode_line_size(field: u64) -> Result<usize, CacheInfoError> {
    let words = 1usize
        .checked_shl(field as u32)
        .ok_or(CacheInfoError::InvalidValue)?;
    Ok(words * WORD_SIZE)
}

#[inline]
fn decode_dminline(ctr: u64) -> Result<usize, CacheInfoError> {
    decode_line_size((ctr >> DMINLINE_SHIFT) & LINE_SIZE_MASK)
}

#[inline]
fn decode_iminline(ctr: u64) -> Result<usize, CacheInfoError> {
    decode_line_size(ctr & LINE_SIZE_MASK)
}

/// Returns the line size in bytes of `level` cache with type `cache_type`.
///
/// The only possibilities for this returning an [`Err`] are if the system does not support cache
/// parameters, in which case we will return `CacheInfoError::NotPresent`, or if the CPU
/// reported an invalid value, in which case we will return `CacheInfoError::InvalidValue`.
#[inline]
pub fn get_cache_line_size(
    level: CacheLevel,
    cache_type: CacheType,
) -> Result<usize, CacheInfoError> {
    let ctr = read_cache_type_register();
    match cache_type {
        CacheType::Instruction if level == CacheLevel::L1 => decode_iminline(ctr),
        CacheType::Data | CacheType::Unified => decode_dminline(ctr),
        _ => Err(CacheInfoError::NotPresent),
    }
}

#[cfg(test)]
mod tests {
    use super::{WORD_SIZE, decode_line_size};

    #[test]
    fn decode_line_size_scaling() {
        assert_eq!(decode_line_size(0).unwrap(), WORD_SIZE);
        assert_eq!(decode_line_size(1).unwrap(), WORD_SIZE * 2);
        assert_eq!(decode_line_size(2).unwrap(), WORD_SIZE * 4);
    }
}
