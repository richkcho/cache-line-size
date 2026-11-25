//! A library to quickly get the line size of your CPU caches.
//!
//! On x86, the crate uses `CPUID` via [`raw_cpuid`](https://docs.rs/raw-cpuid).
//! On ARM, it reads the cache type register where permitted and uses OS APIs on
//! Windows ARM64 and Apple Silicon.
//!
//! Check the [Intel 64 and IA-32 Architectures Software Developers Manual](https://software.intel.com/sites/default/files/managed/39/c5/325462-sdm-vol-1-2abcd-3abcd.pdf)
//! for more information on the `CPUID` instruction.

mod types;
pub use types::{CacheInfoError, CacheLevel, CacheType};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use x86::*;

#[cfg(all(
    any(target_arch = "arm", target_arch = "aarch64"),
    not(all(target_arch = "aarch64", any(target_os = "macos", target_os = "windows")))
))]
mod arm;
#[cfg(all(
    any(target_arch = "arm", target_arch = "aarch64"),
    not(all(target_arch = "aarch64", any(target_os = "macos", target_os = "windows")))
))]
pub use arm::*;

#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
mod appl_arm;
#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
pub use appl_arm::*;

#[cfg(all(target_os = "windows", any(target_arch = "aarch64", test)))]
mod windows_cache;
#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
pub use windows_cache::get_cache_line_size;
