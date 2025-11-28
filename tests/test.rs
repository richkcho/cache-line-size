mod tests {
    use yep_cache_line_size::{get_cache_line_size, CacheLevel, CacheType};

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_cache_line_size_sane() {
        let cl = get_cache_line_size(CacheLevel::L1, CacheType::Data)
            .expect("cache line size should be available on macOS");
        assert!(cl > 0, "cache line size should be positive");
        let page_size = unsafe { libc::getpagesize() as usize };
        assert!(
            cl <= page_size,
            "cache line size should not exceed page size (cl={}, page={})",
            cl,
            page_size
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_test() {
        // walk through sysfs for cpu0 and check that we get the same values
        let base_path = "/sys/devices/system/cpu/cpu0/cache";
        let entries = std::fs::read_dir(base_path).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                let level: u32 = std::fs::read_to_string(path.join("level"))
                    .unwrap()
                    .trim()
                    .parse()
                    .unwrap();
                let cache_type_str = std::fs::read_to_string(path.join("type"))
                    .unwrap()
                    .trim()
                    .to_lowercase();
                let cache_type = match cache_type_str.as_str() {
                    "data" => CacheType::Data,
                    "instruction" => CacheType::Instruction,
                    "unified" => CacheType::Unified,
                    _ => panic!("Unknown cache type: {}", cache_type_str),
                };
                let line_size: usize = std::fs::read_to_string(path.join("coherency_line_size"))
                    .unwrap()
                    .trim()
                    .parse()
                    .unwrap();
                let cl = get_cache_line_size(
                    match level {1 => CacheLevel::L1,
                        2 => CacheLevel::L2,
                        3 => CacheLevel::L3,
                        _ => CacheLevel::Other(level as u8),
                    },
                    cache_type,
                ).unwrap();
                assert_eq!(cl, line_size, "Mismatch for level {} {:?} cache", level, cache_type);
            }
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_test() {
        use windows::Win32::System::SystemInformation::{
            GetLogicalProcessorInformationEx, RelationCache,
            SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX, CacheData, CacheInstruction, CacheUnified,
        };

        unsafe {
            let mut len: u32 = 0;
            // First call to get required buffer size (expect ERROR_INSUFFICIENT_BUFFER)
            let _ = GetLogicalProcessorInformationEx(RelationCache, None, &mut len);
            assert!(len > 0, "Windows API did not report buffer size for processor info");

            let mut buf: Vec<u8> = vec![0u8; len as usize];
            GetLogicalProcessorInformationEx(
                RelationCache,
                Some(buf.as_mut_ptr() as *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX),
                &mut len,
            )
            .expect("GetLogicalProcessorInformationEx failed on second call");

            let mut offset = 0usize;
            while offset < len as usize {
                let entry_ptr =
                    buf.as_ptr().add(offset) as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX;
                // Read possibly unaligned structure safely by value
                let entry = std::ptr::read_unaligned(entry_ptr);

                assert!(entry.Relationship == RelationCache, "Unexpected relationship type in buffer");

                // Access the Cache relationship from the union
                let cache = entry.Anonymous.Cache;

                // Map Windows cache level to our CacheLevel
                let level = match cache.Level {
                    1 => CacheLevel::L1,
                    2 => CacheLevel::L2,
                    3 => CacheLevel::L3,
                    other => CacheLevel::Other(other),
                };

                // Map Windows cache type to our CacheType
                let ctype = match cache.Type {
                    t if t == CacheData => CacheType::Data,
                    t if t == CacheInstruction => CacheType::Instruction,
                    t if t == CacheUnified => CacheType::Unified,
                    _ => {
                        offset += entry.Size as usize;
                        continue;
                    }
                };

                let line_size = cache.LineSize as usize;
                if let Ok(cl) = get_cache_line_size(level, ctype) {
                    assert_eq!(
                        cl, line_size,
                        "Mismatch for level {:?} {:?} cache: library={} windows={}",
                        level, ctype, cl, line_size
                    );
                }

                offset += entry.Size as usize;
            }
        }
    }
}
