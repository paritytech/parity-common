// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

mod iter;

use std::{cmp, collections::HashMap, convert::identity, error, fs, io, mem, path::Path, result};

use parking_lot::{Mutex, MutexGuard, RwLock};
use rocksdb::{
	BlockBasedOptions, ColumnFamily, ColumnFamilyDescriptor, Error, Options, ReadOptions, WriteBatch, WriteOptions, DB,
};

use crate::iter::KeyValuePair;
use elastic_array::ElasticArray32;
use fs_swap::{swap, swap_nonatomic};
use interleaved_ordered::interleave_ordered;
use kvdb::{DBOp, DBTransaction, DBValue, KeyValueDB};
use log::{debug, warn};

#[cfg(target_os = "linux")]
use regex::Regex;
#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::process::Command;

fn other_io_err<E>(e: E) -> io::Error
where
	E: Into<Box<dyn error::Error + Send + Sync>>,
{
	io::Error::new(io::ErrorKind::Other, e)
}

// Used for memory budget.
type MiB = usize;

const KB: usize = 1024;
const MB: usize = 1024 * KB;

/// The default column memory budget in MiB.
pub const DB_DEFAULT_COLUMN_MEMORY_BUDGET_MB: MiB = 128;

/// The default memory budget in MiB.
pub const DB_DEFAULT_MEMORY_BUDGET_MB: MiB = 512;

enum KeyState {
	Insert(DBValue),
	Delete,
}

/// Compaction profile for the database settings
/// Note, that changing these parameters may trigger
/// the compaction process of RocksDB on startup.
/// https://github.com/facebook/rocksdb/wiki/Leveled-Compaction#level_compaction_dynamic_level_bytes-is-true
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct CompactionProfile {
	/// L0-L1 target file size
	/// The minimum size should be calculated in accordance with the
	/// number of levels and the expected size of the database.
	pub initial_file_size: u64,
	/// block size
	pub block_size: usize,
}

impl Default for CompactionProfile {
	/// Default profile suitable for most storage
	fn default() -> CompactionProfile {
		CompactionProfile::ssd()
	}
}

/// Given output of df command return Linux rotational flag file path.
#[cfg(target_os = "linux")]
pub fn rotational_from_df_output(df_out: Vec<u8>) -> Option<PathBuf> {
	use std::str;
	str::from_utf8(df_out.as_slice())
		.ok()
		// Get the drive name.
		.and_then(|df_str| {
			Regex::new(r"/dev/(sd[:alpha:]{1,2})")
				.ok()
				.and_then(|re| re.captures(df_str))
				.and_then(|captures| captures.get(1))
		})
		// Generate path e.g. /sys/block/sda/queue/rotational
		.map(|drive_path| {
			let mut p = PathBuf::from("/sys/block");
			p.push(drive_path.as_str());
			p.push("queue/rotational");
			p
		})
}

impl CompactionProfile {
	/// Attempt to determine the best profile automatically, only Linux for now.
	#[cfg(target_os = "linux")]
	pub fn auto(db_path: &Path) -> CompactionProfile {
		use std::io::Read;
		let hdd_check_file = db_path
			.to_str()
			.and_then(|path_str| Command::new("df").arg(path_str).output().ok())
			.and_then(|df_res| if df_res.status.success() { Some(df_res.stdout) } else { None })
			.and_then(rotational_from_df_output);
		// Read out the file and match compaction profile.
		if let Some(hdd_check) = hdd_check_file {
			if let Ok(mut file) = File::open(hdd_check.as_path()) {
				let mut buffer = [0; 1];
				if file.read_exact(&mut buffer).is_ok() {
					// 0 means not rotational.
					if buffer == [48] {
						return Self::ssd();
					}
					// 1 means rotational.
					if buffer == [49] {
						return Self::hdd();
					}
				}
			}
		}
		// Fallback if drive type was not determined.
		Self::default()
	}

	/// Just default for other platforms.
	#[cfg(not(target_os = "linux"))]
	pub fn auto(_db_path: &Path) -> CompactionProfile {
		Self::default()
	}

	/// Default profile suitable for SSD storage
	pub fn ssd() -> CompactionProfile {
		CompactionProfile { initial_file_size: 64 * MB as u64, block_size: 16 * KB }
	}

	/// Slow HDD compaction profile
	pub fn hdd() -> CompactionProfile {
		CompactionProfile { initial_file_size: 256 * MB as u64, block_size: 64 * KB }
	}
}

