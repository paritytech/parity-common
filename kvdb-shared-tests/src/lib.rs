// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Shared tests for kvdb functionality, to be executed against actual implementations.

use kvdb::{IoStatsKind, KeyValueDB};
use std::io;

/// A test for `KeyValueDB::get`.
pub fn test_put_and_get(db: &dyn KeyValueDB) -> io::Result<()> {
	let key1 = b"key1";

	let mut transaction = db.transaction();
	transaction.put(0, key1, b"horse");
	db.write(transaction)?;
	assert_eq!(&*db.get(0, key1)?.unwrap(), b"horse");
	Ok(())
}

/// A test for `KeyValueDB::get`.
pub fn test_delete_and_get(db: &dyn KeyValueDB) -> io::Result<()> {
	let key1 = b"key1";

	let mut transaction = db.transaction();
	transaction.put(0, key1, b"horse");
	db.write(transaction)?;
	assert_eq!(&*db.get(0, key1)?.unwrap(), b"horse");

	let mut transaction = db.transaction();
	transaction.delete(0, key1);
	db.write(transaction)?;
	assert!(db.get(0, key1)?.is_none());
	Ok(())
}

/// A test for `KeyValueDB::get`.
/// Assumes the `db` has only 1 column.
pub fn test_get_fails_with_non_existing_column(db: &dyn KeyValueDB) -> io::Result<()> {
	assert!(db.get(1, &[]).is_err());
	Ok(())
}

/// A test for `KeyValueDB::write`.
pub fn test_write_clears_buffered_ops(db: &dyn KeyValueDB) -> io::Result<()> {
	let mut batch = db.transaction();
	batch.put(0, b"foo", b"bar");
	db.write(batch)?;

	assert_eq!(db.get(0, b"foo")?.unwrap(), b"bar");

	let mut batch = db.transaction();
	batch.put(0, b"foo", b"baz");
	db.write(batch)?;

	assert_eq!(db.get(0, b"foo")?.unwrap(), b"baz");
	Ok(())
}

/// A test for `KeyValueDB::iter`.
pub fn test_iter(db: &dyn KeyValueDB) -> io::Result<()> {
	let key1 = b"key1";
	let key2 = b"key2";

	let mut transaction = db.transaction();
	transaction.put(0, key1, key1);
	transaction.put(0, key2, key2);
	db.write(transaction)?;

	let contents: Vec<_> = db.iter(0).into_iter().map(Result::unwrap).collect();
	assert_eq!(contents.len(), 2);
	assert_eq!(&*contents[0].0, key1);
	assert_eq!(&*contents[0].1, key1);
	assert_eq!(&*contents[1].0, key2);
	assert_eq!(&*contents[1].1, key2);
	Ok(())
}

/// A test for `KeyValueDB::iter_with_prefix`.
pub fn test_iter_with_prefix(db: &dyn KeyValueDB) -> io::Result<()> {
	let key1 = b"0";
	let key2 = b"ab";
	let key3 = b"abc";
	let key4 = b"abcd";

	let mut batch = db.transaction();
	batch.put(0, key1, key1);
	batch.put(0, key2, key2);
	batch.put(0, key3, key3);
	batch.put(0, key4, key4);
	db.write(batch)?;

	// empty prefix
	let contents: Vec<_> = db.iter_with_prefix(0, b"").into_iter().map(Result::unwrap).collect();
	assert_eq!(contents.len(), 4);
	assert_eq!(&*contents[0].0, key1);
	assert_eq!(&*contents[1].0, key2);
	assert_eq!(&*contents[2].0, key3);
	assert_eq!(&*contents[3].0, key4);

	// prefix a
	let contents: Vec<_> = db.iter_with_prefix(0, b"a").into_iter().map(Result::unwrap).collect();
	assert_eq!(contents.len(), 3);
	assert_eq!(&*contents[0].0, key2);
	assert_eq!(&*contents[1].0, key3);
	assert_eq!(&*contents[2].0, key4);

	// prefix abc
	let contents: Vec<_> = db.iter_with_prefix(0, b"abc").into_iter().map(Result::unwrap).collect();
	assert_eq!(contents.len(), 2);
	assert_eq!(&*contents[0].0, key3);
	assert_eq!(&*contents[1].0, key4);

	// prefix abcde
	let contents: Vec<_> = db.iter_with_prefix(0, b"abcde").into_iter().map(Result::unwrap).collect();
	assert_eq!(contents.len(), 0);

	// prefix 0
	let contents: Vec<_> = db.iter_with_prefix(0, b"0").into_iter().map(Result::unwrap).collect();
	assert_eq!(contents.len(), 1);
	assert_eq!(&*contents[0].0, key1);
	Ok(())
}

/// The number of columns required to run `test_io_stats`.
pub const IO_STATS_NUM_COLUMNS: u32 = 3;

/// A test for `KeyValueDB::io_stats`.
/// Assumes that the `db` has at least 3 columns.
pub fn test_io_stats(db: &dyn KeyValueDB) -> io::Result<()> {
	let key1 = b"kkk";
	let mut batch = db.transaction();
	batch.put(0, key1, key1);
	batch.put(1, key1, key1);
	batch.put(2, key1, key1);

	for _ in 0..10 {
		db.get(0, key1)?;
	}

	db.write(batch)?;

	let io_stats = db.io_stats(IoStatsKind::SincePrevious);
	assert_eq!(io_stats.transactions, 1);
	assert_eq!(io_stats.writes, 3);
	assert_eq!(io_stats.bytes_written, 18);
	assert_eq!(io_stats.reads, 10);
	assert_eq!(io_stats.bytes_read, 30);

	let new_io_stats = db.io_stats(IoStatsKind::SincePrevious);
	// Since we taken previous statistic period,
	// this is expected to be totally empty.
	assert_eq!(new_io_stats.transactions, 0);

	// but the overall should be there
	let new_io_stats = db.io_stats(IoStatsKind::Overall);
	assert_eq!(new_io_stats.bytes_written, 18);

	let mut batch = db.transaction();
	batch.delete(0, key1);
	batch.delete(1, key1);
	batch.delete(2, key1);

	// transaction is not commited yet
	assert_eq!(db.io_stats(IoStatsKind::SincePrevious).writes, 0);

	db.write(batch)?;
	// now it is, and delete is counted as write
	assert_eq!(db.io_stats(IoStatsKind::SincePrevious).writes, 3);
	Ok(())
}

