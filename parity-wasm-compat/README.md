# parity-wasm-compat

Compatibility crate for different wasm output, and to keep existing non wasm usage.

This is a crate hosting common abstraction over multiple missing stdlib compatibility.

Those functionality are mainly here until better alternative are published or usable.

This crate is mainly here to avoid code redundancy and focus on opinionated parity related usage (for instance browser wasm thread could simply be a synchronous execution : that obviously only works for a small category of code and is not equivalent).