/// Database configuration
#[derive(Clone)]
pub struct DatabaseConfig {
	/// Max number of open files.
	pub max_open_files: i32,
	/// Memory budget (in MiB) used for setting block cache size and
	/// write buffer size for each column including the default one.
	/// If the memory budget of a column is not specified,
	/// `DB_DEFAULT_COLUMN_MEMORY_BUDGET_MB` is used for that column.
	pub memory_budget: HashMap<Option<u32>, MiB>,
	/// Compaction profile.
	pub compaction: CompactionProfile,
	/// Set number of columns.
	pub columns: Option<u32>,
	/// Specify the maximum number of info/debug log files to be kept.
	pub keep_log_file_num: i32,
}

impl DatabaseConfig {
	/// Create new `DatabaseConfig` with default parameters and specified set of columns.
	/// Note that cache sizes must be explicitly set.
	pub fn with_columns(columns: Option<u32>) -> Self {
		Self { columns, ..Default::default() }
	}

	/// Returns the total memory budget in bytes.
	pub fn memory_budget(&self) -> MiB {
		match self.columns {
			None => self.memory_budget.get(&None).unwrap_or(&DB_DEFAULT_MEMORY_BUDGET_MB) * MB,
			Some(columns) => (0..columns)
				.map(|i| self.memory_budget.get(&Some(i)).unwrap_or(&DB_DEFAULT_COLUMN_MEMORY_BUDGET_MB) * MB)
				.sum(),
		}
	}

	/// Returns the memory budget of the specified column in bytes.
	fn memory_budget_for_col(&self, col: u32) -> MiB {
		self.memory_budget.get(&Some(col)).unwrap_or(&DB_DEFAULT_COLUMN_MEMORY_BUDGET_MB) * MB
	}

	// Get column family configuration with the given block based options.
	fn column_config(&self, block_opts: &BlockBasedOptions, col: u32) -> Options {
		let column_mem_budget = self.memory_budget_for_col(col);
		let mut opts = Options::default();

		opts.set_level_compaction_dynamic_level_bytes(true);
		opts.set_block_based_table_factory(block_opts);
		opts.optimize_level_style_compaction(column_mem_budget);
		opts.set_target_file_size_base(self.compaction.initial_file_size);
		opts.set_compression_per_level(&[]);

		opts
	}
}

impl Default for DatabaseConfig {
	fn default() -> DatabaseConfig {
		DatabaseConfig {
			max_open_files: 512,
			memory_budget: HashMap::new(),
			compaction: CompactionProfile::default(),
			columns: None,
			keep_log_file_num: 1,
		}
	}
}

struct DBAndColumns {
	db: DB,
	column_names: Vec<String>,
}

impl DBAndColumns {
	fn cf(&self, i: usize) -> &ColumnFamily {
		self.db.cf_handle(&self.column_names[i]).expect("the specified column name is correct; qed")
	}
}

/// Key-Value database.
pub struct Database {
	db: RwLock<Option<DBAndColumns>>,
	config: DatabaseConfig,
	path: String,
	write_opts: WriteOptions,
	read_opts: ReadOptions,
	block_opts: BlockBasedOptions,
	// Dirty values added with `write_buffered`. Cleaned on `flush`.
	overlay: RwLock<Vec<HashMap<ElasticArray32<u8>, KeyState>>>,
	// Values currently being flushed. Cleared when `flush` completes.
	flushing: RwLock<Vec<HashMap<ElasticArray32<u8>, KeyState>>>,
	// Prevents concurrent flushes.
	// Value indicates if a flush is in progress.
	flushing_lock: Mutex<bool>,
}

#[inline]
fn check_for_corruption<T, P: AsRef<Path>>(path: P, res: result::Result<T, Error>) -> io::Result<T> {
	if let Err(ref s) = res {
		if is_corrupted(s) {
			warn!("DB corrupted: {}. Repair will be triggered on next restart", s);
			let _ = fs::File::create(path.as_ref().join(Database::CORRUPTION_FILE_NAME));
		}
	}

	res.map_err(other_io_err)
}

fn is_corrupted(err: &Error) -> bool {
	err.as_ref().starts_with("Corruption:")
		|| err.as_ref().starts_with("Invalid argument: You have to open all column families")
}

/// Generate the options for RocksDB, based on the given `DatabaseConfig`.
fn generate_options(config: &DatabaseConfig) -> Options {
	let mut opts = Options::default();
	let columns = config.columns.unwrap_or(0);

	if columns == 0 {
		let budget = config.memory_budget() / 2;
		opts.set_db_write_buffer_size(budget);
		// from https://github.com/facebook/rocksdb/wiki/Memory-usage-in-RocksDB#memtable
		// Memtable size is controlled by the option `write_buffer_size`.
		// If you increase your memtable size, be sure to also increase your L1 size!
		// L1 size is controlled by the option `max_bytes_for_level_base`.
		opts.set_max_bytes_for_level_base(budget as u64);
	}

	opts.set_use_fsync(false);
	opts.create_if_missing(true);
	opts.set_max_open_files(config.max_open_files);
	opts.set_bytes_per_sync(1 * MB as u64);
	opts.set_keep_log_file_num(1);
	opts.increase_parallelism(cmp::max(1, num_cpus::get() as i32 / 2));

	opts
}

