# [yep-]cache-line-size
[![Crates.io](https://img.shields.io/crates/v/yep-cache-line-size.svg)](https://crates.io/crates/yep-cache-line-size) [![Documentation](https://docs.rs/yep-cache-line-size/badge.svg)](https://docs.rs/yep-cache-line-size) [![Rust](https://github.com/richkcho/cache-line-size/actions/workflows/rust.yml/badge.svg)](https://github.com/richkcho/cache-line-size/actions/workflows/rust.yml)

A library to quickly get the cache line size of your CPU caches. Forked from lovesegfault/cache-size.

On x86, this crate uses `CPUID` via [`raw_cpuid`](https://docs.rs/raw-cpuid). On ARM, it reads the cache type register where permitted and uses OS APIs on Windows ARM64 and Apple Silicon.

Windows ARM64 uses [`GetLogicalProcessorInformationEx(RelationCache)`](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getlogicalprocessorinformationex) instead of reading `CTR_EL0`, which Windows does not permit in user mode. When Windows reports multiple caches for the requested level and type, the crate returns the smallest nonzero line size.

## API overview

The crate exposes two enums, [`CacheType`] and [`CacheLevel`], to describe the type of cache (data, instruction, or unified) and its hierarchy level.
The cache line size is retrieved via `get_cache_line_size`.

We also supply APIs to retrieve the cache line size for cache types and levels. These functions now return a `Result` with a
[`CacheInfoError`] describing whether the
hardware does not expose the requested information, the cache is not present, or the reported values are invalid.

## Known issues
### macOS
MacOS traps the `CTR_EL0` access and kills the process (illegal instruction). We have to take a dependency on libc to issue a sysctl as a result. It's also worth noting that the retrieved cache line size value appears to be an overestimate on emperical testing and comparing with the `CTR_EL0` value. 