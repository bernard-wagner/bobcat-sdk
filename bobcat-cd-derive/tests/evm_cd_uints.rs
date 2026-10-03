use bobcat_cd::serialisation::{EvmCdDeserialise as _, EvmCdSerialise as _};
use bobcat_cd::{EvmCdI24, EvmCdU248, I, U, const_keccak_sel};
use bobcat_cd_derive::{EvmCdDeserialise, EvmCdSerialise};

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
#[evm_selector]
enum Call {
    SetSigned(EvmCdI24),
    SetUnsigned(EvmCdU248),
}

macro_rules! check_uint_widths {
    ($(($bits:literal, $ty:ident, $abi:literal),)*) => {
        $(
            {
                const BYTES: usize = $bits / 8;
                let bytes = [0x5a; BYTES];
                let value = bobcat_cd::$ty::new(bytes);
                let mut encoded = Vec::new();
                value.serialise(&mut encoded).unwrap();

                assert_eq!(encoded.len(), 32);
                assert!(encoded[..32 - BYTES].iter().all(|byte| *byte == 0));
                assert_eq!(&encoded[32 - BYTES..], &bytes);
                assert_eq!(
                    bobcat_cd::$ty::deserialise_reader(&mut encoded.as_slice()).unwrap(),
                    value,
                );

                let word: U = value.into();
                let word_bytes: [u8; 32] = word.into();
                assert_eq!(&word_bytes[32 - BYTES..], &bytes);
                assert_eq!(bobcat_cd::$ty::from(word), value);

                if BYTES < 32 {
                    let mut wide = [0xa5; 32];
                    wide[32 - BYTES..].fill(0x3c);
                    assert_eq!(
                        bobcat_cd::$ty::from(U::from(wide)).into_array(),
                        [0x3c; BYTES]
                    );

                    let mut noncanonical = encoded;
                    noncanonical[0] = 1;
                    assert!(bobcat_cd::$ty::deserialise_reader(
                        &mut noncanonical.as_slice()
                    ).is_err());
                }
            }
        )*
    };
}

macro_rules! check_int_widths {
    ($(($bits:literal, $ty:ident, $abi:literal),)*) => {
        $(
            {
                const BYTES: usize = $bits / 8;
                for (byte, extension) in [(0x5a, 0x00), (0xa5, 0xff)] {
                    let bytes = [byte; BYTES];
                    let value = bobcat_cd::$ty::new(bytes);
                    let mut encoded = Vec::new();
                    value.serialise(&mut encoded).unwrap();

                    assert_eq!(encoded.len(), 32);
                    assert!(encoded[..32 - BYTES]
                        .iter()
                        .all(|actual| *actual == extension));
                    assert_eq!(&encoded[32 - BYTES..], &bytes);
                    assert_eq!(
                        bobcat_cd::$ty::deserialise_reader(&mut encoded.as_slice()).unwrap(),
                        value,
                    );

                    let word: I = value.into();
                    let word_bytes: [u8; 32] = word.into();
                    assert_eq!(word_bytes, encoded.as_slice());
                    assert_eq!(bobcat_cd::$ty::from(word), value);

                    if BYTES < 32 {
                        let mut wide = [!extension; 32];
                        wide[32 - BYTES..].fill(byte);
                        assert_eq!(
                            bobcat_cd::$ty::from(I::from(wide)).into_array(),
                            [byte; BYTES]
                        );

                        let mut noncanonical = encoded;
                        noncanonical[0] = !extension;
                        assert!(bobcat_cd::$ty::deserialise_reader(
                            &mut noncanonical.as_slice()
                        ).is_err());
                    }
                }
            }
        )*
    };
}

#[test]
fn every_non_native_solidity_uint_width_round_trips_and_converts_to_u() {
    check_uint_widths!(
        (24, EvmCdU24, b"uint24"),
        (40, EvmCdU40, b"uint40"),
        (48, EvmCdU48, b"uint48"),
        (56, EvmCdU56, b"uint56"),
        (72, EvmCdU72, b"uint72"),
        (80, EvmCdU80, b"uint80"),
        (88, EvmCdU88, b"uint88"),
        (96, EvmCdU96, b"uint96"),
        (104, EvmCdU104, b"uint104"),
        (112, EvmCdU112, b"uint112"),
        (120, EvmCdU120, b"uint120"),
        (136, EvmCdU136, b"uint136"),
        (144, EvmCdU144, b"uint144"),
        (152, EvmCdU152, b"uint152"),
        (160, EvmCdU160, b"uint160"),
        (168, EvmCdU168, b"uint168"),
        (176, EvmCdU176, b"uint176"),
        (184, EvmCdU184, b"uint184"),
        (192, EvmCdU192, b"uint192"),
        (200, EvmCdU200, b"uint200"),
        (208, EvmCdU208, b"uint208"),
        (216, EvmCdU216, b"uint216"),
        (224, EvmCdU224, b"uint224"),
        (232, EvmCdU232, b"uint232"),
        (240, EvmCdU240, b"uint240"),
        (248, EvmCdU248, b"uint248"),
        (256, EvmCdU256, b"uint256"),
    );
}

#[test]
fn every_non_native_solidity_int_width_round_trips_and_converts_to_i() {
    check_int_widths!(
        (24, EvmCdI24, b"int24"),
        (40, EvmCdI40, b"int40"),
        (48, EvmCdI48, b"int48"),
        (56, EvmCdI56, b"int56"),
        (72, EvmCdI72, b"int72"),
        (80, EvmCdI80, b"int80"),
        (88, EvmCdI88, b"int88"),
        (96, EvmCdI96, b"int96"),
        (104, EvmCdI104, b"int104"),
        (112, EvmCdI112, b"int112"),
        (120, EvmCdI120, b"int120"),
        (136, EvmCdI136, b"int136"),
        (144, EvmCdI144, b"int144"),
        (152, EvmCdI152, b"int152"),
        (160, EvmCdI160, b"int160"),
        (168, EvmCdI168, b"int168"),
        (176, EvmCdI176, b"int176"),
        (184, EvmCdI184, b"int184"),
        (192, EvmCdI192, b"int192"),
        (200, EvmCdI200, b"int200"),
        (208, EvmCdI208, b"int208"),
        (216, EvmCdI216, b"int216"),
        (224, EvmCdI224, b"int224"),
        (232, EvmCdI232, b"int232"),
        (240, EvmCdI240, b"int240"),
        (248, EvmCdI248, b"int248"),
        (256, EvmCdI256, b"int256"),
    );
}

#[test]
fn derived_selectors_recognise_generated_integer_types() {
    let calls = [
        (
            Call::SetSigned(EvmCdI24::new([0xff, 0xff, 0xfe])),
            const_keccak_sel(b"setSigned(int24)"),
        ),
        (
            Call::SetUnsigned(EvmCdU248::new([0x5a; 31])),
            const_keccak_sel(b"setUnsigned(uint248)"),
        ),
    ];

    for (call, selector) in calls {
        let mut encoded = Vec::new();
        call.serialise(&mut encoded).unwrap();
        assert_eq!(&encoded[..4], &selector);
        assert_eq!(
            Call::deserialise_reader(&mut encoded.as_slice()).unwrap(),
            call,
        );
    }
}
