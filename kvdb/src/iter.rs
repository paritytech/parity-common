use owning_ref::{OwningHandle, StableAddress};
use parking_lot::RwLockReadGuard;
use std::ops::{Deref, DerefMut};

pub type KeyValuePair = (Box<[u8]>, Box<[u8]>);


pub struct ReadGuardedIterator<'a, I, T> {
	inner: OwningHandle<
		UnsafeStableAddress<RwLockReadGuard<'a, Option<T>>>,
		DerefWrapper<Option<I>>,
	>,
}


// We can't implement `StableAddress` for a `RwLockReadGuard`
// directly due to orphan rules.
#[repr(transparent)]
struct UnsafeStableAddress<T>(T);

impl<T: Deref> Deref for UnsafeStableAddress<T> {
	type Target = T::Target;
	fn deref(&self) -> &Self::Target {
		self.0.deref()
	}
}

// RwLockReadGuard dereferences to a stable address; qed
unsafe impl<T: Deref> StableAddress for UnsafeStableAddress<T> {}


struct DerefWrapper<T>(T);

impl<T> Deref for DerefWrapper<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> DerefMut for DerefWrapper<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}


impl<'a, I: Iterator, T> Iterator for ReadGuardedIterator<'a, I, T> {
	type Item = I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.deref_mut().as_mut().and_then(|iter| iter.next())
	}
}

pub trait IterationHandler {
	type Iterator: Iterator<Item=KeyValuePair>;

	fn iter(&self, col: u32) -> Self::Iterator;
	fn iter_from_prefix(&self, col: u32, prefix: & [u8]) -> Self::Iterator;
}

impl<'a, T> ReadGuardedIterator<'a, <&'a T as IterationHandler>::Iterator, T>
where
	&'a T: IterationHandler,
{
	pub fn new(read_lock: RwLockReadGuard<'a, Option<T>>, col: u32) -> Self {
		Self {
			inner: OwningHandle::new_with_fn(UnsafeStableAddress(read_lock), move |rlock| {
				let rlock = unsafe { rlock.as_ref().expect("initialized as non-null; qed") };
				DerefWrapper(rlock.as_ref().map(|db| db.iter(col)))
			})
		}
	}

	pub fn new_from_prefix(read_lock: RwLockReadGuard<'a, Option<T>>, col: u32, prefix: &[u8]) -> Self {
		Self {
			inner: OwningHandle::new_with_fn(UnsafeStableAddress(read_lock), move |rlock| {
				let rlock = unsafe { rlock.as_ref().expect("initialized as non-null; qed") };
				DerefWrapper(rlock.as_ref().map(|db| db.iter_from_prefix(col, prefix)))
			})
		}
	}
}
