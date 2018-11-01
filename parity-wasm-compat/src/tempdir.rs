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


//! tempdir compat TODO is this crate still really usefull (only call to "" path)??

#[cfg(not(target_arch = "wasm32"))]
pub use tempdir::Tempdir;

#[cfg(all(target_arch = "wasm32", feature = "browser-wasm"))]
mod tempdir_browser {

	use std::path;
	use std::io;

	pub struct TempDir(pub path::PathBuf);

	impl TempDir {
		pub fn new(prefix: &str) -> io::Result<TempDir> {
			if prefix.len() > 0 {
				// current use is only to get temp_dir
				unimplemented!();
			}
			Ok(TempDir(crate::env::temp_dir()))
		}
		
		pub fn path(&self) -> &path::Path {
			&self.0
		}

	}
}

#[cfg(target_arch = "wasm32")]
pub use self::tempdir_browser::*;
