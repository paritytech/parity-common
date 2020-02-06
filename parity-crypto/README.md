# parity-crypto

General cryptographic utilities for Ethereum.


## Changelog

The 0.4 release removes the dependency on `ring` and replaces it with pure rust alternatives. As a consequence of this, AES GCM support has been removed. `subtle` is used for constant time equality testing and error handling is pared down to the bare minimum required.
