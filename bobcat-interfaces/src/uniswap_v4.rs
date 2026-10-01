//! Uniswap V4 single-pool swap and quote calldata.
//!
//! The swap builder targets Universal Router 2.1.1's `V4_SWAP` command and emits an
//! exact-input plan containing `SWAP_EXACT_IN_SINGLE`, `SETTLE_ALL`, and `TAKE_ALL`.
//! Calling `PoolManager.swap` directly is intentionally unsupported: direct callers must
//! implement and settle the PoolManager unlock callback themselves.
//!
//! The quote builders target `IV4Quoter`'s single-pool exact-input and exact-output
//! entrypoints. These quoter functions are intentionally non-view in Solidity.

use bobcat_cd::{EvmCdAddress, EvmCdError, EvmCdSerialise, EvmCdWrite};
use bobcat_maths::U;

use crate::selectors;

pub type Address = [u8; 20];
pub type U24 = [u8; 3];
/// Big-endian two's-complement `int24`.
pub type I24 = [u8; 3];

const COMMAND_V4_SWAP: u8 = 0x10;
const ACTION_SWAP_EXACT_IN_SINGLE: u8 = 0x06;
const ACTION_SETTLE_ALL: u8 = 0x0c;
const ACTION_TAKE_ALL: u8 = 0x0f;

selectors! {
    SEL_EXECUTE = b"execute(bytes,bytes[],uint256)",
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PoolKey {
    /// The numerically smaller currency address; native ETH is the zero address.
    pub currency0: Address,
    /// The numerically larger currency address.
    pub currency1: Address,
    pub fee: U24,
    /// Big-endian two's-complement `int24` tick spacing.
    pub tick_spacing: I24,
    /// Hook contract address, or the zero address for a hookless pool.
    pub hooks: Address,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExactInputSingle<'a> {
    pub pool_key: PoolKey,
    pub zero_for_one: bool,
    pub amount_in: u128,
    pub amount_out_minimum: u128,
    /// Universal Router 2.1.1's optional per-hop minimum output/input price, scaled by 1e36.
    pub min_hop_price_x36: U,
    /// Opaque bytes forwarded unchanged to the pool's hook callbacks.
    pub hook_data: &'a [u8],
}

/// Parameters shared by `IV4Quoter.quoteExactInputSingle` and
/// `IV4Quoter.quoteExactOutputSingle`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuoteExactSingleParams<'a> {
    pub pool_key: PoolKey,
    pub zero_for_one: bool,
    /// Input amount for an exact-input quote; output amount for an exact-output quote.
    pub exact_amount: u128,
    /// Opaque bytes forwarded unchanged to the pool's hook callbacks.
    pub hook_data: &'a [u8],
}

#[derive(Clone, Copy, EvmCdSerialise)]
struct DerivedPoolKey {
    currency0: EvmCdAddress,
    currency1: EvmCdAddress,
    fee: DerivedU24,
    tick_spacing: DerivedI24,
    hooks: EvmCdAddress,
}

#[derive(Clone, Copy, EvmCdSerialise)]
struct DerivedQuoteExactSingleParams<'a> {
    pool_key: DerivedPoolKey,
    zero_for_one: DerivedBool,
    exact_amount: u128,
    hook_data: DerivedBytes<'a>,
}

#[derive(EvmCdSerialise)]
#[evm_selector]
enum DerivedV4QuoterCall<'a> {
    QuoteExactInputSingle(DerivedQuoteExactSingleParams<'a>),
    QuoteExactOutputSingle(DerivedQuoteExactSingleParams<'a>),
}

#[derive(Clone, Copy)]
struct DerivedBool(bool);

impl EvmCdSerialise for DerivedBool {
    fn serialise_writer<W: EvmCdWrite>(&self, writer: &mut W) -> Result<(), EvmCdError> {
        U::from_u8(u8::from(self.0)).serialise_value(writer)
    }

    fn append_abi_type(
        hasher: bobcat_cd::serialisation::SelectorHasher,
    ) -> bobcat_cd::serialisation::SelectorHasher {
        hasher.update(b"bool")
    }
}

#[derive(Clone, Copy)]
struct DerivedU24(U24);

impl EvmCdSerialise for DerivedU24 {
    fn serialise_writer<W: EvmCdWrite>(&self, writer: &mut W) -> Result<(), EvmCdError> {
        writer.write_all(&[0; 29])?;
        writer.write_all(&self.0)
    }

    fn append_abi_type(
        hasher: bobcat_cd::serialisation::SelectorHasher,
    ) -> bobcat_cd::serialisation::SelectorHasher {
        hasher.update(b"uint24")
    }
}

#[derive(Clone, Copy)]
struct DerivedI24(I24);

impl EvmCdSerialise for DerivedI24 {
    fn serialise_writer<W: EvmCdWrite>(&self, writer: &mut W) -> Result<(), EvmCdError> {
        let sign = if self.0[0] & 0x80 == 0 { 0x00 } else { 0xff };
        writer.write_all(&[sign; 29])?;
        writer.write_all(&self.0)
    }

    fn append_abi_type(
        hasher: bobcat_cd::serialisation::SelectorHasher,
    ) -> bobcat_cd::serialisation::SelectorHasher {
        hasher.update(b"int24")
    }
}

#[derive(Clone, Copy)]
struct DerivedBytes<'a>(&'a [u8]);

