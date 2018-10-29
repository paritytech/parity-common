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


//! mpsc single thread compatibility (very minimal, could only run on particular conditions for
//! instance adding result from threads)

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::mpsc::{ SendError, RecvError };
use std::collections::VecDeque;

pub struct SyncSender<T>(Rc<RefCell<VecDeque<T>>>);

unsafe impl<T: Send> Send for SyncSender<T> {}

impl<T> Clone for SyncSender<T> {
 fn clone(&self) -> SyncSender<T> {
		SyncSender(self.0.clone())
	}
}


pub struct Receiver<T>(Rc<RefCell<VecDeque<T>>>);

pub fn sync_channel<T>(bound: usize) -> (SyncSender<T>, Receiver<T>) {
	let v = Rc::new(RefCell::new(VecDeque::with_capacity(bound)));
	(SyncSender(v.clone()), Receiver(v))
}

impl<T> SyncSender<T> {

	pub fn send(&self, t: T) -> Result<(), SendError<T>> {
		self.0.borrow_mut().push_front(t);
		Ok(())
	}

}


impl<T> Receiver<T> {

	pub fn recv(&self) -> Result<T, RecvError> {
		if let Some(t) = self.0.borrow_mut().pop_front() {
			Ok(t)
		} else {
			Err(RecvError)
		}
	}

}

