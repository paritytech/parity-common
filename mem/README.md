# parity-util-mem

Collection of memory related utilities.

## Features

- volatile-erase : Not set by default, `Memzero` erase memory with `write_volatile`.
- estimate-heapsize : Do not use allocator, but `size_of` or `size_of_val`.
- conditional-metering : Try to avoid counting `Arc` twice. For test only.
Others feature are here to define global allocator, see `src/alloc.rs`.

## Dependency

This crate groups common dependency, `clear_on_drop` is reexported, and a patched copy of unpublished `malloc_size_of` from servo project is used and partially reexported.
