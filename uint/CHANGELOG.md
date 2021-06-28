# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]
- Added `integer_sqrt` method. [#554](https://github.com/paritytech/parity-common/pull/554)

## [0.9.0] - 2021-01-05
- Allow `0x` prefix in `from_str`. [#487](https://github.com/paritytech/parity-common/pull/487)
### Breaking
- Optimized FromStr, made it no_std-compatible. [#468](https://github.com/paritytech/parity-common/pull/468)

## [0.8.5] - 2020-08-12
- Make const matching work again. [#421](https://github.com/paritytech/parity-common/pull/421)

## [0.8.4] - 2020-08-03
- Added a manual impl of `Eq` and `Hash`. [#390](https://github.com/paritytech/parity-common/pull/390)
- Removed some unsafe code and added big-endian support. [#407](https://github.com/paritytech/parity-common/pull/407)
- Added `checked_pow`. [#417](https://github.com/paritytech/parity-common/pull/417)

## [0.8.3] - 2020-04-27
- Added `arbitrary` feature. [#378](https://github.com/paritytech/parity-common/pull/378)
- Fixed UB in `from_big_endian`. [#381](https://github.com/paritytech/parity-common/pull/381)

## [0.8.2] - 2019-10-24
### Fixed
- Fixed 2018 edition imports. [#237](https://github.com/paritytech/parity-common/pull/237)
- Removed `uninitialized` usage. [#238](https://github.com/paritytech/parity-common/pull/238)
### Dependencies
- Updated dependencies. [#239](https://github.com/paritytech/parity-common/pull/239)
### Changed
- Modified AsRef impl. [#196](https://github.com/paritytech/parity-common/pull/196)
