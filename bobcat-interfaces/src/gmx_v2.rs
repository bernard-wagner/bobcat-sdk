//! Minimal GMX V2 core order calldata builders.
//!
//! Collateral and execution fees must be transferred to GMX's OrderVault before
//! creating an order, normally in the same ExchangeRouter multicall.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use bobcat_cd::{EvmCdAddress, EvmCdArray, EvmCdSerialise};
use bobcat_maths::U;

/// An EVM address.
pub type Address = [u8; 20];

/// GMX order types that increase a position.
#[derive(Clone, Copy, Debug, Eq, PartialEq, EvmCdSerialise)]
#[repr(u8)]
pub enum IncreaseOrderType {
    Market = 2,
    Limit = 3,
    Stop = 8,
}

/// GMX order types that decrease a position.
#[derive(Clone, Copy, Debug, Eq, PartialEq, EvmCdSerialise)]
#[repr(u8)]
pub enum DecreaseOrderType {
    Market = 4,
    Limit = 5,
    StopLoss = 6,
}

/// Handling for the PnL token during a decrease.
#[derive(Clone, Copy, Debug, Eq, PartialEq, EvmCdSerialise)]
#[repr(u8)]
pub enum DecreasePositionSwapType {
    NoSwap = 0,
    SwapPnlTokenToCollateralToken = 1,
    SwapCollateralTokenToPnlToken = 2,
}

/// Address fields from GMX's `CreateOrderParamsAddresses`, owning a
/// compile-time-sized swap path without requiring an allocator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrderAddresses<const SWAP_PATH_LEN: usize> {
    pub receiver: Address,
    pub cancellation_receiver: Address,
    pub callback_contract: Address,
    pub ui_fee_receiver: Address,
    pub market: Address,
    pub initial_collateral_token: Address,
    pub swap_path: [EvmCdAddress; SWAP_PATH_LEN],
}

/// Address fields from GMX's `CreateOrderParamsAddresses`, owning the swap path.
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderAddressesVec {
    pub receiver: Address,
    pub cancellation_receiver: Address,
    pub callback_contract: Address,
    pub ui_fee_receiver: Address,
    pub market: Address,
    pub initial_collateral_token: Address,
    pub swap_path: Vec<EvmCdAddress>,
}

/// Numeric fields from GMX's `CreateOrderParamsNumbers`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrderNumbers {
    pub size_delta_usd: U,
    pub initial_collateral_delta_amount: U,
    pub trigger_price: U,
    pub acceptable_price: U,
    pub execution_fee: U,
    pub callback_gas_limit: U,
    pub min_output_amount: U,
    pub valid_from_time: U,
}

/// Shared inputs for GMX increase and decrease orders without allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrderParams<const SWAP_PATH_LEN: usize> {
    pub addresses: OrderAddresses<SWAP_PATH_LEN>,
    pub numbers: OrderNumbers,
    pub decrease_position_swap_type: DecreasePositionSwapType,
    pub is_long: bool,
    pub should_unwrap_native_token: bool,
    pub auto_cancel: bool,
}

/// Shared inputs for GMX increase and decrease orders using an owned vector.
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderParamsVec {
    pub addresses: OrderAddressesVec,
    pub numbers: OrderNumbers,
    pub decrease_position_swap_type: DecreasePositionSwapType,
    pub is_long: bool,
    pub should_unwrap_native_token: bool,
    pub auto_cancel: bool,
}

/// Encoded `createOrder` length for a swap path containing `swap_path_len` addresses.
pub const fn create_order_calldata_len(swap_path_len: usize) -> usize {
    4 + 32 * (26 + swap_path_len)
}

/// Encoded `claimFundingFees` length for the supplied array lengths.
pub const fn claim_funding_fees_calldata_len(markets_len: usize, tokens_len: usize) -> usize {
    4 + 32 * (5 + markets_len + tokens_len)
}

/// Encoded `claimCollateral` length for the supplied array lengths.
pub const fn claim_collateral_calldata_len(
    markets_len: usize,
    tokens_len: usize,
    time_keys_len: usize,
) -> usize {
    4 + 32 * (7 + markets_len + tokens_len + time_keys_len)
}

#[derive(EvmCdSerialise)]
struct DerivedOrderAddresses<SwapPath> {
    receiver: EvmCdAddress,
    cancellation_receiver: EvmCdAddress,
    callback_contract: EvmCdAddress,
    ui_fee_receiver: EvmCdAddress,
    market: EvmCdAddress,
    initial_collateral_token: EvmCdAddress,
    swap_path: SwapPath,
}

