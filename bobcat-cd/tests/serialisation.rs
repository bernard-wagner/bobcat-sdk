use bobcat_cd::serialisation::SelectorHasher;
use bobcat_cd::{
    EvmCdAddress, EvmCdArray, EvmCdArrayError, EvmCdDeserialise, EvmCdSerialise, EvmCdString, U,
    const_keccak_sel,
};

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
#[evm_selector]
enum Call {
    Store(EvmCdAddress, EvmCdArray<u16, 0, 3>),
    SetCount(usize),
    SetSigned(i16),
    SetEnabled(bool),
}

#[cfg(feature = "alloc")]
#[test]
fn serialises_directly_into_a_mut_vec() {
    let mut encoded = vec![0xff];

    0x1234u16.serialise(&mut encoded).unwrap();

    assert_eq!(encoded.len(), 33);
    assert_eq!(encoded[0], 0xff, "serialisation appends to the vector");
    assert!(encoded[1..31].iter().all(|byte| *byte == 0));
    assert_eq!(&encoded[31..], &[0x12, 0x34]);
}

#[test]
fn address_converts_into_a_left_padded_u() {
    let mut bytes = [0u8; 20];
    bytes[0] = 0x12;
    bytes[19] = 0x34;

    let word: U = EvmCdAddress::new(bytes).into();
    let word: [u8; 32] = word.into();

    assert_eq!(&word[..12], &[0; 12]);
    assert_eq!(&word[12..], &bytes);
}

#[test]
fn serialises_to_an_exact_array() {
    let encoded = 0x1234u16.serialise_to_array::<32>().unwrap();

    assert!(encoded[..30].iter().all(|byte| *byte == 0));
    assert_eq!(&encoded[30..], &[0x12, 0x34]);
    assert!(0x1234u16.serialise_to_array::<31>().is_err());
    assert!(0x1234u16.serialise_to_array::<33>().is_err());
}

#[cfg(feature = "alloc")]
#[test]
fn serialises_to_a_new_vec() {
    let encoded = 0x1234u16.serialise_to_vec().unwrap();

    assert_eq!(encoded.len(), 32);
    assert!(encoded[..30].iter().all(|byte| *byte == 0));
    assert_eq!(&encoded[30..], &[0x12, 0x34]);
}

#[test]
fn selector_hasher_supports_signatures_larger_than_512_bytes() {
    let signature = [b'a'; 600];
    let selector = SelectorHasher::new().update(&signature).selector();

    assert_eq!(selector, const_keccak_sel(&signature));
}

fn assert_builtin_buffer_fits<T>(value: &T)
where
    T: EvmCdSerialise + EvmCdDeserialise,
{
    let mut encoded = Vec::new();
    value.serialise(&mut encoded).unwrap();
    let buffer = T::new_buffer(encoded.len()).unwrap();
    assert!(buffer.as_ref().len() >= encoded.len());
}

#[test]
fn built_in_deserialisers_provide_sufficient_buffers() {
    assert_builtin_buffer_fits(&true);
    assert_builtin_buffer_fits(&7u8);
    assert_builtin_buffer_fits(&u32::MAX);
    assert_builtin_buffer_fits(&(u32::MAX as usize));
    assert_builtin_buffer_fits(&bobcat_cd::U::from_u32(11));
    assert_builtin_buffer_fits(&EvmCdAddress::from([0xabu8; 20]));
    assert_builtin_buffer_fits(b"bytes");

    let text = EvmCdString::<0, 100>::try_from("Cerberus").unwrap();
    assert_builtin_buffer_fits(&text);

    let values = EvmCdArray::<u16, 0, 3>::try_from_slice(&[3, 5, 7]).unwrap();
    assert_builtin_buffer_fits(&values);

    assert!(u8::new_buffer(33).is_err());
}

#[test]
fn bool_uses_canonical_abi_encoding_and_selector() {
    for value in [false, true] {
        let call = Call::SetEnabled(value);
        let mut encoded = Vec::new();
        call.serialise(&mut encoded).unwrap();

        assert_eq!(&encoded[..4], &const_keccak_sel(b"setEnabled(bool)"));
        assert!(encoded[4..35].iter().all(|byte| *byte == 0));
        assert_eq!(encoded[35], u8::from(value));
        assert_eq!(
            Call::deserialise_reader(&mut encoded.as_slice()).unwrap(),
            call
        );
    }
}

