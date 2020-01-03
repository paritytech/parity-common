# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]

## [0.4.0] - 2020-01-01
- Added implementation of `MallocSizeOf` for non-std `hashbrown::HashMap` and `lru::LRUMap`. [#293](https://github.com/paritytech/parity-common/pull/293)
- Introduced our own version of `#[derive(MallocSizeOf)]` [#291](https://github.com/paritytech/parity-common/pull/291)
- Added implementation of `MallocSizeOf` for `parking_lot` locking primitives. [#290](https://github.com/paritytech/parity-common/pull/290)
- Added default implementation of `MallocSizeOf` for tuples up to 12. [#300](https://github.com/paritytech/parity-common/pull/300)

## [0.3.0] - 2019-12-19
- Remove `MallocSizeOf` impls for `ElasticArray` and implement it for `SmallVec` (32 and 36). (See [PR #282](https://github.com/paritytech/parity-common/pull/282/files))

## [0.2.1] - 2019-10-24
### Dependencies
- Updated dependencies (https://github.com/paritytech/parity-common/pull/239)
