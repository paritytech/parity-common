use primitive_types::{Error, U128, U256, U512, U64};


#[test]
#[allow(non_snake_case)]
fn from_U64() {
    let a = U64::from(222);

    assert_eq!(U128::from(a), U128::from(222), "U64 -> U128");
    assert_eq!(U256::from(a), U256::from(222), "U64 -> U256");
    assert_eq!(U512::from(a), U512::from(222), "U64 -> U512");		
}

#[test]
#[allow(non_snake_case)]
fn to_U64() {
    // U128 -> U64
    assert_eq!(U64::try_from(U128([222, 0])), Ok(U64::from(222)), "U128 -> U64");
    assert_eq!(U64::try_from(U128([222, 1])), Err(Error::Overflow), "U128 -> U64 :: Overflow");

    // U256 -> U64
    assert_eq!(U64::try_from(U256([222, 0, 0, 0])), Ok(U64::from(222)), "U256 -> U64");
    for i in 1..4 {
        let mut arr = [222, 0, 0, 0];
        arr[i] = 1;
        assert_eq!(U64::try_from(U256(arr)), Err(Error::Overflow), "U256 -> U64 :: Overflow");
    }

    // U512 -> U64
    assert_eq!(U64::try_from(U512([222, 0, 0, 0, 0, 0, 0, 0])),Ok(U64::from(222)), "U512 -> U64");
    for i in 1..8 {
        let mut arr = [222, 0, 0, 0, 0, 0, 0, 0];
        arr[i] = 1;
        assert_eq!(U64::try_from(U512(arr)), Err(Error::Overflow), "U512 -> U64");
    }
}

#[test]
#[allow(non_snake_case)]
fn full_mul_U64() {
    let a = U64::MAX;
    let b = U64::from(2);
    assert_eq!(a.full_mul(b), U128::from(a)*U128::from(b))
}