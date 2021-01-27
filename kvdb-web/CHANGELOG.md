# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]

## [0.9.0] - 2021-01-27
### Breaking
- Updated `kvdb` to 0.9. [#510](https://github.com/paritytech/parity-common/pull/510)
- Updated `kvdb-memorydb` to 0.9. [#510](https://github.com/paritytech/parity-common/pull/510)
- Updated `parity-util-mem` to 0.9. [#510](https://github.com/paritytech/parity-common/pull/510)

## [0.8.0] - 2021-01-05
### Breaking
- Updated dependencies. [#470](https://github.com/paritytech/parity-common/pull/470)

## [0.7.0] - 2020-07-06
- Updated `kvdb` to 0.7.0 [#404](https://github.com/paritytech/parity-common/pull/404)

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

## [0.3.0] - 2019-01-04
- Updated to new `kvdb` and `parity-util-mem` versions. [#299](https://github.com/paritytech/parity-common/pull/299)

## [0.2.0] - 2019-12-19
### Changed
- Default column support removed from the API
  - Column argument type changed from `Option<u32>` to `u32`
  - Migration `None` -> unsupported, `Some(0)` -> `0`, `Some(1)` -> `1`, etc.

## [0.1.1] - 2019-10-24
### Dependencies
- Updated dependencies. [#239](https://github.com/paritytech/parity-common/pull/239)
