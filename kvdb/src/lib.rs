// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Key-Value store abstraction with `RocksDB` backend.

#[macro_use]
extern crate log;

extern crate elastic_array;
extern crate parity_bytes as bytes;
extern crate fs_swap;
extern crate hashbrown;
extern crate parking_lot;
extern crate interleaved_ordered;
extern crate owning_ref;

use std::{io, mem, fs, convert::identity};
use elastic_array::{ElasticArray128, ElasticArray32};
use bytes::Bytes;
use hashbrown::HashMap;
use parking_lot::{Mutex, MutexGuard, RwLock};
use fs_swap::{swap, swap_nonatomic};
use interleaved_ordered::interleave_ordered;

mod iter;

pub use iter::IterationHandler;

/// Required length of prefixes.
pub const PREFIX_LEN: usize = 12;

/// Database value.
pub type DBValue = ElasticArray128<u8>;

/// Write transaction. Batches a sequence of put/delete operations for efficiency.
#[derive(Default, Clone, PartialEq)]
pub struct DBTransaction {
	/// Database operations.
	pub ops: Vec<DBOp>,
}

/// Database operation.
#[derive(Clone, PartialEq)]
pub enum DBOp {
	Insert {
		col: Option<u32>,
		key: ElasticArray32<u8>,
		value: DBValue,
	},
	Delete {
		col: Option<u32>,
		key: ElasticArray32<u8>,
	}
}

impl DBOp {
	/// Returns the key associated with this operation.
	pub fn key(&self) -> &[u8] {
		match *self {
			DBOp::Insert { ref key, .. } => key,
			DBOp::Delete { ref key, .. } => key,
		}
	}

	/// Returns the column associated with this operation.
	pub fn col(&self) -> Option<u32> {
		match *self {
			DBOp::Insert { col, .. } => col,
			DBOp::Delete { col, .. } => col,
		}
	}
}

impl DBTransaction {
	/// Create new transaction.
	pub fn new() -> DBTransaction {
		DBTransaction::with_capacity(256)
	}

	/// Create new transaction with capacity.
	pub fn with_capacity(cap: usize) -> DBTransaction {
		DBTransaction {
			ops: Vec::with_capacity(cap)
		}
	}

	/// Insert a key-value pair in the transaction. Any existing value will be overwritten upon write.
	pub fn put(&mut self, col: Option<u32>, key: &[u8], value: &[u8]) {
		let mut ekey = ElasticArray32::new();
		ekey.append_slice(key);
		self.ops.push(DBOp::Insert {
			col: col,
			key: ekey,
			value: DBValue::from_slice(value),
		});
	}

	/// Insert a key-value pair in the transaction. Any existing value will be overwritten upon write.
	pub fn put_vec(&mut self, col: Option<u32>, key: &[u8], value: Bytes) {
		let mut ekey = ElasticArray32::new();
		ekey.append_slice(key);
		self.ops.push(DBOp::Insert {
			col: col,
			key: ekey,
			value: DBValue::from_vec(value),
		});
	}

	/// Delete value by key.
	pub fn delete(&mut self, col: Option<u32>, key: &[u8]) {
		let mut ekey = ElasticArray32::new();
		ekey.append_slice(key);
		self.ops.push(DBOp::Delete {
			col: col,
			key: ekey,
		});
	}
}

/// Generic key-value database.
///
/// This makes a distinction between "buffered" and "flushed" values. Values which have been
/// written can always be read, but may be present in an in-memory buffer. Values which have
/// been flushed have been moved to backing storage, like a RocksDB instance. There are certain
/// operations which are only guaranteed to operate on flushed data and not buffered,
/// although implementations may differ in this regard.
///
/// The contents of an interior buffer may be explicitly flushed using the `flush` method.
///
/// The `KeyValueDB` also deals in "column families", which can be thought of as distinct
/// stores within a database. Keys written in one column family will not be accessible from
/// any other. The number of column families must be specified at initialization, with a
/// differing interface for each database. The `None` argument in place of a column index
/// is always supported.
///
/// The API laid out here, along with the `Sync` bound implies interior synchronization for
/// implementation.
pub trait KeyValueDB: Sync + Send {
	/// Helper to create a new transaction.
	fn transaction(&self) -> DBTransaction { DBTransaction::new() }