impl EvmCdSerialise for DerivedBytes<'_> {
    fn serialise_writer<W: EvmCdWrite>(&self, writer: &mut W) -> Result<(), EvmCdError> {
        U::from_u32(32).serialise_value(writer)?;
        self.serialise_abi_tail(writer)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        32usize
            .saturating_add(self.0.len())
            .saturating_add(32usize.wrapping_sub(self.0.len() % 32).wrapping_rem(32))
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
        writer.write_all(self.0)?;
        let padding = (32 - self.0.len() % 32) % 32;
        writer.write_all(&[0; 31][..padding])
    }

    fn append_abi_type(
        hasher: bobcat_cd::serialisation::SelectorHasher,
    ) -> bobcat_cd::serialisation::SelectorHasher {
        hasher.update(b"bytes")
    }
}

impl<'a> From<&QuoteExactSingleParams<'a>> for DerivedQuoteExactSingleParams<'a> {
    fn from(params: &QuoteExactSingleParams<'a>) -> Self {
        Self {
            pool_key: DerivedPoolKey {
                currency0: EvmCdAddress::new(params.pool_key.currency0),
                currency1: EvmCdAddress::new(params.pool_key.currency1),
                fee: DerivedU24(params.pool_key.fee),
                tick_spacing: DerivedI24(params.pool_key.tick_spacing),
                hooks: EvmCdAddress::new(params.pool_key.hooks),
            },
            zero_for_one: DerivedBool(params.zero_for_one),
            exact_amount: params.exact_amount,
            hook_data: DerivedBytes(params.hook_data),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    LengthOverflow,
    BufferTooSmall { required: usize },
    SerialisationFailed,
}

const PLAN_BASE_LENGTH: usize = 864;
const CALLDATA_BASE_LENGTH: usize = 1124;
const QUOTE_EXACT_SINGLE_BASE_LENGTH: usize = 324;

fn padded_len(length: usize) -> Option<usize> {
    length.checked_add(31).map(|n| n & !31)
}

/// Required output length for [`make_fn_execute_exact_input_single`].
pub fn exact_input_single_calldata_len(hook_data_len: usize) -> Result<usize, EncodeError> {
    CALLDATA_BASE_LENGTH
        .checked_add(padded_len(hook_data_len).ok_or(EncodeError::LengthOverflow)?)
        .ok_or(EncodeError::LengthOverflow)
}

fn put_usize(output: &mut [u8], offset: usize, value: usize) {
    let bytes = value.to_be_bytes();
    output[offset + 32 - bytes.len()..offset + 32].copy_from_slice(&bytes);
}

fn put_u128(output: &mut [u8], offset: usize, value: u128) {
    output[offset + 16..offset + 32].copy_from_slice(&value.to_be_bytes());
}

fn put_address(output: &mut [u8], offset: usize, value: Address) {
    output[offset + 12..offset + 32].copy_from_slice(&value);
}

fn put_i24(output: &mut [u8], offset: usize, value: I24) {
    if value[0] & 0x80 != 0 {
        output[offset..offset + 29].fill(0xff);
    }
    output[offset + 29..offset + 32].copy_from_slice(&value);
}

/// Required output length for either single-pool `IV4Quoter` quote builder.
pub fn quote_exact_single_calldata_len(hook_data_len: usize) -> Result<usize, EncodeError> {
    QUOTE_EXACT_SINGLE_BASE_LENGTH
        .checked_add(padded_len(hook_data_len).ok_or(EncodeError::LengthOverflow)?)
        .ok_or(EncodeError::LengthOverflow)
}

fn write_derived_quote(
    output: &mut [u8],
    required: usize,
    call: DerivedV4QuoterCall<'_>,
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

/// Encode `IV4Quoter.quoteExactInputSingle` calldata using `bobcat-cd-derive`.
///
/// The function returns `(uint256 amountOut, uint256 gasEstimate)`, encoded as two consecutive
/// ABI words in returndata.
pub fn make_fn_quote_exact_input_single(
    output: &mut [u8],
    params: &QuoteExactSingleParams<'_>,
) -> Result<usize, EncodeError> {
    let required = quote_exact_single_calldata_len(params.hook_data.len())?;
    write_derived_quote(
        output,
        required,
        DerivedV4QuoterCall::QuoteExactInputSingle(params.into()),
    )
}

/// Encode `IV4Quoter.quoteExactOutputSingle` calldata using `bobcat-cd-derive`.
///
/// The function returns `(uint256 amountIn, uint256 gasEstimate)`, encoded as two consecutive ABI
/// words in returndata.
pub fn make_fn_quote_exact_output_single(
    output: &mut [u8],
    params: &QuoteExactSingleParams<'_>,
) -> Result<usize, EncodeError> {
    let required = quote_exact_single_calldata_len(params.hook_data.len())?;
    write_derived_quote(
        output,
        required,
        DerivedV4QuoterCall::QuoteExactOutputSingle(params.into()),
    )
}

/// Encode an exact-input, single-pool V4 swap for Universal Router 2.1.1.
///
/// The generated plan settles at most `amount_in` from the caller and sends at least
/// `amount_out_minimum` back to the caller. ERC-20 input requires the normal Permit2 token
/// approval and Permit2 allowance for the target router. `hook_data` is ABI-encoded as dynamic
/// bytes and forwarded unchanged; `pool_key.hooks` remains the separate hook address.
pub fn make_fn_execute_exact_input_single(
    output: &mut [u8],
    swap: &ExactInputSingle<'_>,
    deadline: &U,
) -> Result<usize, EncodeError> {
    let hook_padded = padded_len(swap.hook_data.len()).ok_or(EncodeError::LengthOverflow)?;
    let required = CALLDATA_BASE_LENGTH
        .checked_add(hook_padded)
        .ok_or(EncodeError::LengthOverflow)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let output = &mut output[..required];
    output.fill(0);

    // UniversalRouter.execute(bytes commands, bytes[] inputs, uint256 deadline)
    output[..4].copy_from_slice(&SEL_EXECUTE);
    put_usize(output, 4, 96);
    put_usize(output, 36, 160);
    output[68..100].copy_from_slice(&deadline.0);
    put_usize(output, 100, 1);
    output[132] = COMMAND_V4_SWAP;
    put_usize(output, 164, 1);
    put_usize(output, 196, 32);

    let plan_len = PLAN_BASE_LENGTH
        .checked_add(hook_padded)
        .ok_or(EncodeError::LengthOverflow)?;
    put_usize(output, 228, plan_len);
    let plan = &mut output[260..260 + plan_len];

    // abi.encode(bytes actions, bytes[] params)
    put_usize(plan, 0, 64);
    put_usize(plan, 32, 128);
    put_usize(plan, 64, 3);
    plan[96..99].copy_from_slice(&[
        ACTION_SWAP_EXACT_IN_SINGLE,
        ACTION_SETTLE_ALL,
        ACTION_TAKE_ALL,
    ]);
    put_usize(plan, 128, 3);
    put_usize(plan, 160, 96);
    put_usize(plan, 192, 512 + hook_padded);
    put_usize(plan, 224, 608 + hook_padded);

    // params[0] = abi.encode(ExactInputSingleParams(...)).
    let swap_param_len = 384 + hook_padded;
    put_usize(plan, 256, swap_param_len);
    let swap_param = &mut plan[288..288 + swap_param_len];
    put_usize(swap_param, 0, 32);
    let tuple = &mut swap_param[32..];
    put_address(tuple, 0, swap.pool_key.currency0);
    put_address(tuple, 32, swap.pool_key.currency1);
    tuple[64 + 29..64 + 32].copy_from_slice(&swap.pool_key.fee);
    put_i24(tuple, 96, swap.pool_key.tick_spacing);
    put_address(tuple, 128, swap.pool_key.hooks);
    tuple[160 + 31] = u8::from(swap.zero_for_one);
    put_u128(tuple, 192, swap.amount_in);
    put_u128(tuple, 224, swap.amount_out_minimum);
    tuple[256..288].copy_from_slice(&swap.min_hop_price_x36.0);
    put_usize(tuple, 288, 320);
    put_usize(tuple, 320, swap.hook_data.len());
    tuple[352..352 + swap.hook_data.len()].copy_from_slice(swap.hook_data);

    let currency_in = if swap.zero_for_one {
        swap.pool_key.currency0
    } else {
        swap.pool_key.currency1
    };
    let currency_out = if swap.zero_for_one {
        swap.pool_key.currency1
    } else {
        swap.pool_key.currency0
    };

    // params[1] = abi.encode(currencyIn, amountIn) for SETTLE_ALL.
    let settle = 672 + hook_padded;
    put_usize(plan, settle, 64);
    put_address(plan, settle + 32, currency_in);
    put_u128(plan, settle + 64, swap.amount_in);

    // params[2] = abi.encode(currencyOut, amountOutMinimum) for TAKE_ALL.
    let take = 768 + hook_padded;
    put_usize(plan, take, 64);
    put_address(plan, take + 32, currency_out);
    put_u128(plan, take + 64, swap.amount_out_minimum);

    Ok(required)
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT_FIXTURE: &str = concat!(
        "aa9d21cb",
        "0000000000000000000000000000000000000000000000000000000000000020",
        "0000000000000000000000000000000000000000000000000000000000000000",
        "0000000000000000000000001111111111111111111111111111111111111111",
        "00000000000000000000000000000000000000000000000000000000000001f4",
        "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff6",
        "0000000000000000000000002222222222222222222222222222222222222222",
        "0000000000000000000000000000000000000000000000000000000000000000",
        "000000000000000000000000000000000000000000000000000000000001e240",
        "0000000000000000000000000000000000000000000000000000000000000100",
        "0000000000000000000000000000000000000000000000000000000000000003",
        "abcdef0000000000000000000000000000000000000000000000000000000000",
    );

    fn fixture() -> QuoteExactSingleParams<'static> {
        QuoteExactSingleParams {
            pool_key: PoolKey {
                currency0: [0; 20],
                currency1: [0x11; 20],
                fee: [0x00, 0x01, 0xf4],
                tick_spacing: [0xff, 0xff, 0xf6],
                hooks: [0x22; 20],
            },
            zero_for_one: false,
            exact_amount: 123_456,
            hook_data: &[0xab, 0xcd, 0xef],
        }
    }

    #[test]
    fn quote_exact_input_single_matches_solidity_abi() {
        let params = fixture();
        let mut output = [0x55; 356];
        let written = make_fn_quote_exact_input_single(&mut output, &params).unwrap();
        let expected = const_hex::decode(INPUT_FIXTURE).unwrap();

        assert_eq!(written, expected.len());
        assert_eq!(&output[..written], expected);
    }

    #[test]
    fn quote_exact_output_single_uses_output_selector() {
        let params = fixture();
        let mut input = [0; 356];
        let mut output = [0; 356];
        make_fn_quote_exact_input_single(&mut input, &params).unwrap();
        make_fn_quote_exact_output_single(&mut output, &params).unwrap();

        assert_eq!(&output[..4], &[0x58, 0x73, 0x30, 0x73]);
        assert_eq!(&output[4..], &input[4..]);
    }

    #[test]
    fn quote_exact_single_handles_true_and_empty_hook_data() {
        let mut params = fixture();
        params.zero_for_one = true;
        params.hook_data = &[];
        let mut output = [0x55; 324];

        assert_eq!(
            make_fn_quote_exact_input_single(&mut output, &params),
            Ok(324)
        );
        assert_eq!(&output[..4], &[0xaa, 0x9d, 0x21, 0xcb]);
        assert_eq!(output[227], 1);
        assert_eq!(&output[292..324], &[0; 32]);
    }

    #[test]
    fn quote_exact_single_checks_lengths() {
        assert_eq!(quote_exact_single_calldata_len(0), Ok(324));
        assert_eq!(quote_exact_single_calldata_len(1), Ok(356));
        assert_eq!(
            quote_exact_single_calldata_len(usize::MAX),
            Err(EncodeError::LengthOverflow)
        );

        let params = fixture();
        let mut output = [0; 355];
        assert_eq!(
            make_fn_quote_exact_input_single(&mut output, &params),
            Err(EncodeError::BufferTooSmall { required: 356 })
        );
    }
}
