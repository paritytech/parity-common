// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Construct a fixed-size hash type.
///
/// # Examples
///
/// Create a public unformatted hash type with 32 bytes size.
///
/// ```
/// # #[macro_use] extern crate fixed_hash;
/// construct_hash!{ pub struct H256(32); }
/// # fn main() {
/// # 	assert_eq!(::std::mem::size_of::<H256>(), 32);
/// # }
/// ```
///
/// With additional attributes and doc comments.
///
/// ```
/// # #[macro_use] extern crate fixed_hash;
/// # extern crate serde;
/// # #[macro_use] extern crate serde_derive;
/// construct_hash!{
/// 	/// My unformatted 160 bytes sized hash type.
/// 	#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
/// 	pub struct H160(20);
/// }
/// # fn main() {
/// # 	assert_eq!(::std::mem::size_of::<H160>(), 20);
/// # }
/// ```
///
/// As always, visibility modifier is optional.
/// So for a private type go ahead with this.
///
/// ```
/// # #[macro_use] extern crate fixed_hash;
/// construct_hash!{ struct H512(64); }
/// # fn main() {
/// # 	assert_eq!(::std::mem::size_of::<H512>(), 64);
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! construct_hash {
	( $(#[$attr:meta])* $visibility:vis struct $name:ident ( $n_bytes:expr ); ) => {
		#[repr(C)]
		$(#[$attr])*
		$visibility struct $name (pub [u8; $n_bytes]);

		impl From<[u8; $n_bytes]> for $name {
			#[inline]
			fn from(bytes: [u8; $n_bytes]) -> Self {
				$name(bytes)
			}
		}

		impl From<$name> for [u8; $n_bytes] {
			#[inline]
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
			/// Returns a new fixed hash where all bits are set.
			#[inline]
			pub fn repeat_byte(byte: u8) -> $name {
				$name([byte; $n_bytes])
			}

			/// Returns a new zero-initialized fixed hash.
			#[inline]
			pub fn zero() -> $name {
				$name::repeat_byte(0u8)
			}

			/// Returns the size of this hash in bytes.
			#[inline]
			pub fn len_bytes() -> usize {
				$n_bytes
			}

			/// Extracts a byte slice containing the entire fixed hash.
			#[inline]
			pub fn as_bytes(&self) -> &[u8] {
				&self.0
			}

			/// Extracts a mutable byte slice containing the entire fixed hash.
			#[inline]
			pub fn as_bytes_mut(&mut self) -> &mut [u8] {
				&mut self.0
			}

			/// Returns a constant raw pointer to the value.
			#[inline]
			pub fn as_ptr(&self) -> *const u8 {
				self.as_bytes().as_ptr()
			}

			/// Returns a mutable raw pointer to the value.
			#[inline]
			pub fn as_mut_ptr(&mut self) -> *mut u8 {
				self.as_bytes_mut().as_mut_ptr()
			}

			/// Assign the bytes from the byte slice `src` to `self`.
			/// 
			/// # Panics
			/// 
			/// If the length of `src` and the number of bytes in `self` do not match.
			pub fn assign_from_slice(&mut self, src: &[u8]) {
				$crate::core::assert_eq!(src.len(), $n_bytes);
				self.as_bytes_mut().clone_from_slice(src);
			}

			/// Create a new fixed-hash from the given slice `src`.
			/// 
			/// # Panics
			/// 
			/// If the length of `src` and the number of bytes in `Self` do not match.
			pub fn from_bytes(src: &[u8]) -> Self {
				$crate::core::assert_eq!(src.len(), $n_bytes);
				let mut ret = Self::zero();
				ret.assign_from_slice(src);
				ret
			}

			/// Returns `true` if all bits set in `b` are also set in `self`.
			#[inline]
			pub fn contains(&self, b: &Self) -> bool {
				&(b & self) == b
			}

			/// Returns `true` if no bits are set.
			#[inline]
			pub fn is_zero(&self) -> bool {
				self.as_bytes().iter().all(|&byte| byte == 0u8)
			}

			/// Returns the lowest 8 bytes interpreted as a big-endian integer.
			pub fn low_u64(&self) -> u64 {
				let mut ret = 0u64;
				for i in 0..$crate::core::cmp::min($n_bytes, 8) {
					ret |= (self.0[$n_bytes - 1 - i] as u64) << (i * 8);
				}
				ret
			}
		}

		impl $crate::core::fmt::Debug for $name {
			fn fmt(&self, f: &mut $crate::core::fmt::Formatter) -> $crate::core::fmt::Result {
				$crate::core::write!(f, "{:#x}", self)
			}
		}

		impl $crate::core::fmt::Display for $name {
			fn fmt(&self, f: &mut $crate::core::fmt::Formatter) -> $crate::core::fmt::Result {
				$crate::core::write!(f, "0x")?;
				for i in &self.0[0..2] {
					$crate::core::write!(f, "{:02x}", i)?;
				}
				$crate::core::write!(f, "…")?;
				for i in &self.0[$n_bytes - 2..$n_bytes] {
					$crate::core::write!(f, "{:02x}", i)?;
				}
				Ok(())
			}
		}

		impl $crate::core::fmt::LowerHex for $name {
			fn fmt(&self, f: &mut $crate::core::fmt::Formatter) -> $crate::core::fmt::Result {
				if f.alternate() {
					$crate::core::write!(f, "0x")?;
				}
				for i in &self.0[..] {
					$crate::core::write!(f, "{:02x}", i)?;
				}
				Ok(())
			}
		}

		impl $crate::core::fmt::UpperHex for $name {
			fn fmt(&self, f: &mut $crate::core::fmt::Formatter) -> $crate::core::fmt::Result {
				if f.alternate() {
					$crate::core::write!(f, "0X")?;
				}
				for i in &self.0[..] {
					$crate::core::write!(f, "{:02X}", i)?;
				}
				Ok(())
			}
		}

		impl $crate::core::marker::Copy for $name {}

		#[cfg_attr(feature="dev", allow(expl_impl_clone_on_copy))]
		impl $crate::core::clone::Clone for $name {
			fn clone(&self) -> $name {
				let mut ret = $name::zero();
				ret.0.copy_from_slice(&self.0);
				ret
			}
		}

		impl $crate::core::cmp::Eq for $name {}

		impl $crate::core::cmp::PartialOrd for $name {
			fn partial_cmp(&self, other: &Self) -> Option<$crate::core::cmp::Ordering> {
				Some(self.cmp(other))
			}
		}

		impl $crate::core::hash::Hash for $name {
			fn hash<H>(&self, state: &mut H) where H: $crate::core::hash::Hasher {
				state.write(&self.0);
				state.finish();
			}
		}

		impl $crate::core::ops::Index<usize> for $name {
			type Output = u8;

			fn index(&self, index: usize) -> &u8 {
				&self.0[index]
			}
		}

		impl $crate::core::ops::IndexMut<usize> for $name {
			fn index_mut(&mut self, index: usize) -> &mut u8 {
				&mut self.0[index]
			}
		}

		impl $crate::core::ops::Index<$crate::core::ops::Range<usize>> for $name {
			type Output = [u8];

			fn index(&self, index: $crate::core::ops::Range<usize>) -> &[u8] {
				&self.0[index]
			}
		}

		impl $crate::core::ops::IndexMut<$crate::core::ops::Range<usize>> for $name {
			fn index_mut(&mut self, index: $crate::core::ops::Range<usize>) -> &mut [u8] {
				&mut self.0[index]
			}
		}

		impl $crate::core::ops::Index<$crate::core::ops::RangeFull> for $name {
			type Output = [u8];

			fn index(&self, _index: $crate::core::ops::RangeFull) -> &[u8] {
				&self.0
			}
		}

		impl $crate::core::ops::IndexMut<$crate::core::ops::RangeFull> for $name {
			fn index_mut(&mut self, _index: $crate::core::ops::RangeFull) -> &mut [u8] {
				&mut self.0
			}
		}

		impl $crate::core::default::Default for $name {
			fn default() -> Self { $name::zero() }
		}

		#[deprecated(
			since = "0.3.0",
			note = "missing clarity for endianess; try to use `From<[u8; 8]>` with byteorder crate instead"
		)]
		impl $crate::core::convert::From<u64> for $name {
			fn from(mut value: u64) -> $name {
				let mut ret = $name::zero();
				for i in 0..8 {
					if i < $n_bytes {
						ret.0[$n_bytes - i - 1] = (value & 0xff) as u8;
						value >>= 8;
					}
				}
				ret
			}
		}

		impl<'a> $crate::core::convert::From<&'a [u8]> for $name {
			fn from(s: &'a [u8]) -> $name {
				$name::from_bytes(s)
			}
		}

		impl_ops_for_hash!($name, BitOr, bitor, BitOrAssign, bitor_assign, |, |=);
		impl_ops_for_hash!($name, BitAnd, bitand, BitAndAssign, bitand_assign, &, &=);
		impl_ops_for_hash!($name, BitXor, bitxor, BitXorAssign, bitxor_assign, ^, ^=);

		impl_rand_for_hash!($name);
		impl_std_for_hash!($name, $n_bytes);
		impl_heapsize_for_hash!($name);
		impl_libc_for_hash!($name);
		impl_quickcheck_arbitrary_for_hash!($name, $n_bytes);
	}
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_ops_for_hash {
	(
		$impl_for:ident,
		$ops_trait_name:ident,
		$ops_fn_name:ident,
		$ops_assign_trait_name:ident,
		$ops_assign_fn_name:ident,
		$ops_tok:tt,
		$ops_assign_tok:tt
	) => {

		impl<'r> $crate::core::ops::$ops_assign_trait_name<&'r $impl_for> for $impl_for {
			fn $ops_assign_fn_name(&mut self, rhs: &'r $impl_for) {
				for (lhs, rhs) in self.as_bytes_mut().iter_mut().zip(rhs.as_bytes()) {
					*lhs $ops_assign_tok rhs;
				}
			}
		}

		impl $crate::core::ops::$ops_assign_trait_name<$impl_for> for $impl_for {
			fn $ops_assign_fn_name(&mut self, rhs: $impl_for) {
				*self $ops_assign_tok &rhs;
			}
		}

		impl<'l, 'r> $crate::core::ops::$ops_trait_name<&'r $impl_for> for &'l $impl_for {
			type Output = $impl_for;

			fn $ops_fn_name(self, rhs: &'r $impl_for) -> Self::Output {
				let mut ret = self.clone();
				ret $ops_assign_tok rhs;
				ret
			}
		}

		impl $crate::core::ops::$ops_trait_name<$impl_for> for $impl_for {
			type Output = $impl_for;

			fn $ops_fn_name(self, rhs: Self) -> Self::Output {
				&self $ops_tok &rhs
			}
		}

	};
}

/// Implements lossy conversions between the given types.
///
/// # Note
///
/// - Both types must be of different sizes.
/// - Type `large_ty` must have a larger memory footprint compared to `small_ty`.
///
/// # Panics
///
/// Both `From` implementations will panic if sizes of the given types
/// do not meet the requirements stated above.
///
/// # Example
///
/// ```
/// #[macro_use] extern crate fixed_hash;
/// #[macro_use] extern crate static_assertions;
/// construct_hash!{ struct H160(20); }
/// construct_hash!{ struct H256(32); }
/// impl_hash_conversions!(H256, H160);
/// // now use it!
/// # fn main() {
/// assert_eq!(H256::from(H160::zero()), H256::zero());
/// assert_eq!(H160::from(H256::zero()), H160::zero());
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! impl_hash_conversions {
	($large_ty:ident, $small_ty:ident) => {
		static_assertions::const_assert!(VALID_SIZES;
			$crate::core::mem::size_of::<$small_ty>() < $crate::core::mem::size_of::<$large_ty>()
		);

		impl From<$small_ty> for $large_ty {
			fn from(value: $small_ty) -> $large_ty {
				let large_ty_size = $large_ty::len_bytes();
				let small_ty_size = $small_ty::len_bytes();

				$crate::core::debug_assert!(
					large_ty_size > small_ty_size
						&& large_ty_size % 2 == 0
						&& small_ty_size % 2 == 0
				);

				let mut ret = $large_ty::zero();
				ret.as_bytes_mut()[(large_ty_size - small_ty_size)..large_ty_size]
					.copy_from_slice(value.as_bytes());
				ret
			}
		}

		impl From<$large_ty> for $small_ty {
			fn from(value: $large_ty) -> $small_ty {
				let large_ty_size = $large_ty::len_bytes();
				let small_ty_size = $small_ty::len_bytes();

				$crate::core::debug_assert!(
					large_ty_size > small_ty_size
						&& large_ty_size % 2 == 0
						&& small_ty_size % 2 == 0
				);

				let mut ret = $small_ty::zero();
				ret.as_bytes_mut().copy_from_slice(
					&value.as_bytes()[(large_ty_size - small_ty_size)..large_ty_size],
				);
				ret
			}
		}
	};
}

#[cfg(all(
	feature = "heapsizeof",
	feature = "libc",
	not(target_os = "unknown")
))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_heapsize_for_hash {
	($name: ident) => {
		impl $crate::heapsize::HeapSizeOf for $name {
			fn heap_size_of_children(&self) -> usize {
				0
			}
		}
	};
}

