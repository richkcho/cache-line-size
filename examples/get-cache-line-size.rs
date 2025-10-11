use cache_size::{cache_line_size, CacheLevel, CacheType};
use clap::{Parser, ValueEnum};
use std::process;

// Could use libc, but that adds a dependency for a simple constant, which feels silly. 
const ENOTSUP: i32 = -134; // Operation not supported

#[derive(Parser)]
#[command(
    name = "cache_line",
    about = "Report cache line size information for a given level and cache type"
)]
struct Args {
    /// Cache level to inspect (L1, L2, or L3)
    #[arg(value_enum)]
    level: ArgCacheLevel,

    /// Cache type to inspect (data, instruction, or unified)
    #[arg(value_enum, name = "type")]
    cache_type: ArgCacheType,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ArgCacheLevel {
    L1,
    L2,
    L3,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ArgCacheType {
    Data,
    Instruction,
    Unified,
}

impl From<ArgCacheType> for CacheType {
    fn from(kind: ArgCacheType) -> CacheType {
        match kind {
            ArgCacheType::Data => CacheType::Data,
            ArgCacheType::Instruction => CacheType::Instruction,
            ArgCacheType::Unified => CacheType::Unified,
        }
    }
}

impl From<ArgCacheLevel> for CacheLevel {
    fn from(level: ArgCacheLevel) -> CacheLevel {
        match level {
            ArgCacheLevel::L1 => CacheLevel::L1,
            ArgCacheLevel::L2 => CacheLevel::L2,
            ArgCacheLevel::L3 => CacheLevel::L3,
        }
    }
}

fn main() {
    let args = Args::parse();
    let level = CacheLevel::from(args.level);
    let cache_type: CacheType = CacheType::from(args.cache_type);

    match cache_line_size(level, cache_type) {
        Some(size) => {
            println!(
                "{:?} {:?} cache line size: {} bytes",
                level, cache_type, size
            );
        }
        None => {
            eprintln!(
                "Unable to retrieve cache line information for {:?} {:?} cache.",
                level, cache_type
            );
            process::exit(ENOTSUP);
        }
    }
}