#[test]
fn bool_rejects_noncanonical_abi_words() {
    let selector = const_keccak_sel(b"setEnabled(bool)");

    let mut value_two = [0u8; 36];
    value_two[..4].copy_from_slice(&selector);
    value_two[35] = 2;
    assert!(Call::deserialise_reader(&mut value_two.as_slice()).is_err());

    let mut nonzero_padding = [0u8; 36];
    nonzero_padding[..4].copy_from_slice(&selector);
    nonzero_padding[4] = 1;
    nonzero_padding[35] = 1;
    assert!(Call::deserialise_reader(&mut nonzero_padding.as_slice()).is_err());
}

#[cfg(feature = "alloc")]
#[test]
fn allocated_deserialisers_size_their_buffer_at_runtime() {
    let values = vec![3u16, 5, 7];
    assert_builtin_buffer_fits(&values);
    assert_eq!(Vec::<u16>::new_buffer(123).unwrap().as_ref().len(), 123);
}

#[test]
fn usize_uses_uint32_abi_encoding() {
    let call = Call::SetCount(u32::MAX as usize);
    let mut encoded = Vec::new();
    call.serialise(&mut encoded).unwrap();

    assert_eq!(&encoded[..4], &const_keccak_sel(b"setCount(uint32)"));
    assert!(encoded[4..32].iter().all(|byte| *byte == 0));
    assert_eq!(&encoded[32..36], &u32::MAX.to_be_bytes());
    assert_eq!(
        Call::deserialise_reader(&mut encoded.as_slice()).unwrap(),
        call
    );

    let mut noncanonical = encoded;
    noncanonical[4] = 1;
    assert!(Call::deserialise_reader(&mut noncanonical.as_slice()).is_err());

    if usize::BITS > 32 {
        let mut output = Vec::new();
        assert!(
            Call::SetCount(u32::MAX as usize + 1)
                .serialise(&mut output)
                .is_err()
        );
    }
}

#[test]
fn signed_rust_integers_use_canonical_abi_encoding() {
    for value in [i16::MIN, -2, 0, 1, i16::MAX] {
        let call = Call::SetSigned(value);
        let mut encoded = Vec::new();
        call.serialise(&mut encoded).unwrap();

        assert_eq!(&encoded[..4], &const_keccak_sel(b"setSigned(int16)"));
        let extension = if value.is_negative() { 0xff } else { 0 };
        assert!(encoded[4..34].iter().all(|byte| *byte == extension));
        assert_eq!(&encoded[34..], &value.to_be_bytes());
        assert_eq!(
            Call::deserialise_reader(&mut encoded.as_slice()).unwrap(),
            call
        );
    }

    let mut noncanonical = [0u8; 32];
    noncanonical[0] = 0xff;
    noncanonical[30..].copy_from_slice(&1i16.to_be_bytes());
    assert!(i16::deserialise_reader(&mut noncanonical.as_slice()).is_err());
}

#[test]
fn address_is_left_padded_and_uses_the_address_abi_type() {
    let address = EvmCdAddress::from([0xabu8; 20]);
    let values = EvmCdArray::try_from_array([7, 9, 0], 2).unwrap();
    let call = Call::Store(address, values);
    let mut encoded = Vec::new();
    call.serialise(&mut encoded).unwrap();

    assert_eq!(&encoded[..4], &const_keccak_sel(b"store(address,uint16[])"));
    assert!(encoded[4..16].iter().all(|byte| *byte == 0));
    assert_eq!(&encoded[16..36], &[0xab; 20]);
    assert_eq!(
        encoded[67], 64,
        "array tail starts after both argument heads"
    );
    assert_eq!(encoded[99], 2);
    assert_eq!(encoded[131], 7);
    assert_eq!(encoded[163], 9);
    assert_eq!(
        Call::deserialise_reader(&mut encoded.as_slice()).unwrap(),
        call
    );
}

