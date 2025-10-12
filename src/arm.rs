use core::arch::asm;

use crate::{CacheLevel, CacheType};

#[derive(Debug, Clone, Copy)]
struct CacheLineSizes {
    data_or_unified: usize,
    instruction: usize,
}

#[inline]
fn cache_line_sizes() -> Option<CacheLineSizes> {
    let ctr = read_cache_type_register()?;
    Some(CacheLineSizes {
        data_or_unified: decode_dminline(ctr)?,
        instruction: decode_iminline(ctr)?,
    })
}

#[cfg(target_arch = "aarch64")]
#[inline]
fn read_cache_type_register() -> Option<u64> {
    let mut value: u64;
    unsafe {
        asm!("mrs {value}, ctr_el0", value = out(reg) value);
    }
    Some(value)
}

#[cfg(target_arch = "arm")]
#[inline]
fn read_cache_type_register() -> Option<u64> {
    let mut value: u32;
    unsafe {
        asm!("mrc p15, 0, {value}, c0, c0, 1", value = out(reg) value);
    }
    Some(value as u64)
}

const WORD_SIZE: usize = 4;
const LINE_SIZE_MASK: u64 = 0xF;
const DMINLINE_SHIFT: u64 = 16;

#[inline]
fn decode_line_size(field: u64) -> Option<usize> {
    let words = 1usize.checked_shl(field as u32)?;
    Some(words * WORD_SIZE)
}

#[inline]
fn decode_dminline(ctr: u64) -> Option<usize> {
    decode_line_size((ctr >> DMINLINE_SHIFT) & LINE_SIZE_MASK)
}

#[inline]
fn decode_iminline(ctr: u64) -> Option<usize> {
    decode_line_size(ctr & LINE_SIZE_MASK)
}

/// Returns the line size in bytes of `level` cache with type `cache_type`.
///
/// The ARM `CTR`/`CTR_EL0` register exposes the minimum data/unified cache line
/// size as well as the minimum L1 instruction cache line size. Data/unified
/// cache line sizes are reported for every level because ARM architectures use a
/// single value representing the minimum size across all data caches.
#[inline]
pub fn get_cache_line_size(level: CacheLevel, cache_type: CacheType) -> Option<usize> {
    let sizes = cache_line_sizes()?;
    match cache_type {
        CacheType::Instruction if level == CacheLevel::L1 => Some(sizes.instruction),
        CacheType::Data | CacheType::Unified => Some(sizes.data_or_unified),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{cache_line_size, decode_line_size, WORD_SIZE};
    use crate::{CacheLevel, CacheType};

    #[test]
    fn decode_line_size_scaling() {
        assert_eq!(decode_line_size(0).unwrap(), WORD_SIZE);
        assert_eq!(decode_line_size(1).unwrap(), WORD_SIZE * 2);
        assert_eq!(decode_line_size(2).unwrap(), WORD_SIZE * 4);
    }
}
