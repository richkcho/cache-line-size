# cache-line-size

A library to quickly get the cache line size of your CPU caches. 

Currently this crate only supports x86 CPUs, since it relies on the `CPUID` instruction, via
the [`raw_cpuid`][raw_cpuid] crate. It is a goal to support other architectures; PRs are
welcome!

## API overview

The crate exposes two enums, [`CacheType`](https://docs.rs/cache-size/latest/cache_size/enum.CacheType.html)
and [`CacheLevel`](https://docs.rs/cache-size/latest/cache_size/enum.CacheLevel.html), to describe
the type of cache (data, instruction, unified, or trace) and its hierarchy level. 

We also supply APIs to retrieve the cache line size for cache types and levels, returning `None` if the operation fails or is not supported. (TODO: refactor into error types)

---
Check the [Intel 64 and IA-32 Architectures Software Developers Manual](https://software.intel.com/sites/default/files/managed/39/c5/325462-sdm-vol-1-2abcd-3abcd.pdf)
for more information on the `CPUID` instruction.

[raw_cpuid]: https://github.com/gz/rust-cpuid
