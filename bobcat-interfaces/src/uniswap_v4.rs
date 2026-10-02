//! Uniswap V4 single-pool swap and quote calldata.
//!
//! The swap builder targets Universal Router 2.1.1's `V4_SWAP` command and emits an
//! exact-input plan containing `SWAP_EXACT_IN_SINGLE`, `SETTLE_ALL`, and `TAKE_ALL`.
//! Calling `PoolManager.swap` directly is intentionally unsupported: direct callers must
//! implement and settle the PoolManager unlock callback themselves.
//!
//! The quote builders target `IV4Quoter`'s single-pool exact-input and exact-output
//! entrypoints. These quoter functions are intentionally non-view in Solidity.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use bobcat_cd::{EvmCdAddress, EvmCdArray, EvmCdError, EvmCdSerialise, EvmCdWrite};
use bobcat_maths::U;

pub type Address = [u8; 20];
pub type U24 = [u8; 3];
/// Big-endian two's-complement `int24`.
pub type I24 = [u8; 3];

const COMMAND_V4_SWAP: u8 = 0x10;
const ACTION_SWAP_EXACT_IN_SINGLE: u8 = 0x06;
const ACTION_SETTLE_ALL: u8 = 0x0c;
const ACTION_TAKE_ALL: u8 = 0x0f;

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
pub struct ExactInputSingle<const HOOK_DATA_LEN: usize> {
    pub pool_key: PoolKey,
    pub zero_for_one: bool,
    pub amount_in: u128,
    pub amount_out_minimum: u128,
    /// Universal Router 2.1.1's optional per-hop minimum output/input price, scaled by 1e36.
    pub min_hop_price_x36: U,
    /// Opaque bytes forwarded unchanged to the pool's hook callbacks.
    pub hook_data: [u8; HOOK_DATA_LEN],
}

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactInputSingleVec {
    pub pool_key: PoolKey,
    pub zero_for_one: bool,
    pub amount_in: u128,
    pub amount_out_minimum: u128,
    pub min_hop_price_x36: U,
    pub hook_data: Vec<u8>,
}

/// Parameters shared by `IV4Quoter.quoteExactInputSingle` and
/// `IV4Quoter.quoteExactOutputSingle`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuoteExactSingleParams<const HOOK_DATA_LEN: usize> {
    pub pool_key: PoolKey,
    pub zero_for_one: bool,
    /// Input amount for an exact-input quote; output amount for an exact-output quote.
    pub exact_amount: u128,
    /// Opaque bytes forwarded unchanged to the pool's hook callbacks.
    pub hook_data: [u8; HOOK_DATA_LEN],
}

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuoteExactSingleParamsVec {
    pub pool_key: PoolKey,
    pub zero_for_one: bool,
    pub exact_amount: u128,
    pub hook_data: Vec<u8>,
}

#[derive(Clone, Copy, EvmCdSerialise)]
struct DerivedPoolKey {
    currency0: EvmCdAddress,
    currency1: EvmCdAddress,
    fee: DerivedU24,
    tick_spacing: DerivedI24,
    hooks: EvmCdAddress,
}

#[derive(EvmCdSerialise)]
struct DerivedQuoteExactSingleParams<HookData> {
    pool_key: DerivedPoolKey,
    zero_for_one: bool,
    exact_amount: u128,
    hook_data: DerivedBytes<HookData>,
}

#[derive(EvmCdSerialise)]
struct DerivedExactInputSingleParams<HookData> {
    pool_key: DerivedPoolKey,
    zero_for_one: bool,
    amount_in: u128,
    amount_out_minimum: u128,
    min_hop_price_x36: U,
    hook_data: DerivedBytes<HookData>,
}

#[derive(EvmCdSerialise)]
struct DerivedSwapParams<HookData> {
    params: DerivedExactInputSingleParams<HookData>,
}

#[derive(EvmCdSerialise)]
struct DerivedCurrencyAmount {
    currency: EvmCdAddress,
    amount: u128,
}

#[derive(EvmCdSerialise)]
struct DerivedPlan<HookData> {
    actions: DerivedBytes<[u8; 3]>,
    params: EvmCdArray<DerivedEncodedBytes<DerivedPlanPayload<HookData>>, 3, 3>,
}

