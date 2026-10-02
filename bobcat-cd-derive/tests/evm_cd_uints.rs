use bobcat_cd::serialisation::{EvmCdDeserialise as _, EvmCdSerialise as _};
use bobcat_cd::{EvmCdU24, EvmCdU192, const_keccak_sel};
use bobcat_cd_derive::{EvmCdDeserialise, EvmCdSerialise};

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
#[evm_selector]
enum Call {
    SetFee(EvmCdU24),
    SetLimit(EvmCdU192),
}

#[test]
fn uint_wrappers_use_canonical_abi_types_and_round_trip() {
    let cases = [
        (
            Call::SetFee(EvmCdU24::new([0x12, 0x34, 0x56])),
            const_keccak_sel(b"setFee(uint24)"),
            29,
            vec![0x12, 0x34, 0x56],
        ),
        (
            Call::SetLimit(EvmCdU192::new([0xab; 24])),
            const_keccak_sel(b"setLimit(uint192)"),
            8,
            vec![0xab; 24],
        ),
    ];

    for (call, selector, padding_len, value) in cases {
        let mut encoded = Vec::new();
        call.serialise(&mut encoded).unwrap();

        assert_eq!(&encoded[..4], &selector);
        assert!(encoded[4..4 + padding_len].iter().all(|byte| *byte == 0));
        assert_eq!(&encoded[4 + padding_len..], value);
        assert_eq!(
            Call::deserialise_reader(&mut encoded.as_slice()).unwrap(),
            call
        );
    }
}

#[test]
fn uint_wrappers_reject_noncanonical_padding() {
    let mut uint24 = [0u8; 36];
    uint24[..4].copy_from_slice(&const_keccak_sel(b"setFee(uint24)"));
    uint24[4] = 1;
    assert!(Call::deserialise_reader(&mut uint24.as_slice()).is_err());

    let mut uint192 = [0u8; 36];
    uint192[..4].copy_from_slice(&const_keccak_sel(b"setLimit(uint192)"));
    uint192[4] = 1;
    assert!(Call::deserialise_reader(&mut uint192.as_slice()).is_err());
}

#[test]
fn uint_wrappers_expose_their_underlying_arrays() {
    let small = [1, 2, 3];
    let large = [4; 24];

    assert_eq!(EvmCdU24::from(small).into_array(), small);
    assert_eq!(EvmCdU192::from(large).into_array(), large);
}
