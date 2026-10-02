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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use alloy_primitives::{
        Address as AlloyAddress, Bytes, U256 as AlloyU256,
        aliases::{I24 as AlloyI24, U24 as AlloyU24},
    };
    use alloy_sol_macro::sol;
    use alloy_sol_types::{SolCall, SolValue};
    use proptest::prelude::*;

    const FIXED_HOOK_LEN: usize = 65;
    const FIXED_QUOTE_LEN: usize = quote_exact_single_calldata_len(FIXED_HOOK_LEN);
    const FIXED_EXECUTE_LEN: usize = exact_input_single_calldata_len(FIXED_HOOK_LEN);

    sol! {
        struct OraclePoolKey {
            address currency0;
            address currency1;
            uint24 fee;
            int24 tickSpacing;
            address hooks;
        }

        struct OracleQuoteExactSingleParams {
            OraclePoolKey poolKey;
            bool zeroForOne;
            uint128 exactAmount;
            bytes hookData;
        }

        struct OracleExactInputSingleParams {
            OraclePoolKey poolKey;
            bool zeroForOne;
            uint128 amountIn;
            uint128 amountOutMinimum;
            uint256 minHopPriceX36;
            bytes hookData;
        }

        function quoteExactInputSingle(OracleQuoteExactSingleParams memory params)
            external returns (uint256 amountOut, uint256 gasEstimate);
        function quoteExactOutputSingle(OracleQuoteExactSingleParams memory params)
            external returns (uint256 amountIn, uint256 gasEstimate);
        function execute(bytes commands, bytes[] inputs, uint256 deadline);
    }

    fn pool_key(
        currency0: Address,
        currency1: Address,
        fee: U24,
        tick_spacing: I24,
        hooks: Address,
    ) -> PoolKey {
        PoolKey {
            currency0,
            currency1,
            fee,
            tick_spacing,
            hooks,
        }
    }

    fn oracle_pool_key(pool: PoolKey) -> OraclePoolKey {
        let sign = if pool.tick_spacing[0] & 0x80 == 0 {
            0
        } else {
            0xff
        };
        OraclePoolKey {
            currency0: AlloyAddress::from(pool.currency0),
            currency1: AlloyAddress::from(pool.currency1),
            fee: AlloyU24::from_be_bytes(pool.fee),
            tickSpacing: AlloyI24::try_from(i32::from_be_bytes([
                sign,
                pool.tick_spacing[0],
                pool.tick_spacing[1],
                pool.tick_spacing[2],
            ]))
            .unwrap(),
            hooks: AlloyAddress::from(pool.hooks),
        }
    }

    fn oracle_quote(
        pool: PoolKey,
        zero_for_one: bool,
        exact_amount: u128,
        hook_data: &[u8],
    ) -> OracleQuoteExactSingleParams {
        OracleQuoteExactSingleParams {
            poolKey: oracle_pool_key(pool),
            zeroForOne: zero_for_one,
            exactAmount: exact_amount,
            hookData: Bytes::copy_from_slice(hook_data),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn oracle_execute(
        pool: PoolKey,
        zero_for_one: bool,
        amount_in: u128,
        amount_out_minimum: u128,
        min_hop_price_x36: U,
        hook_data: &[u8],
        deadline: U,
    ) -> Vec<u8> {
        let swap = (OracleExactInputSingleParams {
            poolKey: oracle_pool_key(pool),
            zeroForOne: zero_for_one,
            amountIn: amount_in,
            amountOutMinimum: amount_out_minimum,
            minHopPriceX36: AlloyU256::from_be_bytes(*min_hop_price_x36),
            hookData: Bytes::copy_from_slice(hook_data),
        },)
            .abi_encode_params();
        let (currency_in, currency_out) = if zero_for_one {
            (pool.currency0, pool.currency1)
        } else {
            (pool.currency1, pool.currency0)
        };
        let settle = (AlloyAddress::from(currency_in), amount_in).abi_encode_params();
        let take = (AlloyAddress::from(currency_out), amount_out_minimum).abi_encode_params();
        let plan = (
            Bytes::copy_from_slice(&[0x06, 0x0c, 0x0f]),
            vec![Bytes::from(swap), Bytes::from(settle), Bytes::from(take)],
        )
            .abi_encode_params();

        executeCall {
            commands: Bytes::copy_from_slice(&[0x10]),
            inputs: vec![Bytes::from(plan)],
            deadline: AlloyU256::from_be_bytes(*deadline),
        }
        .abi_encode()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        #[test]
        fn fixed_quote_builders_match_alloy(
            currency0 in any::<Address>(),
            currency1 in any::<Address>(),
            fee in any::<U24>(),
            tick_spacing in any::<I24>(),
            hooks in any::<Address>(),
            zero_for_one in any::<bool>(),
            exact_amount in any::<u128>(),
            hook_data in any::<[u8; FIXED_HOOK_LEN]>(),
        ) {
            let pool = pool_key(currency0, currency1, fee, tick_spacing, hooks);
            let params = QuoteExactSingleParams {
                pool_key: pool,
                zero_for_one,
                exact_amount,
                hook_data,
            };

            let input: [u8; FIXED_QUOTE_LEN] = make_fn_quote_exact_input_single_array(params);
            let expected_input = quoteExactInputSingleCall {
                params: oracle_quote(pool, zero_for_one, exact_amount, &hook_data),
            }
            .abi_encode();
            prop_assert_eq!(input.as_slice(), expected_input.as_slice());

            let output: [u8; FIXED_QUOTE_LEN] = make_fn_quote_exact_output_single_array(params);
            let expected_output = quoteExactOutputSingleCall {
                params: oracle_quote(pool, zero_for_one, exact_amount, &hook_data),
            }
            .abi_encode();
            prop_assert_eq!(output.as_slice(), expected_output.as_slice());
        }

        #[test]
        fn zero_hook_quote_builder_matches_alloy(
            currency0 in any::<Address>(),
            currency1 in any::<Address>(),
            fee in any::<U24>(),
            tick_spacing in any::<I24>(),
            hooks in any::<Address>(),
            zero_for_one in any::<bool>(),
            exact_amount in any::<u128>(),
        ) {
            let pool = pool_key(currency0, currency1, fee, tick_spacing, hooks);
            let actual = make_fn_quote_exact_input_zero_hooks(QuoteExactSingleParams {
                pool_key: pool,
                zero_for_one,
                exact_amount,
                hook_data: [],
            });
            let expected = quoteExactInputSingleCall {
                params: oracle_quote(pool, zero_for_one, exact_amount, &[]),
            }
            .abi_encode();

            prop_assert_eq!(actual.as_slice(), expected.as_slice());
        }

        #[test]
        fn fixed_execute_builder_matches_alloy(
            currency0 in any::<Address>(),
            currency1 in any::<Address>(),
            fee in any::<U24>(),
            tick_spacing in any::<I24>(),
            hooks in any::<Address>(),
            zero_for_one in any::<bool>(),
            amount_in in any::<u128>(),
            amount_out_minimum in any::<u128>(),
            min_hop_price_x36 in any::<U>(),
            hook_data in any::<[u8; FIXED_HOOK_LEN]>(),
            deadline in any::<U>(),
        ) {
            let pool = pool_key(currency0, currency1, fee, tick_spacing, hooks);
            let actual: [u8; FIXED_EXECUTE_LEN] = make_fn_execute_exact_input_single_array(
                ExactInputSingle {
                    pool_key: pool,
                    zero_for_one,
                    amount_in,
                    amount_out_minimum,
                    min_hop_price_x36,
                    hook_data,
                },
                deadline,
            );
            let expected = oracle_execute(
                pool,
                zero_for_one,
                amount_in,
                amount_out_minimum,
                min_hop_price_x36,
                &hook_data,
                deadline,
            );

            prop_assert_eq!(actual.as_slice(), expected.as_slice());
        }
    }

    #[cfg(feature = "alloc")]
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        #[test]
        fn vec_quote_builders_match_alloy(
            currency0 in any::<Address>(),
            currency1 in any::<Address>(),
            fee in any::<U24>(),
            tick_spacing in any::<I24>(),
            hooks in any::<Address>(),
            zero_for_one in any::<bool>(),
            exact_amount in any::<u128>(),
            hook_data in prop::collection::vec(any::<u8>(), 0..=2048),
        ) {
            let pool = pool_key(currency0, currency1, fee, tick_spacing, hooks);
            let expected_input = quoteExactInputSingleCall {
                params: oracle_quote(pool, zero_for_one, exact_amount, &hook_data),
            }
            .abi_encode();
            let expected_output = quoteExactOutputSingleCall {
                params: oracle_quote(pool, zero_for_one, exact_amount, &hook_data),
            }
            .abi_encode();

            let input = make_fn_quote_exact_input_single_vec(QuoteExactSingleParamsVec {
                pool_key: pool,
                zero_for_one,
                exact_amount,
                hook_data: hook_data.clone(),
            });
            let output = make_fn_quote_exact_output_single_vec(QuoteExactSingleParamsVec {
                pool_key: pool,
                zero_for_one,
                exact_amount,
                hook_data,
            });

            prop_assert_eq!(input, expected_input);
            prop_assert_eq!(output, expected_output);
        }

        #[test]
        fn vec_execute_builder_matches_alloy(
            currency0 in any::<Address>(),
            currency1 in any::<Address>(),
            fee in any::<U24>(),
            tick_spacing in any::<I24>(),
            hooks in any::<Address>(),
            zero_for_one in any::<bool>(),
            amount_in in any::<u128>(),
            amount_out_minimum in any::<u128>(),
            min_hop_price_x36 in any::<U>(),
            hook_data in prop::collection::vec(any::<u8>(), 0..=2048),
            deadline in any::<U>(),
        ) {
            let pool = pool_key(currency0, currency1, fee, tick_spacing, hooks);
            let expected = oracle_execute(
                pool,
                zero_for_one,
                amount_in,
                amount_out_minimum,
                min_hop_price_x36,
                &hook_data,
                deadline,
            );
            let actual = make_fn_execute_exact_input_single_vec(
                ExactInputSingleVec {
                    pool_key: pool,
                    zero_for_one,
                    amount_in,
                    amount_out_minimum,
                    min_hop_price_x36,
                    hook_data,
                },
                deadline,
            );

            prop_assert_eq!(actual, expected);
        }
    }
}
