use crate::{CacheLevel, CacheType};
use raw_cpuid::{self, CpuId, CpuIdReaderNative};

/// Uses the CPUID family info to detect Zen architecture CPUs.
///
/// Data pulled from https://en.wikichip.org/wiki/amd/cpuid.
#[inline]
fn amd_is_zen(cpuid: &CpuId<CpuIdReaderNative>) -> Option<bool> {
    let info = cpuid.get_feature_info()?;
    match (info.base_family_id(), info.extended_family_id()) {
        (0xF, 0x8..=0xA) => Some(true),
        _ => Some(false),
    }
}

/// Uses cache parameters to get cache line size at a given level with the provided cache type.
#[inline]
fn generic_cache_line_size(
    cpuid: CpuId<CpuIdReaderNative>,
    level: CacheLevel,
    cache_type: CacheType,
) -> Option<usize> {
    let level_id: u8 = level.into();
    let cache_kind: raw_cpuid::CacheType = cache_type.into();
    cpuid
        .get_cache_parameters()?
        .filter(|cparams| cparams.level() == level_id && cparams.cache_type() == cache_kind)
        .map(|cparams| cparams.coherency_line_size())
        .min()
}

/// This is computed using tlb info. Instruction and data cache line sizes
/// are available separately for the L1 cache, but only unified is available for L2 and L3 caches.
#[inline]
fn amd_cache_line_size(
    cpuid: CpuId<CpuIdReaderNative>,
    level: CacheLevel,
    cache_type: CacheType,
) -> Option<usize> {
    match (level, cache_type) {
        (CacheLevel::L1, CacheType::Instruction) => cpuid
            .get_l1_cache_and_tlb_info()
            .map(|i| i.icache_line_size() as usize),
        (CacheLevel::L1, CacheType::Data) => cpuid
            .get_l1_cache_and_tlb_info()
            .map(|i| i.dcache_line_size() as usize),
        (CacheLevel::L2, CacheType::Unified) => cpuid
            .get_l2_l3_cache_and_tlb_info()
            .map(|i| i.l2cache_line_size() as usize),
        (CacheLevel::L3, CacheType::Unified) => cpuid
            .get_l2_l3_cache_and_tlb_info()
            .map(|i| i.l3cache_line_size() as usize),
        _ => None,
    }
}

/// Returns the line size in bytes of `level` cache with type `cache_type`.
///
/// The only possibilities for this returning `None` are if the system does not support cache
/// parameters, in which case [`get_cache_parameters()`](raw_cpuid::CpuId::get_cache_parameters) will
/// fail, or if the selected cache level and/or type does not exist.
///
/// On an AMD Zen architecture this is computed using tlb info. Instruction and data cache line
/// sizes are available separately for the L1 cache, but only unified is available for L2 and L3
/// caches.
///
/// On other x86 architectures this is computed from
/// [`coherency_line_size()`](raw_cpuid::CacheParameter::coherency_line_size),
/// and if there are multiple caches available, it returns the size of the **smallest** cache.
#[inline]
pub fn get_cache_line_size(level: CacheLevel, cache_type: CacheType) -> Option<usize> {
    let cpuid = CpuId::new();
    match cpuid.get_vendor_info()?.as_str() {
        "AuthenticAMD" if amd_is_zen(&cpuid).unwrap_or(false) => {
            amd_cache_line_size(cpuid, level, cache_type)
        }
        _ => generic_cache_line_size(cpuid, level, cache_type),
    }
}