	/// Get a value by key.
	fn get(&self, col: Option<u32>, key: &[u8]) -> io::Result<Option<DBValue>>;

	/// Get a value by partial key. Only works for flushed data.
	fn get_by_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<Box<[u8]>>;

	/// Write a transaction of changes to the buffer.
	fn write_buffered(&self, transaction: DBTransaction);

	/// Write a transaction of changes to the backing store.
	fn write(&self, transaction: DBTransaction) -> io::Result<()> {
		self.write_buffered(transaction);
		self.flush()
	}

	/// Flush all buffered data.
	fn flush(&self) -> io::Result<()>;

	/// Iterate over flushed data for a given column.
	fn iter<'a>(&'a self, col: Option<u32>) -> Box<Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a>;

	/// Iterate over flushed data for a given column, starting from a given prefix.
	fn iter_from_prefix<'a>(&'a self, col: Option<u32>, prefix: &'a [u8])
		-> Box<Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a>;

	/// Attempt to replace this database with a new one located at the given path.
	fn restore(&self, new_db: &str) -> io::Result<()>;
}


/// Converts an error to an `io::Error`.
pub fn other_io_err<E>(e: E) -> io::Error where E: Into<Box<std::error::Error + Send + Sync>> {
	io::Error::new(io::ErrorKind::Other, e)
}

/// An abstraction over a concrete database write transaction implementation.
pub trait WriteTransaction {
	/// Insert a key-value pair in the transaction. Any existing value will be overwritten upon write.
	fn put(&mut self, col: usize, key: &[u8], value: &[u8]) -> io::Result<()>;
	/// Delete value by key.
	fn delete(&mut self, col: usize, key: &[u8]) -> io::Result<()>;
	/// Commit the transaction.
	fn commit(self: Box<Self>) -> io::Result<()>;
}

/// An abstraction over a concrete database read-only transaction implementation.
pub trait ReadTransaction {
	/// Get value by key.
	fn get(self: Box<Self>, col: usize, key: &[u8]) -> io::Result<Option<DBValue>>;
}

pub trait TransactionHandler {
	// TODO: how to avoid boxing?
	fn write_transaction<'a> (&'a self) -> Box<WriteTransaction + 'a>;
	fn read_transaction<'a>(&'a self) -> Box<ReadTransaction + 'a>;
}


enum KeyState {
	Insert(DBValue),
	Delete,
}

pub trait OpenHandler<DB>: Send + Sync {
	/// Database configuration type.
	type Config: NumColumns + Default + Clone + Send + Sync;

	/// Opens the database path. Creates if it does not exist.
	fn open(config: &Self::Config, path: &str) -> io::Result<DB>;
}

pub trait NumColumns {
	/// Number of non-default columns.
	fn num_columns(&self) -> usize;
}

/// Allows dropping and appending columns to the DB.
pub trait MigrationHandler<DB: OpenHandler<DB>>: NumColumns {
	/// Appends a new column to the database.
	fn add_column(&mut self, config: &<DB as OpenHandler<DB>>::Config) -> io::Result<()>;
	/// Drops the last column from the database.
	fn drop_column(&mut self) -> io::Result<()>;
}

pub struct DatabaseWithCache<DB>
where
	DB: OpenHandler<DB> + TransactionHandler,
{
	db: RwLock<Option<DB>>,
	config: <DB as OpenHandler<DB>>::Config,
	path: String,
	// Dirty values added with `write_buffered`. Cleaned on `flush`.
	overlay: RwLock<Vec<HashMap<ElasticArray32<u8>, KeyState>>>,
	// Values currently being flushed. Cleared when `flush` completes.
	flushing: RwLock<Vec<HashMap<ElasticArray32<u8>, KeyState>>>,
	// Prevents concurrent flushes.
	// Value indicates if a flush is in progress.
	flushing_lock: Mutex<bool>,
}