/// The number of columns required to run `test_delete_prefix`.
pub const DELETE_PREFIX_NUM_COLUMNS: u32 = 7;

/// A test for `KeyValueDB::delete_prefix`.
pub fn test_delete_prefix(db: &dyn KeyValueDB) -> io::Result<()> {
	let keys = [
		&[][..],
		&[0u8][..],
		&[0, 1][..],
		&[1][..],
		&[1, 0][..],
		&[1, 255][..],
		&[1, 255, 255][..],
		&[2][..],
		&[2, 0][..],
		&[2, 255][..],
		&[255; 16][..],
	];
	let init_db = |ix: u32| -> io::Result<()> {
		let mut batch = db.transaction();
		for (i, key) in keys.iter().enumerate() {
			batch.put(ix, key, &[i as u8]);
		}
		db.write(batch)?;
		Ok(())
	};
	let check_db = |ix: u32, content: [bool; 11]| -> io::Result<()> {
		let mut state = [true; 11];
		for (c, key) in keys.iter().enumerate() {
			state[c] = db.get(ix, key)?.is_some();
		}
		assert_eq!(state, content, "at {}", ix);
		Ok(())
	};
	let tests: [_; DELETE_PREFIX_NUM_COLUMNS as usize] = [
		// standard
		(&[1u8][..], [true, true, true, false, false, false, false, true, true, true, true]),
		// edge
		(&[1u8, 255, 255][..], [true, true, true, true, true, true, false, true, true, true, true]),
		// none 1
		(&[1, 2][..], [true, true, true, true, true, true, true, true, true, true, true]),
		// none 2
		(&[8][..], [true, true, true, true, true, true, true, true, true, true, true]),
		// last value
		(&[255, 255][..], [true, true, true, true, true, true, true, true, true, true, false]),
		// last value, limit prefix
		(&[255][..], [true, true, true, true, true, true, true, true, true, true, false]),
		// all
		(&[][..], [false, false, false, false, false, false, false, false, false, false, false]),
	];
	for (ix, test) in tests.iter().enumerate() {
		let ix = ix as u32;
		init_db(ix)?;
		let mut batch = db.transaction();
		batch.delete_prefix(ix, test.0);
		db.write(batch)?;
		check_db(ix, test.1)?;
	}

	Ok(())
}

/// A complex test.
pub fn test_complex(db: &dyn KeyValueDB) -> io::Result<()> {
	let key1 = b"02c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc";
	let key2 = b"03c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc";
	let key3 = b"04c00000000b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc";
	let key4 = b"04c01111110b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc";
	let key5 = b"04c02222220b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc";

	let mut batch = db.transaction();
	batch.put(0, key1, b"cat");
	batch.put(0, key2, b"dog");
	batch.put(0, key3, b"caterpillar");
	batch.put(0, key4, b"beef");
	batch.put(0, key5, b"fish");
	db.write(batch)?;

	assert_eq!(&*db.get(0, key1)?.unwrap(), b"cat");

	let contents: Vec<_> = db.iter(0).into_iter().map(Result::unwrap).collect();
	assert_eq!(contents.len(), 5);
	assert_eq!(contents[0].0.to_vec(), key1.to_vec());
	assert_eq!(&*contents[0].1, b"cat");
	assert_eq!(contents[1].0.to_vec(), key2.to_vec());
	assert_eq!(&*contents[1].1, b"dog");

	let mut prefix_iter = db.iter_with_prefix(0, b"04c0");
	assert_eq!(*prefix_iter.next().unwrap().unwrap().1, b"caterpillar"[..]);
	assert_eq!(*prefix_iter.next().unwrap().unwrap().1, b"beef"[..]);
	assert_eq!(*prefix_iter.next().unwrap().unwrap().1, b"fish"[..]);

	let mut batch = db.transaction();
	batch.delete(0, key1);
	db.write(batch)?;

	assert!(db.get(0, key1)?.is_none());

	let mut batch = db.transaction();
	batch.put(0, key1, b"cat");
	db.write(batch)?;

	let mut transaction = db.transaction();
	transaction.put(0, key3, b"elephant");
	transaction.delete(0, key1);
	db.write(transaction)?;
	assert!(db.get(0, key1)?.is_none());
	assert_eq!(&*db.get(0, key3)?.unwrap(), b"elephant");

	assert_eq!(&*db.get_by_prefix(0, key3).unwrap().unwrap(), b"elephant");
	assert_eq!(&*db.get_by_prefix(0, key2).unwrap().unwrap(), b"dog");

	let mut transaction = db.transaction();
	transaction.put(0, key1, b"horse");
	transaction.delete(0, key3);
	db.write(transaction)?;
	assert!(db.get(0, key3)?.is_none());
	assert_eq!(&*db.get(0, key1)?.unwrap(), b"horse");

	assert!(db.get(0, key3)?.is_none());
	assert_eq!(&*db.get(0, key1)?.unwrap(), b"horse");
	Ok(())
}