#[derive(EvmCdSerialise)]
#[evm_selector]
enum DerivedV4QuoterCall<HookData> {
    #[evm_selector(
        "quoteExactInputSingle(((address,address,uint24,int24,address),bool,uint128,bytes))"
    )]
    QuoteExactInputSingle(DerivedQuoteExactSingleParams<HookData>),
    #[evm_selector(
        "quoteExactOutputSingle(((address,address,uint24,int24,address),bool,uint128,bytes))"
    )]
    QuoteExactOutputSingle(DerivedQuoteExactSingleParams<HookData>),
}

#[derive(EvmCdSerialise)]
#[evm_selector]
enum DerivedUniversalRouterCall<HookData> {
    #[evm_selector("execute(bytes,bytes[],uint256)")]
    Execute(
        DerivedBytes<[u8; 1]>,
        EvmCdArray<DerivedEncodedBytes<DerivedPlan<HookData>>, 1, 1>,
        U,
    ),
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

struct DerivedBytes<T>(T);

impl<T: AsRef<[u8]>> EvmCdSerialise for DerivedBytes<T> {
    fn serialise_writer<W: EvmCdWrite>(&self, writer: &mut W) -> Result<(), EvmCdError> {
        U::from_u32(32).serialise_value(writer)?;
        self.serialise_abi_tail(writer)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        32usize.saturating_add(padded_len(self.0.as_ref().len()))
    }

    fn serialise_abi_head<W: EvmCdWrite>(
        &self,
        tail_offset: usize,
        writer: &mut W,
    ) -> Result<(), EvmCdError> {
        U::from_usize(tail_offset).serialise_value(writer)
    }

    fn serialise_abi_tail<W: EvmCdWrite>(&self, writer: &mut W) -> Result<(), EvmCdError> {
        let bytes = self.0.as_ref();
        U::from_usize(bytes.len()).serialise_value(writer)?;
        writer.write_all(bytes)?;
        write_padding(writer, bytes.len())
    }

    fn append_abi_type(
        hasher: bobcat_cd::serialisation::SelectorHasher,
    ) -> bobcat_cd::serialisation::SelectorHasher {
        hasher.update(b"bytes")
    }
}

struct DerivedEncodedBytes<T> {
    value: T,
    encoded_len: usize,
}

impl<T: EvmCdSerialise> EvmCdSerialise for DerivedEncodedBytes<T> {
    fn serialise_writer<W: EvmCdWrite>(&self, writer: &mut W) -> Result<(), EvmCdError> {
        U::from_u32(32).serialise_value(writer)?;
        self.serialise_abi_tail(writer)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        32usize.saturating_add(padded_len(self.encoded_len))
    }

    fn serialise_abi_head<W: EvmCdWrite>(
        &self,
        tail_offset: usize,
        writer: &mut W,
    ) -> Result<(), EvmCdError> {
        U::from_usize(tail_offset).serialise_value(writer)
    }

    fn serialise_abi_tail<W: EvmCdWrite>(&self, writer: &mut W) -> Result<(), EvmCdError> {
        U::from_usize(self.encoded_len).serialise_value(writer)?;
        self.value.serialise_writer(writer)?;
        write_padding(writer, self.encoded_len)
    }

    fn append_abi_type(
        hasher: bobcat_cd::serialisation::SelectorHasher,
    ) -> bobcat_cd::serialisation::SelectorHasher {
        hasher.update(b"bytes")
    }
}

struct DerivedRawPayload<T>(T);

impl<T: AsRef<[u8]>> EvmCdSerialise for DerivedRawPayload<T> {
    fn serialise_writer<W: EvmCdWrite>(&self, writer: &mut W) -> Result<(), EvmCdError> {
        writer.write_all(self.0.as_ref())
    }