impl<DB> KeyValueDB for DatabaseWithCache<DB>
where
	DB: OpenHandler<DB> + TransactionHandler,
	for<'a> &'a DB: IterationHandler,
{
	fn get(&self, col: Option<u32>, key: &[u8]) -> io::Result<Option<DBValue>> {
		DatabaseWithCache::get(self, col, key)
	}

	fn get_by_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<Box<[u8]>> {
		DatabaseWithCache::get_by_prefix(self, col, prefix)
	}

	fn write_buffered(&self, transaction: DBTransaction) {
		DatabaseWithCache::write_buffered(self, transaction)
	}

	fn write(&self, transaction: DBTransaction) -> io::Result<()> {
		DatabaseWithCache::write(self, transaction)
	}

	fn flush(&self) -> io::Result<()> {
		DatabaseWithCache::flush(self)
	}

	fn restore(&self, new_db: &str) -> io::Result<()> {
		DatabaseWithCache::restore(self, new_db)
	}

	fn iter<'a>(&'a self, col: Option<u32>) -> Box<Iterator<Item=iter::KeyValuePair> + 'a> {
		let unboxed = DatabaseWithCache::iter(self, col);
		Box::new(unboxed)
	}

	fn iter_from_prefix<'a>(&'a self, col: Option<u32>, prefix: &'a [u8])
		-> Box<Iterator<Item=iter::KeyValuePair> + 'a>
	{
		let unboxed = DatabaseWithCache::iter_from_prefix(self, col, prefix);
		Box::new(unboxed)
	}
}

impl<DB> DatabaseWithCache<DB>
where
	DB: OpenHandler<DB> + TransactionHandler,
	for<'a> &'a DB: IterationHandler,
{
	/// Helper to create a new transaction.
	pub fn transaction(&self) -> DBTransaction { DBTransaction::new() }
	/// Commit transaction to database.
	pub fn write_buffered(&self, tr: DBTransaction) {
		let mut overlay = self.overlay.write();
		let ops = tr.ops;
		for op in ops {
			match op {
				DBOp::Insert { col, key, value } => {
					let c = Self::to_overlay_column(col);
					overlay[c].insert(key, KeyState::Insert(value));
				},
				DBOp::Delete { col, key } => {
					let c = Self::to_overlay_column(col);
					overlay[c].insert(key, KeyState::Delete);
				},
			}
		};
	}

	/// Commit transaction to database.
	pub fn write(&self, tr: DBTransaction) -> io::Result<()> {
		match *self.db.read() {
			Some(ref db) => {
				let mut txn = db.write_transaction();
				let ops = tr.ops;
				for op in ops {
					let c = Self::to_overlay_column(op.col());
					// remove any buffered operation for this key
					self.overlay.write()[c].remove(op.key());

					match op {
						DBOp::Insert { key, value, .. } => txn.put(c, &key, &value)?,
						DBOp::Delete { key, .. } => txn.delete(c, &key)?,
					};
				}

				txn.commit()
			},
			None => Err(other_io_err("Database is closed")),
		}
	}

	/// Get value by key.
	pub fn get(&self, col: Option<u32>, key: &[u8]) -> io::Result<Option<DBValue>> {
		match *self.db.read() {
			Some(ref db) => {
				let c = Self::to_overlay_column(col);
				let overlay = &self.overlay.read()[c];
				match overlay.get(key) {
					Some(&KeyState::Insert(ref value)) => Ok(Some(value.clone())),
					Some(&KeyState::Delete) => Ok(None),
					None => {
						let flushing = &self.flushing.read()[c];
						match flushing.get(key) {
							Some(&KeyState::Insert(ref value)) => Ok(Some(value.clone())),
							Some(&KeyState::Delete) => Ok(None),
							None => {
								let txn = db.read_transaction();
								txn.get(c, &key)
							},
						}
					},
				}
			},
			None => Ok(None),
		}
	}

	/// Get value by partial key. Prefix size should match configured prefix size. Only searches flushed values.
	pub fn get_by_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<Box<[u8]>> {
		match self.iter_from_prefix(col, prefix).next() {
			Some((k, v)) => if k.starts_with(prefix) { Some(v) } else { None },
			_ => None
		}
	}

	/// Restore the database from a copy at given path.
	pub fn restore(&self, new_db: &str) -> io::Result<()> {
		self.close();

		// swap is guaranteed to be atomic
		match swap(new_db, &self.path) {
			Ok(_) => {
				// ignore errors
				let _ = fs::remove_dir_all(new_db);
			},
			Err(err) => {
				debug!("DB atomic swap failed: {}", err);
				match swap_nonatomic(new_db, &self.path) {
					Ok(_) => {
						// ignore errors
						let _ = fs::remove_dir_all(new_db);
					},
					Err(err) => {
						warn!("Failed to swap DB directories: {:?}", err);
						return Err(io::Error::new(io::ErrorKind::Other, "DB restoration failed: could not swap DB directories"));
					}
				}
			}
		}

		// reopen the database and steal handles into self
		let db = Self::open(&self.config, &self.path)?;

		*self.db.write() = mem::replace(&mut *db.db.write(), None);
		*self.overlay.write() = mem::replace(&mut *db.overlay.write(), Vec::new());
		*self.flushing.write() = mem::replace(&mut *db.flushing.write(), Vec::new());

		Ok(())
	}
}



