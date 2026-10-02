//! Calldata builders for Uniswap V2 Router02 exact-input swaps.
//!
//! Fixed builders own const-sized paths and return exact arrays without allocation.
//! `alloc`-gated variants own vectors and return dynamically sized calldata.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use bobcat_cd::{EvmCdAddress, EvmCdArray, EvmCdSerialise};
use bobcat_maths::U;

/// An EVM address.
pub type Address = [u8; 20];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SwapExactTokensForTokensParams<const PATH_LEN: usize> {
    pub amount_in: U,
    pub amount_out_min: U,
    pub path: [Address; PATH_LEN],
    pub to: Address,
    pub deadline: U,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SwapExactEthForTokensParams<const PATH_LEN: usize> {
    pub amount_out_min: U,
    pub path: [Address; PATH_LEN],
    pub to: Address,
    pub deadline: U,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SwapExactTokensForEthParams<const PATH_LEN: usize> {
    pub amount_in: U,
    pub amount_out_min: U,
    pub path: [Address; PATH_LEN],
    pub to: Address,
    pub deadline: U,
}

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapExactTokensForTokensParamsVec {
    pub amount_in: U,
    pub amount_out_min: U,
    pub path: Vec<Address>,
    pub to: Address,
    pub deadline: U,
}

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapExactEthForTokensParamsVec {
    pub amount_out_min: U,
    pub path: Vec<Address>,
    pub to: Address,
    pub deadline: U,
}

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapExactTokensForEthParamsVec {
    pub amount_in: U,
    pub amount_out_min: U,
    pub path: Vec<Address>,
    pub to: Address,
    pub deadline: U,
}

#[derive(EvmCdSerialise)]
#[evm_selector]
enum DerivedV2RouterCall<Path> {
    #[evm_selector("swapExactTokensForTokens(uint256,uint256,address[],address,uint256)")]
    SwapExactTokensForTokens(U, U, Path, EvmCdAddress, U),
    #[evm_selector("swapExactETHForTokens(uint256,address[],address,uint256)")]
    SwapExactEthForTokens(U, Path, EvmCdAddress, U),
    #[evm_selector("swapExactTokensForETH(uint256,uint256,address[],address,uint256)")]
    SwapExactTokensForEth(U, U, Path, EvmCdAddress, U),
}

const fn calldata_len(head_words: usize, path_len: usize) -> usize {
    assert!(
        path_len >= 2,
        "Uniswap V2 path must contain at least two addresses"
    );
    let Some(words) = path_len.checked_add(head_words + 1) else {
        panic!("Uniswap V2 path length overflows calldata length");
    };
    let Some(bytes) = words.checked_mul(32) else {
        panic!("Uniswap V2 path length overflows calldata length");
    };
    let Some(bytes) = bytes.checked_add(4) else {
        panic!("Uniswap V2 path length overflows calldata length");
    };
    bytes
}

pub const fn swap_exact_tokens_for_tokens_calldata_len(n: usize) -> usize {
    calldata_len(5, n)
}

pub const fn swap_exact_eth_for_tokens_calldata_len(n: usize) -> usize {
    calldata_len(4, n)
}

pub const fn swap_exact_tokens_for_eth_calldata_len(n: usize) -> usize {
    calldata_len(5, n)
}

fn derived_path_array<const PATH_LEN: usize>(
    path: [Address; PATH_LEN],
) -> EvmCdArray<EvmCdAddress, PATH_LEN, PATH_LEN> {
    EvmCdArray::try_from_array(path.map(EvmCdAddress::new), PATH_LEN)
        .expect("the path fills its const-sized array")
}

#[cfg(feature = "alloc")]
fn derived_path_vec(path: Vec<Address>) -> Vec<EvmCdAddress> {
    path.into_iter().map(EvmCdAddress::new).collect()
}

fn encode_array<const ALL: usize>(call: &impl EvmCdSerialise, expected: usize) -> [u8; ALL] {
    assert_eq!(ALL, expected, "calldata output array has the wrong length");
    let mut output = [0; ALL];
    call.serialise(&mut output)
        .expect("Uniswap V2 calldata serialization failed");
    output
}

#[cfg(feature = "alloc")]
fn encode_vec(call: &impl EvmCdSerialise) -> Vec<u8> {
    let mut output = Vec::new();
    call.serialise(&mut output)
        .expect("Uniswap V2 calldata serialization failed");
    output
}

pub fn make_fn_swap_exact_tokens_for_tokens_array<const PATH_LEN: usize, const ALL: usize>(
    params: SwapExactTokensForTokensParams<PATH_LEN>,
) -> [u8; ALL] {
    let required = swap_exact_tokens_for_tokens_calldata_len(PATH_LEN);
    encode_array(
        &DerivedV2RouterCall::SwapExactTokensForTokens(
            params.amount_in,
            params.amount_out_min,
            derived_path_array(params.path),
            EvmCdAddress::new(params.to),
            params.deadline,
        ),
        required,
    )
}

pub fn make_fn_swap_exact_eth_for_tokens_array<const PATH_LEN: usize, const ALL: usize>(
    params: SwapExactEthForTokensParams<PATH_LEN>,
) -> [u8; ALL] {
    let required = swap_exact_eth_for_tokens_calldata_len(PATH_LEN);
    encode_array(
        &DerivedV2RouterCall::SwapExactEthForTokens(
            params.amount_out_min,
            derived_path_array(params.path),
            EvmCdAddress::new(params.to),
            params.deadline,
        ),
        required,
    )
}

pub fn make_fn_swap_exact_tokens_for_eth_array<const PATH_LEN: usize, const ALL: usize>(
    params: SwapExactTokensForEthParams<PATH_LEN>,
) -> [u8; ALL] {
    let required = swap_exact_tokens_for_eth_calldata_len(PATH_LEN);
    encode_array(
        &DerivedV2RouterCall::SwapExactTokensForEth(
            params.amount_in,
            params.amount_out_min,
            derived_path_array(params.path),
            EvmCdAddress::new(params.to),
            params.deadline,
        ),
        required,
    )
}

#[cfg(feature = "alloc")]
pub fn make_fn_swap_exact_tokens_for_tokens_vec(
    params: SwapExactTokensForTokensParamsVec,
) -> Vec<u8> {
    swap_exact_tokens_for_tokens_calldata_len(params.path.len());
    encode_vec(&DerivedV2RouterCall::SwapExactTokensForTokens(
        params.amount_in,
        params.amount_out_min,
        derived_path_vec(params.path),
        EvmCdAddress::new(params.to),
        params.deadline,
    ))
}

#[cfg(feature = "alloc")]
pub fn make_fn_swap_exact_eth_for_tokens_vec(params: SwapExactEthForTokensParamsVec) -> Vec<u8> {
    swap_exact_eth_for_tokens_calldata_len(params.path.len());
    encode_vec(&DerivedV2RouterCall::SwapExactEthForTokens(
        params.amount_out_min,
        derived_path_vec(params.path),
        EvmCdAddress::new(params.to),
        params.deadline,
    ))
}

#[cfg(feature = "alloc")]
pub fn make_fn_swap_exact_tokens_for_eth_vec(params: SwapExactTokensForEthParamsVec) -> Vec<u8> {
    swap_exact_tokens_for_eth_calldata_len(params.path.len());
    encode_vec(&DerivedV2RouterCall::SwapExactTokensForEth(
        params.amount_in,
        params.amount_out_min,
        derived_path_vec(params.path),
        EvmCdAddress::new(params.to),
        params.deadline,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATH: [Address; 2] = [[0x11; 20], [0x22; 20]];
    const TO: Address = [0x33; 20];

    fn tokens_for_tokens_params() -> SwapExactTokensForTokensParams<2> {
        SwapExactTokensForTokensParams {
            amount_in: U::from_u8(1),
            amount_out_min: U::from_u8(2),
            path: PATH,
            to: TO,
            deadline: U::from_u8(3),
        }
    }

    #[test]
    fn fixed_api_owns_a_const_sized_path() {
        let output: [u8; 260] =
            make_fn_swap_exact_tokens_for_tokens_array(tokens_for_tokens_params());

        assert_eq!(&output[..4], &[0x38, 0xed, 0x17, 0x39]);
        assert_eq!(output[35], 1);
        assert_eq!(output[67], 2);
        assert_eq!(output[99], 160);
        assert_eq!(&output[112..132], &TO);
        assert_eq!(output[163], 3);
        assert_eq!(output[195], 2);
        assert_eq!(&output[208..228], &PATH[0]);
        assert_eq!(&output[240..260], &PATH[1]);
    }

    #[test]
    fn native_swap_variants_use_canonical_selectors() {
        let eth_for_tokens: [u8; 228] =
            make_fn_swap_exact_eth_for_tokens_array(SwapExactEthForTokensParams {
                amount_out_min: U::from_u8(1),
                path: PATH,
                to: TO,
                deadline: U::from_u8(2),
            });
        let tokens_for_eth: [u8; 260] =
            make_fn_swap_exact_tokens_for_eth_array(SwapExactTokensForEthParams {
                amount_in: U::from_u8(1),
                amount_out_min: U::from_u8(2),
                path: PATH,
                to: TO,
                deadline: U::from_u8(3),
            });

        assert_eq!(&eth_for_tokens[..4], &[0x7f, 0xf3, 0x6a, 0xb5]);
        assert_eq!(&tokens_for_eth[..4], &[0x18, 0xcb, 0xaf, 0xe5]);
    }

    #[test]
    #[should_panic(expected = "Uniswap V2 path must contain at least two addresses")]
    fn fixed_api_panics_for_a_short_path() {
        let short = SwapExactTokensForTokensParams {
            amount_in: U::ZERO,
            amount_out_min: U::ZERO,
            path: [[0; 20]],
            to: TO,
            deadline: U::ZERO,
        };
        let _: [u8; 228] = make_fn_swap_exact_tokens_for_tokens_array(short);
    }

    #[test]
    #[should_panic(expected = "calldata output array has the wrong length")]
    fn fixed_api_panics_for_the_wrong_output_length() {
        let _: [u8; 259] = make_fn_swap_exact_tokens_for_tokens_array(tokens_for_tokens_params());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn vector_variant_matches_const_sized_variant() {
        let fixed: [u8; 260] =
            make_fn_swap_exact_tokens_for_tokens_array(tokens_for_tokens_params());
        let dynamic = make_fn_swap_exact_tokens_for_tokens_vec(SwapExactTokensForTokensParamsVec {
            amount_in: U::from_u8(1),
            amount_out_min: U::from_u8(2),
            path: PATH.into(),
            to: TO,
            deadline: U::from_u8(3),
        });

        assert_eq!(fixed.to_vec(), dynamic);
    }
}
