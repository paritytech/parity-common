use ethereum_types::{U128, H128};

#[test]
fn should_format_and_debug_correctly() {
    let test = |x: usize, hex: &'static str, display: &'static str| {
		let hash = H128::from(U128::from(x));
        assert_eq!(format!("{}", hash), format!("0x{}", display));
        assert_eq!(format!("{:?}", hash), format!("0x{}", hex));
        assert_eq!(format!("{:x}", hash), hex);
    };

    test(0x1, "00000000000000000000000000000001", "0000…0001");
    test(0xf, "0000000000000000000000000000000f", "0000…000f");
    test(0x10, "00000000000000000000000000000010", "0000…0010");
    test(0xff, "000000000000000000000000000000ff", "0000…00ff");
    test(0x100, "00000000000000000000000000000100", "0000…0100");
    test(0xfff, "00000000000000000000000000000fff", "0000…0fff");
    test(0x1000, "00000000000000000000000000001000", "0000…1000");
}

