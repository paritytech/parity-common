# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]

## [0.5.2] - 2022-10-21
- Add optional `derive` feature. [#613](https://github.com/paritytech/parity-common/pull/613)

## [0.5.1] - 2021-07-30
- Fix rlp encoding/decoding for bool. [#572](https://github.com/paritytech/parity-common/pull/572)

## [0.5.0] - 2021-01-05
### Breaking
- Use BytesMut for `RlpStream`'s backing buffer. [#453](https://github.com/paritytech/parity-common/pull/453)

## [0.4.6] - 2020-09-29
- Implement Encodable, Decodable for boxed types. [#427](https://github.com/paritytech/parity-common/pull/427)

## [0.4.5] - 2020-03-16
### Dependencies
- Updated dependencies. [#361](https://github.com/paritytech/parity-common/pull/361)

## [0.4.4] - 2019-11-20
### Added
- Method `Rlp::at_with_offset`. [#269](https://github.com/paritytech/parity-common/pull/269)

## [0.4.3] - 2019-10-24
### Dependencies
- Updated dependencies. [#239](https://github.com/paritytech/parity-common/pull/239)
### Fixed
- Fixed nested unbounded lists. [#203](https://github.com/paritytech/parity-common/pull/203)
### Added
- Added no-std support. [#206](https://github.com/paritytech/parity-common/pull/206)