#[derive(EvmCdSerialise)]
struct DerivedOrderNumbers {
    size_delta_usd: U,
    initial_collateral_delta_amount: U,
    trigger_price: U,
    acceptable_price: U,
    execution_fee: U,
    callback_gas_limit: U,
    min_output_amount: U,
    valid_from_time: U,
}

#[derive(EvmCdSerialise)]
struct DerivedCreateOrderParams<SwapPath, DataList, OrderType> {
    addresses: DerivedOrderAddresses<SwapPath>,
    numbers: DerivedOrderNumbers,
    order_type: OrderType,
    decrease_position_swap_type: DecreasePositionSwapType,
    is_long: bool,
    should_unwrap_native_token: bool,
    auto_cancel: bool,
    referral_code: [u8; 32],
    data_list: DataList,
}

#[derive(EvmCdSerialise)]
#[evm_selector]
enum DerivedCreateOrderCall<SwapPath, DataList, OrderType> {
    CreateOrder(DerivedCreateOrderParams<SwapPath, DataList, OrderType>),
}

#[derive(EvmCdSerialise)]
#[evm_selector]
enum DerivedCancelOrderCall {
    CancelOrder([u8; 32]),
}

#[derive(EvmCdSerialise)]
#[evm_selector]
enum DerivedClaimFundingFeesCall<Markets, Tokens> {
    ClaimFundingFees(Markets, Tokens, EvmCdAddress),
}

#[derive(EvmCdSerialise)]
#[evm_selector]
enum DerivedClaimCollateralCall<Markets, Tokens, TimeKeys> {
    ClaimCollateral(Markets, Tokens, TimeKeys, EvmCdAddress),
}

fn derived_order_numbers(numbers: OrderNumbers) -> DerivedOrderNumbers {
    DerivedOrderNumbers {
        size_delta_usd: numbers.size_delta_usd,
        initial_collateral_delta_amount: numbers.initial_collateral_delta_amount,
        trigger_price: numbers.trigger_price,
        acceptable_price: numbers.acceptable_price,
        execution_fee: numbers.execution_fee,
        callback_gas_limit: numbers.callback_gas_limit,
        min_output_amount: numbers.min_output_amount,
        valid_from_time: numbers.valid_from_time,
    }
}

fn derived_create_order_params_array<const SWAP_PATH_LEN: usize, OrderType>(
    params: OrderParams<SWAP_PATH_LEN>,
    order_type: OrderType,
) -> DerivedCreateOrderParams<
    EvmCdArray<EvmCdAddress, SWAP_PATH_LEN, SWAP_PATH_LEN>,
    EvmCdArray<[u8; 32], 0, 0>,
    OrderType,
> {
    DerivedCreateOrderParams {
        addresses: DerivedOrderAddresses {
            receiver: EvmCdAddress::new(params.addresses.receiver),
            cancellation_receiver: EvmCdAddress::new(params.addresses.cancellation_receiver),
            callback_contract: EvmCdAddress::new(params.addresses.callback_contract),
            ui_fee_receiver: EvmCdAddress::new(params.addresses.ui_fee_receiver),
            market: EvmCdAddress::new(params.addresses.market),
            initial_collateral_token: EvmCdAddress::new(params.addresses.initial_collateral_token),
            swap_path: EvmCdArray::try_from_array(params.addresses.swap_path, SWAP_PATH_LEN)
                .expect("the swap path fills its fixed-size array"),
        },
        numbers: derived_order_numbers(params.numbers),
        order_type,
        decrease_position_swap_type: params.decrease_position_swap_type,
        is_long: params.is_long,
        should_unwrap_native_token: params.should_unwrap_native_token,
        auto_cancel: params.auto_cancel,
        referral_code: [0; 32],
        data_list: EvmCdArray::try_from_array([], 0)
            .expect("the empty data list fits its zero-length array"),
    }
}

#[cfg(feature = "alloc")]
fn derived_create_order_params_vec<OrderType>(
    params: OrderParamsVec,
    order_type: OrderType,
) -> DerivedCreateOrderParams<Vec<EvmCdAddress>, Vec<[u8; 32]>, OrderType> {
    DerivedCreateOrderParams {
        addresses: DerivedOrderAddresses {
            receiver: EvmCdAddress::new(params.addresses.receiver),
            cancellation_receiver: EvmCdAddress::new(params.addresses.cancellation_receiver),
            callback_contract: EvmCdAddress::new(params.addresses.callback_contract),
            ui_fee_receiver: EvmCdAddress::new(params.addresses.ui_fee_receiver),
            market: EvmCdAddress::new(params.addresses.market),
            initial_collateral_token: EvmCdAddress::new(params.addresses.initial_collateral_token),
            swap_path: params.addresses.swap_path,
        },
        numbers: derived_order_numbers(params.numbers),
        order_type,
        decrease_position_swap_type: params.decrease_position_swap_type,
        is_long: params.is_long,
        should_unwrap_native_token: params.should_unwrap_native_token,
        auto_cancel: params.auto_cancel,
        referral_code: [0; 32],
        data_list: Vec::new(),
    }
}

