//! Calldata builders for Uniswap V3's canonical `ISwapRouter`.
//!
//! This module supports the deadline-bearing, single-hop `exactInputSingle` and
//! `exactOutputSingle` entrypoints. It does not target the different
//! `SwapRouter02` ABI.

use bobcat_cd::{EvmCdAddress, EvmCdSerialise};
use bobcat_maths::U;

pub type Address = [u8; 20];
pub type U24 = [u8; 3];
pub type U160 = [u8; 20];

#[derive(EvmCdSerialise)]
#[evm_selector]
enum DerivedV3RouterCall {
    #[evm_selector(
        "exactInputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))"
    )]
    ExactInputSingle(
        EvmCdAddress,
        EvmCdAddress,
        u32,
        EvmCdAddress,
        U,
        U,
        U,
        EvmCdAddress,
    ),
    #[evm_selector(
        "exactOutputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))"
    )]
    ExactOutputSingle(
        EvmCdAddress,
        EvmCdAddress,
        u32,
        EvmCdAddress,
        U,
        U,
        U,
        EvmCdAddress,
    ),
}

fn u24_as_u32(value: U24) -> u32 {
    u32::from_be_bytes([0, value[0], value[1], value[2]])
}

/// Encode an exact-input swap through one Uniswap V3 pool.
#[allow(clippy::too_many_arguments)]
pub fn make_fn_exact_input_single(
    token_in: Address,
    token_out: Address,
    fee: U24,
    recipient: Address,
    deadline: U,
    amount_in: U,
    amount_out_minimum: U,
    sqrt_price_limit_x96: U160,
) -> [u8; 4 + 32 * 8] {
    DerivedV3RouterCall::ExactInputSingle(
        EvmCdAddress::new(token_in),
        EvmCdAddress::new(token_out),
        u24_as_u32(fee),
        EvmCdAddress::new(recipient),
        deadline,
        amount_in,
        amount_out_minimum,
        EvmCdAddress::new(sqrt_price_limit_x96),
    )
    .to_evm_array()
    .expect("exactInputSingle calldata has a fixed 260-byte encoding")
}

/// Encode an exact-output swap through one Uniswap V3 pool.
#[allow(clippy::too_many_arguments)]
pub fn make_fn_exact_output_single(
    token_in: Address,
    token_out: Address,
    fee: U24,
    recipient: Address,
    deadline: U,
    amount_out: U,
    amount_in_maximum: U,
    sqrt_price_limit_x96: U160,
) -> [u8; 4 + 32 * 8] {
    DerivedV3RouterCall::ExactOutputSingle(
        EvmCdAddress::new(token_in),
        EvmCdAddress::new(token_out),
        u24_as_u32(fee),
        EvmCdAddress::new(recipient),
        deadline,
        amount_out,
        amount_in_maximum,
        EvmCdAddress::new(sqrt_price_limit_x96),
    )
    .to_evm_array()
    .expect("exactOutputSingle calldata has a fixed 260-byte encoding")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_common_words(encoded: &[u8; 260]) {
        assert_eq!(&encoded[16..36], &[0x11; 20]);
        assert_eq!(&encoded[48..68], &[0x22; 20]);
        assert_eq!(&encoded[68..100], &{
            let mut word = [0; 32];
            word[29..].copy_from_slice(&[0x00, 0x01, 0xf4]);
            word
        });
        assert_eq!(&encoded[112..132], &[0x33; 20]);
        assert_eq!(encoded[163], 1);
        assert_eq!(encoded[195], 2);
        assert_eq!(encoded[227], 3);
        assert_eq!(&encoded[240..260], &[0x44; 20]);
    }

    #[test]
    fn exact_input_single_uses_fixed_derived_encoding() {
        let encoded = make_fn_exact_input_single(
            [0x11; 20],
            [0x22; 20],
            [0x00, 0x01, 0xf4],
            [0x33; 20],
            U::from_u8(1),
            U::from_u8(2),
            U::from_u8(3),
            [0x44; 20],
        );

        assert_eq!(&encoded[..4], &[0x41, 0x4b, 0xf3, 0x89]);
        assert_common_words(&encoded);
    }

    #[test]
    fn exact_output_single_uses_fixed_derived_encoding() {
        let encoded = make_fn_exact_output_single(
            [0x11; 20],
            [0x22; 20],
            [0x00, 0x01, 0xf4],
            [0x33; 20],
            U::from_u8(1),
            U::from_u8(2),
            U::from_u8(3),
            [0x44; 20],
        );

        assert_eq!(&encoded[..4], &[0xdb, 0x3e, 0x21, 0x98]);
        assert_common_words(&encoded);
    }
}
