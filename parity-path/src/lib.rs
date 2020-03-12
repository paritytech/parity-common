// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Path utilities
use std::path::Path;
use std::path::PathBuf;

use home::home_dir;

#[cfg(target_os = "macos")]
/// Get the config path for application `name`.
/// `name` should be capitalized, e.g. `"Ethereum"`, `"Parity"`.
pub fn config_path(name: &str) -> PathBuf {
	let mut home = home_dir().expect("Failed to get home dir");
	home.push("Library");
	home.push(name);
	home
}

#[cfg(windows)]
/// Get the config path for application `name`.
/// `name` should be capitalized, e.g. `"Ethereum"`, `"Parity"`.
pub fn config_path(name: &str) -> PathBuf {
	let mut home = home_dir().expect("Failed to get home dir");
	home.push("AppData");
	home.push("Roaming");
	home.push(name);
	home
}

#[cfg(not(any(target_os = "macos", windows)))]
/// Get the config path for application `name`.
/// `name` should be capitalized, e.g. `"Ethereum"`, `"Parity"`.
pub fn config_path(name: &str) -> PathBuf {
	let mut home = home_dir().expect("Failed to get home dir");
	home.push(format!(".{}", name.to_lowercase()));
	home
}

/// Get the specific folder inside a config path.
pub fn config_path_with(name: &str, then: &str) -> PathBuf {
	let mut path = config_path(name);
	path.push(then);
	path
}

/// Default ethereum paths
pub mod ethereum {
	use std::path::PathBuf;

	/// Default path for ethereum installation on Mac Os
	pub fn default() -> PathBuf {
		super::config_path("Ethereum")
	}

	/// Default path for ethereum installation (testnet)
	pub fn test() -> PathBuf {
		let mut path = default();
		path.push("testnet");
		path
	}

	/// Get the specific folder inside default ethereum installation
	pub fn with_default(s: &str) -> PathBuf {
		let mut path = default();
		path.push(s);
		path
	}

	/// Get the specific folder inside default ethereum installation configured for testnet
	pub fn with_testnet(s: &str) -> PathBuf {
		let mut path = default();
		path.push("testnet");
		path.push(s);
		path
	}
}

/// Restricts the permissions of given path only to the owner.
#[cfg(unix)]
pub fn restrict_permissions_owner(file_path: &Path, write: bool, executable: bool) -> Result<(), String> {
	let perms =
		::std::os::unix::fs::PermissionsExt::from_mode(0o400 + write as u32 * 0o200 + executable as u32 * 0o100);
	::std::fs::set_permissions(file_path, perms).map_err(|e| format!("{:?}", e))
}

/// Restricts the permissions of given path only to the owner.
#[cfg(not(unix))]
pub fn restrict_permissions_owner(_file_path: &Path, _write: bool, _executable: bool) -> Result<(), String> {
	//TODO: implement me
	Ok(())
}
