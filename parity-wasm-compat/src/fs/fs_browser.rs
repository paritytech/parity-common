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


//! fs compatibility (very limited)


use std::io::{ self, Read, Write, Seek, SeekFrom };
use std::path::Path;
use wasm_bindgen::prelude::*;

// put in module env for now
#[wasm_bindgen(module = "env")]
extern "C" {

	pub type JsFile;

	#[wasm_bindgen(catch)]
	fn open_browserfs(path: &str, option: i32) -> Result<JsFile, JsValue>;

	#[wasm_bindgen(catch)]
	fn read_browserfs(jsfile: &JsFile, buff: *mut u8, len: u32) -> Result<u32, JsValue>;

	#[wasm_bindgen(catch)]
	fn write_browserfs(jsfile: &JsFile, buff: *const u8, len: u32) -> Result<u32, JsValue>;

	#[wasm_bindgen(catch)]
	fn flush_write_browserfs(jsfile: &JsFile) -> Result<(), JsValue>;

	#[wasm_bindgen(catch)]
	fn set_len_browserfs(jsfile: &JsFile, len: u64) -> Result<(), JsValue>;

	fn close_browserfs(jsfile: &JsFile);

	#[wasm_bindgen(catch)]
	fn seek_browserfs(jsfile: &JsFile, mov: i64) -> Result<u32, JsValue>;

	fn seek_browserfs_from_start(jsfile: &JsFile, mov: u32) -> u32;

	#[wasm_bindgen(catch)]
	fn seek_browserfs_from_end(jsfile: &JsFile, mov: i64) -> Result<u32, JsValue>;

	fn len_browserfs(jsfile: &JsFile) -> u64;

}

pub struct File(pub JsFile);

#[allow(non_camel_case_types)]
type c_int = i32;

#[allow(non_camel_case_types)]
type mode_t = u32;

mod libc {
	use super::c_int;
	// reexport unix libc constant for file
	//
	pub const O_RDONLY: c_int = 0;
	pub const O_WRONLY: c_int = 1;
	pub const O_RDWR: c_int = 2;

	pub const O_APPEND: c_int = 1024;

	pub const O_CREAT: c_int = 64;
	pub const O_EXCL: c_int = 128;
	pub const O_TRUNC: c_int = 512;

	pub const EINVAL : c_int = 22;
}
/// straight copy of unix code plus create and new
#[derive(Clone, Debug)]
pub struct OpenOptions {
		// generic
		read: bool,
		write: bool,
		append: bool,
		truncate: bool,
		create: bool,
		create_new: bool,
		// system-specific
		custom_flags: i32,
		mode: mode_t,
}


