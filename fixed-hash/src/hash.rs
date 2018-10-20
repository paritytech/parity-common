// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Return the given string `s` without the `0x` at the beginning of it, if any.
pub fn clean_0x(s: &str) -> &str {
	if s.starts_with("0x") {
		&s[2..]
	} else {
		s
	}
}

/// Construct a fixed-size hash type.
/// Takes the name of the type and the size in bytes and an optional third argument for meta data
/// Example: `construct_hash!(H256, 32);`
/// Example: `construct_hash!(H160, 20, cfg_attr(feature = "serialize", derive(Serialize, Deserialize)));`
#[macro_export]
macro_rules! construct_hash {
	($(#[$attr:meta])* $visibility:vis struct $name:ident ( $n_bytes:expr );) => {
		#[repr(C)]
		$(#[$attr])*
		$visibility struct $name (pub [u8; $n_bytes]);

		impl From<[u8; $n_bytes]> for $name {
			fn from(bytes: [u8; $n_bytes]) -> Self {
				$name(bytes)
			}
		}

		impl From<$name> for [u8; $n_bytes] {
			fn from(s: $name) -> Self {
				s.0
			}
		}

		impl AsRef<[u8]> for $name {
			#[inline]
			fn as_ref(&self) -> &[u8] {
				self.as_bytes()
			}
		}

		impl AsMut<[u8]> for $name {
			#[inline]
			fn as_mut(&mut self) -> &mut [u8] {
				self.as_bytes_mut()
			}
		}

		impl $name {
			/// Create a new, zero-initialised, instance.
			pub fn new() -> $name {
				$name([0; $n_bytes])
			}

			/// Synonym for `new()`. Prefer to new as it's more readable.
			pub fn zero() -> $name {
				$name([0; $n_bytes])
			}

			/// Get the size of this object in bytes.
			pub fn len() -> usize {
				$n_bytes
			}

			/// Extracts a byte slice containing the entire fixed hash.
			pub fn as_bytes(&self) -> &[u8] {
				&self.0
			}

			/// Extracts a mutable byte slice containing the entire fixed hash.
			pub fn as_bytes_mut(&mut self) -> &mut [u8] {
				&mut self.0
			}

            /// Returns a constant raw pointer to the value
            pub fn as_ptr(&self) -> *const u8 {
                self.0.as_ptr()
            }

            pub fn as_mut_ptr(&mut self) -> *mut u8 {
                (&mut self.0).as_mut_ptr()
            }

			#[inline]
			/// Assign self to be of the same value as a slice of bytes of length `len()`.
			pub fn clone_from_slice(&mut self, src: &[u8]) -> usize {
				let min = ::core::cmp::min($n_bytes, src.len());
				self.0[..min].copy_from_slice(&src[..min]);
				min
			}

			/// Convert a slice of bytes of length `len()` to an instance of this type.
			pub fn from_slice(src: &[u8]) -> Self {
				let mut r = Self::new();
				r.clone_from_slice(src);
				r
			}

			/// Copy the data of this object into some mutable slice of length `len()`.
			pub fn copy_to(&self, dest: &mut[u8]) {
				let min = ::core::cmp::min($n_bytes, dest.len());
				dest[..min].copy_from_slice(&self.0[..min]);
			}

			/// Returns `true` if all bits set in `b` are also set in `self`.
			pub fn contains<'a>(&'a self, b: &'a Self) -> bool {
				&(b & self) == b
			}

			/// Returns `true` if no bits are set.
			pub fn is_zero(&self) -> bool {
				self.eq(&Self::new())
			}

			/// Returns the lowest 8 bytes interpreted as a BigEndian integer.
			pub fn low_u64(&self) -> u64 {
				let mut ret = 0u64;
				for i in 0..::core::cmp::min($n_bytes, 8) {
					ret |= (self.0[$n_bytes - 1 - i] as u64) << (i * 8);
				}
				ret
			}

			impl_std_for_hash_internals!($name, $n_bytes);
		}

		impl ::core::fmt::Debug for $name {
			fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				write!(f, "{:#x}", self)
			}
		}

		impl ::core::fmt::Display for $name {
			fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				write!(f, "0x")?;
				for i in &self.0[0..2] {
					write!(f, "{:02x}", i)?;
				}
				write!(f, "…")?;
				for i in &self.0[$n_bytes - 2..$n_bytes] {
					write!(f, "{:02x}", i)?;
				}
				Ok(())
			}
		}

		impl ::core::fmt::LowerHex for $name {
			fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				if f.alternate() {
					write!(f, "0x")?;
				}
				for i in &self.0[..] {
					write!(f, "{:02x}", i)?;
				}
				Ok(())
			}
		}

		impl Copy for $name {}
		#[cfg_attr(feature="dev", allow(expl_impl_clone_on_copy))]
		impl Clone for $name {
			fn clone(&self) -> $name {
				let mut ret = $name::new();
				ret.0.copy_from_slice(&self.0);
				ret
			}
		}

		impl Eq for $name {}

		impl PartialOrd for $name {
			fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
				Some(self.cmp(other))
			}
		}

		impl ::core::hash::Hash for $name {
			fn hash<H>(&self, state: &mut H) where H: ::core::hash::Hasher {
				state.write(&self.0);
				state.finish();
			}
		}

		impl ::core::ops::Index<usize> for $name {
			type Output = u8;

			fn index(&self, index: usize) -> &u8 {
				&self.0[index]
			}
		}
		impl ::core::ops::IndexMut<usize> for $name {
			fn index_mut(&mut self, index: usize) -> &mut u8 {
				&mut self.0[index]
			}
		}
		impl ::core::ops::Index<::core::ops::Range<usize>> for $name {
			type Output = [u8];

			fn index(&self, index: ::core::ops::Range<usize>) -> &[u8] {
				&self.0[index]
			}
		}
		impl ::core::ops::IndexMut<::core::ops::Range<usize>> for $name {
			fn index_mut(&mut self, index: ::core::ops::Range<usize>) -> &mut [u8] {
				&mut self.0[index]
			}
		}
		impl ::core::ops::Index<::core::ops::RangeFull> for $name {
			type Output = [u8];

			fn index(&self, _index: ::core::ops::RangeFull) -> &[u8] {
				&self.0
			}
		}
		impl ::core::ops::IndexMut<::core::ops::RangeFull> for $name {
			fn index_mut(&mut self, _index: ::core::ops::RangeFull) -> &mut [u8] {
				&mut self.0
			}
		}

		/// `BitOr` on references
		impl<'a> ::core::ops::BitOr for &'a $name {
			type Output = $name;

			fn bitor(self, rhs: Self) -> Self::Output {
				let mut ret: $name = $name::default();
				for i in 0..$n_bytes {
					ret.0[i] = self.0[i] | rhs.0[i];
				}
				ret
			}
		}

		/// Moving `BitOr`
		impl ::core::ops::BitOr for $name {
			type Output = $name;

			fn bitor(self, rhs: Self) -> Self::Output {
				&self | &rhs
			}
		}

		/// `BitAnd` on references
		impl <'a> ::core::ops::BitAnd for &'a $name {
			type Output = $name;

			fn bitand(self, rhs: Self) -> Self::Output {
				let mut ret: $name = $name::default();
				for i in 0..$n_bytes {
					ret.0[i] = self.0[i] & rhs.0[i];
				}
				ret
			}
		}

		/// Moving `BitAnd`
		impl ::core::ops::BitAnd for $name {
			type Output = $name;

			fn bitand(self, rhs: Self) -> Self::Output {
				&self & &rhs
			}
		}

		/// `BitXor` on references
		impl <'a> ::core::ops::BitXor for &'a $name {
			type Output = $name;

			fn bitxor(self, rhs: Self) -> Self::Output {
				let mut ret: $name = $name::default();
				for i in 0..$n_bytes {
					ret.0[i] = self.0[i] ^ rhs.0[i];
				}
				ret
			}
		}

		/// Moving `BitXor`
		impl ::core::ops::BitXor for $name {
			type Output = $name;

			fn bitxor(self, rhs: Self) -> Self::Output {
				&self ^ &rhs
			}
		}

		impl Default for $name {
			fn default() -> Self { $name::new() }
		}

		impl From<u64> for $name {
			fn from(mut value: u64) -> $name {
				let mut ret = $name::new();
				for i in 0..8 {
					if i < $n_bytes {
						ret.0[$n_bytes - i - 1] = (value & 0xff) as u8;
						value >>= 8;
					}
				}
				ret
			}
		}

		impl<'a> From<&'a [u8]> for $name {
			fn from(s: &'a [u8]) -> $name {
				$name::from_slice(s)
			}
		}

		impl_std_for_hash!($name, $n_bytes);
		impl_heapsize_for_hash!($name);
		impl_libc_for_hash!($name, $n_bytes);
		impl_quickcheck_arbitrary_for_hash!($name, $n_bytes);
	}
}

