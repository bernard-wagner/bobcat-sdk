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
use bobcat_cd::{
    EvmCdAddress, EvmCdArray, EvmCdBytes, EvmCdBytes1, EvmCdBytes3, EvmCdI24, EvmCdSerialise,
    EvmCdU24,
};
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
    fee: EvmCdU24,
    tick_spacing: EvmCdI24,
    hooks: EvmCdAddress,
}

#[derive(EvmCdSerialise)]
struct DerivedQuoteExactSingleParams<HookData> {
    pool_key: DerivedPoolKey,
    zero_for_one: bool,
    exact_amount: u128,
    hook_data: HookData,
}

#[derive(EvmCdSerialise)]
struct DerivedExactInputSingleParams<HookData> {
    pool_key: DerivedPoolKey,
    zero_for_one: bool,
    amount_in: u128,
    amount_out_minimum: u128,
    min_hop_price_x36: U,
    hook_data: HookData,
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
struct DerivedPlan<Bytes> {
    actions: EvmCdBytes3,
    params: EvmCdArray<Bytes, 3, 3>,
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
enum DerivedUniversalRouterCall<Bytes> {
    #[evm_selector("execute(bytes,bytes[],uint256)")]
    Execute(EvmCdBytes1, EvmCdArray<Bytes, 1, 1>, U),
}

fn derived_pool_key(pool_key: PoolKey) -> DerivedPoolKey {
    DerivedPoolKey {
        currency0: EvmCdAddress::new(pool_key.currency0),
        currency1: EvmCdAddress::new(pool_key.currency1),
        fee: EvmCdU24::new(pool_key.fee),
        tick_spacing: EvmCdI24::new(pool_key.tick_spacing),
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
        hook_data,
    }
}

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

pub fn make_fn_quote_exact_input_single_array<const HOOK_DATA_LEN: usize, const ALL: usize>(
    params: QuoteExactSingleParams<HOOK_DATA_LEN>,
) -> [u8; ALL] {
    let required = quote_exact_single_calldata_len(HOOK_DATA_LEN);
    assert_eq!(ALL, required, "calldata output array has the wrong length");
    DerivedV4QuoterCall::QuoteExactInputSingle(derived_quote(
        params.pool_key,
        params.zero_for_one,
        params.exact_amount,
        EvmCdBytes::<HOOK_DATA_LEN>::from(params.hook_data),
    ))
    .serialise_to_array()
    .expect("Uniswap V4 calldata serialization failed")
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
    assert_eq!(ALL, required, "calldata output array has the wrong length");
    DerivedV4QuoterCall::QuoteExactOutputSingle(derived_quote(
        params.pool_key,
        params.zero_for_one,
        params.exact_amount,
        EvmCdBytes::<HOOK_DATA_LEN>::from(params.hook_data),
    ))
    .serialise_to_array()
    .expect("Uniswap V4 calldata serialization failed")
}

#[cfg(feature = "alloc")]
pub fn make_fn_quote_exact_input_single_vec(params: QuoteExactSingleParamsVec) -> Vec<u8> {
    quote_exact_single_calldata_len(params.hook_data.len());
    DerivedV4QuoterCall::QuoteExactInputSingle(derived_quote(
        params.pool_key,
        params.zero_for_one,
        params.exact_amount,
        params.hook_data,
    ))
    .serialise_to_vec()
    .expect("Uniswap V4 calldata serialization failed")
}

#[cfg(feature = "alloc")]
pub fn make_fn_quote_exact_output_single_vec(params: QuoteExactSingleParamsVec) -> Vec<u8> {
    quote_exact_single_calldata_len(params.hook_data.len());
    DerivedV4QuoterCall::QuoteExactOutputSingle(derived_quote(
        params.pool_key,
        params.zero_for_one,
        params.exact_amount,
        params.hook_data,
    ))
    .serialise_to_vec()
    .expect("Uniswap V4 calldata serialization failed")
}

#[allow(clippy::too_many_arguments)]
fn derived_swap_params<HookData>(
    pool_key: PoolKey,
    zero_for_one: bool,
    amount_in: u128,
    amount_out_minimum: u128,
    min_hop_price_x36: U,
    hook_data: HookData,
) -> DerivedSwapParams<HookData> {
    DerivedSwapParams {
        params: DerivedExactInputSingleParams {
            pool_key: derived_pool_key(pool_key),
            zero_for_one,
            amount_in,
            amount_out_minimum,
            min_hop_price_x36,
            hook_data,
        },
    }
}

fn derived_currency_params(
    pool_key: PoolKey,
    zero_for_one: bool,
    amount_in: u128,
    amount_out_minimum: u128,
) -> (DerivedCurrencyAmount, DerivedCurrencyAmount) {
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
    (
        DerivedCurrencyAmount {
            currency: EvmCdAddress::new(currency_in),
            amount: amount_in,
        },
        DerivedCurrencyAmount {
            currency: EvmCdAddress::new(currency_out),
            amount: amount_out_minimum,
        },
    )
}

fn derived_plan<Bytes>(swap: Bytes, settle: Bytes, take: Bytes) -> DerivedPlan<Bytes> {
    DerivedPlan {
        actions: EvmCdBytes3::from([
            ACTION_SWAP_EXACT_IN_SINGLE,
            ACTION_SETTLE_ALL,
            ACTION_TAKE_ALL,
        ]),
        params: EvmCdArray::try_from_array([swap, settle, take], 3)
            .expect("failed to construct the V4 action parameters"),
    }
}

fn derived_execute_call<Bytes>(plan: Bytes, deadline: U) -> DerivedUniversalRouterCall<Bytes> {
    let inputs = EvmCdArray::try_from_array([plan], 1)
        .expect("failed to construct the Universal Router inputs");
    DerivedUniversalRouterCall::Execute(EvmCdBytes1::from([COMMAND_V4_SWAP]), inputs, deadline)
}

pub fn make_fn_execute_exact_input_single_array<const HOOK_DATA_LEN: usize, const ALL: usize>(
    swap: ExactInputSingle<HOOK_DATA_LEN>,
    deadline: U,
) -> [u8; ALL] {
    let required = exact_input_single_calldata_len(HOOK_DATA_LEN);
    assert_eq!(ALL, required, "calldata output array has the wrong length");
    let swap_params = derived_swap_params(
        swap.pool_key,
        swap.zero_for_one,
        swap.amount_in,
        swap.amount_out_minimum,
        swap.min_hop_price_x36,
        EvmCdBytes::<HOOK_DATA_LEN>::from(swap.hook_data),
    );
    let (settle, take) = derived_currency_params(
        swap.pool_key,
        swap.zero_for_one,
        swap.amount_in,
        swap.amount_out_minimum,
    );
    let swap_params = EvmCdBytes::<ALL>::try_from_serialised(&swap_params)
        .expect("failed to serialize the swap parameter");
    let settle = EvmCdBytes::<ALL>::try_from_serialised(&settle)
        .expect("failed to serialize the settle parameter");
    let take = EvmCdBytes::<ALL>::try_from_serialised(&take)
        .expect("failed to serialize the take parameter");
    let plan = derived_plan(swap_params, settle, take);
    let plan =
        EvmCdBytes::<ALL>::try_from_serialised(&plan).expect("failed to serialize the V4 plan");
    let call = derived_execute_call(plan, deadline);
    call.serialise_to_array()
        .expect("Uniswap V4 calldata serialization failed")
}

#[cfg(feature = "alloc")]
pub fn make_fn_execute_exact_input_single_vec(swap: ExactInputSingleVec, deadline: U) -> Vec<u8> {
    exact_input_single_calldata_len(swap.hook_data.len());
    let swap_params = derived_swap_params(
        swap.pool_key,
        swap.zero_for_one,
        swap.amount_in,
        swap.amount_out_minimum,
        swap.min_hop_price_x36,
        swap.hook_data,
    );
    let (settle, take) = derived_currency_params(
        swap.pool_key,
        swap.zero_for_one,
        swap.amount_in,
        swap.amount_out_minimum,
    );
    let plan = derived_plan(
        swap_params
            .serialise_to_vec()
            .expect("failed to serialize V4 swap parameters"),
        settle
            .serialise_to_vec()
            .expect("failed to serialize V4 settle parameters"),
        take.serialise_to_vec()
            .expect("failed to serialize V4 take parameters"),
    );
    let plan = plan
        .serialise_to_vec()
        .expect("failed to serialize the V4 plan");
    derived_execute_call(plan, deadline)
        .serialise_to_vec()
        .expect("Uniswap V4 calldata serialization failed")
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
    fn execute_supports_the_largest_generated_hook_byte_capacity() {
        let swap = ExactInputSingle {
            pool_key: pool_key(),
            zero_for_one: true,
            amount_in: 123,
            amount_out_minimum: 45,
            min_hop_price_x36: U::from_u8(7),
            hook_data: [0xabu8; 1024],
        };

        let actual: [u8; 2148] = make_fn_execute_exact_input_single_array(swap, U::from_u8(9));

        assert_eq!(&actual[..4], &[0x35, 0x93, 0x56, 0x4c]);
        assert_eq!(actual.len(), exact_input_single_calldata_len(1024));
    }

    #[test]
    fn execute_has_no_intermediate_byte_capacity_limit() {
        const HOOK_LEN: usize = 2049;
        const CALLDATA_LEN: usize = exact_input_single_calldata_len(HOOK_LEN);
        let swap = ExactInputSingle {
            pool_key: pool_key(),
            zero_for_one: true,
            amount_in: 42,
            amount_out_minimum: 24,
            min_hop_price_x36: U::ZERO,
            hook_data: [0xabu8; HOOK_LEN],
        };

        let actual: [u8; CALLDATA_LEN] =
            make_fn_execute_exact_input_single_array(swap, U::from_u64(123));

        assert_eq!(&actual[..4], &[0x35, 0x93, 0x56, 0x4c]);
        assert_eq!(actual.len(), CALLDATA_LEN);
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

    #[cfg(feature = "alloc")]
    #[test]
    fn execute_vec_variant_preserves_hook_data_above_generated_alias_range() {
        const HOOK_LEN: usize = 1025;
        const CALLDATA_LEN: usize = exact_input_single_calldata_len(HOOK_LEN);
        let hook_data = [0xabu8; HOOK_LEN];
        let fixed: [u8; CALLDATA_LEN] = make_fn_execute_exact_input_single_array(
            ExactInputSingle {
                pool_key: pool_key(),
                zero_for_one: false,
                amount_in: 123,
                amount_out_minimum: 45,
                min_hop_price_x36: U::from_u8(7),
                hook_data,
            },
            U::from_u8(9),
        );
        let dynamic = make_fn_execute_exact_input_single_vec(
            ExactInputSingleVec {
                pool_key: pool_key(),
                zero_for_one: false,
                amount_in: 123,
                amount_out_minimum: 45,
                min_hop_price_x36: U::from_u8(7),
                hook_data: hook_data.to_vec(),
            },
            U::from_u8(9),
        );

        assert_eq!(fixed.to_vec(), dynamic);
    }
}
