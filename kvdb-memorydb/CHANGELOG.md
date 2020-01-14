# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]
### Breaking
- Updated to the new `kvdb` interface. [#313](https://github.com/paritytech/parity-common/pull/313) 

## [0.3.0] - 2019-01-03
- InMemory key-value database now can report memory used (via `MallocSizeOf`). [#292](https://github.com/paritytech/parity-common/pull/292)

## [0.2.0] - 2019-12-19
### Fixed
- `iter_from_prefix` behaviour synced with the `kvdb-rocksdb`

### Changed
- Default column support removed from the API
  - Column argument type changed from `Option<u32>` to `u32`
  - Migration `None` -> unsupported, `Some(0)` -> `0`, `Some(1)` -> `1`, etc.
