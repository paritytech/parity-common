# Changelog

The format is based on [Keep a Changelog]. 

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]
### Fixed
- `iter_from_prefix` behaviour synced with the `kvdb-rocksdb`
### Changed
- Default column support removed from the API
  - Column argument type changed from `Option<u32>` to `u32`
  - Migration `None` -> `0`, `Some(0)` -> `1`, `Some(1)` -> `2`, etc.