impl<DB> DatabaseWithCache<DB>
where
	DB: OpenHandler<DB> + TransactionHandler,
{
	/// Opens or creates the database from the specified config and path.
	pub fn open(config: &<DB as OpenHandler<DB>>::Config, path: &str) -> io::Result<Self> {
		let db = DB::open(config, path)?;
		let num_cols = config.num_columns();
		Ok(Self {
			db: RwLock::new(Some(db)),
			config: config.clone(),
			path: path.to_owned(),
			overlay: RwLock::new((0..(num_cols + 1)).map(|_| HashMap::new()).collect()),
			flushing: RwLock::new((0..(num_cols + 1)).map(|_| HashMap::new()).collect()),
			flushing_lock: Mutex::new(false),
		})
	}

	pub fn open_default(path: &str) -> io::Result<Self> {
		Self::open(&<DB as OpenHandler<DB>>::Config::default(), path)
	}

	fn to_overlay_column(col: Option<u32>) -> usize {
		col.map_or(0, |c| (c + 1) as usize)
	}

	/// Close the database
	fn close(&self) {
		*self.db.write() = None;
		self.overlay.write().clear();
		self.flushing.write().clear();
	}
}

impl<DB> NumColumns for DatabaseWithCache<DB>
where
	DB: OpenHandler<DB> + TransactionHandler + NumColumns,
{
	fn num_columns(&self) -> usize {
		self.db
			.read()
			.as_ref()
			.map(|db| db.num_columns())
			.unwrap_or(0)
	}
}

impl<DB> DatabaseWithCache<DB>
where
	DB: OpenHandler<DB> + TransactionHandler + MigrationHandler<DB>,
{
	/// Appends a new column to the database.
	pub fn add_column(&self) -> io::Result<()> {
		match *self.db.write() {
			Some(ref mut db) => {
				db.add_column(&self.config)?;
				// TODO: this was not present in the previous implementation
				self.overlay.write().push(Default::default());
				self.flushing.write().push(Default::default());
				Ok(())
			},
			None => Ok(()),
		}
	}

	/// Drops the last column from the database.
	pub fn drop_column(&self) -> io::Result<()> {
		match *self.db.write() {
			Some(ref mut db) => {
				db.drop_column()?;
				// TODO: this was not present in the previous implementation
				self.overlay.write().pop();
				self.flushing.write().pop();
				Ok(())
			},
			None => Ok(()),
		}
	}
}

