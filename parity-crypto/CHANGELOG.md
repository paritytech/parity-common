# Changelog

The format is based on [Keep a Changelog]. 

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]
- Remove `inv()` from `SecretKey` (breaking)
- `Generate::generate()` does not return error
- `Secp256k1` is no longer exported 
- Remove `public_is_valid()` as it is now impossible to create invalid public keys
- 0-valued `Secp::Message`s are disallowed (signatures on them are forgeable for all keys)
- updates to upstream `rust-secp256k1` at v0.17.2
