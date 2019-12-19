# Changelog

The format is based on [Keep a Changelog]. 

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]
- Use `get_pinned` API to save one allocation for each call to `get()` (See [PR #274](https://github.com/paritytech/parity-common/pull/274) for details)
- Rename `drop_column` to `remove_last_column` (See [PR #274](https://github.com/paritytech/parity-common/pull/274) for details)
- Rename `get_cf` to `cf` (See [PR #274](https://github.com/paritytech/parity-common/pull/274) for details)
- Default column support removed from the API (See [PR #278](https://github.com/paritytech/parity-common/pull/278) for details)
  - Column argument type changed from `Option<u32>` to `u32`
  - Migration
    - Column index `None` -> `0`, `Some(0)` -> `1`, `Some(1)` -> `2`, etc.
    - Database must be opened with at least one column and existing DBs has to be opened with a number of columns increased by 1 to avoid having to migrate the data, e.g. before: `Some(9)`, after: `10`.
  - `DatabaseConfig::default()` defaults to 1 column
  - `Database::with_columns` still accepts `u32`, but panics if `0` is provided 
  - `Database::open` panics if configuration with 0 columns is provided
- Add `num_keys(col)` to get an estimate of the number of keys in a column (See [PR #285](https://github.com/paritytech/parity-common/pull/285)).

### Breaking 
- Remove `ElasticArray` and use the new `DBValue` (alias for `Vec<u8>`) and `DBKey` types from `kvdb`. (See [PR #282](https://github.com/paritytech/parity-common/pull/282/files))  

## [0.2.0] - 2019-11-28
- Switched away from using [parity-rocksdb](https://crates.io/crates/parity-rocksdb) in favour of upstream [rust-rocksdb](https://crates.io/crates/rocksdb) (see [PR #257](https://github.com/paritytech/parity-common/pull/257) for details)
- Revamped configuration handling, allowing per-column memory budgeting (see [PR #256](https://github.com/paritytech/parity-common/pull/256) for details)
### Dependencies
- rust-rocksdb v0.13

## [0.1.6] - 2019-10-24
- Updated to 2018 edition idioms (https://github.com/paritytech/parity-common/pull/237)
### Dependencies
- Updated dependencies (https://github.com/paritytech/parity-common/pull/239)
