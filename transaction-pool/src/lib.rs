// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Generic Transaction Pool
//!
//! An extensible and performant implementation of Ethereum Transaction Pool.
//! The pool stores ordered, verified transactions according to some pluggable
//! `Scoring` implementation.
//! The pool also allows you to construct a set of `pending` transactions according
//! to some notion of `Readiness` (pluggable).
//!
//! The pool is generic over transactions and should make no assumptions about them.
//! The only thing we can rely on is the `Scoring` that defines:
//!  - the ordering of transactions from a single sender
//!  - the priority of the transaction compared to other transactions from different senders
//!
//! NOTE: the transactions from a single sender are not ordered by priority,
//! but still when constructing pending set we always need to maintain the ordering
//! (i.e. `txs[1]` always needs to be included after `txs[0]` even if it has higher priority)
//!
//! ### Design Details
//!
//! Performance assumptions:
//! - Possibility to handle tens of thousands of transactions
//! - Fast insertions and replacements `O(per-sender + log(senders))`
//! - Reasonably fast removal of stalled transactions `O(per-sender)`
//! - Reasonably fast construction of pending set `O(txs * (log(senders) + log(per-sender))`
//!
//! The removal performance could be improved by trading some memory. Currently `SmallVec` is used
//! to store senders transactions, instead we could use `VecDeque` and efficiently `pop_front`
//! the best transactions.
//!
//! The pending set construction and insertion complexity could be reduced by introducing
//! a notion of `nonce` - an absolute, numeric ordering of transactions.
//! We don't do that because of possible implications of EIP208 where nonce might not be
//! explicitly available.
//!
//! 1. The pool groups transactions from particular sender together
//!    and stores them ordered by `Scoring` within that group
//!    i.e. `HashMap<Sender, Vec<Transaction>>`.
//! 2. Additionally we maintain the best and the worst transaction from each sender
//!    (by `Scoring` not `priority`) ordered by `priority`.
//!    It means that we can easily identify the best transaction inside the entire pool
//!    and the worst transaction.
//! 3. Whenever new transaction is inserted to the queue:
//!    - first check all the limits (overall, memory, per-sender)
//!    - retrieve all transactions from a sender
//!    - binary search for position to insert the transaction
//!    - decide if we are replacing existing transaction (3 outcomes: drop, replace, insert)
//!    - update best and worst transaction from that sender if affected
//! 4. Pending List construction:
//!    - Take the best transaction (by priority) from all senders to the List
//!    - Replace the transaction with next transaction (by ordering) from that sender (if any)
//!    - Repeat

#![warn(missing_docs)]

#[cfg(test)]
mod tests;

mod error;
mod listener;
mod options;
mod pool;
mod ready;
mod replace;
mod status;
mod transactions;
mod verifier;

pub mod scoring;

pub use self::error::Error;
pub use self::listener::{Listener, NoopListener};
pub use self::options::Options;
pub use self::pool::{PendingIterator, Pool, Transaction, UnorderedIterator};
pub use self::ready::{Readiness, Ready};
pub use self::replace::{ReplaceTransaction, ShouldReplace};
pub use self::scoring::Scoring;
pub use self::status::{LightStatus, Status};
pub use self::verifier::Verifier;

use std::fmt;
use std::hash::Hash;

/// Already verified transaction that can be safely queued.
pub trait VerifiedTransaction: fmt::Debug {
	/// Transaction hash type.
	type Hash: fmt::Debug + fmt::LowerHex + Eq + Clone + Hash;

	/// Transaction sender type.
	type Sender: fmt::Debug + Eq + Clone + Hash + Send;

	/// Transaction hash
	fn hash(&self) -> &Self::Hash;

	/// Memory usage
	fn mem_usage(&self) -> usize;

	/// Transaction sender
	fn sender(&self) -> &Self::Sender;
}