fn encode_array_call<const ALL: usize>(
    call: &impl EvmCdSerialise,
    expected_len: usize,
) -> [u8; ALL] {
    assert!(
        ALL == expected_len,
        "ALL is inconsistent with the calldata length"
    );
    let mut out = [0; ALL];
    call.serialise(&mut out)
        .expect("GMX calldata serialisation must succeed");
    out
}

#[cfg(feature = "alloc")]
fn encode_dynamic_call(call: &impl EvmCdSerialise) -> Vec<u8> {
    let mut out = Vec::new();
    call.serialise(&mut out)
        .expect("GMX calldata serialisation must succeed");
    out
}

/// Encode a GMX V2 increase order without allocation.
pub fn make_fn_create_increase_order_array<const SWAP_PATH_LEN: usize, const ALL: usize>(
    params: OrderParams<SWAP_PATH_LEN>,
    order_type: IncreaseOrderType,
) -> [u8; ALL] {
    encode_array_call(
        &DerivedCreateOrderCall::CreateOrder(derived_create_order_params_array(
            params, order_type,
        )),
        create_order_calldata_len(SWAP_PATH_LEN),
    )
}

/// Encode a GMX V2 decrease order without allocation.
pub fn make_fn_create_decrease_order_array<const SWAP_PATH_LEN: usize, const ALL: usize>(
    params: OrderParams<SWAP_PATH_LEN>,
    order_type: DecreaseOrderType,
) -> [u8; ALL] {
    encode_array_call(
        &DerivedCreateOrderCall::CreateOrder(derived_create_order_params_array(
            params, order_type,
        )),
        create_order_calldata_len(SWAP_PATH_LEN),
    )
}

/// Encode a GMX V2 increase order using allocation.
#[cfg(feature = "alloc")]
pub fn make_fn_create_increase_order_vec(
    params: OrderParamsVec,
    order_type: IncreaseOrderType,
) -> Vec<u8> {
    encode_dynamic_call(&DerivedCreateOrderCall::CreateOrder(
        derived_create_order_params_vec(params, order_type),
    ))
}

/// Encode a GMX V2 decrease order using allocation.
#[cfg(feature = "alloc")]
pub fn make_fn_create_decrease_order_vec(
    params: OrderParamsVec,
    order_type: DecreaseOrderType,
) -> Vec<u8> {
    encode_dynamic_call(&DerivedCreateOrderCall::CreateOrder(
        derived_create_order_params_vec(params, order_type),
    ))
}

/// Encode `ExchangeRouter.cancelOrder` for either an increase or decrease key.
pub fn make_fn_cancel_order(key: [u8; 32]) -> [u8; 36] {
    DerivedCancelOrderCall::CancelOrder(key)
        .to_evm_array()
        .expect("cancelOrder calldata has a fixed 36-byte encoding")
}

/// Encode a claim for positive funding fees without allocation.
pub fn make_fn_claim_funding_fees_array<
    const MARKETS_LEN: usize,
    const TOKENS_LEN: usize,
    const ALL: usize,
>(
    markets: [EvmCdAddress; MARKETS_LEN],
    tokens: [EvmCdAddress; TOKENS_LEN],
    receiver: Address,
) -> [u8; ALL] {
    let markets = EvmCdArray::<EvmCdAddress, MARKETS_LEN, MARKETS_LEN>::try_from_array(
        markets,
        MARKETS_LEN,
    )
    .expect("the markets fill their fixed-size array");
    let tokens = EvmCdArray::<EvmCdAddress, TOKENS_LEN, TOKENS_LEN>::try_from_array(
        tokens,
        TOKENS_LEN,
    )
    .expect("the tokens fill their fixed-size array");
    encode_array_call(
        &DerivedClaimFundingFeesCall::ClaimFundingFees(
            markets,
            tokens,
            EvmCdAddress::new(receiver),
        ),
        claim_funding_fees_calldata_len(MARKETS_LEN, TOKENS_LEN),
    )
}

