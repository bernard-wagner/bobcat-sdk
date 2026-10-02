//! Calldata builders for Uniswap V2 Router02 exact-input swaps.
//!
//! This module targets the canonical `IUniswapV2Router02` interface for
//! exact-input token and native-asset swaps on Arbitrum. All three
//! exact-input entrypoints share a dynamic `address[]` path parameter;
//! builders write into a caller-supplied buffer and return the number of
//! bytes written, matching the convention used by other bobcat interface
//! modules with dynamic ABI types.
//!
//! Token approvals and any native token call value remain the caller's
//! responsibility.
//!
//! ABI reference:
//! - <https://github.com/Uniswap/v2-periphery/blob/master/contracts/interfaces/IUniswapV2Router02.sol>

use bobcat_cd::{EvmCdAddress, EvmCdError, EvmCdSerialise, EvmCdWrite};
use bobcat_maths::U;

/// An EVM address.
pub type Address = [u8; 20];

/// Error returned when calldata cannot be encoded into the supplied buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// Path must contain at least 2 addresses.
    PathTooShort,
    /// Provided buffer was too small; `required` is the minimum length.
    BufferTooSmall { required: usize },
    /// The derived calldata serialiser rejected the values.
    SerialisationFailed,
}

#[derive(Clone, Copy)]
struct DerivedAddressPath<'a>(&'a [Address]);

impl EvmCdSerialise for DerivedAddressPath<'_> {
    fn serialise_writer<W: EvmCdWrite>(&self, writer: &mut W) -> Result<(), EvmCdError> {
        U::from_u32(32).serialise_value(writer)?;
        self.serialise_abi_tail(writer)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        32usize.saturating_add(self.0.len().saturating_mul(32))
    }

    fn serialise_abi_head<W: EvmCdWrite>(
        &self,
        tail_offset: usize,
        writer: &mut W,
    ) -> Result<(), EvmCdError> {
        U::from_usize(tail_offset).serialise_value(writer)
    }

    fn serialise_abi_tail<W: EvmCdWrite>(&self, writer: &mut W) -> Result<(), EvmCdError> {
        U::from_usize(self.0.len()).serialise_value(writer)?;
        for address in self.0 {
            EvmCdAddress::new(*address).serialise_value(writer)?;
        }
        Ok(())
    }

    fn append_abi_type(
        hasher: bobcat_cd::serialisation::SelectorHasher,
    ) -> bobcat_cd::serialisation::SelectorHasher {
        EvmCdAddress::append_abi_type(hasher).update(b"[]")
    }
}

