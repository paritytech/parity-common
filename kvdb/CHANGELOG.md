# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]

## [0.3.0] - 2020-01-03
- I/O statistics API. [#294](https://github.com/paritytech/parity-common/pull/294)
- Removed `KeyValueDBHandler` trait. [#304](https://github.com/paritytech/parity-common/pull/304)

## [0.2.0] - 2019-12-19
### Changed
- Default column support removed from the API
  - Column argument type changed from `Option<u32>` to `u32`
  - Migration `None` -> unsupported, `Some(0)` -> `0`, `Some(1)` -> `1`, etc.
- Remove `ElasticArray` and change `DBValue` to be a type alias for `Vec<u8>` and add a `DBKey` backed by a `SmallVec`.  (See [PR #282](https://github.com/paritytech/parity-common/pull/282/files))

## [0.1.1] - 2019-10-24
### Dependencies
- Updated dependencies (https://github.com/paritytech/parity-common/pull/239)
### Changed
- Migrated to 2018 edition (https://github.com/paritytech/parity-common/pull/205)