/// Encode a claim for positive funding fees using allocation.
#[cfg(feature = "alloc")]
pub fn make_fn_claim_funding_fees_vec(
    markets: Vec<EvmCdAddress>,
    tokens: Vec<EvmCdAddress>,
    receiver: Address,
) -> Vec<u8> {
    encode_dynamic_call(&DerivedClaimFundingFeesCall::ClaimFundingFees(
        markets,
        tokens,
        EvmCdAddress::new(receiver),
    ))
}

/// Encode a claim for collateral retained after capped negative price impact without allocation.
pub fn make_fn_claim_collateral_array<
    const MARKETS_LEN: usize,
    const TOKENS_LEN: usize,
    const TIME_KEYS_LEN: usize,
    const ALL: usize,
>(
    markets: [EvmCdAddress; MARKETS_LEN],
    tokens: [EvmCdAddress; TOKENS_LEN],
    time_keys: [U; TIME_KEYS_LEN],
    receiver: Address,
) -> [u8; ALL] {
    let markets = EvmCdArray::<EvmCdAddress, MARKETS_LEN, MARKETS_LEN>::try_from_array(
        markets,
        MARKETS_LEN,
    )
    .expect("the markets fill their fixed-size array");
    let tokens = EvmCdArray::<EvmCdAddress, TOKENS_LEN, TOKENS_LEN>::try_from_array(
        tokens,
        TOKENS_LEN,
    )
    .expect("the tokens fill their fixed-size array");
    let time_keys = EvmCdArray::<U, TIME_KEYS_LEN, TIME_KEYS_LEN>::try_from_array(
        time_keys,
        TIME_KEYS_LEN,
    )
    .expect("the time keys fill their fixed-size array");
    encode_array_call(
        &DerivedClaimCollateralCall::ClaimCollateral(
            markets,
            tokens,
            time_keys,
            EvmCdAddress::new(receiver),
        ),
        claim_collateral_calldata_len(MARKETS_LEN, TOKENS_LEN, TIME_KEYS_LEN),
    )
}

