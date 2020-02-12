// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A transactions ordering abstraction.

use crate::pool::Transaction;
use std::{cmp, fmt};

/// Represents a decision what to do with
/// a new transaction that tries to enter the pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
	/// New transaction should be rejected
	/// (i.e. the old transaction that occupies the same spot
	/// is better).
	RejectNew,
	/// The old transaction should be dropped
	/// in favour of the new one.
	ReplaceOld,
	/// The new transaction should be inserted
	/// and both (old and new) should stay in the pool.
	InsertNew,
}

/// Describes a reason why the `Score` of transactions
/// should be updated.
/// The `Scoring` implementations can use this information
/// to update the `Score` table more efficiently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change<T = ()> {
	/// New transaction has been inserted at given index.
	/// The Score at that index is initialized with default value
	/// and needs to be filled in.
	InsertedAt(usize),
	/// The transaction has been removed at given index and other transactions
	/// shifted to it's place.
	/// The scores were removed and shifted as well.
	/// For simple scoring algorithms no action is required here.
	RemovedAt(usize),
	/// The transaction at given index has replaced a previous transaction.
	/// The score at that index needs to be update (it contains value from previous transaction).
	ReplacedAt(usize),
	/// Given number of stalled transactions has been culled from the beginning.
	/// The scores has been removed from the beginning as well.
	/// For simple scoring algorithms no action is required here.
	Culled(usize),
	/// Custom event to update the score triggered outside of the pool.
	/// Handling this event is up to scoring implementation.
	Event(T),
}

/// A transaction ordering.
///
/// The implementation should decide on order of transactions in the pool.
/// Each transaction should also get assigned a `Score` which is used to later
/// prioritize transactions in the pending set.
///
/// Implementation notes:
/// - Returned `Score`s should match ordering of `compare` method.
/// - `compare` will be called only within a context of transactions from the same sender.
/// - `choose` may be called even if `compare` returns `Ordering::Equal`
/// - `Score`s and `compare` should align with `Ready` implementation.
///
/// Example: Natural ordering of Ethereum transactions.
/// - `compare`: compares transaction `nonce` ()
/// - `choose`: compares transactions `gasPrice` (decides if old transaction should be replaced)
/// - `update_scores`: score defined as `gasPrice` if `n==0` and `max(scores[n-1], gasPrice)` if `n>0`
///
pub trait Scoring<T>: fmt::Debug {
	/// A score of a transaction.
	type Score: cmp::Ord + Clone + Default + fmt::Debug + Send + fmt::LowerHex;
	/// Custom scoring update event type.
	type Event: fmt::Debug;

	/// Decides on ordering of `T`s from a particular sender.
	fn compare(&self, old: &T, other: &T) -> cmp::Ordering;

	/// Decides how to deal with two transactions from a sender that seem to occupy the same slot in the queue.
	fn choose(&self, old: &T, new: &T) -> Choice;

	/// Updates the transaction scores given a list of transactions and a change to previous scoring.
	/// NOTE: you can safely assume that both slices have the same length.
	/// (i.e. score at index `i` represents transaction at the same index)
	fn update_scores(&self, txs: &[Transaction<T>], scores: &mut [Self::Score], change: Change<Self::Event>);

	/// Decides if the transaction should ignore per-sender limit in the pool.
	///
	/// If you return `true` for given transaction it's going to be accepted even though
	/// the per-sender limit is exceeded.
	fn should_ignore_sender_limit(&self, _new: &T) -> bool {
		false
	}
}

/// A score with a reference to the transaction.
#[derive(Debug)]
pub struct ScoreWithRef<T, S> {
	/// Score
	pub score: S,
	/// Shared transaction
	pub transaction: Transaction<T>,
}

impl<T, S> ScoreWithRef<T, S> {
	/// Creates a new `ScoreWithRef`
	pub fn new(score: S, transaction: Transaction<T>) -> Self {
		ScoreWithRef { score, transaction }
	}
}

impl<T, S: Clone> Clone for ScoreWithRef<T, S> {
	fn clone(&self) -> Self {
		ScoreWithRef { score: self.score.clone(), transaction: self.transaction.clone() }
	}
}

impl<S: cmp::Ord, T> Ord for ScoreWithRef<T, S> {
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		other.score.cmp(&self.score).then(self.transaction.insertion_id.cmp(&other.transaction.insertion_id))
	}
}

impl<S: cmp::Ord, T> PartialOrd for ScoreWithRef<T, S> {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<S: cmp::Ord, T> PartialEq for ScoreWithRef<T, S> {
	fn eq(&self, other: &Self) -> bool {
		self.score == other.score && self.transaction.insertion_id == other.transaction.insertion_id
	}
}

impl<S: cmp::Ord, T> Eq for ScoreWithRef<T, S> {}

#[cfg(test)]
mod tests {
	use super::*;

	fn score(score: u64, insertion_id: u64) -> ScoreWithRef<(), u64> {
		ScoreWithRef { score, transaction: Transaction { insertion_id, transaction: Default::default() } }
	}

	#[test]
	fn scoring_comparison() {
		// the higher the score the better
		assert_eq!(score(10, 0).cmp(&score(0, 0)), cmp::Ordering::Less);
		assert_eq!(score(0, 0).cmp(&score(10, 0)), cmp::Ordering::Greater);

		// equal is equal
		assert_eq!(score(0, 0).cmp(&score(0, 0)), cmp::Ordering::Equal);

		// lower insertion id is better
		assert_eq!(score(0, 0).cmp(&score(0, 10)), cmp::Ordering::Less);
		assert_eq!(score(0, 10).cmp(&score(0, 0)), cmp::Ordering::Greater);
	}
}
