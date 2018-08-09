# HashDB
`HashDB` defines a common interface for databases of byte-slices keyed to their hash. It is generic over hash type through the `Hasher` trait.

The `Hasher` trait can be used in a `no_std` context.