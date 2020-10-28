use super::U256;

impl U256 {
	/// Lossy saturating conversion from a `f64` to a `U256`. Like for floating point to
	/// primitive integer type conversions, this truncates fractional parts.
	///
	/// The conversion follows the same rules as converting `f64` to other
	/// primitive integer types. Namely, the conversion of `value: f64` behaves as
	/// follows:
	/// - `NaN` => `0`
	/// - `(-∞, 0]` => `0`
	/// - `(0, u256::MAX]` => `value as u256`
	/// - `(u256::MAX, +∞)` => `u256::MAX`
	pub fn from_f64_lossy(value: f64) -> U256 {
		if value >= 1.0 {
			let bits = value.to_bits();
			// NOTE: Don't consider the sign or check that the subtraction will
			//   underflow since we already checked that the value is greater
			//   than 1.0.
			let exponent = ((bits >> 52) & 0x7ff) - 1023;
			let mantissa = (bits & 0x0f_ffff_ffff_ffff) | 0x10_0000_0000_0000;
			if exponent <= 52 {
				U256::from(mantissa >> (52 - exponent))
			} else if exponent >= 256 {
				U256::MAX
			} else {
				U256::from(mantissa) << U256::from(exponent - 52)
			}
		} else {
			0.into()
		}
	}

	/// Lossy conversion of `U256` to `f64`.
	pub fn to_f64_lossy(self) -> f64 {
		let (res, factor) = match self {
			U256([_, _, 0, 0]) => (self, 1.0),
			U256([_, _, _, 0]) => (self >> 64, 2.0f64.powi(64)),
			U256([_, _, _, _]) => (self >> 128, 2.0f64.powi(128)),
		};
		(res.low_u128() as f64) * factor
	}
}
