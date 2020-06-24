# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]

## [0.7.0] - 2020-06-24
- Updated `parity-util-mem` to 0.7. [#402](https://github.com/paritytech/parity-common/pull/402)

## [0.6.0] - 2020-05-05
### Breaking
- Removed `write_buffered` and `flush` methods. [#313](https://github.com/paritytech/parity-common/pull/313)
- Introduced a new `DeletePrefix` database operation. [#360](https://github.com/paritytech/parity-common/pull/360)
- Renamed prefix iteration to `iter_with_prefix`. [#365](https://github.com/paritytech/parity-common/pull/365)

## [0.5.0] - 2020-03-16
- License changed from GPL3 to dual MIT/Apache2. [#342](https://github.com/paritytech/parity-common/pull/342)
- Remove dependency on parity-bytes. [#351](https://github.com/paritytech/parity-common/pull/351)
- Updated dependencies. [#361](https://github.com/paritytech/parity-common/pull/361)

## [0.4.0] - 2019-01-06
- Bump parking_lot to 0.10. [#332](https://github.com/paritytech/parity-common/pull/332)

## [0.3.1] - 2019-01-06
- Updated features and feature dependencies. [#307](https://github.com/paritytech/parity-common/pull/307)

## [0.3.0] - 2020-01-03
- I/O statistics API. [#294](https://github.com/paritytech/parity-common/pull/294)
- Removed `KeyValueDBHandler` trait. [#304](https://github.com/paritytech/parity-common/pull/304)

## [0.2.0] - 2019-12-19
### Changed
- Default column support removed from the API
  - Column argument type changed from `Option<u32>` to `u32`
  - Migration `None` -> unsupported, `Some(0)` -> `0`, `Some(1)` -> `1`, etc.
- Remove `ElasticArray` and change `DBValue` to be a type alias for `Vec<u8>` and add a `DBKey` backed by a `SmallVec`.  [#282](https://github.com/paritytech/parity-common/pull/282)

## [0.1.1] - 2019-10-24
### Dependencies
- Updated dependencies. [#239](https://github.com/paritytech/parity-common/pull/239)
### Changed
- Migrated to 2018 edition. [#205](https://github.com/paritytech/parity-common/pull/205)
