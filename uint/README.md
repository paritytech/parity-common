# Uint

## Description

Provides facilities to construct big unsigned integer types.
Also provides commonly used `U256` and `U512` out of the box.

The focus on the provided big unsigned integer types is performance and cross-platform availability.
Support a very similar API as the built-in primitive integer types.

## Usage

In your `Cargo.toml` paste

```
uint = "0.5.0-beta"
```

Construct your own big unsigned integer type as follows.

```
// U1024 with 1024 bits consisting of 16 x 64-bit words
construct_uint!(U1024; 16);
```

## Tests

### Basic tests

```
cargo test --release
```

### Basic tests + property tests

```
cargo test --release --features=quickcheck
```

### Benchmark tests

```
cargo bench
```

## Crate Features

- `std`: Use Rust's standard library.
	- Enables `byteorder/std`, `rustc-hex/std`
	- Enabled by default.
- `common`: Provide commonly used `U256` and `U512` big unsigned integer types.
	- Enabled by default.
- `quickcheck`: Enable quickcheck-style property testing
	- Use with `cargo test --release --features=quickcheck`.