#[derive(EvmCdSerialise)]
#[evm_selector]
enum DerivedV2RouterCall<'a> {
    SwapExactTokensForTokens(U, U, DerivedAddressPath<'a>, EvmCdAddress, U),
    #[evm_selector("swapExactETHForTokens(uint256,address[],address,uint256)")]
    SwapExactEthForTokens(U, DerivedAddressPath<'a>, EvmCdAddress, U),
    #[evm_selector("swapExactTokensForETH(uint256,uint256,address[],address,uint256)")]
    SwapExactTokensForEth(U, U, DerivedAddressPath<'a>, EvmCdAddress, U),
}

fn calldata_len(head_words: usize, path_len: usize) -> Result<usize, EncodeError> {
    if path_len < 2 {
        return Err(EncodeError::PathTooShort);
    }
    path_len
        .checked_add(head_words + 1)
        .and_then(|words| words.checked_mul(32))
        .and_then(|bytes| bytes.checked_add(4))
        .ok_or(EncodeError::PathTooShort)
}

fn write_derived_call(
    output: &mut [u8],
    required: usize,
    call: DerivedV2RouterCall<'_>,
) -> Result<usize, EncodeError> {
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let written = call
        .write_slice(&mut output[..required])
        .map_err(|_| EncodeError::SerialisationFailed)?
        .len();
    if written != required {
        return Err(EncodeError::SerialisationFailed);
    }
    Ok(written)
}

/// Required output length for [`make_fn_swap_exact_tokens_for_tokens`].
///
/// `swapExactTokensForTokens(uint256,uint256,address[],address,uint256)`
/// encodes as: selector(4) + 5 head words + length(32) + N path words.
pub fn swap_exact_tokens_for_tokens_calldata_len(n: usize) -> Result<usize, EncodeError> {
    calldata_len(5, n)
}

/// Encode an exact-input token-for-token swap into `output`. Returns the
/// number of bytes written.
///
/// `path` must contain at least 2 addresses. The caller must supply a buffer
/// of at least [`swap_exact_tokens_for_tokens_calldata_len`]`(path.len())`
/// bytes.
pub fn make_fn_swap_exact_tokens_for_tokens(
    output: &mut [u8],
    amount_in: &U,
    amount_out_min: &U,
    path: &[Address],
    to: Address,
    deadline: &U,
) -> Result<usize, EncodeError> {
    let required = swap_exact_tokens_for_tokens_calldata_len(path.len())?;
    write_derived_call(
        output,
        required,
        DerivedV2RouterCall::SwapExactTokensForTokens(
            *amount_in,
            *amount_out_min,
            DerivedAddressPath(path),
            EvmCdAddress::new(to),
            *deadline,
        ),
    )
}

/// Required output length for [`make_fn_swap_exact_eth_for_tokens`].
///
/// `swapExactETHForTokens(uint256,address[],address,uint256)` encodes as:
/// selector(4) + 4 head words + length(32) + N path words.
pub fn swap_exact_eth_for_tokens_calldata_len(n: usize) -> Result<usize, EncodeError> {
    calldata_len(4, n)
}

/// Encode an exact-input native-asset-for-tokens swap into `output`. Returns
/// the number of bytes written. The caller must send `msg.value` equal to
/// the desired input amount.
///
/// `path[0]` must be the WETH address. The caller must supply a buffer of at
/// least [`swap_exact_eth_for_tokens_calldata_len`]`(path.len())` bytes.
pub fn make_fn_swap_exact_eth_for_tokens(
    output: &mut [u8],
    amount_out_min: &U,
    path: &[Address],
    to: Address,
    deadline: &U,
) -> Result<usize, EncodeError> {
    let required = swap_exact_eth_for_tokens_calldata_len(path.len())?;
    write_derived_call(
        output,
        required,
        DerivedV2RouterCall::SwapExactEthForTokens(
            *amount_out_min,
            DerivedAddressPath(path),
            EvmCdAddress::new(to),
            *deadline,
        ),
    )
}

/// Required output length for [`make_fn_swap_exact_tokens_for_eth`].
///
/// `swapExactTokensForETH(uint256,uint256,address[],address,uint256)` encodes
/// as: selector(4) + 5 head words + length(32) + N path words.
pub fn swap_exact_tokens_for_eth_calldata_len(n: usize) -> Result<usize, EncodeError> {
    calldata_len(5, n)
}

/// Encode an exact-input tokens-for-native-asset swap into `output`. Returns
/// the number of bytes written.
///
/// `path[path.len()-1]` must be the WETH address. The caller must supply a
/// buffer of at least [`swap_exact_tokens_for_eth_calldata_len`]`(path.len())`
/// bytes.
pub fn make_fn_swap_exact_tokens_for_eth(
    output: &mut [u8],
    amount_in: &U,
    amount_out_min: &U,
    path: &[Address],
    to: Address,
    deadline: &U,
) -> Result<usize, EncodeError> {
    let required = swap_exact_tokens_for_eth_calldata_len(path.len())?;
    write_derived_call(
        output,
        required,
        DerivedV2RouterCall::SwapExactTokensForEth(
            *amount_in,
            *amount_out_min,
            DerivedAddressPath(path),
            EvmCdAddress::new(to),
            *deadline,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATH: [Address; 2] = [[0x11; 20], [0x22; 20]];
    const TO: Address = [0x33; 20];

    #[test]
    fn swap_exact_tokens_for_tokens_matches_solidity_abi() {
        let mut output = [0; 260];
        let written = make_fn_swap_exact_tokens_for_tokens(
            &mut output,
            &U::from_u8(1),
            &U::from_u8(2),
            &PATH,
            TO,
            &U::from_u8(3),
        )
        .unwrap();

        assert_eq!(written, 260);
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
        let mut eth_for_tokens = [0; 228];
        let mut tokens_for_eth = [0; 260];
        make_fn_swap_exact_eth_for_tokens(
            &mut eth_for_tokens,
            &U::from_u8(1),
            &PATH,
            TO,
            &U::from_u8(2),
        )
        .unwrap();
        make_fn_swap_exact_tokens_for_eth(
            &mut tokens_for_eth,
            &U::from_u8(1),
            &U::from_u8(2),
            &PATH,
            TO,
            &U::from_u8(3),
        )
        .unwrap();

        assert_eq!(&eth_for_tokens[..4], &[0x7f, 0xf3, 0x6a, 0xb5]);
        assert_eq!(&tokens_for_eth[..4], &[0x18, 0xcb, 0xaf, 0xe5]);
    }

    #[test]
    fn swaps_validate_path_and_buffer_lengths() {
        assert_eq!(
            swap_exact_tokens_for_tokens_calldata_len(1),
            Err(EncodeError::PathTooShort)
        );
        let mut output = [0; 259];
        assert_eq!(
            make_fn_swap_exact_tokens_for_tokens(
                &mut output,
                &U::from_u8(1),
                &U::from_u8(2),
                &PATH,
                TO,
                &U::from_u8(3),
            ),
            Err(EncodeError::BufferTooSmall { required: 260 })
        );
    }
}
