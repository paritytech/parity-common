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


//! time wasm compat (mainly access to system time through `now`)

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::{ Instant, SystemTime, Duration, SystemTimeError };


#[cfg(all(target_arch = "wasm32", feature = "browser-wasm"))]
mod impl_browser {
	use std::ops::{ Deref, DerefMut, Add, AddAssign, Sub, SubAssign };
	use std::fmt;
	use std::time::{ Duration, SystemTime, SystemTimeError };

	// TODO bench but might be should efficient with internal f64
	#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct Instant(pub Duration);
	#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct SystemTimeB(pub SystemTime);

	fn websys_instant() -> Duration {
		let secs_f64 = web_sys::window().unwrap().performance().unwrap().now();
		Duration::from_float_secs(secs_f64)
	}

	fn jssys_from_epoch() -> Duration {
		let ms_f64 = js_sys::Date::now();
		Duration::from_millis(ms_f64 as u64)
	}

	impl Instant {
		/// see std::time::Instant;
		pub fn now() -> Instant {
			Instant(websys_instant())
		}
		/// see std::time::Instant;
		pub fn duration_since(&self, earlier: Instant) -> Duration {
			self.0.sub(earlier.0)
		}
		/// see std::time::Instant;
		pub fn elapsed(&self) -> Duration {
			Instant::now().0 - self.0
		}
	}

	impl Add<Duration> for Instant {
		type Output = Instant;

		fn add(self, other: Duration) -> Instant {
			Instant(self.0.add(other))
		}
	}

	impl AddAssign<Duration> for Instant {
		fn add_assign(&mut self, other: Duration) {
			self.0 = self.0 + other;
		}
	}

	impl Sub<Duration> for Instant {
		type Output = Instant;

		fn sub(self, other: Duration) -> Instant {
			Instant(self.0.sub(other))
		}
	}

	impl SubAssign<Duration> for Instant {
		fn sub_assign(&mut self, other: Duration) {
			self.0 = self.0 - other;
		}
	}

	impl Sub<Instant> for Instant {
		type Output = Duration;

		fn sub(self, other: Instant) -> Duration {
			self.duration_since(other)
		}
	}

	impl fmt::Debug for Instant {
		fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
			self.0.fmt(f)
		}
	}

	impl SystemTimeB {
		/// see std::time::SystemTime;
		pub const UNIX_EPOCH: SystemTimeB = SystemTimeB(SystemTime::UNIX_EPOCH);

		/// see std::time::SystemTime;
		pub fn now() -> SystemTimeB {
			let now = SystemTime::UNIX_EPOCH + jssys_from_epoch();
			SystemTimeB(now)
		}

		/// see std::time::SystemTime;
		pub fn elapsed(&self) -> Result<Duration, SystemTimeError> {
			SystemTimeB::now().duration_since(self.0)
		}
	}

	impl Deref for SystemTimeB {
		type Target = SystemTime;

		fn deref(&self) -> &SystemTime {
			&self.0
		}
	}
	impl DerefMut for SystemTimeB {
		fn deref_mut(&mut self) -> &mut SystemTime {
			&mut self.0
		}
	}

	impl fmt::Debug for SystemTimeB {
		fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
			self.0.fmt(f)
		}
	}
}

#[cfg(all(target_arch = "wasm32", feature = "browser-wasm"))]
pub use self::impl_browser::{ Instant, SystemTimeB as SystemTime };
#[cfg(all(target_arch = "wasm32", feature = "browser-wasm"))]
pub use std::time::{ Duration };
