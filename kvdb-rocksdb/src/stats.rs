// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::Instant;

pub struct RawDbStats {
	pub reads: u64,
	pub writes: u64,
	pub bytes_written: u64,
	pub bytes_read: u64,
	pub transactions: u64,
}

impl RawDbStats {
	fn combine(&self, other: &RawDbStats) -> Self {
		RawDbStats {
			reads: self.reads + other.reads,
			writes: self.writes + other.writes,
			bytes_written: self.bytes_written + other.bytes_written,
			bytes_read: self.bytes_read + other.bytes_written,
			transactions: self.transactions + other.transactions,
		}
	}
}

struct OverallDbStats {
	stats: RawDbStats,
	last_taken: Instant,
	started: Instant,
}

impl OverallDbStats {
	fn new() -> Self {
		OverallDbStats {
			stats: RawDbStats { reads: 0, writes: 0, bytes_written: 0, bytes_read: 0, transactions: 0 },
			last_taken: Instant::now(),
			started: Instant::now(),
		}
	}
}

pub struct RunningDbStats {
	reads: AtomicU64,
	writes: AtomicU64,
	bytes_written: AtomicU64,
	bytes_read: AtomicU64,
	transactions: AtomicU64,
	overall: RwLock<OverallDbStats>,
}

pub struct TakenDbStats {
	pub raw: RawDbStats,
	pub started: Instant,
}

impl RunningDbStats {
	pub fn new() -> Self {
		Self {
			reads: 0.into(),
			bytes_read: 0.into(),
			writes: 0.into(),
			bytes_written: 0.into(),
			transactions: 0.into(),
			overall: OverallDbStats::new().into(),
		}
	}

	pub fn tally_reads(&self, val: u64) {
		self.reads.fetch_add(val, AtomicOrdering::Relaxed);
	}

	pub fn tally_bytes_read(&self, val: u64) {
		self.bytes_read.fetch_add(val, AtomicOrdering::Relaxed);
	}

	pub fn tally_writes(&self, val: u64) {
		self.writes.fetch_add(val, AtomicOrdering::Relaxed);
	}

	pub fn tally_bytes_written(&self, val: u64) {
		self.bytes_written.fetch_add(val, AtomicOrdering::Relaxed);
	}

	pub fn tally_transactions(&self, val: u64) {
		self.transactions.fetch_add(val, AtomicOrdering::Relaxed);
	}

	fn take_current(&self) -> RawDbStats {
		RawDbStats {
			reads: self.reads.swap(0, AtomicOrdering::Relaxed),
			writes: self.writes.swap(0, AtomicOrdering::Relaxed),
			bytes_written: self.bytes_written.swap(0, AtomicOrdering::Relaxed),
			bytes_read: self.bytes_read.swap(0, AtomicOrdering::Relaxed),
			transactions: self.transactions.swap(0, AtomicOrdering::Relaxed),
		}
	}

	fn peek_current(&self) -> RawDbStats {
		RawDbStats {
			reads: self.reads.load(AtomicOrdering::Relaxed),
			writes: self.writes.load(AtomicOrdering::Relaxed),
			bytes_written: self.bytes_written.load(AtomicOrdering::Relaxed),
			bytes_read: self.bytes_read.load(AtomicOrdering::Relaxed),
			transactions: self.transactions.load(AtomicOrdering::Relaxed),
		}
	}

	pub fn since_previous(&self) -> TakenDbStats {
		let mut overall_lock = self.overall.write();

		let current = self.take_current();

		overall_lock.stats = overall_lock.stats.combine(&current);

		let stats = TakenDbStats { raw: current, started: overall_lock.last_taken };

		overall_lock.last_taken = Instant::now();

		stats
	}

	pub fn overall(&self) -> TakenDbStats {
		let overall_lock = self.overall.read();

		let current = self.peek_current();

		TakenDbStats { raw: overall_lock.stats.combine(&current), started: overall_lock.started }
	}
}
