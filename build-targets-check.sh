#!/bin/sh
# checks combination features are working in real no std and in std other features working for basic types

# uses build instead of check as sometimes some issues are not caught by check
cargo_deep_check() {
    cargo build --locked --no-default-features $@
}

cargo_deep_check --target thumbv7em-none-eabi --package primitive-types --features=codec,serde_no_std,byteorder,rustc-hex,codec
cargo_deep_check --target wasm32-unknown-unknown --package primitive-types --features=codec,serde_no_std,byteorder,rustc-hex,codec,num-traits
cargo_deep_check --package primitive-types --features=std,serde,json-schema,byteorder,rustc-hex,codec
cargo_deep_check --target thumbv7em-none-eabi --package bounded-collections
cargo_deep_check --package primitive-types --features=json-schema
cargo_deep_check --target wasm32-unknown-unknown --package bounded-collections