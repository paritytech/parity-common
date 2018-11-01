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


//! File compatibility, at this point it is only to allow very specific case (no directory
//! management, `is_file` defaulting to true is good for it). And probably only our last usecase
//! (open file and read_to_end).

#[cfg(all(target_arch = "wasm32", feature = "browser-wasm"))]
mod fs_browser;

#[cfg(all(target_arch = "wasm32", feature = "browser-wasm"))]
pub use self::fs_browser::*;

#[cfg(not(target_arch = "wasm32"))]
mod fs_crate {
	pub use std::fs::{ File, OpenOptions };
}

#[cfg(not(target_arch = "wasm32"))]
pub use self::fs_crate::*;