#[cfg(any(
	not(feature = "heapsizeof"),
	not(feature = "libc"),
	target_os = "unknown"
))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_heapsize_for_hash {
	($name: ident) => {};
}

#[cfg(feature = "std")]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_std_for_hash {
	($from: ident, $size: tt) => {
		impl $crate::rand::Rand for $from {
			fn rand<R: $crate::rand::Rng>(r: &mut R) -> Self {
				let mut hash = $from::zero();
				r.fill_bytes(&mut hash.0);
				hash
			}
		}

		impl $crate::core::str::FromStr for $from {
			type Err = $crate::rustc_hex::FromHexError;

			fn from_str(s: &str) -> Result<$from, $crate::rustc_hex::FromHexError> {
				use $crate::rustc_hex::FromHex;
				let a: Vec<u8> = s.from_hex()?;
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
				let s = if s.starts_with("0x") { &s[2..] } else { s };
				if s.len() % 2 == 1 {
					("0".to_owned() + s).parse().unwrap()
				} else {
					s.parse().unwrap()
				}
			}
		}
	};
}

#[cfg(not(feature = "std"))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_std_for_hash {
	($from: ident, $size: tt) => {};
}

#[cfg(feature = "std")]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_rand_for_hash {
	( $impl_for:ident ) => {
		impl $impl_for {
			/// Create a new, cryptographically random, instance.
			pub fn random() -> $impl_for {
				let mut hash = $impl_for::zero();
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
}

#[cfg(not(feature = "std"))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_rand_for_hash {
	($from: ident) => {};
}

#[cfg(all(feature = "libc", not(target_os = "unknown")))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_libc_for_hash {
	( $name: ident ) => {
		impl $crate::core::cmp::PartialEq for $name {
			fn eq(&self, other: &Self) -> bool {
				unsafe {
					$crate::libc::memcmp(
						self.as_bytes().as_ptr() as *const $crate::libc::c_void,
						other.as_bytes().as_ptr() as *const $crate::libc::c_void,
						Self::len_bytes(),
					) == 0
				}
			}
		}

		impl $crate::core::cmp::Ord for $name {
			fn cmp(&self, other: &Self) -> $crate::core::cmp::Ordering {
				let r = unsafe {
					$crate::libc::memcmp(
						self.as_bytes().as_ptr() as *const $crate::libc::c_void,
						other.as_bytes().as_ptr() as *const $crate::libc::c_void,
						Self::len_bytes(),
					)
				};
				if r < 0 {
					return $crate::core::cmp::Ordering::Less;
				}
				if r > 0 {
					return $crate::core::cmp::Ordering::Greater;
				}
				$crate::core::cmp::Ordering::Equal
			}
		}
	};
}

