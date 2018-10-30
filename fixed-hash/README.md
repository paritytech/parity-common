# Fixed Hash

Provides macros to construct custom fixed-size hash types.

## Examples

Simple 256 bit (32 bytes) hash type.

```rust
#[macro_use] extern crate fixed_hash;

construct_fixed_hash! {
    /// My 256 bit hash type.
    pub struct H256(32);
}
```

Opt-in to add conversions between differently sized hashes.

```rust
construct_fixed_hash!{ struct H256(32); }
construct_fixed_hash!{ struct H160(20); }
// auto-implement conversions between H256 and H160
impl_fixed_hash_conversions!(H256, H160);
// now use the generated conversions
assert_eq!(H256::from(H160::zero()), H256::zero());
assert_eq!(H160::from(H256::zero()), H160::zero());
```

It is possible to add attributes to your types, for example to make them serializable.

```rust
extern crate serde;
#[macro_use] extern crate serde_derive;

construct_fixed_hash!{
    /// My serializable hash type.
    #[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
    struct H160(20);
}
```