/// Implements conversion to and from hash types of different sizes. Uses the
/// last bytes, e.g. `From<H256> for H160` uses bytes 12..32
/// CAUTION: make sure to call with correct sizes and the bigger type first or
/// bad things will happen!
#[macro_export]
macro_rules! impl_hash_conversions {
	($a: ident, $a_size: expr, $b: ident, $b_size: expr) => {
		impl From<$b> for $a {
			fn from(value: $b) -> $a {
				debug_assert!($a_size > $b_size && $a_size % 2 == 0 && $b_size %2 == 0);
				let mut ret = $a::new();
				ret.0[($a_size - $b_size)..$a_size].copy_from_slice(value.as_bytes());
				ret
			}
		}

		impl From<$a> for $b {
			fn from(value: $a) -> $b {
				debug_assert!($a_size > $b_size && $a_size % 2 == 0 && $b_size %2 == 0);
				let mut ret = $b::new();
				ret.0.copy_from_slice(&value[($a_size - $b_size)..$a_size]);
				ret
			}
		}

		impl<'a> From<&'a $b> for $a {
			fn from(value: &'a $b) -> $a {
				let mut ret = $a::new();
				ret.0[($a_size - $b_size)..$a_size].copy_from_slice(value.as_bytes());
				ret
			}
		}
	}
}