/// Generate the block based options for RocksDB, based on the given `DatabaseConfig`.
fn generate_block_based_options(config: &DatabaseConfig) -> BlockBasedOptions {
	let mut block_opts = BlockBasedOptions::default();
	block_opts.set_block_size(config.compaction.block_size);
	// Set cache size as recommended by
	// https://github.com/facebook/rocksdb/wiki/Setup-Options-and-Basic-Tuning#block-cache-size
	let cache_size = config.memory_budget() / 3;
	block_opts.set_lru_cache(cache_size);
	// "index and filter blocks will be stored in block cache, together with all other data blocks."
	// See: https://github.com/facebook/rocksdb/wiki/Memory-usage-in-RocksDB#indexes-and-filter-blocks
	block_opts.set_cache_index_and_filter_blocks(true);
	// Don't evict L0 filter/index blocks from the cache
	block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
	block_opts.set_bloom_filter(10, true);

	block_opts
}

impl Database {
	const CORRUPTION_FILE_NAME: &'static str = "CORRUPTED";

	/// Open database with default settings.
	pub fn open_default(path: &str) -> io::Result<Database> {
		Database::open(&DatabaseConfig::default(), path)
	}

	/// Open database file. Creates if it does not exist.
	pub fn open(config: &DatabaseConfig, path: &str) -> io::Result<Database> {
		let opts = generate_options(config);
		let block_opts = generate_block_based_options(config);
		let columns = config.columns.unwrap_or(0);

		if config.columns.is_some() && config.memory_budget.contains_key(&None) {
			warn!("Memory budget for the default column (None) is ignored if columns.is_some()");
		}

		// attempt database repair if it has been previously marked as corrupted
		let db_corrupted = Path::new(path).join(Database::CORRUPTION_FILE_NAME);
		if db_corrupted.exists() {
			warn!("DB has been previously marked as corrupted, attempting repair");
			DB::repair(&opts, path).map_err(other_io_err)?;
			fs::remove_file(db_corrupted)?;
		}

		let column_names: Vec<_> = (0..columns).map(|c| format!("col{}", c)).collect();

		let write_opts = WriteOptions::default();
		let mut read_opts = ReadOptions::default();
		read_opts.set_verify_checksums(false);

		let db = if config.columns.is_some() {
			let cf_descriptors: Vec<_> = (0..columns)
				.map(|i| ColumnFamilyDescriptor::new(&column_names[i as usize], config.column_config(&block_opts, i)))
				.collect();

			match DB::open_cf_descriptors(&opts, path, cf_descriptors) {
				Err(_) => {
					// retry and create CFs
					match DB::open_cf(&opts, path, &[] as &[&str]) {
						Ok(mut db) => {
							for (i, name) in column_names.iter().enumerate() {
								let _ = db
									.create_cf(name, &config.column_config(&block_opts, i as u32))
									.map_err(other_io_err)?;
							}
							Ok(db)
						}
						err => err,
					}
				}
				ok => ok,
			}
		} else {
			DB::open(&opts, path)
		};

		let db = match db {
			Ok(db) => db,
			Err(ref s) if is_corrupted(s) => {
				warn!("DB corrupted: {}, attempting repair", s);
				DB::repair(&opts, path).map_err(other_io_err)?;

				if config.columns.is_some() {
					let cf_descriptors: Vec<_> = (0..columns)
						.map(|i| {
							ColumnFamilyDescriptor::new(&column_names[i as usize], config.column_config(&block_opts, i))
						})
						.collect();

					DB::open_cf_descriptors(&opts, path, cf_descriptors).map_err(other_io_err)?
				} else {
					DB::open(&opts, path).map_err(other_io_err)?
				}
			}
			Err(s) => return Err(other_io_err(s)),
		};
		Ok(Database {
			db: RwLock::new(Some(DBAndColumns { db, column_names })),
			config: config.clone(),
			overlay: RwLock::new((0..=columns).map(|_| HashMap::new()).collect()),
			flushing: RwLock::new((0..=columns).map(|_| HashMap::new()).collect()),
			flushing_lock: Mutex::new(false),
			path: path.to_owned(),
			read_opts,
			write_opts,
			block_opts,
		})
	}

	/// Helper to create new transaction for this database.
	pub fn transaction(&self) -> DBTransaction {
		DBTransaction::new()
	}

	fn to_overlay_column(col: Option<u32>) -> usize {
		col.map_or(0, |c| (c + 1) as usize)
	}