#[test]
fn fixed_capacity_dynamic_array_enforces_bounds_without_allocating() {
    type Values = EvmCdArray<u16, 1, 3>;

    assert_eq!(
        Values::try_from_array([0, 0, 0], 0),
        Err(EvmCdArrayError::TooShort),
    );
    assert_eq!(
        Values::try_from_array([0, 0, 0], 4),
        Err(EvmCdArrayError::TooLong),
    );
    assert_eq!(
        EvmCdArray::<u16, 4, 3>::try_from_array([0, 0, 0], 3),
        Err(EvmCdArrayError::InvalidBounds),
    );

    let values = Values::try_from_slice(&[3, 5]).unwrap();
    assert_eq!(values.as_slice(), &[3, 5]);
    assert_eq!(values.len(), 2);
    assert_eq!(values.capacity(), 3);
}

#[test]
fn fixed_capacity_array_supports_dynamic_elements() {
    type Text = EvmCdString<0, 4>;
    type Texts = EvmCdArray<Text, 0, 2>;

    let values = Texts::try_from_array(
        [Text::try_from("a").unwrap(), Text::try_from("bc").unwrap()],
        2,
    )
    .unwrap();
    let mut encoded = Vec::new();
    values.serialise(&mut encoded).unwrap();

    assert_eq!(encoded.len(), 256);
    assert_eq!(encoded[31], 32, "standalone array head points at its tail");
    assert_eq!(encoded[63], 2);
    assert_eq!(
        encoded[95], 64,
        "first string offset follows both element heads"
    );
    assert_eq!(encoded[127], 128);
    assert_eq!(encoded[159], 1);
    assert_eq!(encoded[160], b'a');
    assert_eq!(encoded[223], 2);
    assert_eq!(&encoded[224..226], b"bc");

    let decoded = Texts::deserialise_reader(&mut encoded.as_slice()).unwrap();
    assert_eq!(decoded, values);
}

#[cfg(feature = "alloc")]
#[test]
fn allocated_vec_supports_dynamic_elements() {
    type Text = EvmCdString<0, 4>;

    let values = vec![Text::try_from("a").unwrap(), Text::try_from("bc").unwrap()];
    let mut encoded = Vec::new();
    values.serialise(&mut encoded).unwrap();

    assert_eq!(encoded.len(), 256);
    assert_eq!(encoded[31], 32, "standalone array head points at its tail");
    assert_eq!(encoded[63], 2);
    assert_eq!(encoded[95], 64, "offsets are relative to after the length");
    assert_eq!(encoded[127], 128);
    assert_eq!(encoded[159], 1);
    assert_eq!(encoded[160], b'a');
    assert_eq!(encoded[223], 2);
    assert_eq!(&encoded[224..226], b"bc");

    let decoded = Vec::<Text>::deserialise_reader(&mut encoded.as_slice()).unwrap();
    assert_eq!(decoded, values);
}

#[cfg(feature = "alloc")]
#[test]
fn allocated_vec_supports_nested_arrays_and_bytes() {
    let arrays = vec![vec![7u16, 9], vec![11u16]];
    let mut encoded = Vec::new();
    arrays.serialise(&mut encoded).unwrap();

    assert_eq!(encoded.len(), 288);
    assert_eq!(encoded[31], 32);
    assert_eq!(encoded[63], 2);
    assert_eq!(encoded[95], 64);
    assert_eq!(encoded[127], 160);
    assert_eq!(encoded[159], 2);
    assert_eq!(encoded[191], 7);
    assert_eq!(encoded[223], 9);
    assert_eq!(encoded[255], 1);
    assert_eq!(encoded[287], 11);
    assert_eq!(
        Vec::<Vec<u16>>::deserialise_reader(&mut encoded.as_slice()).unwrap(),
        arrays
    );

    let blobs = vec![vec![0xaau8], vec![0xbbu8, 0xcc]];
    let mut encoded = Vec::new();
    blobs.serialise(&mut encoded).unwrap();
    assert_eq!(
        Vec::<Vec<u8>>::deserialise_reader(&mut encoded.as_slice()).unwrap(),
        blobs
    );
}

#[cfg(feature = "alloc")]
#[test]
fn allocated_vec_rejects_lengths_exceeding_the_decode_budget() {
    let mut encoded = [0u8; 64];
    encoded[31] = 32;
    encoded[56..64].copy_from_slice(&524_289u64.to_be_bytes());

    assert!(Vec::<u16>::deserialise(&encoded).is_err());
}