    fn append_abi_type(
        hasher: bobcat_cd::serialisation::SelectorHasher,
    ) -> bobcat_cd::serialisation::SelectorHasher {
        hasher
    }
}

enum DerivedPlanPayload<HookData> {
    Swap(DerivedSwapParams<HookData>),
    Raw(DerivedRawPayload<[u8; 64]>),
}

impl<HookData: AsRef<[u8]>> EvmCdSerialise for DerivedPlanPayload<HookData> {
    fn serialise_writer<W: EvmCdWrite>(&self, writer: &mut W) -> Result<(), EvmCdError> {
        match self {
            Self::Swap(value) => value.serialise_writer(writer),
            Self::Raw(value) => value.serialise_writer(writer),
        }
    }

    fn append_abi_type(
        hasher: bobcat_cd::serialisation::SelectorHasher,
    ) -> bobcat_cd::serialisation::SelectorHasher {
        hasher
    }
}

fn write_padding<W: EvmCdWrite>(writer: &mut W, length: usize) -> Result<(), EvmCdError> {
    let padding = (32 - length % 32) % 32;
    writer.write_all(&[0; 31][..padding])
}

fn derived_pool_key(pool_key: PoolKey) -> DerivedPoolKey {
    DerivedPoolKey {
        currency0: EvmCdAddress::new(pool_key.currency0),
        currency1: EvmCdAddress::new(pool_key.currency1),
        fee: DerivedU24(pool_key.fee),
        tick_spacing: DerivedI24(pool_key.tick_spacing),
        hooks: EvmCdAddress::new(pool_key.hooks),
    }
}

fn derived_quote<HookData>(
    pool_key: PoolKey,
    zero_for_one: bool,
    exact_amount: u128,
    hook_data: HookData,
) -> DerivedQuoteExactSingleParams<HookData> {
    DerivedQuoteExactSingleParams {
        pool_key: derived_pool_key(pool_key),
        zero_for_one,
        exact_amount,
        hook_data: DerivedBytes(hook_data),
    }
}

const PLAN_BASE_LENGTH: usize = 864;
const CALLDATA_BASE_LENGTH: usize = 1124;
const QUOTE_EXACT_SINGLE_BASE_LENGTH: usize = 324;

const fn padded_len(length: usize) -> usize {
    let Some(padded) = length.checked_add(31) else {
        panic!("hook data length overflows calldata length");
    };
    padded & !31
}

pub const fn exact_input_single_calldata_len(hook_data_len: usize) -> usize {
    let Some(length) = CALLDATA_BASE_LENGTH.checked_add(padded_len(hook_data_len)) else {
        panic!("hook data length overflows calldata length");
    };
    length
}

pub const fn quote_exact_single_calldata_len(hook_data_len: usize) -> usize {
    let Some(length) = QUOTE_EXACT_SINGLE_BASE_LENGTH.checked_add(padded_len(hook_data_len)) else {
        panic!("hook data length overflows calldata length");
    };
    length
}

fn encode_array<const ALL: usize>(call: &impl EvmCdSerialise, expected: usize) -> [u8; ALL] {
    assert_eq!(ALL, expected, "calldata output array has the wrong length");
    let mut output = [0; ALL];
    call.serialise(&mut output)
        .expect("Uniswap V4 calldata serialization failed");
    output
}

#[cfg(feature = "alloc")]
fn encode_vec(call: &impl EvmCdSerialise) -> Vec<u8> {
    let mut output = Vec::new();
    call.serialise(&mut output)
        .expect("Uniswap V4 calldata serialization failed");
    output
}

pub fn make_fn_quote_exact_input_single_array<const HOOK_DATA_LEN: usize, const ALL: usize>(
    params: QuoteExactSingleParams<HOOK_DATA_LEN>,
) -> [u8; ALL] {
    let required = quote_exact_single_calldata_len(HOOK_DATA_LEN);
    encode_array(
        &DerivedV4QuoterCall::QuoteExactInputSingle(derived_quote(
            params.pool_key,
            params.zero_for_one,
            params.exact_amount,
            params.hook_data,
        )),
        required,
    )
}

/// Encode `quoteExactInputSingle` with empty hook data.
///
/// The hook-data and calldata lengths are fixed by this function, so callers do not need to
/// provide either const generic explicitly.
pub fn make_fn_quote_exact_input_zero_hooks(
    params: QuoteExactSingleParams<0>,
) -> [u8; QUOTE_EXACT_SINGLE_BASE_LENGTH] {
    make_fn_quote_exact_input_single_array(params)
}

pub fn make_fn_quote_exact_output_single_array<const HOOK_DATA_LEN: usize, const ALL: usize>(
    params: QuoteExactSingleParams<HOOK_DATA_LEN>,
) -> [u8; ALL] {
    let required = quote_exact_single_calldata_len(HOOK_DATA_LEN);
    encode_array(
        &DerivedV4QuoterCall::QuoteExactOutputSingle(derived_quote(
            params.pool_key,
            params.zero_for_one,
            params.exact_amount,
            params.hook_data,
        )),
        required,
    )
}

#[cfg(feature = "alloc")]
pub fn make_fn_quote_exact_input_single_vec(params: QuoteExactSingleParamsVec) -> Vec<u8> {
    quote_exact_single_calldata_len(params.hook_data.len());
    encode_vec(&DerivedV4QuoterCall::QuoteExactInputSingle(derived_quote(
        params.pool_key,
        params.zero_for_one,
        params.exact_amount,
        params.hook_data,
    )))
}

#[cfg(feature = "alloc")]
pub fn make_fn_quote_exact_output_single_vec(params: QuoteExactSingleParamsVec) -> Vec<u8> {
    quote_exact_single_calldata_len(params.hook_data.len());
    encode_vec(&DerivedV4QuoterCall::QuoteExactOutputSingle(derived_quote(
        params.pool_key,
        params.zero_for_one,
        params.exact_amount,
        params.hook_data,
    )))
}

#[allow(clippy::too_many_arguments)]
fn derived_execute_call<HookData: AsRef<[u8]>>(
    pool_key: PoolKey,
    zero_for_one: bool,
    amount_in: u128,
    amount_out_minimum: u128,
    min_hop_price_x36: U,
    hook_data: HookData,
    deadline: U,
) -> (DerivedUniversalRouterCall<HookData>, usize) {
    let hook_padded = padded_len(hook_data.as_ref().len());
    let required = exact_input_single_calldata_len(hook_data.as_ref().len());
    let plan_len = PLAN_BASE_LENGTH
        .checked_add(hook_padded)
        .expect("hook data length overflows plan length");
    let swap_param_len = 384usize
        .checked_add(hook_padded)
        .expect("hook data length overflows swap parameter length");
    let currency_in = if zero_for_one {
        pool_key.currency0
    } else {
        pool_key.currency1
    };
    let currency_out = if zero_for_one {
        pool_key.currency1
    } else {
        pool_key.currency0
    };
    let settle = DerivedCurrencyAmount {
        currency: EvmCdAddress::new(currency_in),
        amount: amount_in,
    }
    .to_evm_array()
    .expect("failed to serialize the settle parameter");
    let take = DerivedCurrencyAmount {
        currency: EvmCdAddress::new(currency_out),
        amount: amount_out_minimum,
    }
    .to_evm_array()
    .expect("failed to serialize the take parameter");
    let plan = DerivedPlan {
        actions: DerivedBytes([
            ACTION_SWAP_EXACT_IN_SINGLE,
            ACTION_SETTLE_ALL,
            ACTION_TAKE_ALL,
        ]),
        params: EvmCdArray::try_from_array(
            [
                DerivedEncodedBytes {
                    value: DerivedPlanPayload::Swap(DerivedSwapParams {
                        params: DerivedExactInputSingleParams {
                            pool_key: derived_pool_key(pool_key),
                            zero_for_one,
                            amount_in,
                            amount_out_minimum,
                            min_hop_price_x36,
                            hook_data: DerivedBytes(hook_data),
                        },
                    }),
                    encoded_len: swap_param_len,
                },
                DerivedEncodedBytes {
                    value: DerivedPlanPayload::Raw(DerivedRawPayload(settle)),
                    encoded_len: 64,
                },
                DerivedEncodedBytes {
                    value: DerivedPlanPayload::Raw(DerivedRawPayload(take)),
                    encoded_len: 64,
                },
            ],
            3,
        )
        .expect("failed to construct the V4 action parameters"),
    };
    let inputs = EvmCdArray::try_from_array(
        [DerivedEncodedBytes {
            value: plan,
            encoded_len: plan_len,
        }],
        1,
    )
    .expect("failed to construct the Universal Router inputs");
    (
        DerivedUniversalRouterCall::Execute(DerivedBytes([COMMAND_V4_SWAP]), inputs, deadline),
        required,
    )
}

pub fn make_fn_execute_exact_input_single_array<const HOOK_DATA_LEN: usize, const ALL: usize>(
    swap: ExactInputSingle<HOOK_DATA_LEN>,
    deadline: U,
) -> [u8; ALL] {
    let (call, required) = derived_execute_call(
        swap.pool_key,
        swap.zero_for_one,
        swap.amount_in,
        swap.amount_out_minimum,
        swap.min_hop_price_x36,
        swap.hook_data,
        deadline,
    );
    encode_array(&call, required)
}

#[cfg(feature = "alloc")]
pub fn make_fn_execute_exact_input_single_vec(swap: ExactInputSingleVec, deadline: U) -> Vec<u8> {
    let (call, _) = derived_execute_call(
        swap.pool_key,
        swap.zero_for_one,
        swap.amount_in,
        swap.amount_out_minimum,
        swap.min_hop_price_x36,
        swap.hook_data,
        deadline,
    );
    encode_vec(&call)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{
        Address as AlloyAddress, Bytes, U256 as AlloyU256,
        aliases::{I24 as AlloyI24, U24 as AlloyU24},
    };
    use alloy_sol_macro::sol;
    use alloy_sol_types::{SolCall, SolValue};

    sol! {
        struct OraclePoolKey {
            address currency0;
            address currency1;
            uint24 fee;
            int24 tickSpacing;
            address hooks;
        }

        struct OracleExactInputSingleParams {
            OraclePoolKey poolKey;
            bool zeroForOne;
            uint128 amountIn;
            uint128 amountOutMinimum;
            uint256 minHopPriceX36;
            bytes hookData;
        }

        function execute(bytes commands, bytes[] inputs, uint256 deadline);
    }

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

    fn pool_key() -> PoolKey {
        PoolKey {
            currency0: [0; 20],
            currency1: [0x11; 20],
            fee: [0x00, 0x01, 0xf4],
            tick_spacing: [0xff, 0xff, 0xf6],
            hooks: [0x22; 20],
        }
    }

    fn fixture() -> QuoteExactSingleParams<3> {
        QuoteExactSingleParams {
            pool_key: pool_key(),
            zero_for_one: false,
            exact_amount: 123_456,
            hook_data: [0xab, 0xcd, 0xef],
        }
    }

    #[test]
    fn fixed_quote_type_owns_const_sized_hook_data() {
        let encoded: [u8; 356] = make_fn_quote_exact_input_single_array(fixture());
        let expected: [u8; 356] = const_hex::decode(INPUT_FIXTURE)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(encoded, expected);
    }

    #[test]
    fn exact_output_quote_uses_output_selector_and_same_body() {
        let input: [u8; 356] = make_fn_quote_exact_input_single_array(fixture());
        let output: [u8; 356] = make_fn_quote_exact_output_single_array(fixture());

        assert_eq!(&output[..4], &[0x58, 0x73, 0x30, 0x73]);
        assert_eq!(&output[4..], &input[4..]);
    }

    #[test]
    fn empty_hook_data_is_carried_by_the_zero_length_type() {
        let params = QuoteExactSingleParams {
            pool_key: pool_key(),
            zero_for_one: true,
            exact_amount: 123_456,
            hook_data: [],
        };
        let output = make_fn_quote_exact_input_zero_hooks(params);

        assert_eq!(&output[..4], &[0xaa, 0x9d, 0x21, 0xcb]);
        assert_eq!(output[227], 1);
        assert_eq!(&output[292..324], &[0; 32]);
    }

    #[test]
    fn execute_exact_input_single_matches_alloy() {
        let swap = ExactInputSingle {
            pool_key: PoolKey {
                currency0: [0x11; 20],
                currency1: [0x22; 20],
                fee: [0x00, 0x01, 0xf4],
                tick_spacing: [0xff, 0xff, 0xf6],
                hooks: [0x33; 20],
            },
            zero_for_one: true,
            amount_in: 123,
            amount_out_minimum: 45,
            min_hop_price_x36: U::from_u8(7),
            hook_data: [0xab, 0xcd, 0xef],
        };
        let actual: [u8; 1156] = make_fn_execute_exact_input_single_array(swap, U::from_u8(9));

        let swap_param = (OracleExactInputSingleParams {
            poolKey: OraclePoolKey {
                currency0: AlloyAddress::from(swap.pool_key.currency0),
                currency1: AlloyAddress::from(swap.pool_key.currency1),
                fee: AlloyU24::from(500u16),
                tickSpacing: AlloyI24::try_from(-10i32).unwrap(),
                hooks: AlloyAddress::from(swap.pool_key.hooks),
            },
            zeroForOne: swap.zero_for_one,
            amountIn: swap.amount_in,
            amountOutMinimum: swap.amount_out_minimum,
            minHopPriceX36: AlloyU256::from(7),
            hookData: Bytes::copy_from_slice(&swap.hook_data),
        },)
            .abi_encode_params();
        let settle =
            (AlloyAddress::from(swap.pool_key.currency0), swap.amount_in).abi_encode_params();
        let take = (
            AlloyAddress::from(swap.pool_key.currency1),
            swap.amount_out_minimum,
        )
            .abi_encode_params();
        let plan = (
            Bytes::copy_from_slice(&[
                ACTION_SWAP_EXACT_IN_SINGLE,
                ACTION_SETTLE_ALL,
                ACTION_TAKE_ALL,
            ]),
            vec![
                Bytes::from(swap_param),
                Bytes::from(settle),
                Bytes::from(take),
            ],
        )
            .abi_encode_params();
        let expected = executeCall {
            commands: Bytes::copy_from_slice(&[COMMAND_V4_SWAP]),
            inputs: vec![Bytes::from(plan)],
            deadline: AlloyU256::from(9),
        }
        .abi_encode();

        assert_eq!(actual.to_vec(), expected);
    }

    #[test]
    fn length_helpers_return_plain_lengths() {
        assert_eq!(quote_exact_single_calldata_len(0), 324);
        assert_eq!(quote_exact_single_calldata_len(1), 356);
        assert_eq!(quote_exact_single_calldata_len(33), 388);
    }

    #[test]
    #[should_panic(expected = "hook data length overflows calldata length")]
    fn length_helpers_panic_on_overflow() {
        let _ = quote_exact_single_calldata_len(usize::MAX);
    }

    #[test]
    #[should_panic(expected = "calldata output array has the wrong length")]
    fn fixed_builders_panic_for_the_wrong_array_length() {
        let _: [u8; 355] = make_fn_quote_exact_input_single_array(fixture());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn quote_vec_variant_matches_const_sized_variant() {
        let fixed: [u8; 356] = make_fn_quote_exact_input_single_array(fixture());
        let dynamic = make_fn_quote_exact_input_single_vec(QuoteExactSingleParamsVec {
            pool_key: pool_key(),
            zero_for_one: false,
            exact_amount: 123_456,
            hook_data: vec![0xab, 0xcd, 0xef],
        });

        assert_eq!(fixed.to_vec(), dynamic);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn execute_vec_variant_matches_const_sized_variant() {
        let fixed_swap = ExactInputSingle {
            pool_key: pool_key(),
            zero_for_one: false,
            amount_in: 123,
            amount_out_minimum: 45,
            min_hop_price_x36: U::from_u8(7),
            hook_data: [0xab, 0xcd, 0xef],
        };
        let fixed: [u8; 1156] = make_fn_execute_exact_input_single_array(fixed_swap, U::from_u8(9));
        let dynamic = make_fn_execute_exact_input_single_vec(
            ExactInputSingleVec {
                pool_key: pool_key(),
                zero_for_one: false,
                amount_in: 123,
                amount_out_minimum: 45,
                min_hop_price_x36: U::from_u8(7),
                hook_data: vec![0xab, 0xcd, 0xef],
            },
            U::from_u8(9),
        );

        assert_eq!(fixed.to_vec(), dynamic);
    }
}
