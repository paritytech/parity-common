# parity-util-mem

Collection of memory related utilities.

## WARNING

When `parity-util-mem` is used as a dependency with any of the global allocator features enabled,
it must be the sole place where a global allocator is defined.
The only exception to this rule is when used in a `no_std` context or when the `estimate-heapsize` feature is used.

Because of that, it must be present in the dependency tree with a single version.
Starting from version 0.6.1, having duplicate versions of `parity-util-mem` will lead
to a compile-time error. It still will be possible to have 0.5 and 0.6.1 versions in the same binary though.

Unless heeded you risk UB; see discussion in [issue 364].

[issue 364]: https://github.com/paritytech/parity-common/issues/364

## Features

- estimate-heapsize : Do not use allocator, but `size_of` or `size_of_val`.

Others features define global allocator, see `src/alloc.rs`.

## Dependency

This crate groups common dependency, a patched copy of unpublished [`malloc_size_of`](https://github.com/servo/servo/tree/master/components/malloc_size_of) from servo project is copied and partially reexported.

`Malloc_size_of` code is used internally as a module with a few modification to be able to implement type locally.

For existing code using deprecated `HeapsizeOf` crate, calls to `heapsize_of_children` should be replace by calls to `size_of`.
