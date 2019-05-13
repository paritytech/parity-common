# parity-crypto

General cryptographic utilities for Ethereum.

By default, this library is compiled with the `secp256k1` feature, which provides ECDH and ECIES capability on that curve. It can be compiled without to avoid a dependency on the `libsecp256k1` library.


## Changelog

The 0.4 release removes the dependency on `ring` and replaces it with prue-rust alternatives. As a consequence of this, AES GCM support has been removed. `subtle` replaces the `constant_time_eq` crate for constant time equality testing.