impl OpenOptions {
	/// only non copied function from unix code
	pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
		let tag = self.get_access_mode()? |
			self.get_creation_mode()? |
			(self.custom_flags as c_int);
		open_browserfs(path.as_ref().to_str().expect("input from browser in utf8"), tag)
		.map_err(|jsval|io::Error::new(io::ErrorKind::Other, format!("could not open file in browser: {:?}",jsval)))
		.map(|jsfile|File(jsfile))
	
	}
 

	/// see unix openoption
	pub fn new() -> OpenOptions {
		OpenOptions {
			// generic
			read: false,
			write: false,
			append: false,
			truncate: false,
			create: false,
			create_new: false,
			// system-specific
			custom_flags: 0,
			mode: 0o666,
		}
	}

	/// see unix openoption
	pub fn read(mut self, read: bool) -> Self { self.read = read; self }
	/// see unix openoption
	pub fn write(mut self, write: bool) -> Self { self.write = write; self }
	/// see unix openoption
	pub fn append(mut self, append: bool) -> Self { self.append = append; self }
	/// see unix openoption
	pub fn truncate(mut self, truncate: bool) -> Self { self.truncate = truncate; self }
	/// see unix openoption
	pub fn create(mut self, create: bool) -> Self { self.create = create; self }
	/// see unix openoption
	pub fn create_new(mut self, create_new: bool) -> Self { self.create_new = create_new; self }

	/// see unix openoption
	pub fn custom_flags(mut self, flags: i32) -> Self { self.custom_flags = flags; self }
	/// warning curently not used TODO usable in node fs
	pub fn mode(mut self, mode: u32) -> Self { self.mode = mode as mode_t; self }

	/// see unix openoption
	fn get_access_mode(&self) -> io::Result<c_int> {
		match (self.read, self.write, self.append) {
			(true, false, false) => Ok(libc::O_RDONLY),
			(false, true, false) => Ok(libc::O_WRONLY),
			(true, true, false) => Ok(libc::O_RDWR),
			(false, _, true)	=> Ok(libc::O_WRONLY | libc::O_APPEND),
			(true, _, true)	=> Ok(libc::O_RDWR | libc::O_APPEND),
			(false, false, false) => Err(io::Error::from_raw_os_error(libc::EINVAL)),
		}
	}

	/// see unix openoption
	fn get_creation_mode(&self) -> io::Result<c_int> {
		match (self.write, self.append) {
			(true, false) => {}
			(false, false) =>
				if self.truncate || self.create || self.create_new {
					return Err(io::Error::from_raw_os_error(libc::EINVAL));
				},
			(_, true) =>
				if self.truncate && !self.create_new {
					return Err(io::Error::from_raw_os_error(libc::EINVAL));
				},
		}

		Ok(match (self.create, self.truncate, self.create_new) {
			(false, false, false) => 0,
			(true, false, false) => libc::O_CREAT,
			(false, true, false) => libc::O_TRUNC,
			(true, true, false) => libc::O_CREAT | libc::O_TRUNC,
			(_, _, true) => libc::O_CREAT | libc::O_EXCL,
		})
	}
}

/*
#[derive(Clone, Debug)]
pub struct OpenOptions {
	// generic
	read: bool,
	write: bool,
	append: bool,
	truncate: bool,
	create: bool,
	create_new: bool,
}
impl OpenOptions {
	pub fn new() -> OpenOptions {
		OpenOptions {
			// generic
			read: false,
			write: false,
			append: false,
			truncate: false,
			create: false,
			create_new: false,
		}
	}

	pub fn read(&mut self, read: bool) { self.read = read; }
	pub fn write(&mut self, write: bool) { self.write = write; }
	pub fn append(&mut self, append: bool) { self.append = append; }
	pub fn truncate(&mut self, truncate: bool) { self.truncate = truncate; }
	pub fn create(&mut self, create: bool) { self.create = create; }
	pub fn create_new(&mut self, create_new: bool) { self.create_new = create_new; }

	// mapping from https://nodejs.org/api/fs.html#fs_file_system_flags
	fn get_node_fs_tag(&self) -> io::Result<&str> {
	const tag_r : &str = "r"; // reading, should exist
	const tag_rp : &str = "r+"; // reading, should exist, + write
	const tag_w : &str = "w"; // writing, truncate if exist and create if don't
	const tag_wx : &str = "wx"; // writing, truncate if exist and create if don't, fail if exist
	const tag_wp : &str = "w+"; // writing, truncate if exist and create if don't
	const tag_wxp : &str = "wx+"; // writing, truncate if exist and create if don't, fail if exist
	const tag_a : &str = "a"; // append , create if does not exist
	const tag_ax : &str = "ax"; // append , create if does not exist, fail if exist
	const tag_ap : &str = "a+"; // append , create if does not exist
	const tag_axp : &str = "ax+"; // append , create if does not exist, fail if exist
		match (self.read, self.write, self.append, self.truncate, self.create, self.create_new) {
			(true, false, false, false, false, false) => Ok(&tag_r),
			(true, true, false, false, false, false) => Ok(&tag_rp),
			(false, true, false, true, true, false) => Ok(&tag_w),
			(false, true, false, true, true, true) => Ok(&tag_wx),
			(true, true, false, true, true, false) => Ok(&tag_wp),
			(true, true, false, true, true, true) => Ok(&tag_wxp),
			(false, _, true, false, true, false) => Ok(&tag_a),
			(false, _, true, false, true, true) => Ok(&tag_ax),
			(true, _, true, false, true, false) => Ok(&tag_ap),
			(true, _, true, false, true, true) => Ok(&tag_axp),
			_ => Err(io::Error::new(io::ErrorKind::Other, "unimplemented or non allowed openoption configuration")),
		}
	}
}
*/
impl File {

