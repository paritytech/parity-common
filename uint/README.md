# Big unsigned integer types

Implementation of a various large-but-fixed sized unsigned integer types.
The functions here are designed to be fast. There are optional `x86_64`
implementations for even more speed, hidden behind the `x64_arithmetic`
feature flag.

Run tests with `cargo test --features=std,impl_quickcheck_arbitrary`.