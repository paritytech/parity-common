# Changelog

The format is based on [Keep a Changelog]. 

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]
- use `get_pinned` API to save one allocation for each call to `get()` (See [PR #274](https://github.com/paritytech/parity-common/pull/274) for details)
- rename `drop_column` to `remove_last_column` (See [PR #274](https://github.com/paritytech/parity-common/pull/274) for details)
- rename `get_cf` to `cf` (See [PR #274](https://github.com/paritytech/parity-common/pull/274) for details)

## [0.2.0] - 2019-11-28
- Switched away from using [parity-rocksdb](https://crates.io/crates/parity-rocksdb) in favour of upstream [rust-rocksdb](https://crates.io/crates/rocksdb) (see [PR #257](https://github.com/paritytech/parity-common/pull/257) for details)
- Revamped configuration handling, allowing per-column memory budgeting (see [PR #256](https://github.com/paritytech/parity-common/pull/256) for details)
### Dependencies
- rust-rocksdb v0.13

## [0.1.6] - 2019-10-24
- Updated to 2018 edition idioms (https://github.com/paritytech/parity-common/pull/237)
### Dependencies
- Updated dependencies (https://github.com/paritytech/parity-common/pull/239)