/// Encode a claim for collateral retained after capped negative price impact using allocation.
#[cfg(feature = "alloc")]
pub fn make_fn_claim_collateral_vec(
    markets: Vec<EvmCdAddress>,
    tokens: Vec<EvmCdAddress>,
    time_keys: Vec<U>,
    receiver: Address,
) -> Vec<u8> {
    encode_dynamic_call(&DerivedClaimCollateralCall::ClaimCollateral(
        markets,
        tokens,
        time_keys,
        EvmCdAddress::new(receiver),
    ))
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use alloy_primitives::{Address as AlloyAddress, FixedBytes, U256 as AlloyU256};
    use alloy_sol_macro::sol;
    use alloy_sol_types::SolCall;
    use proptest::{collection::vec, prelude::*};

    sol! {
        struct CreateOrderParamsAddresses {
            address receiver;
            address cancellationReceiver;
            address callbackContract;
            address uiFeeReceiver;
            address market;
            address initialCollateralToken;
            address[] swapPath;
        }

        struct CreateOrderParamsNumbers {
            uint256 sizeDeltaUsd;
            uint256 initialCollateralDeltaAmount;
            uint256 triggerPrice;
            uint256 acceptablePrice;
            uint256 executionFee;
            uint256 callbackGasLimit;
            uint256 minOutputAmount;
            uint256 validFromTime;
        }

        struct CreateOrderParams {
            CreateOrderParamsAddresses addresses;
            CreateOrderParamsNumbers numbers;
            uint8 orderType;
            uint8 decreasePositionSwapType;
            bool isLong;
            bool shouldUnwrapNativeToken;
            bool autoCancel;
            bytes32 referralCode;
            bytes32[] dataList;
        }

        function createOrder(CreateOrderParams calldata params)
            external
            payable
            returns (bytes32 key);

        function claimFundingFees(address[] calldata markets, address[] calldata tokens, address receiver)
            external
            returns (uint256[] memory claimedAmounts);

        function claimCollateral(
            address[] calldata markets,
            address[] calldata tokens,
            uint256[] calldata timeKeys,
            address receiver
        ) external returns (uint256[] memory claimedAmounts);
    }

    fn increase_order_type() -> impl Strategy<Value = IncreaseOrderType> {
        prop_oneof![
            Just(IncreaseOrderType::Market),
            Just(IncreaseOrderType::Limit),
            Just(IncreaseOrderType::Stop),
        ]
    }

    fn decrease_order_type() -> impl Strategy<Value = DecreaseOrderType> {
        prop_oneof![
            Just(DecreaseOrderType::Market),
            Just(DecreaseOrderType::Limit),
            Just(DecreaseOrderType::StopLoss),
        ]
    }

    fn decrease_position_swap_type() -> impl Strategy<Value = DecreasePositionSwapType> {
        prop_oneof![
            Just(DecreasePositionSwapType::NoSwap),
            Just(DecreasePositionSwapType::SwapPnlTokenToCollateralToken),
            Just(DecreasePositionSwapType::SwapCollateralTokenToPnlToken),
        ]
    }

    fn order_params(
        addresses: [[u8; 20]; 6],
        swap_path: Vec<[u8; 20]>,
        numbers: [U; 8],
        decrease_position_swap_type: DecreasePositionSwapType,
        flags: [bool; 3],
    ) -> OrderParamsVec {
        OrderParamsVec {
            addresses: OrderAddressesVec {
                receiver: addresses[0],
                cancellation_receiver: addresses[1],
                callback_contract: addresses[2],
                ui_fee_receiver: addresses[3],
                market: addresses[4],
                initial_collateral_token: addresses[5],
                swap_path: swap_path.into_iter().map(EvmCdAddress::new).collect(),
            },
            numbers: OrderNumbers {
                size_delta_usd: numbers[0],
                initial_collateral_delta_amount: numbers[1],
                trigger_price: numbers[2],
                acceptable_price: numbers[3],
                execution_fee: numbers[4],
                callback_gas_limit: numbers[5],
                min_output_amount: numbers[6],
                valid_from_time: numbers[7],
            },
            decrease_position_swap_type,
            is_long: flags[0],
            should_unwrap_native_token: flags[1],
            auto_cancel: flags[2],
        }
    }

    fn alloy_u(value: U) -> AlloyU256 {
        AlloyU256::from_be_bytes(*value)
    }

    fn alloy_create_order(params: &OrderParamsVec, order_type: u8) -> Vec<u8> {
        createOrderCall {
            params: CreateOrderParams {
                addresses: CreateOrderParamsAddresses {
                    receiver: AlloyAddress::from(params.addresses.receiver),
                    cancellationReceiver: AlloyAddress::from(
                        params.addresses.cancellation_receiver,
                    ),
                    callbackContract: AlloyAddress::from(params.addresses.callback_contract),
                    uiFeeReceiver: AlloyAddress::from(params.addresses.ui_fee_receiver),
                    market: AlloyAddress::from(params.addresses.market),
                    initialCollateralToken: AlloyAddress::from(
                        params.addresses.initial_collateral_token,
                    ),
                    swapPath: params
                        .addresses
                        .swap_path
                        .iter()
                        .map(|address| AlloyAddress::from(*address.as_array()))
                        .collect(),
                },
                numbers: CreateOrderParamsNumbers {
                    sizeDeltaUsd: alloy_u(params.numbers.size_delta_usd),
                    initialCollateralDeltaAmount: alloy_u(
                        params.numbers.initial_collateral_delta_amount,
                    ),
                    triggerPrice: alloy_u(params.numbers.trigger_price),
                    acceptablePrice: alloy_u(params.numbers.acceptable_price),
                    executionFee: alloy_u(params.numbers.execution_fee),
                    callbackGasLimit: alloy_u(params.numbers.callback_gas_limit),
                    minOutputAmount: alloy_u(params.numbers.min_output_amount),
                    validFromTime: alloy_u(params.numbers.valid_from_time),
                },
                orderType: order_type,
                decreasePositionSwapType: params.decrease_position_swap_type as u8,
                isLong: params.is_long,
                shouldUnwrapNativeToken: params.should_unwrap_native_token,
                autoCancel: params.auto_cancel,
                referralCode: FixedBytes::ZERO,
                dataList: Vec::new(),
            },
        }
        .abi_encode()
    }

    proptest! {
        #[test]
        fn create_increase_order_matches_alloy(
            addresses in any::<[[u8; 20]; 6]>(),
            swap_path in vec(any::<[u8; 20]>(), 0..8),
            numbers in any::<[U; 8]>(),
            swap_type in decrease_position_swap_type(),
            flags in any::<[bool; 3]>(),
            order_type in increase_order_type(),
        ) {
            let params = order_params(addresses, swap_path, numbers, swap_type, flags);
            let expected = alloy_create_order(&params, order_type as u8);
            let actual = make_fn_create_increase_order_vec(params, order_type);
            prop_assert_eq!(actual, expected);
        }

        #[test]
        fn create_decrease_order_matches_alloy(
            addresses in any::<[[u8; 20]; 6]>(),
            swap_path in vec(any::<[u8; 20]>(), 0..8),
            numbers in any::<[U; 8]>(),
            swap_type in decrease_position_swap_type(),
            flags in any::<[bool; 3]>(),
            order_type in decrease_order_type(),
        ) {
            let params = order_params(addresses, swap_path, numbers, swap_type, flags);
            let expected = alloy_create_order(&params, order_type as u8);
            let actual = make_fn_create_decrease_order_vec(params, order_type);
            prop_assert_eq!(actual, expected);
        }

        #[test]
        fn claim_funding_fees_matches_alloy(
            markets in vec(any::<[u8; 20]>(), 0..8),
            tokens in vec(any::<[u8; 20]>(), 0..8),
            receiver in any::<[u8; 20]>(),
        ) {
            let expected = claimFundingFeesCall {
                markets: markets.iter().copied().map(AlloyAddress::from).collect(),
                tokens: tokens.iter().copied().map(AlloyAddress::from).collect(),
                receiver: AlloyAddress::from(receiver),
            }
            .abi_encode();
            let actual = make_fn_claim_funding_fees_vec(
                markets.into_iter().map(EvmCdAddress::new).collect(),
                tokens.into_iter().map(EvmCdAddress::new).collect(),
                receiver,
            );
            prop_assert_eq!(actual, expected);
        }

        #[test]
        fn claim_collateral_matches_alloy(
            markets in vec(any::<[u8; 20]>(), 0..8),
            tokens in vec(any::<[u8; 20]>(), 0..8),
            time_keys in vec(any::<U>(), 0..8),
            receiver in any::<[u8; 20]>(),
        ) {
            let expected = claimCollateralCall {
                markets: markets.iter().copied().map(AlloyAddress::from).collect(),
                tokens: tokens.iter().copied().map(AlloyAddress::from).collect(),
                timeKeys: time_keys.iter().copied().map(alloy_u).collect(),
                receiver: AlloyAddress::from(receiver),
            }
            .abi_encode();
            let actual = make_fn_claim_collateral_vec(
                markets.into_iter().map(EvmCdAddress::new).collect(),
                tokens.into_iter().map(EvmCdAddress::new).collect(),
                time_keys,
                receiver,
            );
            prop_assert_eq!(actual, expected);
        }
    }
}