/// Implements conversion to and from a hash type and the equally sized unsigned int.
/// CAUTION: Bad things will happen if the two types are not of the same size!
#[cfg(feature="uint_conversions")]
#[macro_export]
macro_rules! impl_hash_uint_conversions {
	($hash: ident, $uint: ident) => {
		debug_assert_eq!(::core::mem::size_of::<$hash>(), ::core::mem::size_of::<$uint>(), "Can't convert between differently sized uint/hash.");
		impl From<$uint> for $hash {
			fn from(value: $uint) -> $hash {
				let mut ret = $hash::new();
				value.to_big_endian(&mut ret);
				ret
			}
		}

		impl<'a> From<&'a $uint> for $hash {
			fn from(value: &'a $uint) -> $hash {
				let mut ret: $hash = $hash::new();
				value.to_big_endian(&mut ret);
				ret
			}
		}

		impl From<$hash> for $uint {
			fn from(value: $hash) -> $uint {
				$uint::from(&value as &[u8])
			}
		}

		impl<'a> From<&'a $hash> for $uint {
			fn from(value: &'a $hash) -> $uint {
				$uint::from(value.as_ref() as &[u8])
			}
		}

	}
}

#[cfg(all(feature="heapsizeof", feature="libc", not(target_os = "unknown")))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_heapsize_for_hash {
	($name: ident) => {
		impl $crate::heapsize::HeapSizeOf for $name {
			fn heap_size_of_children(&self) -> usize {
				0
			}
		}
	}
}

#[cfg(any(not(feature="heapsizeof"), not(feature="libc"), target_os = "unknown"))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_heapsize_for_hash {
	($name: ident) => {}
}

