# parity-util-mem

Collection of memory related utilities.

## Features

- volatile-erase : Not set by default, `Memzero` struct will be erasing memory through a simple [`write_volatile`](https://doc.rust-lang.org/std/ptr/fn.write_volatile.html) call.
- estimate-heapsize : Do not use allocator, but `size_of` or `size_of_val`.

Others features define global allocator, see `src/alloc.rs`.

## Dependency

This crate groups common dependency, [`clear_on_drop`](https://crates.io/crates/clear_on_drop) is reexported, and a patched copy of unpublished [`malloc_size_of`](https://github.com/servo/servo/tree/master/components/malloc_size_of) from servo project is copied and partially reexported.

`Malloc_size_of` code is used internally as a module with a few modification to be able to implement type locally.