	/// see std::fs::File
	pub fn open<P: AsRef<Path>>(path: P) -> io::Result<File> {
		open_browserfs(path.as_ref().to_str().expect("input from browser in utf8"), 0)
		.map_err(|jsval|io::Error::new(io::ErrorKind::Other, format!("could not open file in browser: {:?}",jsval)))
		.map(|jsfile|File(jsfile))
	}

	fn inner_read(&self, buf: &mut [u8]) -> io::Result<usize> {
		let l = buf.len();
		match read_browserfs(&self.0, buf.as_mut_ptr(), l as u32) {
			Ok(n) => Ok(n as usize),
			Err(jsval) => Err(io::Error::new(
				io::ErrorKind::Other, format!("could not read from file in browser: {:?}",jsval)
			)),
		}
	}

	fn inner_write(&self, buf: &[u8]) -> io::Result<usize> {
		let l = buf.len();
		match write_browserfs(&self.0, buf.as_ptr(), l as u32) {
			Ok(n) => Ok(n as usize),
			Err(jsval) => Err(io::Error::new(
				io::ErrorKind::Other, format!("could not write in file in browser: {:?}",jsval)
			)),
		}
	}

	fn inner_flush(&self) -> io::Result<()> {
		match flush_write_browserfs(&self.0) {
			Ok(n) => Ok(n),
			Err(jsval) => Err(io::Error::new(
				io::ErrorKind::Other, format!("error when flushing file in browser: {:?}",jsval)
			)),
		}
	}

	// Note that if we stored jsfile in rust memory (doable (just get the fd with open), probably better design), 
	// we would not need to call js (except to get length for end) here
	// TODO redesign if this wasm compat get used (and choice of memfs is ok), at least it
	// demonstrate wasmbindgen capability (but it is a waste here). Will get a slight issue with
	// mutability
	fn inner_seek(&self, pos: SeekFrom) -> io::Result<u64> {
		match match pos {
			SeekFrom::Current(nb) => seek_browserfs(&self.0, nb),
			SeekFrom::Start(nb) => Ok(seek_browserfs_from_start(&self.0, nb as u32) as u32),
			SeekFrom::End(nb) => seek_browserfs_from_end(&self.0, nb),
		} {
			Ok(n) => Ok(n as u64),
			Err(jsval) => Err(io::Error::new(
				io::ErrorKind::Other, format!("seek to under zero position: {:?}",jsval)
			)),
		}
	}

	pub fn set_len(&self, size: u64) -> io::Result<()> {
		match set_len_browserfs(&self.0, size) {
			Ok(n) => Ok(n),
			Err(jsval) => Err(io::Error::new(
				io::ErrorKind::Other, format!("error changing file length in browser: {:?}",jsval)
			)),
		}
	}

	/// warning non conform use : would need to query stat as metada, TODO keep it with not many use
	/// case
	pub fn metadata(&self) -> io::Result<&Self> {
		Ok(self)
	}

	/// from metadata use case, move it to metadata if using stat value as metadata
	/// TODO this is bad semantic indeed (can return an error)
	pub fn len(&self) -> u64 {
		len_browserfs(&self.0)
	}

}
impl Read for File {

	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.inner_read(buf)
	}

}

impl Write for File {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.inner_write(buf)
	}

	fn flush(&mut self) -> io::Result<()> {
		self.inner_flush()
	}
}

impl Seek for File {
	
	fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
		self.inner_seek(pos)
	}
}

impl Drop for File {
	fn drop(&mut self) {
		close_browserfs(&self.0);
	}
}

impl<'a> Read for &'a File {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.inner_read(buf)
	}
}

impl<'a> Write for &'a File {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.inner_write(buf)
	}

	fn flush(&mut self) -> io::Result<()> { 
		self.inner_flush()
	}
}

impl<'a> Seek for &'a File {
	fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
		self.inner_seek(pos)
	}
}


