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


//! Hook to display stdout and stderr as websys console calls

use std::io::{ Write, Result, set_panic, set_print };
use web_sys::console;
use wasm_bindgen::JsValue;
use js_sys::Array;

fn write_out(v: &str) {
	if v.len() > 0 {
		console::log(&Array::of1(&JsValue::from(v)))
	}
}
fn write_err(v: &str) {
	if v.len() > 0 {
		console::warn(&Array::of1(&JsValue::from(v)))
	}
}

struct Writer(pub fn(&str), pub String);

struct NoBuffWriter(fn(&str));

impl Write for NoBuffWriter {

 fn write(&mut self, buf: &[u8]) -> Result<usize> {
	 let str_w = unsafe { std::str::from_utf8_unchecked(buf) };
	 (self.0)(str_w);
	 Ok(buf.len())
 }

 fn flush(&mut self) -> Result<()> { Ok(()) }

}

impl Write for Writer {

 fn write(&mut self, buf: &[u8]) -> Result<usize> {
	 if let Some(p) = buf.iter().rposition(|b|*b == '\n' as u8) {
		 self.1.push_str(&String::from_utf8_lossy(&buf[..p]));
		 (self.0)(&self.1);
		 self.1.clear();
		 self.1.push_str(&String::from_utf8_lossy(&buf[p..]));
	 } else {
		 // TODO add a max length for flushing
		 self.1.push_str(&String::from_utf8_lossy(&buf[..]));
	 }
	 Ok(buf.len())
 }

 fn flush(&mut self) -> Result<()> { 
	 (self.0)(&self.1);
	 self.1.clear();
	 Ok(())
 }
 
}

/// Sets stdout and stderr
pub fn hook_std_io_no_buff () {
	let wout = NoBuffWriter(write_out);
	let werr = NoBuffWriter(write_err);
	set_print(Some(Box::new(wout)));
	set_panic(Some(Box::new(werr)));
}

/// Sets stdout and stderr, use a display buffer
pub fn hook_std_io () {
	let wout = Writer(write_out,String::new());
	let werr = Writer(write_err,String::new());
	set_print(Some(Box::new(wout)));
	set_panic(Some(Box::new(werr)));
}

