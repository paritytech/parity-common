use U256;

impl_hash!(H32, 4);
impl_hash!(H64, 8);
impl_hash!(H128, 16);
impl_hash!(H160, 20);
impl_hash!(H256, 32);
impl_hash!(H264, 33);
impl_hash!(H512, 64);
impl_hash!(H520, 65);
impl_hash!(H1024, 128);
impl_hash!(H2048, 256);

impl From<U256> for H256 {
	fn from(value: U256) -> H256 {
		let mut ret = H256::new();
		value.to_big_endian(&mut ret);
		ret
	}
}

impl<'a> From<&'a U256> for H256 {
	fn from(value: &'a U256) -> H256 {
		let mut ret: H256 = H256::new();
		value.to_big_endian(&mut ret);
		ret
	}
}

impl From<H256> for U256 {
	fn from(value: H256) -> U256 {
		U256::from(&value)
	}
}

impl<'a> From<&'a H256> for U256 {
	fn from(value: &'a H256) -> U256 {
		U256::from(value.as_ref() as &[u8])
	}
}

impl From<H256> for H160 {
	fn from(value: H256) -> H160 {
		let mut ret = H160::new();
		ret.0.copy_from_slice(&value[12..32]);
		ret
	}
}

impl From<H256> for H64 {
	fn from(value: H256) -> H64 {
		let mut ret = H64::new();
		ret.0.copy_from_slice(&value[20..28]);
		ret
	}
}

impl From<H160> for H256 {
	fn from(value: H160) -> H256 {
		let mut ret = H256::new();
		ret.0[12..32].copy_from_slice(&value);
		ret
	}
}

impl<'a> From<&'a H160> for H256 {
	fn from(value: &'a H160) -> H256 {
		let mut ret = H256::new();
		ret.0[12..32].copy_from_slice(value);
		ret
	}
}