#[cfg(test)]
mod array_tests {
    extern crate std;

    use super::*;
    use alloy_primitives::{Address as AlloyAddress, FixedBytes, U256 as AlloyU256};
    use alloy_sol_macro::sol;
    use alloy_sol_types::SolCall;
    use proptest::prelude::*;
    use std::vec::Vec;

    sol! {
        struct SliceCreateOrderParamsAddresses {
            address receiver;
            address cancellationReceiver;
            address callbackContract;
            address uiFeeReceiver;
            address market;
            address initialCollateralToken;
            address[] swapPath;
        }

        struct SliceCreateOrderParamsNumbers {
            uint256 sizeDeltaUsd;
            uint256 initialCollateralDeltaAmount;
            uint256 triggerPrice;
            uint256 acceptablePrice;
            uint256 executionFee;
            uint256 callbackGasLimit;
            uint256 minOutputAmount;
            uint256 validFromTime;
        }

        struct SliceCreateOrderParams {
            SliceCreateOrderParamsAddresses addresses;
            SliceCreateOrderParamsNumbers numbers;
            uint8 orderType;
            uint8 decreasePositionSwapType;
            bool isLong;
            bool shouldUnwrapNativeToken;
            bool autoCancel;
            bytes32 referralCode;
            bytes32[] dataList;
        }

        function createOrder(SliceCreateOrderParams calldata params)
            external
            payable
            returns (bytes32 key);

        function claimFundingFees(
            address[] calldata markets,
            address[] calldata tokens,
            address receiver
        ) external returns (uint256[] memory claimedAmounts);

        function claimCollateral(
            address[] calldata markets,
            address[] calldata tokens,
            uint256[] calldata timeKeys,
            address receiver
        ) external returns (uint256[] memory claimedAmounts);
    }

    fn alloy_u(value: U) -> AlloyU256 {
        AlloyU256::from_be_bytes(*value)
    }