impl<DB> DatabaseWithCache<DB>
where
	DB: OpenHandler<DB> + TransactionHandler,
	for<'a> &'a DB: IterationHandler,
{
	/// Get database iterator for flushed data.
	pub fn iter<'a>(&'a self, col: Option<u32>) -> impl Iterator<Item=iter::KeyValuePair> + 'a {
		let read_lock = self.db.read();
		let optional = if read_lock.is_some() {
			let c = Self::to_overlay_column(col);
			let overlay = &self.overlay.read()[c];
			let mut overlay_data = overlay.iter()
				.filter_map(|(k, v)| match *v {
					KeyState::Insert(ref value) =>
						Some((k.clone().into_vec().into_boxed_slice(), value.clone().into_vec().into_boxed_slice())),
					KeyState::Delete => None,
				}).collect::<Vec<_>>();
			overlay_data.sort();

			let guarded = iter::ReadGuardedIterator::new(read_lock, c);
			Some(interleave_ordered(overlay_data, guarded))
		} else {
			None
		};
		optional.into_iter().flat_map(identity)
	}

	/// Get database iterator from prefix for flushed data.
	pub fn iter_from_prefix<'a>(
		&'a self,
		col: Option<u32>,
		prefix: &[u8],
	) -> impl Iterator<Item=iter::KeyValuePair> + 'a {
		let read_lock = self.db.read();
		let c = Self::to_overlay_column(col);
		let optional = if read_lock.is_some() {
			let guarded = iter::ReadGuardedIterator::new_from_prefix(read_lock, c, prefix);
			Some(interleave_ordered(Vec::new(), guarded))
		} else {
			None
		};
		optional.into_iter().flat_map(identity)
	}
}

impl<DB> DatabaseWithCache<DB>
where
	DB: OpenHandler<DB> + TransactionHandler,
{
	/// Commit buffered changes to database. Must be called under `flush_lock`
	fn write_flushing_with_lock(&self, _lock: &mut MutexGuard<bool>) -> io::Result<()> {
		match *self.db.read() {
			Some(ref db) => {
				let mut txn = db.write_transaction();
				mem::swap(&mut *self.overlay.write(), &mut *self.flushing.write());
				{
					for (c, column) in self.flushing.read().iter().enumerate() {
						for (key, state) in column.iter() {
							match *state {
								KeyState::Delete => {
									txn.delete(c, &key)?;
								},
								KeyState::Insert(ref value) => {
									txn.put(c, &key, &value)?;
								},
							}
						}
					}
				}

				txn.commit()?;

				for column in self.flushing.write().iter_mut() {
					column.clear();
					column.shrink_to_fit();
				}

				Ok(())
			},
			None => Err(other_io_err("Database is closed"))
		}
	}

	/// Commit buffered changes to database.
	pub fn flush(&self) -> io::Result<()> {
		let mut lock = self.flushing_lock.lock();
		// If batch allocation fails the thread gets terminated and the lock is released.
		// The value inside the lock is used to detect that.
		if *lock {
			// This can only happen if another flushing thread is terminated unexpectedly.
			return Err(other_io_err("Database write failure. Running low on memory perhaps?"))
		}
		*lock = true;
		let result = self.write_flushing_with_lock(&mut lock);
		*lock = false;
		result
	}
}

impl<DB> Drop for DatabaseWithCache<DB>
where
	DB: OpenHandler<DB> + TransactionHandler,
{
	fn drop(&mut self) {
		if let Err(error) = self.flush() {
			warn!("database flush failed while closing: {}", error);
		}
	}
}
