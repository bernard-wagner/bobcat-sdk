use bobcat_cd::{
    EvmCdBytes, EvmCdBytes0, EvmCdBytes64, EvmCdBytes1024, EvmCdDeserialise, EvmCdSerialise,
    const_keccak_sel,
};

#[derive(
    Debug, PartialEq, Eq, bobcat_cd_derive::EvmCdSerialise, bobcat_cd_derive::EvmCdDeserialise,
)]
struct BytesValue {
    bytes: EvmCdBytes64,
}

#[derive(bobcat_cd_derive::EvmCdSerialise)]
#[evm_selector]
enum BytesCall {
    Store(EvmCdBytes64),
}

#[derive(bobcat_cd_derive::EvmCdSerialise)]
#[evm_selector]
enum GenericBytesCall<const CAP: usize> {
    Store(EvmCdBytes<CAP>),
}

#[derive(bobcat_cd_derive::EvmCdSerialise)]
#[evm_selector]
enum ConcreteGenericBytesCall {
    Store(EvmCdBytes<64>),
}

#[derive(bobcat_cd_derive::EvmCdSerialise)]
#[evm_selector]
enum QualifiedBytesCall {
    Store(bobcat_cd::EvmCdBytes64),
}

#[derive(bobcat_cd_derive::EvmCdSerialise)]
#[evm_selector]
enum EmptyBytesCall {
    Store(EvmCdBytes0),
}

#[test]
fn fixed_capacity_bytes_encode_like_vec_without_allocating() {
    let value = EvmCdBytes64::try_from_slice(&[0xab, 0xcd]).unwrap();
    let mut fixed = [0u8; 96];
    value.serialise(&mut fixed).unwrap();

    let mut allocated = Vec::new();
    vec![0xabu8, 0xcd].serialise(&mut allocated).unwrap();

    assert_eq!(fixed.as_slice(), allocated);
    assert_eq!(&fixed[..32], &{
        let mut word = [0u8; 32];
        word[31] = 32;
        word
    });
    assert_eq!(fixed[63], 2);
    assert_eq!(&fixed[64..66], &[0xab, 0xcd]);
    assert_eq!(&fixed[66..], &[0; 30]);
}

#[test]
fn fixed_capacity_bytes_call_has_a_compile_time_capacity_array() {
    let bytes = EvmCdBytes64::try_from_slice(&[1, 2, 3]).unwrap();
    let encoded: [u8; 132] = BytesCall::Store(bytes).to_evm_array().unwrap();

    assert_eq!(&encoded[..4], &const_keccak_sel(b"store(bytes)"));
    assert_eq!(encoded[35], 32);
    assert_eq!(encoded[67], 3);
    assert_eq!(&encoded[68..71], &[1, 2, 3]);
    assert!(encoded[71..].iter().all(|byte| *byte == 0));

    let generic_capacity: [u8; 132] =
        GenericBytesCall::<64>::Store(EvmCdBytes::<64>::try_from_slice(&[1, 2, 3]).unwrap())
            .to_evm_array::<132>()
            .unwrap();
    assert_eq!(generic_capacity, encoded);

    let generic: [u8; 132] =
        ConcreteGenericBytesCall::Store(EvmCdBytes::<64>::try_from_slice(&[1, 2, 3]).unwrap())
            .to_evm_array()
            .unwrap();
    assert_eq!(generic, encoded);
}

#[test]
fn aliases_cover_zero_through_1024() {
    let empty = EvmCdBytes0::new();
    let largest = EvmCdBytes1024::try_from_slice(&[7; 1024]).unwrap();

    assert!(empty.is_empty());
    assert_eq!(largest.len(), 1024);
    assert_eq!(largest.capacity(), 1024);
    assert!(EvmCdBytes::<0>::try_from_slice(&[1]).is_err());
    assert!(EvmCdBytes64::try_from_slice(&[1; 65]).is_err());
}

