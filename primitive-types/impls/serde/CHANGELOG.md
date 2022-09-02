# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [0.4.0] - 2022-09-02
- Support deserializing H256 et al from bytes or sequences of bytes, too. [#668](https://github.com/paritytech/parity-common/pull/668)
- Support deserializing H256 et al from newtype structs containing anything compatible, too. [#672](https://github.com/paritytech/parity-common/pull/672)
- Migrated to 2021 edition, enforcing MSRV of `1.56.1`. [#601](https://github.com/paritytech/parity-common/pull/601)

## [0.3.2] - 2021-11-10
- Supported decoding of hex strings without `0x` prefix. [#598](https://github.com/paritytech/parity-common/pull/598)

## [0.3.1] - 2020-05-05
- Added `no_std` support. [#385](https://github.com/paritytech/parity-common/pull/385)

## [0.2.3] - 2019-10-29
### Fixed
- Fixed a bug in empty slice serialization. [#253](https://github.com/paritytech/parity-common/pull/253)
