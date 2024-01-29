# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [0.2.0] - 2024-01-29
- Added `try_rotate_left` and `try_rotate_right` to `BoundedVec`. [#800](https://github.com/paritytech/parity-common/pull/800)

## [0.1.9] - 2023-10-10
- Added `serde` support for `BoundedBTreeSet`. [#781](https://github.com/paritytech/parity-common/pull/781)

## [0.1.8] - 2023-06-11
- Altered return types of `BoundedVec::force_insert_keep_` functions to return the element in case of error.
- Added `new` and `clear` to `BoundedVec`.

## [0.1.7] - 2023-05-05
- Added `serde` feature, which can be enabled for no `std` deployments.

## [0.1.6] - 2023-04-27
- Added `Clone` and `Default` derive to the `impl_const_get!` macro and thereby all `Const*` types.
- Fixed `Debug` impl for `impl_const_get!` and all `Const*` types to also print the value and not just the type name.

## [0.1.5] - 2023-02-13
- Fixed `Hash` impl (previously it could not be used in practice, because the size bound was required to also implement `Hash`).

## [0.1.4] - 2023-01-28
- Fixed unnecessary decoding and allocations for bounded types, when the decoded length is greater than the allowed bound.
- Add `Hash` derivation (when `feature = "std"`) for bounded types.

## [0.1.3] - 2023-01-27
- Removed non-existent `bounded` mod reference. [#715](https://github.com/paritytech/parity-common/pull/715)

## [0.1.2] - 2023-01-27
- Ensured `bounded-collections` crate compiles under `no_std`. [#712](https://github.com/paritytech/parity-common/pull/712)

## [0.1.1] - 2023-01-26
- Made `alloc` public. [#711](https://github.com/paritytech/parity-common/pull/711)
- Removed a reference to `sp_core` in the comments. [#710](https://github.com/paritytech/parity-common/pull/710)

## [0.1.0] - 2023-01-26
- Wrote better description for `bounded-collections`. [#709](https://github.com/paritytech/parity-common/pull/709)
- Added `bounded-collections` crate. [#708](https://github.com/paritytech/parity-common/pull/708)
