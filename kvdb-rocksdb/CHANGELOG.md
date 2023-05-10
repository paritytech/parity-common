# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]

## [0.19.0] - 2023-05-10
- Updated `rocksdb` to 0.21. [#750](https://github.com/paritytech/parity-common/pull/750)

## [0.18.0] - 2023-04-21
- Updated `rocksdb` to 0.20.1. [#743](https://github.com/paritytech/parity-common/pull/743)

## [0.17.0] - 2022-11-29
- Removed `parity-util-mem` support. [#696](https://github.com/paritytech/parity-common/pull/696)

## [0.16.0] - 2022-09-20
- Removed `owning_ref` from dependencies :tada:. [#662](https://github.com/paritytech/parity-common/pull/662)
- No longer attempt to repair on `open`. [#667](https://github.com/paritytech/parity-common/pull/667)
### Breaking
- Updated `kvdb` to 0.12. [#662](https://github.com/paritytech/parity-common/pull/662)
  - `add_column` and `remove_last_column` now require `&mut self`

## [0.15.2] - 2022-03-20
- Disable `jemalloc` feature for `rocksdb` where it is not working. [#633](https://github.com/paritytech/parity-common/pull/633)

## [0.15.1] - 2022-02-18
- Updated `rocksdb` to 0.18 and enable `jemalloc` feature. [#629](https://github.com/paritytech/parity-common/pull/629)

## [0.15.0] - 2022-02-04
### Breaking
- Migrated to 2021 edition, enforcing MSRV of `1.56.1`. [#601](https://github.com/paritytech/parity-common/pull/601)
- Bumped `kvdb` and `parity-util-mem`. [#623](https://github.com/paritytech/parity-common/pull/623)

## [0.14.0] - 2021-08-05
### Breaking
- `Database` api uses now template argument `P: AsRef<Path>` instead of `&str` [#579](https://github.com/paritytech/parity-common/pull/579)

## [0.13.0] - 2021-08-04
### Breaking
- `DatabaseConfig` is now `#[non_exhaustive]`. [#576](https://github.com/paritytech/parity-common/pull/576)
- Added `create_if_missing` to `DatabaseConfig`. [#576](https://github.com/paritytech/parity-common/pull/576)

## [0.12.1] - 2021-07-30
- Bumped `rocksdb` to 0.17. [#573](https://github.com/paritytech/parity-common/pull/573)

## [0.12.0] - 2021-07-02
### Breaking
- Updated `kvdb` to 0.10. [#556](https://github.com/paritytech/parity-common/pull/556)
- Updated `parity-util-mem` to 0.10. [#556](https://github.com/paritytech/parity-common/pull/556)

## [0.11.1] - 2021-05-03
- Updated `rocksdb` to 0.16. [#537](https://github.com/paritytech/parity-common/pull/537)

## [0.11.0] - 2021-01-27
### Breaking
- Updated `kvdb` to 0.9. [#510](https://github.com/paritytech/parity-common/pull/510)
- Updated `parity-util-mem` to 0.9. [#510](https://github.com/paritytech/parity-common/pull/510)

## [0.10.0] - 2021-01-05
### Breaking
- Updated dependencies. [#470](https://github.com/paritytech/parity-common/pull/470)

## [0.9.1] - 2020-08-26
- Updated rocksdb to 0.15. [#424](https://github.com/paritytech/parity-common/pull/424)
- Set `format_version` to 5. [#395](https://github.com/paritytech/parity-common/pull/395)

## [0.9.0] - 2020-06-24
- Updated `kvdb` to 0.7. [#402](https://github.com/paritytech/parity-common/pull/402)

## [0.8.0] - 2020-05-05
- Updated RocksDB to 6.7.3. [#379](https://github.com/paritytech/parity-common/pull/379)
### Breaking
- Updated to the new `kvdb` interface. [#313](https://github.com/paritytech/parity-common/pull/313)
- Rename and optimize prefix iteration. [#365](https://github.com/paritytech/parity-common/pull/365)
- Added Secondary Instance API. [#384](https://github.com/paritytech/parity-common/pull/384)

## [0.7.0] - 2020-03-16
- Updated dependencies. [#361](https://github.com/paritytech/parity-common/pull/361)

## [0.6.0] - 2020-02-28
- License changed from GPL3 to dual MIT/Apache2. [#342](https://github.com/paritytech/parity-common/pull/342)
- Added `get_statistics` method and `enable_statistics` config parameter. [#347](https://github.com/paritytech/parity-common/pull/347)

## [0.5.0] - 2019-02-05
- Bump parking_lot to 0.10. [#332](https://github.com/paritytech/parity-common/pull/332)

## [0.4.2] - 2019-02-04
### Fixes
- Fixed `iter_from_prefix` being slow. [#326](https://github.com/paritytech/parity-common/pull/326)

## [0.4.1] - 2019-01-06
- Updated features and feature dependencies. [#307](https://github.com/paritytech/parity-common/pull/307)

## [0.4.0] - 2019-01-03
- Add I/O statistics for RocksDB. [#294](https://github.com/paritytech/parity-common/pull/294)
- Support querying memory footprint via `MallocSizeOf` trait. [#292](https://github.com/paritytech/parity-common/pull/292)

## [0.3.0] - 2019-12-19
- Use `get_pinned` API to save one allocation for each call to `get()`. [#274](https://github.com/paritytech/parity-common/pull/274)
- Rename `drop_column` to `remove_last_column`. [#274](https://github.com/paritytech/parity-common/pull/274)
- Rename `get_cf` to `cf`. [#274](https://github.com/paritytech/parity-common/pull/274)
- Default column support removed from the API. [#278](https://github.com/paritytech/parity-common/pull/278)
  - Column argument type changed from `Option<u32>` to `u32`
  - Migration
    - Column index `None` -> unsupported, `Some(0)` -> `0`, `Some(1)` -> `1`, etc.
    - Database must be opened with at least one column and existing DBs has to be opened with a number of columns increased by 1 to avoid having to migrate the data, e.g. before: `Some(9)`, after: `10`.
  - `DatabaseConfig::default()` defaults to 1 column
  - `Database::with_columns` still accepts `u32`, but panics if `0` is provided
  - `Database::open` panics if configuration with 0 columns is provided
- Add `num_keys(col)` to get an estimate of the number of keys in a column. [#285](https://github.com/paritytech/parity-common/pull/285)
- Remove `ElasticArray` and use the new `DBValue` (alias for `Vec<u8>`) and `DBKey` types from `kvdb`. [#282](https://github.com/paritytech/parity-common/pull/282)

## [0.2.0] - 2019-11-28
- Switched away from using [parity-rocksdb](https://crates.io/crates/parity-rocksdb) in favour of upstream [rust-rocksdb](https://crates.io/crates/rocksdb). [#257](https://github.com/paritytech/parity-common/pull/257)
- Revamped configuration handling, allowing per-column memory budgeting. [#256](https://github.com/paritytech/parity-common/pull/256)
### Dependencies
- rust-rocksdb v0.13

## [0.1.6] - 2019-10-24
- Updated to 2018 edition idioms. [#237](https://github.com/paritytech/parity-common/pull/237)
### Dependencies
- Updated dependencies. [#239](https://github.com/paritytech/parity-common/pull/239)
