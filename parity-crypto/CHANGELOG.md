# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]

### Breaking
- Updated `ethereum-types` to 0.12. [#556](https://github.com/paritytech/parity-common/pull/556)

## [0.8.0] - 2021-01-27
### Breaking
- Updated `ethereum-types` to 0.11. [#510](https://github.com/paritytech/parity-common/pull/510)

## [0.7.0] - 2021-01-05
### Breaking
- Bump `rust-secp256k1` to v0.19, always allow zero signatures. [#438](https://github.com/paritytech/parity-common/pull/438)
- Updated `rlp` to 0.5. [#463](https://github.com/paritytech/parity-common/pull/463)
- Updated dependencies. [#483](https://github.com/paritytech/parity-common/pull/483)
- Remove deprecated trait impls `FromStr`/`TryFrom` for `Secret` [#495](https://github.com/paritytech/parity-common/pull/495)

## [0.6.2] - 2020-06-19
- Put `Secret` memory on heap. [#400](https://github.com/paritytech/parity-common/pull/400)
- Add `copy_from_str` conversion methods for `Secret`.
- Deprecate `From<&str>` in favor of `copy_from_str`.

## [0.6.1] - 2020-04-11
- Add `recover_allowing_all_zero_message()` and `ZeroesAllowedMessage` to accomodate ethereum's `ecrecover` builtin. [#369](https://github.com/paritytech/parity-common/pull/369)

## [0.6.0] - 2020-03-16
- License changed from GPL3 to dual MIT/Apache2. [#342](https://github.com/paritytech/parity-common/pull/342)
- Updated dependencies. [#361](https://github.com/paritytech/parity-common/pull/361)

## [0.5.0] - 2020-02-08
- Remove `inv()` from `SecretKey` (breaking). [#258](https://github.com/paritytech/parity-common/pull/258)
- `Generate::generate()` does not return error. [#258](https://github.com/paritytech/parity-common/pull/258)
- `Secp256k1` is no longer exported. [#258](https://github.com/paritytech/parity-common/pull/258)
- Remove `public_is_valid()` as it is now impossible to create invalid public keys. [#258](https://github.com/paritytech/parity-common/pull/258)
- 0-valued `Secp::Message`s are disallowed (signatures on them are forgeable for all keys). [#258](https://github.com/paritytech/parity-common/pull/258)
- Switch to upstream `rust-secp256k1` at v0.17.2. [#258](https://github.com/paritytech/parity-common/pull/258)
- make `rustc_hex` dependency optional. [#337](https://github.com/paritytech/parity-common/pull/337)