#[cfg(feature="std")]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_std_for_hash {
	($from: ident, $size: tt) => {
		impl $from {
			/// Get a hex representation.
			#[deprecated(note="Use LowerHex or Debug formatting instead.")]
			pub fn hex(&self) -> String {
				format!("{:?}", self)
			}
		}

		impl $crate::rand::Rand for $from {
			fn rand<R: $crate::rand::Rng>(r: &mut R) -> Self {
				let mut hash = $from::new();
				r.fill_bytes(&mut hash.0);
				hash
			}
		}

		impl ::core::str::FromStr for $from {
			type Err = $crate::rustc_hex::FromHexError;

			fn from_str(s: &str) -> Result<$from, $crate::rustc_hex::FromHexError> {
				use $crate::rustc_hex::FromHex;
				let a : Vec<u8> = s.from_hex()?;
				if a.len() != $size {
					return Err($crate::rustc_hex::FromHexError::InvalidHexLength);
				}

				let mut ret = [0; $size];
				ret.copy_from_slice(&a);
				Ok($from(ret))
			}
		}

		impl From<&'static str> for $from {
			fn from(s: &'static str) -> $from {
				let s = $crate::clean_0x(s);
				if s.len() % 2 == 1 {
					("0".to_owned() + s).parse().unwrap()
				} else {
					s.parse().unwrap()
				}
			}
		}
	}
}


#[cfg(not(feature="std"))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_std_for_hash {
	($from: ident, $size: tt) => {}
}


#[cfg(feature="std")]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_std_for_hash_internals {
	($from: ident, $size: tt) => {
		/// Create a new, cryptographically random, instance.
		pub fn random() -> $from {
			let mut hash = $from::new();
			hash.randomize();
			hash
		}

		/// Assign self have a cryptographically random value.
		pub fn randomize(&mut self) {
			let mut rng = $crate::rand::OsRng::new().unwrap();
			*self = $crate::rand::Rand::rand(&mut rng);
		}
	}
}

#[cfg(not(feature="std"))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_std_for_hash_internals {
	($from: ident, $size: tt) => {}
}

#[cfg(all(feature="libc", not(target_os = "unknown")))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_libc_for_hash {
	($from: ident, $size: expr) => {
		impl PartialEq for $from {
			fn eq(&self, other: &Self) -> bool {
				unsafe { $crate::libc::memcmp(self.0.as_ptr() as *const $crate::libc::c_void, other.0.as_ptr() as *const $crate::libc::c_void, $size) == 0 }
			}
		}

		impl Ord for $from {
			fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
				let r = unsafe { $crate::libc::memcmp(self.0.as_ptr() as *const $crate::libc::c_void, other.0.as_ptr() as *const $crate::libc::c_void, $size) };
				if r < 0 { return ::core::cmp::Ordering::Less }
				if r > 0 { return ::core::cmp::Ordering::Greater }
				return ::core::cmp::Ordering::Equal;
			}
		}
	}
}

#[cfg(any(not(feature="libc"), target_os = "unknown"))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_libc_for_hash {
	($from: ident, $size: expr) => {
		impl PartialEq for $from {
			fn eq(&self, other: &Self) -> bool {
				&self.0[..] == &other.0[..]
			}
		}

		impl Ord for $from {
			fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
				self.0[..].cmp(&other.0[..])
			}
		}
	}
}

#[cfg(feature="impl_quickcheck_arbitrary")]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_quickcheck_arbitrary_for_hash {
	($name: ty, $n_bytes: tt) => {
		impl $crate::quickcheck::Arbitrary for $name {
			fn arbitrary<G: $crate::quickcheck::Gen>(g: &mut G) -> Self {
				let mut res = [0u8; $n_bytes];
				g.fill_bytes(&mut res[..$n_bytes]);
				res.as_ref().into()
			}
		}
	}
}

#[cfg(not(feature="impl_quickcheck_arbitrary"))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_quickcheck_arbitrary_for_hash {
	($name: ty, $n_bytes: tt) => {}
}

#[cfg(test)]
mod tests {
	construct_hash!{
		/// Unformatted hash type with 32 bits length.
		pub struct H32(4);
	}

	construct_hash!{
		/// Unformatted hash type with 64 bits length.
		pub struct H64(8);
	}

	construct_hash!{
		/// Unformatted hash type with 256 bits length.
		pub struct H128(16);
	}

