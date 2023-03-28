# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]

## [0.13.0] - 2022-11-29
- Removed `parity-util-mem` support. [#696](https://github.com/paritytech/parity-common/pull/696)

## [0.12.0] - 2022-09-20
### Breaking
- Updated `kvdb` to 0.12. [662](https://github.com/paritytech/parity-common/pull/662)
- Updated `parity-util-mem` to 0.12. [#680](https://github.com/paritytech/parity-common/pull/680)

## [0.11.0] - 2022-02-04
### Breaking
- Migrated to 2021 edition, enforcing MSRV of `1.56.1`. [#601](https://github.com/paritytech/parity-common/pull/601)
- Updated `kvdb` to 0.11. [#623](https://github.com/paritytech/parity-common/pull/623)

## [0.10.0] - 2021-07-02
### Breaking
- Updated `parity-util-mem` to 0.10. [#556](https://github.com/paritytech/parity-common/pull/556)
- Updated `kvdb` to 0.10. [#556](https://github.com/paritytech/parity-common/pull/556)

## [0.9.0] - 2021-01-27
### Breaking
- Updated `parity-util-mem` to 0.9. [#510](https://github.com/paritytech/parity-common/pull/510)
- Updated `kvdb` to 0.9. [#510](https://github.com/paritytech/parity-common/pull/510)

## [0.8.0] - 2021-01-05
### Breaking
- Updated dependencies. [#470](https://github.com/paritytech/parity-common/pull/470)

## [0.7.0] - 2020-06-24
- Updated `kvdb` to 0.7. [#402](https://github.com/paritytech/parity-common/pull/402)

## [0.6.0] - 2020-05-05
### Breaking
- Updated to the new `kvdb` interface. [#313](https://github.com/paritytech/parity-common/pull/313)

## [0.5.0] - 2020-03-16
- License changed from GPL3 to dual MIT/Apache2. [#342](https://github.com/paritytech/parity-common/pull/342)
- Updated dependencies. [#361](https://github.com/paritytech/parity-common/pull/361)

## [0.4.0] - 2019-02-05
- Bump parking_lot to 0.10. [#332](https://github.com/paritytech/parity-common/pull/332)

## [0.3.1] - 2019-01-06
- Updated features and feature dependencies. [#307](https://github.com/paritytech/parity-common/pull/307)

## [0.3.0] - 2019-01-03
- InMemory key-value database now can report memory used (via `MallocSizeOf`). [#292](https://github.com/paritytech/parity-common/pull/292)

## [0.2.0] - 2019-12-19
### Fixed
- `iter_from_prefix` behaviour synced with the `kvdb-rocksdb`
### Changed
- Default column support removed from the API
  - Column argument type changed from `Option<u32>` to `u32`
  - Migration `None` -> unsupported, `Some(0)` -> `0`, `Some(1)` -> `1`, etc.
