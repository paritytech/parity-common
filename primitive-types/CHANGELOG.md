# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]

## [0.12.2] - 2023-10-10
- Added `schemars` support via `json-schema` feature. [#785](https://github.com/paritytech/parity-common/pull/785)

## [0.12.1] - 2022-10-27
- Added `H384` and `H768` types. [#684](https://github.com/paritytech/parity-common/pull/684)

## [0.12.0] - 2022-09-20
### Breaking
- Updated `fixed-hash` to 0.8. [#680](https://github.com/paritytech/parity-common/pull/680)
- Uses weak-dependency feature of cargo. [#664](https://github.com/paritytech/parity-common/pull/664)

## [0.11.1] - 2022-02-07
- Updated `scale-info` to ">=0.9, <3". [#627](https://github.com/paritytech/parity-common/pull/627)

## [0.11.0] - 2022-02-04
### Breaking
- Migrated to 2021 edition, enforcing MSRV of `1.56.1`. [#601](https://github.com/paritytech/parity-common/pull/601)
- Updated `impl-codec` to 0.6. [#623](https://github.com/paritytech/parity-common/pull/623)

## [0.10.1] - 2021-07-02
### Added
- Implemented `parity_scale_codec::MaxEncodedLen` trait for `{U128, U256, U512}` and `{H128, H160, H256, H512}` types.

## [0.10.0] - 2021-07-02
### Added
- Added `U128::full_mul` method. [#546](https://github.com/paritytech/parity-common/pull/546)
### Breaking
- Updated `scale-info` to 0.9. [#556](https://github.com/paritytech/parity-common/pull/556)
### Removed
- Removed `parity-scale-codec` direct dependency. [#556](https://github.com/paritytech/parity-common/pull/556)

## [0.9.0] - 2021-01-27
### Breaking
- Updated `impl-codec` to 0.5. [#510](https://github.com/paritytech/parity-common/pull/510)
- Updated `scale-info` to 0.5. [#510](https://github.com/paritytech/parity-common/pull/510)

## [0.8.0] - 2021-01-05
- Added `num-traits` feature. [#480](https://github.com/paritytech/parity-common/pull/480)
### Breaking
- Updated `impl-rlp` to `rlp` 0.5. [#463](https://github.com/paritytech/parity-common/pull/463)
- Updated `uint` to 0.9. [#486](https://github.com/paritytech/parity-common/pull/486)

## [0.7.3] - 2020-11-12
- Added `scale_info` support. [#312](https://github.com/paritytech/parity-common/pull/312)
- Added `H128` type. [#434](https://github.com/paritytech/parity-common/pull/434)
- Added `fp-conversion` feature: `U256` <-> `f64`. [#436](https://github.com/paritytech/parity-common/pull/436)

## [0.7.2] - 2020-05-05
- Added `serde_no_std` feature. [#385](https://github.com/paritytech/parity-common/pull/385)

## [0.7.1] - 2020-04-27
- Added `arbitrary` feature. [#378](https://github.com/paritytech/parity-common/pull/378)

## [0.7.0] - 2020-03-16
- Removed `libc` feature. [#317](https://github.com/paritytech/parity-common/pull/317)

## [0.6.2] - 2019-01-03
- Expose to_hex and from_hex from impl-serde. [#302](https://github.com/paritytech/parity-common/pull/302)

## [0.6.1] - 2019-10-24
### Dependencies
- Updated dependencies. [#239](https://github.com/paritytech/parity-common/pull/239)
