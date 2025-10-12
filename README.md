# cache-line-size

A library to quickly get the cache line size of your CPU caches. 

Currently this crate supports x86 CPUs via the `CPUID` instruction, using the [`raw_cpuid`][raw_cpuid] crate. ARM (64-bit) is supported via the corresponding assembly instructions for ARM and AArch64. If you can support other architectures, PRs are welcome!

## API overview

The crate exposes two enums, [`CacheType`](https://docs.rs/cache-size/latest/cache_size/enum.CacheType.html)
and [`CacheLevel`](https://docs.rs/cache-size/latest/cache_size/enum.CacheLevel.html), to describe
the type of cache (data, instruction, unified, or trace) and its hierarchy level. 

We also supply APIs to retrieve the cache line size for cache types and levels. These functions now return a `Result` with a
[`CacheInfoError`](https://docs.rs/cache-line-size/latest/cache_line_size/enum.CacheInfoError.html) describing whether the
hardware does not expose the requested information, the cache is not present, or the reported values are invalid.
