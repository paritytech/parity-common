# Changelog

The format is based on [Keep a Changelog]. 

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]
### Changed
- [BREAKING] Default column support removed from the API
  - Column argument type changed from `Option<u32>` to `u32`
  - Migration `None` -> `0`, `Some(0)` -> `1`, `Some(1)` -> `2`, etc.
- [BREAKING] Remove `ElasticArray` and change `DBValue` to be a type alias for `Vec<u8>` and add a `DBKey` backed by a `SmallVec`.  (See [PR #282](https://github.com/paritytech/parity-common/pull/282/files))
## [0.1.1] - 2019-10-24
### Dependencies
- Updated dependencies (https://github.com/paritytech/parity-common/pull/239)
### Changed
- Migrated to 2018 edition (https://github.com/paritytech/parity-common/pull/205)
