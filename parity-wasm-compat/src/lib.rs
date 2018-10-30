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

//! Parity wasm compat crate.
#![feature(fn_traits)]

pub mod rng;
pub mod threadpool;
pub mod mpsc;
pub mod memmap;
pub mod snappy;

pub mod home {
	#[cfg(not(target_arch = "wasm32"))]
	extern crate home;
	#[cfg(not(target_arch = "wasm32"))]
	pub use home::home_dir;

	#[cfg(all(target_arch = "wasm32", feature = "browser-wasm"))]
	use std::path::PathBuf;
	#[cfg(all(target_arch = "wasm32", feature = "browser-wasm"))]
	pub fn home_dir() -> Option<PathBuf> {
		// need a dummy dir for whatever browser mapping we use
		Some(PathBuf::from("/home"))
	}
}
