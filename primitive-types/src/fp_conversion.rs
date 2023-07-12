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
		// Reference: https://blog.m-ou.se/floats/
		// Step 1: Get leading zeroes
		let leading_zeroes = self.leading_zeros();
		// Step 2: Get msb to be farthest left bit
		let left_aligned = self << leading_zeroes;
		// Step 3: Shift msb to fit in lower 53 bits of the first u64 (64-53=11)
		let quarter_aligned = left_aligned >> 11;
		let mantissa = quarter_aligned.0[3];
		// Step 4: For the dropped bits (all bits beyond the 53 most significant
		// We want to know only 2 things. If the msb of the dropped bits is 1 or 0,
		// and if any of the other bits are 1. (See blog for explanation)
		// So we take care to preserve the msb bit, while jumbling the rest of the bits
		// together so that any 1s will survive. If all 0s, then the result will also be 0.
		let dropped_bits = quarter_aligned.0[1] | quarter_aligned.0[0] | (left_aligned.0[0] & 0xFFFF_FFFF);
		let dropped_bits = (dropped_bits & 0x7FFF_FFFF_FFFF_FFFF) | (dropped_bits >> 63);
		let dropped_bits = quarter_aligned.0[2] | dropped_bits;
		// Step 5: dropped_bits contains the msb of the original bits and an OR-mixed 63 bits.
		// If msb of dropped bits is 0, it is mantissa + 0
		// If msb of dropped bits is 1, it is mantissa + 0 only if mantissa lowest bit is 0
		// and other bits of the dropped bits are all 0 (which both can be tested with the below all at once)
		let mantissa = mantissa + ((dropped_bits - (dropped_bits >> 63 & !mantissa)) >> 63);
		// Step 6: Calculate the exponent
		// If self is 0, exponent should be 0 (special meaning) and mantissa will end up 0 too
		// Otherwise, (255 - n) + 1022 so it simplifies to 1277 - n
		// 1023 and 1022 are the cutoffs for the exponent having the msb next to the decimal point
		let exponent = if self.is_zero() { 0 } else { 1277 - leading_zeroes as u64 };
		// Step 7: sign bit is always 0, exponent is shifted into place
		// Use addition instead of bitwise OR to saturate the exponent if mantissa overflows
		f64::from_bits((exponent << 52) + mantissa)
	}
}