	construct_hash!{
		/// Unformatted hash type with 160 bits length.
		///
		/// # Note
		///
		/// Mainly used for addresses in ethereum and solidity context.
		struct H160(20);
	}

	construct_hash!{
		/// Unformatted hash type with 256 bits length.
		pub struct H256(32);
	}

	impl_hash_conversions!(H256, 32, H160, 20);

	#[test]
	fn test_construct_hash() {
		assert_eq!(H128::default(), H128::new());
		assert_eq!(H128::new(), H128::zero());
		assert_eq!(H128::len(), 16);
	}

	#[cfg(feature="heapsizeof")]
	#[test]
	fn test_heapsizeof() {
		use heapsize::HeapSizeOf;
		let h = H128::zero();
		assert_eq!(h.heap_size_of_children(),0);
	}

	#[cfg(feature="std")]
	#[test]
	fn should_format_and_debug_correctly() {
		let test = |x: u64, hex: &'static str, display: &'static str| {
			let hash = H128::from(x);
			assert_eq!(format!("{}", hash), format!("0x{}", display));
			assert_eq!(format!("{:?}", hash), format!("0x{}", hex));
			assert_eq!(format!("{:x}", hash), hex);
			assert_eq!(format!("{:#x}", hash), format!("0x{}", hex));
		};

		test(0x1, "00000000000000000000000000000001", "0000…0001");
		test(0xf, "0000000000000000000000000000000f", "0000…000f");
		test(0x10, "00000000000000000000000000000010", "0000…0010");
		test(0xff, "000000000000000000000000000000ff", "0000…00ff");
		test(0x100, "00000000000000000000000000000100", "0000…0100");
		test(0xfff, "00000000000000000000000000000fff", "0000…0fff");
		test(0x1000, "00000000000000000000000000001000", "0000…1000");
	}

	#[test]
	fn hash_bitor() {
		let a = H64([1; 8]);
		let b = H64([2; 8]);
		let c = H64([3; 8]);

		// borrow
		assert_eq!(&a | &b, c);

		// move
		assert_eq!(a | b, c);
	}

	#[cfg(feature="std")]
	#[test]
	fn from_and_to_address() {
		let address: H160 = "ef2d6d194084c2de36e0dabfce45d046b37d1106".into();
		let h = H256::from(address.clone());
		let a = H160::from(h);
		assert_eq!(address, a);
	}

	#[cfg(feature="std")]
	#[test]
	fn from_u64() {
		use core::str::FromStr;

		assert_eq!(H128::from(0x1234567890abcdef), H128::from_str("00000000000000001234567890abcdef").unwrap());
		assert_eq!(H64::from(0x1234567890abcdef), H64::from_str("1234567890abcdef").unwrap());
		assert_eq!(H32::from(0x1234567890abcdef), H32::from_str("90abcdef").unwrap());
	}

	#[cfg(feature="std")]
	#[test]
	fn from_str() {
		assert_eq!(H64::from(0x1234567890abcdef), H64::from("0x1234567890abcdef"));
		assert_eq!(H64::from(0x1234567890abcdef), H64::from("1234567890abcdef"));
		assert_eq!(H64::from(0x234567890abcdef), H64::from("0x234567890abcdef"));
	}

	#[cfg(feature = "uint_conversions")]
	#[test]
	fn from_and_to_u256() {
		use uint::U256;

		impl_hash_uint_conversions!(H256, U256);

		let u: U256 = 0x123456789abcdef0u64.into();
		let h = H256::from(u);
		assert_eq!(H256::from(u), H256::from("000000000000000000000000000000000000000000000000123456789abcdef0"));
		let h_ref = H256::from(&u);
		assert_eq!(h, h_ref);
		let r_ref: U256 = From::from(&h);
		assert!(r_ref == u);
		let r: U256 = From::from(h);
		assert!(r == u)
	}

	#[cfg(feature="uint_conversions")]
	#[test]
	#[should_panic(expected = "Can't convert between differently sized uint/hash.")]
	fn converting_differently_sized_types_panics() {
		use uint::U512;

		impl_hash_uint_conversions!(H256, U512);
	}
}
