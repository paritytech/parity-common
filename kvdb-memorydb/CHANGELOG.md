# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]

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
