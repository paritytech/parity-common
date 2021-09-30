# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]

## [0.12.1] - 2021-09-30
- Combined `scale-info` feature into `codec`. [#593](https://github.com/paritytech/parity-common/pull/593)

## [0.12.0] - 2021-07-02
### Breaking
- Updated `primitive-types` to 0.10. [#556](https://github.com/paritytech/parity-common/pull/556)

## [0.11.0] - 2021-01-27
### Breaking
- Updated `ethbloom` to 0.11. [#510](https://github.com/paritytech/parity-common/pull/510)
- Updated `primitive-types` to 0.9. [#510](https://github.com/paritytech/parity-common/pull/510)
- Updated `impl-codec` to 0.5. [#510](https://github.com/paritytech/parity-common/pull/510)

### Potentially-breaking
- `serialize` feature no longer pulls `std`. [#503](https://github.com/paritytech/parity-common/pull/503)

## [0.10.0] - 2021-01-05
### Breaking
- Updated `rlp` to 0.5. [#463](https://github.com/paritytech/parity-common/pull/463)
- Updated `uint` to 0.9. [#486](https://github.com/paritytech/parity-common/pull/486)

## [0.9.2] - 2020-05-18
- Added `codec` feature. [#393](https://github.com/paritytech/parity-common/pull/393)

## [0.9.1] - 2020-04-27
- Added `arbitrary` feature. [#378](https://github.com/paritytech/parity-common/pull/378)

## [0.9.0] - 2020-03-16
- License changed from MIT to dual MIT/Apache2. [#342](https://github.com/paritytech/parity-common/pull/342)
- Updated dependencies. [#361](https://github.com/paritytech/parity-common/pull/361)

### Added
- Uint error type is re-exported. [#244](https://github.com/paritytech/parity-common/pull/244)