	/// Commit transaction to database.
	pub fn write_buffered(&self, tr: DBTransaction) {
		let mut overlay = self.overlay.write();
		let ops = tr.ops;
		for op in ops {
			match op {
				DBOp::Insert { col, key, value } => {
					let c = Self::to_overlay_column(col);
					overlay[c].insert(key, KeyState::Insert(value));
				}
				DBOp::Delete { col, key } => {
					let c = Self::to_overlay_column(col);
					overlay[c].insert(key, KeyState::Delete);
				}
			}
		}
	}

	/// Commit buffered changes to database. Must be called under `flush_lock`
	fn write_flushing_with_lock(&self, _lock: &mut MutexGuard<'_, bool>) -> io::Result<()> {
		match *self.db.read() {
			Some(ref cfs) => {
				let mut batch = WriteBatch::default();
				mem::swap(&mut *self.overlay.write(), &mut *self.flushing.write());
				{
					for (c, column) in self.flushing.read().iter().enumerate() {
						for (key, state) in column.iter() {
							match *state {
								KeyState::Delete => {
									if c > 0 {
										let cf = cfs.cf(c - 1);
										batch.delete_cf(cf, key).map_err(other_io_err)?;
									} else {
										batch.delete(key).map_err(other_io_err)?;
									}
								}
								KeyState::Insert(ref value) => {
									if c > 0 {
										let cf = cfs.cf(c - 1);
										batch.put_cf(cf, key, value).map_err(other_io_err)?;
									} else {
										batch.put(key, value).map_err(other_io_err)?;
									}
								}
							}
						}
					}
				}

				check_for_corruption(&self.path, cfs.db.write_opt(batch, &self.write_opts))?;

				for column in self.flushing.write().iter_mut() {
					column.clear();
					column.shrink_to_fit();
				}
				Ok(())
			}
			None => Err(other_io_err("Database is closed")),
		}
	}

	/// Commit buffered changes to database.
	pub fn flush(&self) -> io::Result<()> {
		let mut lock = self.flushing_lock.lock();
		// If RocksDB batch allocation fails the thread gets terminated and the lock is released.
		// The value inside the lock is used to detect that.
		if *lock {
			// This can only happen if another flushing thread is terminated unexpectedly.
			return Err(other_io_err("Database write failure. Running low on memory perhaps?"));
		}
		*lock = true;
		let result = self.write_flushing_with_lock(&mut lock);
		*lock = false;
		result
	}

	/// Commit transaction to database.
	pub fn write(&self, tr: DBTransaction) -> io::Result<()> {
		match *self.db.read() {
			Some(ref cfs) => {
				let mut batch = WriteBatch::default();
				let ops = tr.ops;
				for op in ops {
					// remove any buffered operation for this key
					self.overlay.write()[Self::to_overlay_column(op.col())].remove(op.key());

					match op {
						DBOp::Insert { col, key, value } => match col {
							None => batch.put(&key, &value).map_err(other_io_err)?,
							Some(c) => batch.put_cf(cfs.cf(c as usize), &key, &value).map_err(other_io_err)?,
						},
						DBOp::Delete { col, key } => match col {
							None => batch.delete(&key).map_err(other_io_err)?,
							Some(c) => batch.delete_cf(cfs.cf(c as usize), &key).map_err(other_io_err)?,
						},
					}
				}

				check_for_corruption(&self.path, cfs.db.write_opt(batch, &self.write_opts))
			}
			None => Err(other_io_err("Database is closed")),
		}
	}

	/// Get value by key.
	pub fn get(&self, col: Option<u32>, key: &[u8]) -> io::Result<Option<DBValue>> {
		match *self.db.read() {
			Some(ref cfs) => {
				let overlay = &self.overlay.read()[Self::to_overlay_column(col)];
				match overlay.get(key) {
					Some(&KeyState::Insert(ref value)) => Ok(Some(value.clone())),
					Some(&KeyState::Delete) => Ok(None),
					None => {
						let flushing = &self.flushing.read()[Self::to_overlay_column(col)];
						match flushing.get(key) {
							Some(&KeyState::Insert(ref value)) => Ok(Some(value.clone())),
							Some(&KeyState::Delete) => Ok(None),
							None => col
								.map_or_else(
									|| {
										cfs.db
											.get_pinned_opt(key, &self.read_opts)
											.map(|r| r.map(|v| DBValue::from_slice(&v)))
									},
									|c| {
										cfs.db
											.get_pinned_cf_opt(cfs.cf(c as usize), key, &self.read_opts)
											.map(|r| r.map(|v| DBValue::from_slice(&v)))
									},
								)
								.map_err(other_io_err),
						}
					}
				}
			}
			None => Ok(None),
		}
	}

