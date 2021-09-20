# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [0.10.2] - 2021-09-20
- Switched from `jemallocator` to `tikv-jemallocator`. [#589](https://github.com/paritytech/parity-common/pull/589)

## [0.10.1] - 2021-09-15
- Added support for memory stats gathering, ported over from `polkadot`. [#588](https://github.com/paritytech/parity-common/pull/588)

## [0.10.0] - 2021-07-02
- Fixed `malloc_usable_size` for FreeBSD. [#553](https://github.com/paritytech/parity-common/pull/553)

### Breaking
- Updated `ethereum-types` to 0.12. [#556](https://github.com/paritytech/parity-common/pull/556)
- Updated `primitive-types` to 0.10. [#556](https://github.com/paritytech/parity-common/pull/556)
- Updated `hashbrown` to 0.11. [#533](https://github.com/paritytech/parity-common/pull/533)

## [0.9.0] - 2021-01-27
### Breaking
- Updated `ethereum-types` to 0.11. [#510](https://github.com/paritytech/parity-common/pull/510)
- Updated `primitive-types` to 0.9. [#510](https://github.com/paritytech/parity-common/pull/510)

## [0.8.0] - 2021-01-05
- Updated dlmalloc to 0.2.1. [#452](https://github.com/paritytech/parity-common/pull/452)
### Breaking
- Updated `ethereum-types` to 0.10. [#463](https://github.com/paritytech/parity-common/pull/463)
- Updated `parking_lot` to 0.11.1. [#470](https://github.com/paritytech/parity-common/pull/470)

## [0.7.0] - 2020-06-24
- Added `const_size` to `MallocSizeOf` to optimize it for flat collections. [#398](https://github.com/paritytech/parity-common/pull/398)
- Exported `MallocShallowSizeOf`. [#399](https://github.com/paritytech/parity-common/pull/399)
- Updated dependencies.

## [0.6.1] - 2020-04-15
- Fix compilation on Windows for no-std. [#375](https://github.com/paritytech/parity-common/pull/375)
- Prevent multiple versions from being linked into the same program. [#363](https://github.com/paritytech/parity-common/pull/363)

## [0.6.0] - 2020-03-13
- Updated dependencies. [#361](https://github.com/paritytech/parity-common/pull/361)

## [0.5.2] - 2020-03-13
- License changed from GPL3 to dual MIT/Apache2. [#342](https://github.com/paritytech/parity-common/pull/342)
- Updated mimalloc dependency. [#352](https://github.com/paritytech/parity-common/pull/352)
- Use malloc for `usable_size` on Android. [#355](https://github.com/paritytech/parity-common/pull/355)

## [0.5.1] - 2019-02-05
- Add different mode for malloc_size_of_is_0 macro dealing with generics. [#334](https://github.com/paritytech/parity-common/pull/334)

## [0.5.0] - 2019-02-05
- Bump parking_lot to 0.10. [#332](https://github.com/paritytech/parity-common/pull/332)

## [0.4.2] - 2020-02-04
- Implementation of `MallocSizeOf` for `BTreeSet`. [#325](https://github.com/paritytech/parity-common/pull/325)
- Split off implementation of `MallocSizeOf` for `primitive-types`. [#323](https://github.com/paritytech/parity-common/pull/323)

## [0.4.1] - 2020-01-06
- Implementation of `MallocSizeOf` for SmallVec no longer requires ethereum `ethereum-impls` feature. [#307](https://github.com/paritytech/parity-common/pull/307)

## [0.4.0] - 2020-01-01
- Added implementation of `MallocSizeOf` for non-std `hashbrown::HashMap` and `lru::LRUMap`. [#293](https://github.com/paritytech/parity-common/pull/293)
- Introduced our own version of `#[derive(MallocSizeOf)]` [#291](https://github.com/paritytech/parity-common/pull/291)
- Added implementation of `MallocSizeOf` for `parking_lot` locking primitives. [#290](https://github.com/paritytech/parity-common/pull/290)
- Added default implementation of `MallocSizeOf` for tuples up to 12. [#300](https://github.com/paritytech/parity-common/pull/300)

## [0.3.0] - 2019-12-19
- Remove `MallocSizeOf` impls for `ElasticArray` and implement it for `SmallVec` (32 and 36). [#282](https://github.com/paritytech/parity-common/pull/282)

## [0.2.1] - 2019-10-24
### Dependencies
- Updated dependencies. [#239](https://github.com/paritytech/parity-common/pull/239)
