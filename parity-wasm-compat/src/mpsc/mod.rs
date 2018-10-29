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


//! mpsc compatibility

#[cfg(all(target_arch = "wasm32", feature = "browser-wasm"))]
mod mpsc_browser_single;

#[cfg(all(target_arch = "wasm32", feature = "browser-wasm"))]
pub use self::mpsc_browser_single::*;

#[cfg(not(target_arch = "wasm32"))]
mod mpsc_crate {
	pub use std::sync::mpsc::{ sync_channel, SyncSender, Receiver }; // need build then from pool max_count and execute
}

#[cfg(not(target_arch = "wasm32"))]
pub use self::mpsc_crate::*;