	/// Get value by partial key. Prefix size should match configured prefix size. Only searches flushed values.
	// TODO: support prefix seek for unflushed data
	pub fn get_by_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<Box<[u8]>> {
		self.iter_from_prefix(col, prefix).next().map(|(_, v)| v)
	}

	/// Get database iterator for flushed data.
	/// Will hold a lock until the iterator is dropped
	/// preventing the database from being closed.
	pub fn iter<'a>(&'a self, col: Option<u32>) -> impl Iterator<Item = KeyValuePair> + 'a {
		let read_lock = self.db.read();
		let optional = if read_lock.is_some() {
			let c = Self::to_overlay_column(col);
			let overlay_data = {
				let overlay = &self.overlay.read()[c];
				let mut overlay_data = overlay
					.iter()
					.filter_map(|(k, v)| match *v {
						KeyState::Insert(ref value) => {
							Some((k.clone().into_vec().into_boxed_slice(), value.clone().into_vec().into_boxed_slice()))
						}
						KeyState::Delete => None,
					})
					.collect::<Vec<_>>();
				overlay_data.sort();
				overlay_data
			};

			let guarded = iter::ReadGuardedIterator::new(read_lock, col);
			Some(interleave_ordered(overlay_data, guarded))
		} else {
			None
		};
		optional.into_iter().flat_map(identity)
	}
	/// Get database iterator from prefix for flushed data.
	/// Will hold a lock until the iterator is dropped
	/// preventing the database from being closed.
	fn iter_from_prefix<'a>(
		&'a self,
		col: Option<u32>,
		prefix: &'a [u8],
	) -> impl Iterator<Item = iter::KeyValuePair> + 'a {
		let read_lock = self.db.read();
		let optional = if read_lock.is_some() {
			let guarded = iter::ReadGuardedIterator::new_from_prefix(read_lock, col, prefix);
			Some(interleave_ordered(Vec::new(), guarded))
		} else {
			None
		};
		// workaround for https://github.com/facebook/rocksdb/issues/2343
		optional.into_iter().flat_map(identity).filter(move |(k, _)| k.starts_with(prefix))
	}

	/// Close the database
	fn close(&self) {
		*self.db.write() = None;
		self.overlay.write().clear();
		self.flushing.write().clear();
	}

	/// Restore the database from a copy at given path.
	pub fn restore(&self, new_db: &str) -> io::Result<()> {
		self.close();

		// swap is guaranteed to be atomic
		match swap(new_db, &self.path) {
			Ok(_) => {
				// ignore errors
				let _ = fs::remove_dir_all(new_db);
			}
			Err(err) => {
				debug!("DB atomic swap failed: {}", err);
				match swap_nonatomic(new_db, &self.path) {
					Ok(_) => {
						// ignore errors
						let _ = fs::remove_dir_all(new_db);
					}
					Err(err) => {
						warn!("Failed to swap DB directories: {:?}", err);
						return Err(io::Error::new(
							io::ErrorKind::Other,
							"DB restoration failed: could not swap DB directories",
						));
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

	/// The number of non-default column families.
	pub fn num_columns(&self) -> u32 {
		self.db
			.read()
			.as_ref()
			.and_then(|db| if db.column_names.is_empty() { None } else { Some(db.column_names.len()) })
			.map(|n| n as u32)
			.unwrap_or(0)
	}

	/// Remove the last column family in the database. The deletion is definitive.
	pub fn remove_last_column(&self) -> io::Result<()> {
		match *self.db.write() {
			Some(DBAndColumns { ref mut db, ref mut column_names }) => {
				if let Some(name) = column_names.pop() {
					db.drop_cf(&name).map_err(other_io_err)?;
				}
				Ok(())
			}
			None => Ok(()),
		}
	}

	/// Add a new column family to the DB.
	pub fn add_column(&self) -> io::Result<()> {
		match *self.db.write() {
			Some(DBAndColumns { ref mut db, ref mut column_names }) => {
				let col = column_names.len() as u32;
				let name = format!("col{}", col);
				let col_config = self.config.column_config(&self.block_opts, col as u32);
				let _ = db.create_cf(&name, &col_config).map_err(other_io_err)?;
				column_names.push(name);
				Ok(())
			}
			None => Ok(()),
		}
	}
}

// duplicate declaration of methods here to avoid trait import in certain existing cases
// at time of addition.
impl KeyValueDB for Database {
	fn get(&self, col: Option<u32>, key: &[u8]) -> io::Result<Option<DBValue>> {
		Database::get(self, col, key)
	}

	fn get_by_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<Box<[u8]>> {
		Database::get_by_prefix(self, col, prefix)
	}

	fn write_buffered(&self, transaction: DBTransaction) {
		Database::write_buffered(self, transaction)
	}

	fn write(&self, transaction: DBTransaction) -> io::Result<()> {
		Database::write(self, transaction)
	}

	fn flush(&self) -> io::Result<()> {
		Database::flush(self)
	}

	fn iter<'a>(&'a self, col: Option<u32>) -> Box<dyn Iterator<Item = KeyValuePair> + 'a> {
		let unboxed = Database::iter(self, col);
		Box::new(unboxed.into_iter())
	}

	fn iter_from_prefix<'a>(
		&'a self,
		col: Option<u32>,
		prefix: &'a [u8],
	) -> Box<dyn Iterator<Item = KeyValuePair> + 'a> {
		let unboxed = Database::iter_from_prefix(self, col, prefix);
		Box::new(unboxed.into_iter())
	}

	fn restore(&self, new_db: &str) -> io::Result<()> {
		Database::restore(self, new_db)
	}
}

