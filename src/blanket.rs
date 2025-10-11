use crate::{CacheLevel, CacheType};

/// Returns the line size in bytes of `level` cache with type `cache_type`.
///
/// This is the implementation for unsupported architectures, and always returns None.
#[inline]
pub fn get_cache_line_size(_level: CacheLevel, _cache_type: CacheType) -> Option<usize> {
    None
}

/// Returns the line size in bytes of the L1 data cache.
///
/// This is the implementation for unsupported architectures, and always returns None.
#[inline]
pub fn get_l1_cache_line_size() -> Option<usize> {
    get_cache_line_size(CacheLevel::L1, CacheType::Data)
}

/// Returns the total size in bytes of the unified L2 cache.
///
/// This is the implementation for unsupported architectures, and always returns None.
#[inline]
pub fn l2_cache_size() -> Option<usize> {
    cache_size(CacheLevel::L2, CacheType::Unified)
}

/// Returns the line size in bytes of the unified L2 cache.
///
/// This is the implementation for unsupported architectures, and always returns None.
#[inline]
pub fn l2_cache_line_size() -> Option<usize> {
    cache_line_size(CacheLevel::L2, CacheType::Unified)
}

/// Returns the total size in bytes of the unified L3 cache.
///
/// This is the implementation for unsupported architectures, and always returns None.
#[inline]
pub fn l3_cache_size() -> Option<usize> {
    cache_size(CacheLevel::L3, CacheType::Unified)
}

/// Returns the line size in bytes of the unified L3 cache.
///
/// This is the implementation for unsupported architectures, and always returns None.
#[inline]
pub fn l3_cache_line_size() -> Option<usize> {
    cache_line_size(CacheLevel::L3, CacheType::Unified)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_l1_cache_size() {
        assert_eq!(l1_cache_size(), None);
    }
    #[test]
    fn test_l1_cache_line_size() {
        assert_eq!(l1_cache_line_size(), None)
    }
    #[test]
    fn test_l2_cache_size() {
        assert_eq!(l2_cache_size(), None);
    }
    #[test]
    fn test_l2_cache_line_size() {
        assert_eq!(l2_cache_line_size(), None);
    }
    #[test]
    fn test_l3_cache_size() {
        assert_eq!(l3_cache_size(), None);
    }
    #[test]
    fn test_l3_cache_line_size() {
        assert_eq!(l3_cache_line_size(), None);
    }
}