#[cfg(any(not(feature = "libc"), target_os = "unknown"))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_libc_for_hash {
	($from: ident, $size: expr) => {
		impl PartialEq for $from {
			fn eq(&self, other: &Self) -> bool {
				self.as_bytes() == other.as_bytes()
			}
		}

		impl Ord for $from {
			fn cmp(&self, other: &Self) -> $crate::core::cmp::Ordering {
				self.as_bytes().cmp(other.as_bytes())
			}
		}
	};
}

#[cfg(feature = "impl_quickcheck_arbitrary")]
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
	};
}

#[cfg(not(feature = "impl_quickcheck_arbitrary"))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_quickcheck_arbitrary_for_hash {
	($name: ty, $n_bytes: tt) => {};
}

#[cfg(test)]
mod tests {
	construct_hash!{
		/// Unformatted hash type with 32 bits length.
		struct H32(4);
	}

	construct_hash!{
		/// Unformatted hash type with 64 bits length.
		struct H64(8);
	}

	construct_hash!{
		/// Unformatted hash type with 128 bits length.
		struct H128(16);
	}

	construct_hash!{
		/// Unformatted hash type with 160 bits length.
		struct H160(20);
	}

	construct_hash!{
		/// Unformatted hash type with 256 bits length.
		struct H256(32);
	}

