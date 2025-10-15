# [yep-]cache-line-size
[![Crates.io](https://img.shields.io/crates/v/yep-cache-line-size.svg)](https://crates.io/crates/yep-cache-line-size) [![Documentation](https://docs.rs/yep-cache-line-size/badge.svg)](https://docs.rs/yep-cache-line-size) [![Rust](https://github.com/richkcho/cache-line-size/actions/workflows/rust.yml/badge.svg)](https://github.com/richkcho/cache-line-size/actions/workflows/rust.yml)

A library to quickly get the cache line size of your CPU caches. Forked from lovesegfault/cache-size.

Currently this crate supports x86 CPUs via the `CPUID` instruction, using the [`raw_cpuid`][raw_cpuid] crate. ARM (64-bit) is supported via the corresponding assembly instructions for ARM and AArch64. If you can support other architectures, PRs are welcome!

## API overview

The crate exposes two enums, [`CacheType`] and [`CacheLevel`], to describe the type of cache (data, instruction, unified, or trace) and its hierarchy level. 
The cache line size is retrieved via `get_cache_line_size`.

We also supply APIs to retrieve the cache line size for cache types and levels. These functions now return a `Result` with a
[`CacheInfoError`] describing whether the
hardware does not expose the requested information, the cache is not present, or the reported values are invalid.
