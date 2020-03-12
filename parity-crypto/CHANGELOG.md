# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]
- License changed from GPL3 to dual MIT/Apache2. [#342](https://github.com/paritytech/parity-common/pull/342)

## [0.5.0] - 2020-02-08
- Remove `inv()` from `SecretKey` (breaking). [#258](https://github.com/paritytech/parity-common/pull/258)
- `Generate::generate()` does not return error. [#258](https://github.com/paritytech/parity-common/pull/258)
- `Secp256k1` is no longer exported. [#258](https://github.com/paritytech/parity-common/pull/258)
- Remove `public_is_valid()` as it is now impossible to create invalid public keys. [#258](https://github.com/paritytech/parity-common/pull/258)
- 0-valued `Secp::Message`s are disallowed (signatures on them are forgeable for all keys). [#258](https://github.com/paritytech/parity-common/pull/258)
- Switch to upstream `rust-secp256k1` at v0.17.2. [#258](https://github.com/paritytech/parity-common/pull/258)
- make `rustc_hex` dependency optional. [#337](https://github.com/paritytech/parity-common/pull/337)