	impl_hash_conversions!(H256, H160);

	#[test]
	fn test_construct_hash() {
		assert_eq!(H128::default(), H128::zero());
		assert_eq!(H128::zero(), H128::zero());
		assert_eq!(H128::len_bytes(), 16);
	}

	#[cfg(feature = "heapsizeof")]
	#[test]
	fn test_heapsizeof() {
		use heapsize::HeapSizeOf;
		let h = H128::zero();
		assert_eq!(h.heap_size_of_children(), 0);
	}

	#[cfg(feature = "std")]
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

	#[cfg(feature = "std")]
	#[test]
	fn from_and_to_address() {
		let address: H160 = "ef2d6d194084c2de36e0dabfce45d046b37d1106".into();
		let h = H256::from(address.clone());
		let a = H160::from(h);
		assert_eq!(address, a);
	}

	#[cfg(feature = "std")]
	#[test]
	fn from_u64() {
		use core::str::FromStr;

		assert_eq!(
			H128::from(0x1234567890abcdef),
			H128::from_str("00000000000000001234567890abcdef").unwrap()
		);
		assert_eq!(
			H64::from(0x1234567890abcdef),
			H64::from_str("1234567890abcdef").unwrap()
		);
		assert_eq!(
			H32::from(0x1234567890abcdef),
			H32::from_str("90abcdef").unwrap()
		);
	}

	#[cfg(feature = "std")]
	#[test]
	fn from_str() {
		assert_eq!(
			H64::from(0x1234567890abcdef),
			H64::from("0x1234567890abcdef")
		);
		assert_eq!(H64::from(0x1234567890abcdef), H64::from("1234567890abcdef"));
		assert_eq!(H64::from(0x234567890abcdef), H64::from("0x234567890abcdef"));
	}
}