impl Drop for Database {
	fn drop(&mut self) {
		// write all buffered changes if we can.
		let _ = self.flush();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use ethereum_types::H256;
	use std::io::Read;
	use std::str::FromStr;
	use tempdir::TempDir;

	fn test_db(config: &DatabaseConfig) {
		let tempdir = TempDir::new("").unwrap();
		let db = Database::open(config, tempdir.path().to_str().unwrap()).unwrap();
		let key1 = H256::from_str("02c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc").unwrap();
		let key2 = H256::from_str("03c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc").unwrap();
		let key3 = H256::from_str("04c00000000b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc").unwrap();
		let key4 = H256::from_str("04c01111110b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc").unwrap();
		let key5 = H256::from_str("04c02222220b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc").unwrap();

		let mut batch = db.transaction();
		batch.put(None, key1.as_bytes(), b"cat");
		batch.put(None, key2.as_bytes(), b"dog");
		batch.put(None, key3.as_bytes(), b"caterpillar");
		batch.put(None, key4.as_bytes(), b"beef");
		batch.put(None, key5.as_bytes(), b"fish");
		db.write(batch).unwrap();

		assert_eq!(&*db.get(None, key1.as_bytes()).unwrap().unwrap(), b"cat");

		let contents: Vec<_> = db.iter(None).into_iter().collect();
		assert_eq!(contents.len(), 5);
		assert_eq!(&*contents[0].0, key1.as_bytes());
		assert_eq!(&*contents[0].1, b"cat");
		assert_eq!(&*contents[1].0, key2.as_bytes());
		assert_eq!(&*contents[1].1, b"dog");

		let mut prefix_iter = db.iter_from_prefix(None, &[0x04, 0xc0]);
		assert_eq!(*prefix_iter.next().unwrap().1, b"caterpillar"[..]);
		assert_eq!(*prefix_iter.next().unwrap().1, b"beef"[..]);
		assert_eq!(*prefix_iter.next().unwrap().1, b"fish"[..]);

		let mut batch = db.transaction();
		batch.delete(None, key1.as_bytes());
		db.write(batch).unwrap();

		assert!(db.get(None, key1.as_bytes()).unwrap().is_none());

		let mut batch = db.transaction();
		batch.put(None, key1.as_bytes(), b"cat");
		db.write(batch).unwrap();

		let mut transaction = db.transaction();
		transaction.put(None, key3.as_bytes(), b"elephant");
		transaction.delete(None, key1.as_bytes());
		db.write(transaction).unwrap();
		assert!(db.get(None, key1.as_bytes()).unwrap().is_none());
		assert_eq!(&*db.get(None, key3.as_bytes()).unwrap().unwrap(), b"elephant");

		assert_eq!(&*db.get_by_prefix(None, key3.as_bytes()).unwrap(), b"elephant");
		assert_eq!(&*db.get_by_prefix(None, key2.as_bytes()).unwrap(), b"dog");

		let mut transaction = db.transaction();
		transaction.put(None, key1.as_bytes(), b"horse");
		transaction.delete(None, key3.as_bytes());
		db.write_buffered(transaction);
		assert!(db.get(None, key3.as_bytes()).unwrap().is_none());
		assert_eq!(&*db.get(None, key1.as_bytes()).unwrap().unwrap(), b"horse");

		db.flush().unwrap();
		assert!(db.get(None, key3.as_bytes()).unwrap().is_none());
		assert_eq!(&*db.get(None, key1.as_bytes()).unwrap().unwrap(), b"horse");
	}

	#[test]
	fn kvdb() {
		let tempdir = TempDir::new("").unwrap();
		let _ = Database::open_default(tempdir.path().to_str().unwrap()).unwrap();
		test_db(&DatabaseConfig::default());
	}

	#[test]
	#[cfg(target_os = "linux")]
	fn df_to_rotational() {
		use std::path::PathBuf;
		// Example df output.
		let example_df = vec![
			70, 105, 108, 101, 115, 121, 115, 116, 101, 109, 32, 32, 32, 32, 32, 49, 75, 45, 98, 108, 111, 99, 107,
			115, 32, 32, 32, 32, 32, 85, 115, 101, 100, 32, 65, 118, 97, 105, 108, 97, 98, 108, 101, 32, 85, 115, 101,
			37, 32, 77, 111, 117, 110, 116, 101, 100, 32, 111, 110, 10, 47, 100, 101, 118, 47, 115, 100, 97, 49, 32,
			32, 32, 32, 32, 32, 32, 54, 49, 52, 48, 57, 51, 48, 48, 32, 51, 56, 56, 50, 50, 50, 51, 54, 32, 32, 49, 57,
			52, 52, 52, 54, 49, 54, 32, 32, 54, 55, 37, 32, 47, 10,
		];
		let expected_output = Some(PathBuf::from("/sys/block/sda/queue/rotational"));
		assert_eq!(rotational_from_df_output(example_df), expected_output);
	}

	#[test]
	fn add_columns() {
		let config = DatabaseConfig::default();
		let config_5 = DatabaseConfig::with_columns(Some(5));

		let tempdir = TempDir::new("").unwrap();

		// open empty, add 5.
		{
			let db = Database::open(&config, tempdir.path().to_str().unwrap()).unwrap();
			assert_eq!(db.num_columns(), 0);

			for i in 1..=5 {
				db.add_column().unwrap();
				assert_eq!(db.num_columns(), i);
			}
		}

		// reopen as 5.
		{
			let db = Database::open(&config_5, tempdir.path().to_str().unwrap()).unwrap();
			assert_eq!(db.num_columns(), 5);
		}
	}

	#[test]
	fn remove_columns() {
		let config = DatabaseConfig::default();
		let config_5 = DatabaseConfig::with_columns(Some(5));

		let tempdir = TempDir::new("drop_columns").unwrap();

		// open 5, remove all.
		{
			let db = Database::open(&config_5, tempdir.path().to_str().unwrap()).expect("open with 5 columns");
			assert_eq!(db.num_columns(), 5);

			for i in (0..5).rev() {
				db.remove_last_column().unwrap();
				assert_eq!(db.num_columns(), i);
			}
		}

		// reopen as 0.
		{
			let db = Database::open(&config, tempdir.path().to_str().unwrap()).unwrap();
			assert_eq!(db.num_columns(), 0);
		}
	}

	#[test]
	fn test_iter_by_prefix() {
		let tempdir = TempDir::new("").unwrap();
		let config = DatabaseConfig::default();
		let db = Database::open(&config, tempdir.path().to_str().unwrap()).unwrap();

		let key1 = b"0";
		let key2 = b"ab";
		let key3 = b"abc";
		let key4 = b"abcd";

		let mut batch = db.transaction();
		batch.put(None, key1, key1);
		batch.put(None, key2, key2);
		batch.put(None, key3, key3);
		batch.put(None, key4, key4);
		db.write(batch).unwrap();

		// empty prefix
		let contents: Vec<_> = db.iter_from_prefix(None, b"").into_iter().collect();
		assert_eq!(contents.len(), 4);
		assert_eq!(&*contents[0].0, key1);
		assert_eq!(&*contents[1].0, key2);
		assert_eq!(&*contents[2].0, key3);
		assert_eq!(&*contents[3].0, key4);

		// prefix a
		let contents: Vec<_> = db.iter_from_prefix(None, b"a").into_iter().collect();
		assert_eq!(contents.len(), 3);
		assert_eq!(&*contents[0].0, key2);
		assert_eq!(&*contents[1].0, key3);
		assert_eq!(&*contents[2].0, key4);

		// prefix abc
		let contents: Vec<_> = db.iter_from_prefix(None, b"abc").into_iter().collect();
		assert_eq!(contents.len(), 2);
		assert_eq!(&*contents[0].0, key3);
		assert_eq!(&*contents[1].0, key4);

		// prefix abcde
		let contents: Vec<_> = db.iter_from_prefix(None, b"abcde").into_iter().collect();
		assert_eq!(contents.len(), 0);

		// prefix 0
		let contents: Vec<_> = db.iter_from_prefix(None, b"0").into_iter().collect();
		assert_eq!(contents.len(), 1);
		assert_eq!(&*contents[0].0, key1);
	}

	#[test]
	fn write_clears_buffered_ops() {
		let tempdir = TempDir::new("").unwrap();
		let config = DatabaseConfig::default();
		let db = Database::open(&config, tempdir.path().to_str().unwrap()).unwrap();

		let mut batch = db.transaction();
		batch.put(None, b"foo", b"bar");
		db.write_buffered(batch);

		let mut batch = db.transaction();
		batch.put(None, b"foo", b"baz");
		db.write(batch).unwrap();

		assert_eq!(db.get(None, b"foo").unwrap().unwrap().as_ref(), b"baz");
	}

	#[test]
	fn default_memory_budget() {
		let c = DatabaseConfig::default();
		assert_eq!(c.columns, None);
		assert_eq!(c.memory_budget(), DB_DEFAULT_MEMORY_BUDGET_MB * MB, "total memory budget is default");
		assert_eq!(
			c.memory_budget_for_col(0),
			DB_DEFAULT_COLUMN_MEMORY_BUDGET_MB * MB,
			"total memory budget for column 0 is the default"
		);
		assert_eq!(
			c.memory_budget_for_col(999),
			DB_DEFAULT_COLUMN_MEMORY_BUDGET_MB * MB,
			"total memory budget for any column is the default"
		);
	}

	#[test]
	fn memory_budget() {
		let mut c = DatabaseConfig::with_columns(Some(3));
		c.memory_budget = [(0, 10), (1, 15), (2, 20)].iter().cloned().map(|(c, b)| (Some(c), b)).collect();
		assert_eq!(c.memory_budget(), 45 * MB, "total budget is the sum of the column budget");
	}

	#[test]
	fn rocksdb_settings() {
		const NUM_COLS: usize = 2;
		let mut cfg = DatabaseConfig::with_columns(Some(NUM_COLS as u32));
		cfg.max_open_files = 123; // is capped by the OS fd limit (typically 1024)
		cfg.compaction.block_size = 323232;
		cfg.compaction.initial_file_size = 102030;
		cfg.memory_budget = [(0, 30), (1, 300)].iter().cloned().map(|(c, b)| (Some(c), b)).collect();

		let db_path = TempDir::new("config_test").expect("the OS can create tmp dirs");
		let _db = Database::open(&cfg, db_path.path().to_str().unwrap()).expect("can open a db");
		let mut rocksdb_log = std::fs::File::open(format!("{}/LOG", db_path.path().to_str().unwrap()))
			.expect("rocksdb creates a LOG file");
		let mut settings = String::new();
		rocksdb_log.read_to_string(&mut settings).unwrap();
		// Check column count
		assert!(settings.contains("Options for column family [default]"), "no default col");
		assert!(settings.contains("Options for column family [col0]"), "no col0");
		assert!(settings.contains("Options for column family [col1]"), "no col1");

		// Check max_open_files
		assert!(settings.contains("max_open_files: 123"));

		// Check block size
		assert!(settings.contains(" block_size: 323232"));

		// LRU cache (default column)
		assert!(settings.contains("block_cache_options:\n    capacity : 8388608"));
		// LRU cache for non-default columns is ⅓ of memory budget (including default column)
		let lru_size = (330 * MB) / 3;
		let needle = format!("block_cache_options:\n    capacity : {}", lru_size);
		let lru = settings.match_indices(&needle).collect::<Vec<_>>().len();
		assert_eq!(lru, NUM_COLS);

		// Index/filters share cache
		let include_indexes = settings.matches("cache_index_and_filter_blocks: 1").collect::<Vec<_>>().len();
		assert_eq!(include_indexes, NUM_COLS);
		// Pin index/filters on L0
		let pins = settings.matches("pin_l0_filter_and_index_blocks_in_cache: 1").collect::<Vec<_>>().len();
		assert_eq!(pins, NUM_COLS);

		// Check target file size, aka initial file size
		let l0_sizes = settings.matches("target_file_size_base: 102030").collect::<Vec<_>>().len();
		assert_eq!(l0_sizes, NUM_COLS);
		// The default column uses the default of 64Mb regardless of the setting.
		assert!(settings.contains("target_file_size_base: 67108864"));

		// Check compression settings
		let snappy_compression = settings.matches("Options.compression: Snappy").collect::<Vec<_>>().len();
		// All columns use Snappy
		assert_eq!(snappy_compression, NUM_COLS + 1);
		// …even for L7
		let snappy_bottommost = settings.matches("Options.bottommost_compression: Disabled").collect::<Vec<_>>().len();
		assert_eq!(snappy_bottommost, NUM_COLS + 1);

		// 7 levels
		let levels = settings.matches("Options.num_levels: 7").collect::<Vec<_>>().len();
		assert_eq!(levels, NUM_COLS + 1);

		// Don't fsync every store
		assert!(settings.contains("Options.use_fsync: 0"));

		// We're using the old format
		assert!(settings.contains("format_version: 2"));
	}
}