#[test]
fn zero_capacity_bytes_round_trip_as_empty_dynamic_bytes() {
    let value = EvmCdBytes0::new();
    let mut encoded = [0u8; 64];
    value.serialise(&mut encoded).unwrap();

    assert_eq!(encoded[31], 32);
    assert_eq!(EvmCdBytes0::deserialise(&encoded).unwrap(), value);
}

#[test]
fn zero_capacity_bytes_call_has_a_compile_time_sized_array() {
    let encoded: [u8; 68] = EmptyBytesCall::Store(EvmCdBytes0::new())
        .to_evm_array()
        .unwrap();

    assert_eq!(&encoded[..4], &const_keccak_sel(b"store(bytes)"));
    assert_eq!(encoded[35], 32);
    assert!(encoded[36..].iter().all(|byte| *byte == 0));
}

#[test]
fn deserialisation_rejects_a_length_larger_than_capacity() {
    let value = EvmCdBytes64::try_from_slice(&[1, 2, 3]).unwrap();
    let mut encoded = [0u8; 96];
    value.serialise(&mut encoded).unwrap();
    encoded[63] = 65;

    assert!(EvmCdBytes64::deserialise(&encoded).is_err());
}

#[test]
fn fixed_capacity_bytes_are_an_allocation_free_write_target() {
    let mut bytes = EvmCdBytes64::new();
    7u8.serialise(&mut bytes).unwrap();
    9u8.serialise(&mut bytes).unwrap();

    assert_eq!(bytes.len(), 64);
    assert_eq!(bytes.as_slice()[31], 7);
    assert_eq!(bytes.as_slice()[63], 9);
    assert!(1u8.serialise(&mut bytes).is_err());
}

#[test]
fn clearing_removes_stale_bytes_from_value_semantics() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut reused = EvmCdBytes64::try_from_slice(&[1, 2]).unwrap();
    reused.clear();
    reused.extend_from_slice(&[1]).unwrap();
    let fresh = EvmCdBytes64::try_from_slice(&[1]).unwrap();

    let hash = |value: &EvmCdBytes64| {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    };
    assert_eq!(reused, fresh);
    assert_eq!(hash(&reused), hash(&fresh));
}

#[test]
fn fixed_capacity_bytes_round_trip_as_dynamic_bytes() {
    let value = BytesValue {
        bytes: EvmCdBytes64::try_from_slice(&[1, 2, 3]).unwrap(),
    };
    let mut encoded = [0u8; 96];
    value.serialise(&mut encoded).unwrap();

    assert_eq!(BytesValue::deserialise(&encoded).unwrap(), value);
}

#[test]
fn fixed_capacity_bytes_resolve_to_the_bytes_selector_type() {
    let bytes = EvmCdBytes64::try_from_slice(&[1, 2, 3]).unwrap();
    let expected_selector = const_keccak_sel(b"store(bytes)");

    let mut alias_encoded = [0u8; 100];
    BytesCall::Store(bytes.clone())
        .serialise(&mut alias_encoded)
        .unwrap();
    let mut generic_encoded = [0u8; 100];
    GenericBytesCall::<64>::Store(bytes.clone())
        .serialise(&mut generic_encoded)
        .unwrap();
    let mut qualified_encoded = [0u8; 100];
    QualifiedBytesCall::Store(bytes)
        .serialise(&mut qualified_encoded)
        .unwrap();

    assert_eq!(&alias_encoded[..4], &expected_selector);
    assert_eq!(&generic_encoded[..4], &expected_selector);
    assert_eq!(&qualified_encoded[..4], &expected_selector);
}

#[test]
fn generic_fixed_capacity_bytes_have_the_same_runtime_type() {
    let value: EvmCdBytes<64> = EvmCdBytes::try_from_slice(&[1, 2, 3]).unwrap();
    let alias = EvmCdBytes64::try_from_slice(&[1, 2, 3]).unwrap();

    assert_eq!(value, alias);
}
