# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]

## [0.14.1] - 2022-11-29
- Added `if_ethbloom` conditional macro. [#682](https://github.com/paritytech/parity-common/pull/682)

## [0.14.0] - 2022-09-20
- Updated `fixed-hash` to 0.8. [#680](https://github.com/paritytech/parity-common/pull/680)
- Updated `primitive-types` to 0.12. [#680](https://github.com/paritytech/parity-common/pull/680)
- Updated `ethbloom` to 0.13. [#680](https://github.com/paritytech/parity-common/pull/680)
- Made `ethbloom` optional. [#625](https://github.com/paritytech/parity-common/pull/625)

## [0.13.1] - 2022-02-07
- Updated `scale-info` to ">=1.0, <3". [#627](https://github.com/paritytech/parity-common/pull/627)

## [0.13.0] - 2022-02-04
### Breaking
- Migrated to 2021 edition, enforcing MSRV of `1.56.1`. [#601](https://github.com/paritytech/parity-common/pull/601)
- Updated `impl-codec` to 0.6. [#623](https://github.com/paritytech/parity-common/pull/623)
- Updated `primitive-types` to 0.11. [#623](https://github.com/paritytech/parity-common/pull/623)
- Updated `ethbloom` to 0.12. [#623](https://github.com/paritytech/parity-common/pull/623)

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
