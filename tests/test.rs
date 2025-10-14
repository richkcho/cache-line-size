mod tests {
    use cache_line_size::{get_cache_line_size, CacheLevel, CacheType};

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
}