    proptest! {
        #[test]
        fn create_increase_order_array_matches_alloy(
            addresses in any::<[[u8; 20]; 6]>(),
            swap_path in any::<[[u8; 20]; 3]>(),
            numbers in any::<[U; 8]>(),
            is_long in any::<bool>(),
            should_unwrap_native_token in any::<bool>(),
            auto_cancel in any::<bool>(),
        ) {
            const SWAP_PATH_LEN: usize = 3;
            const ALL: usize = create_order_calldata_len(SWAP_PATH_LEN);
            let bobcat_swap_path = swap_path.map(EvmCdAddress::new);
            let params = OrderParams {
                addresses: OrderAddresses {
                    receiver: addresses[0],
                    cancellation_receiver: addresses[1],
                    callback_contract: addresses[2],
                    ui_fee_receiver: addresses[3],
                    market: addresses[4],
                    initial_collateral_token: addresses[5],
                    swap_path: bobcat_swap_path,
                },
                numbers: OrderNumbers {
                    size_delta_usd: numbers[0],
                    initial_collateral_delta_amount: numbers[1],
                    trigger_price: numbers[2],
                    acceptable_price: numbers[3],
                    execution_fee: numbers[4],
                    callback_gas_limit: numbers[5],
                    min_output_amount: numbers[6],
                    valid_from_time: numbers[7],
                },
                decrease_position_swap_type: DecreasePositionSwapType::SwapPnlTokenToCollateralToken,
                is_long,
                should_unwrap_native_token,
                auto_cancel,
            };
            let expected = createOrderCall {
                params: SliceCreateOrderParams {
                    addresses: SliceCreateOrderParamsAddresses {
                        receiver: AlloyAddress::from(addresses[0]),
                        cancellationReceiver: AlloyAddress::from(addresses[1]),
                        callbackContract: AlloyAddress::from(addresses[2]),
                        uiFeeReceiver: AlloyAddress::from(addresses[3]),
                        market: AlloyAddress::from(addresses[4]),
                        initialCollateralToken: AlloyAddress::from(addresses[5]),
                        swapPath: swap_path.into_iter().map(AlloyAddress::from).collect(),
                    },
                    numbers: SliceCreateOrderParamsNumbers {
                        sizeDeltaUsd: alloy_u(numbers[0]),
                        initialCollateralDeltaAmount: alloy_u(numbers[1]),
                        triggerPrice: alloy_u(numbers[2]),
                        acceptablePrice: alloy_u(numbers[3]),
                        executionFee: alloy_u(numbers[4]),
                        callbackGasLimit: alloy_u(numbers[5]),
                        minOutputAmount: alloy_u(numbers[6]),
                        validFromTime: alloy_u(numbers[7]),
                    },
                    orderType: IncreaseOrderType::Market as u8,
                    decreasePositionSwapType: DecreasePositionSwapType::SwapPnlTokenToCollateralToken as u8,
                    isLong: is_long,
                    shouldUnwrapNativeToken: should_unwrap_native_token,
                    autoCancel: auto_cancel,
                    referralCode: FixedBytes::ZERO,
                    dataList: Vec::new(),
                },
            }
            .abi_encode();
            let actual = make_fn_create_increase_order_array::<SWAP_PATH_LEN, ALL>(
                params,
                IncreaseOrderType::Market,
            );
            let expected: [u8; ALL] = expected.try_into().expect("Alloy encoded the expected length");
            prop_assert_eq!(actual, expected);
        }

        #[test]
        fn create_decrease_order_array_matches_alloy(
            addresses in any::<[[u8; 20]; 6]>(),
            swap_path in any::<[[u8; 20]; 3]>(),
            numbers in any::<[U; 8]>(),
            is_long in any::<bool>(),
            should_unwrap_native_token in any::<bool>(),
            auto_cancel in any::<bool>(),
        ) {
            const SWAP_PATH_LEN: usize = 3;
            const ALL: usize = create_order_calldata_len(SWAP_PATH_LEN);
            let bobcat_swap_path = swap_path.map(EvmCdAddress::new);
            let params = OrderParams {
                addresses: OrderAddresses {
                    receiver: addresses[0],
                    cancellation_receiver: addresses[1],
                    callback_contract: addresses[2],
                    ui_fee_receiver: addresses[3],
                    market: addresses[4],
                    initial_collateral_token: addresses[5],
                    swap_path: bobcat_swap_path,
                },
                numbers: OrderNumbers {
                    size_delta_usd: numbers[0],
                    initial_collateral_delta_amount: numbers[1],
                    trigger_price: numbers[2],
                    acceptable_price: numbers[3],
                    execution_fee: numbers[4],
                    callback_gas_limit: numbers[5],
                    min_output_amount: numbers[6],
                    valid_from_time: numbers[7],
                },
                decrease_position_swap_type: DecreasePositionSwapType::SwapPnlTokenToCollateralToken,
                is_long,
                should_unwrap_native_token,
                auto_cancel,
            };
            let expected = createOrderCall {
                params: SliceCreateOrderParams {
                    addresses: SliceCreateOrderParamsAddresses {
                        receiver: AlloyAddress::from(addresses[0]),
                        cancellationReceiver: AlloyAddress::from(addresses[1]),
                        callbackContract: AlloyAddress::from(addresses[2]),
                        uiFeeReceiver: AlloyAddress::from(addresses[3]),
                        market: AlloyAddress::from(addresses[4]),
                        initialCollateralToken: AlloyAddress::from(addresses[5]),
                        swapPath: swap_path.into_iter().map(AlloyAddress::from).collect(),
                    },
                    numbers: SliceCreateOrderParamsNumbers {
                        sizeDeltaUsd: alloy_u(numbers[0]),
                        initialCollateralDeltaAmount: alloy_u(numbers[1]),
                        triggerPrice: alloy_u(numbers[2]),
                        acceptablePrice: alloy_u(numbers[3]),
                        executionFee: alloy_u(numbers[4]),
                        callbackGasLimit: alloy_u(numbers[5]),
                        minOutputAmount: alloy_u(numbers[6]),
                        validFromTime: alloy_u(numbers[7]),
                    },
                    orderType: DecreaseOrderType::StopLoss as u8,
                    decreasePositionSwapType: DecreasePositionSwapType::SwapPnlTokenToCollateralToken as u8,
                    isLong: is_long,
                    shouldUnwrapNativeToken: should_unwrap_native_token,
                    autoCancel: auto_cancel,
                    referralCode: FixedBytes::ZERO,
                    dataList: Vec::new(),
                },
            }
            .abi_encode();
            let actual = make_fn_create_decrease_order_array::<SWAP_PATH_LEN, ALL>(
                params,
                DecreaseOrderType::StopLoss,
            );
            let expected: [u8; ALL] = expected.try_into().expect("Alloy encoded the expected length");
            prop_assert_eq!(actual, expected);
        }

        #[test]
        fn claim_funding_fees_array_matches_alloy(
            markets in any::<[[u8; 20]; 2]>(),
            tokens in any::<[[u8; 20]; 3]>(),
            receiver in any::<[u8; 20]>(),
        ) {
            const ALL: usize = claim_funding_fees_calldata_len(2, 3);
            let bobcat_markets = markets.map(EvmCdAddress::new);
            let bobcat_tokens = tokens.map(EvmCdAddress::new);
            let expected = claimFundingFeesCall {
                markets: markets.into_iter().map(AlloyAddress::from).collect(),
                tokens: tokens.into_iter().map(AlloyAddress::from).collect(),
                receiver: AlloyAddress::from(receiver),
            }
            .abi_encode();
            let actual = make_fn_claim_funding_fees_array::<2, 3, ALL>(
                bobcat_markets,
                bobcat_tokens,
                receiver,
            );
            let expected: [u8; ALL] = expected.try_into().expect("Alloy encoded the expected length");
            prop_assert_eq!(actual, expected);
        }

        #[test]
        fn claim_collateral_array_matches_alloy(
            markets in any::<[[u8; 20]; 2]>(),
            tokens in any::<[[u8; 20]; 3]>(),
            time_keys in any::<[U; 4]>(),
            receiver in any::<[u8; 20]>(),
        ) {
            const ALL: usize = claim_collateral_calldata_len(2, 3, 4);
            let bobcat_markets = markets.map(EvmCdAddress::new);
            let bobcat_tokens = tokens.map(EvmCdAddress::new);
            let expected = claimCollateralCall {
                markets: markets.into_iter().map(AlloyAddress::from).collect(),
                tokens: tokens.into_iter().map(AlloyAddress::from).collect(),
                timeKeys: time_keys.iter().copied().map(alloy_u).collect(),
                receiver: AlloyAddress::from(receiver),
            }
            .abi_encode();
            let actual = make_fn_claim_collateral_array::<2, 3, 4, ALL>(
                bobcat_markets,
                bobcat_tokens,
                time_keys,
                receiver,
            );
            let expected: [u8; ALL] = expected.try_into().expect("Alloy encoded the expected length");
            prop_assert_eq!(actual, expected);
        }
    }
}

#[cfg(test)]
mod fixed_size_tests {
    use super::make_fn_cancel_order;
    use alloy_primitives::FixedBytes;
    use alloy_sol_macro::sol;
    use alloy_sol_types::SolCall;
    use proptest::prelude::*;

    sol! {
        function cancelOrder(bytes32 key) external payable;
    }

    proptest! {
        #[test]
        fn cancel_order_matches_alloy(key in any::<[u8; 32]>()) {
            let expected = cancelOrderCall {
                key: FixedBytes::from(key),
            }
            .abi_encode();
            let actual = make_fn_cancel_order(key);
            let expected: [u8; 36] = expected.try_into().expect("Alloy encoded the expected length");
            prop_assert_eq!(actual, expected);
        }
    }
}
