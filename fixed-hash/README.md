Macros to construct fixed-size hash types. Does not export any types.

Examples:

```rust
construct_hash!(H256, 32);
```

Add conversions between differently sized hashes:

```rust
construct_hash!(H256, 32);
construct_hash!(H160, 20);
impl_hash_conversions!(H256, 32, H160, 20);
```

Add conversions between a hash type and the equivalently sized unsigned int:

```rust
extern crate uint;
construct_hash!(H256, 32);
use uint::U256;
impl_hash_uint_conversions!(H256, U256);
```

Build a serde serializable hash type:

```rust
construct_hash!(H160, 20, cfg_attr(feature = "serialize", derive(Serialize, Deserialize)));
```