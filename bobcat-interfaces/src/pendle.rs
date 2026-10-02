//! Narrow Pendle V2 router calldata builders for core end-user flows.
//!
//! These builders cover PT/YT swaps and single-token liquidity using tokens
//! accepted directly by a market's SY. External swap aggregators and limit-order
//! fills are intentionally unsupported.

use bobcat_cd::{EvmCdAddress, EvmCdSerialise};
use bobcat_maths::U;

pub type Address = [u8; 20];

/// Pendle's on-chain approximation bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApproxParams {
    pub guess_min: U,
    pub guess_max: U,
    pub guess_offchain: U,
    pub max_iteration: U,
    pub eps: U,
}

/// Input accepted directly by an SY, without an external swap aggregator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SimpleTokenInput {
    pub token: Address,
    pub amount: U,
}

/// Output redeemed directly from an SY, without an external swap aggregator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SimpleTokenOutput {
    pub token: Address,
    pub min_amount: U,
}

pub const TOKEN_IN_CALLDATA_LEN: usize = 4 + 32 * 28;
pub const TOKEN_OUT_CALLDATA_LEN: usize = 4 + 32 * 23;

// The public API deliberately permits only createTokenInputSimple and an empty
// LimitOrderData. Their otherwise-dynamic ABI tails therefore have fixed contents
// and size. These private calls describe those canonical words as static fields so
// the derive can provide allocation-free exact-array encoders.
#[derive(EvmCdSerialise)]
#[evm_selector]
enum DerivedTokenInCall {
    #[evm_selector(
        "swapExactTokenForPt(address,address,uint256,(uint256,uint256,uint256,uint256,uint256),(address,uint256,address,address,(uint8,address,bytes,bool)),(address,uint256,((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],bytes))"
    )]
    SwapExactTokenForPt(
        EvmCdAddress,
        EvmCdAddress,
        U,
        U,
        U,
        U,
        U,
        U,
        U,
        U,
        EvmCdAddress,
        U,
        EvmCdAddress,
        EvmCdAddress,
        U,
        u8,
        EvmCdAddress,
        U,
        bool,
        U,
        EvmCdAddress,
        U,
        U,
        U,
        U,
        U,
        U,
        U,
    ),
    #[evm_selector(
        "swapExactTokenForYt(address,address,uint256,(uint256,uint256,uint256,uint256,uint256),(address,uint256,address,address,(uint8,address,bytes,bool)),(address,uint256,((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],bytes))"
    )]
    SwapExactTokenForYt(
        EvmCdAddress,
        EvmCdAddress,
        U,
        U,
        U,
        U,
        U,
        U,
        U,
        U,
        EvmCdAddress,
        U,
        EvmCdAddress,
        EvmCdAddress,
        U,
        u8,
        EvmCdAddress,
        U,
        bool,
        U,
        EvmCdAddress,
        U,
        U,
        U,
        U,
        U,
        U,
        U,
    ),
    #[evm_selector(
        "addLiquiditySingleToken(address,address,uint256,(uint256,uint256,uint256,uint256,uint256),(address,uint256,address,address,(uint8,address,bytes,bool)),(address,uint256,((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],bytes))"
    )]
    AddLiquiditySingleToken(
        EvmCdAddress,
        EvmCdAddress,
        U,
        U,
        U,
        U,
        U,
        U,
        U,
        U,
        EvmCdAddress,
        U,
        EvmCdAddress,
        EvmCdAddress,
        U,
        u8,
        EvmCdAddress,
        U,
        bool,
        U,
        EvmCdAddress,
        U,
        U,
        U,
        U,
        U,
        U,
        U,
    ),
}

#[derive(EvmCdSerialise)]
#[evm_selector]
enum DerivedTokenOutCall {
    #[evm_selector(
        "swapExactPtForToken(address,address,uint256,(address,uint256,address,address,(uint8,address,bytes,bool)),(address,uint256,((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],bytes))"
    )]
    SwapExactPtForToken(
        EvmCdAddress,
        EvmCdAddress,
        U,
        U,
        U,
        EvmCdAddress,
        U,
        EvmCdAddress,
        EvmCdAddress,
        U,
        u8,
        EvmCdAddress,
        U,
        bool,
        U,
        EvmCdAddress,
        U,
        U,
        U,
        U,
        U,
        U,
        U,
    ),
    #[evm_selector(
        "swapExactYtForToken(address,address,uint256,(address,uint256,address,address,(uint8,address,bytes,bool)),(address,uint256,((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],bytes))"
    )]
    SwapExactYtForToken(
        EvmCdAddress,
        EvmCdAddress,
        U,
        U,
        U,
        EvmCdAddress,
        U,
        EvmCdAddress,
        EvmCdAddress,
        U,
        u8,
        EvmCdAddress,
        U,
        bool,
        U,
        EvmCdAddress,
        U,
        U,
        U,
        U,
        U,
        U,
        U,
    ),
    #[evm_selector(
        "removeLiquiditySingleToken(address,address,uint256,(address,uint256,address,address,(uint8,address,bytes,bool)),(address,uint256,((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],bytes))"
    )]
    RemoveLiquiditySingleToken(
        EvmCdAddress,
        EvmCdAddress,
        U,
        U,
        U,
        EvmCdAddress,
        U,
        EvmCdAddress,
        EvmCdAddress,
        U,
        u8,
        EvmCdAddress,
        U,
        bool,
        U,
        EvmCdAddress,
        U,
        U,
        U,
        U,
        U,
        U,
        U,
    ),
}

#[derive(Clone, Copy)]
enum TokenInFunction {
    SwapExactTokenForPt,
    SwapExactTokenForYt,
    AddLiquiditySingleToken,
}

#[derive(Clone, Copy)]
enum TokenOutFunction {
    SwapExactPtForToken,
    SwapExactYtForToken,
    RemoveLiquiditySingleToken,
}

macro_rules! token_in_call {
    ($variant:ident, $receiver:expr, $market:expr, $minimum_out:expr, $approx:expr, $input:expr) => {
        DerivedTokenInCall::$variant(
            EvmCdAddress::new($receiver),
            EvmCdAddress::new($market),
            *$minimum_out,
            $approx.guess_min,
            $approx.guess_max,
            $approx.guess_offchain,
            $approx.max_iteration,
            $approx.eps,
            U::from_usize(32 * 10), // TokenInput offset.
            U::from_usize(32 * 20), // LimitOrderData offset.
            EvmCdAddress::new($input.token),
            $input.amount,
            EvmCdAddress::new($input.token),
            EvmCdAddress::new([0; 20]), // pendleSwap.
            U::from_usize(32 * 5),      // SwapData offset in TokenInput.
            0,                          // SwapType::NONE.
            EvmCdAddress::new([0; 20]), // extRouter.
            U::from_usize(32 * 4),      // extCalldata offset in SwapData.
            false,                      // needScale.
            U::ZERO,                    // extCalldata length.
            EvmCdAddress::new([0; 20]), // limitRouter.
            U::ZERO,                    // epsSkipMarket.
            U::from_usize(32 * 5),      // normalFills offset.
            U::from_usize(32 * 6),      // flashFills offset.
            U::from_usize(32 * 7),      // optData offset.
            U::ZERO,                    // normalFills length.
            U::ZERO,                    // flashFills length.
            U::ZERO,                    // optData length.
        )
    };
}

macro_rules! token_out_call {
    ($variant:ident, $receiver:expr, $market:expr, $exact_in:expr, $output:expr) => {
        DerivedTokenOutCall::$variant(
            EvmCdAddress::new($receiver),
            EvmCdAddress::new($market),
            *$exact_in,
            U::from_usize(32 * 5),  // TokenOutput offset.
            U::from_usize(32 * 15), // LimitOrderData offset.
            EvmCdAddress::new($output.token),
            $output.min_amount,
            EvmCdAddress::new($output.token),
            EvmCdAddress::new([0; 20]), // pendleSwap.
            U::from_usize(32 * 5),      // SwapData offset in TokenOutput.
            0,                          // SwapType::NONE.
            EvmCdAddress::new([0; 20]), // extRouter.
            U::from_usize(32 * 4),      // extCalldata offset in SwapData.
            false,                      // needScale.
            U::ZERO,                    // extCalldata length.
            EvmCdAddress::new([0; 20]), // limitRouter.
            U::ZERO,                    // epsSkipMarket.
            U::from_usize(32 * 5),      // normalFills offset.
            U::from_usize(32 * 6),      // flashFills offset.
            U::from_usize(32 * 7),      // optData offset.
            U::ZERO,                    // normalFills length.
            U::ZERO,                    // flashFills length.
            U::ZERO,                    // optData length.
        )
    };
}

fn make_token_in_call(
    function: TokenInFunction,
    receiver: Address,
    market: Address,
    minimum_out: &U,
    approx: &ApproxParams,
    input: &SimpleTokenInput,
) -> [u8; TOKEN_IN_CALLDATA_LEN] {
    let call = match function {
        TokenInFunction::SwapExactTokenForPt => token_in_call!(
            SwapExactTokenForPt,
            receiver,
            market,
            minimum_out,
            approx,
            input
        ),
        TokenInFunction::SwapExactTokenForYt => token_in_call!(
            SwapExactTokenForYt,
            receiver,
            market,
            minimum_out,
            approx,
            input
        ),
        TokenInFunction::AddLiquiditySingleToken => token_in_call!(
            AddLiquiditySingleToken,
            receiver,
            market,
            minimum_out,
            approx,
            input
        ),
    };
    call.to_evm_array()
        .expect("Pendle token-input calldata has a fixed 900-byte encoding")
}

fn make_token_out_call(
    function: TokenOutFunction,
    receiver: Address,
    market: Address,
    exact_in: &U,
    output: &SimpleTokenOutput,
) -> [u8; TOKEN_OUT_CALLDATA_LEN] {
    let call = match function {
        TokenOutFunction::SwapExactPtForToken => {
            token_out_call!(SwapExactPtForToken, receiver, market, exact_in, output)
        }
        TokenOutFunction::SwapExactYtForToken => {
            token_out_call!(SwapExactYtForToken, receiver, market, exact_in, output)
        }
        TokenOutFunction::RemoveLiquiditySingleToken => token_out_call!(
            RemoveLiquiditySingleToken,
            receiver,
            market,
            exact_in,
            output
        ),
    };
    call.to_evm_array()
        .expect("Pendle token-output calldata has a fixed 740-byte encoding")
}

/// Encode a swap from an SY-supported token to PT.
pub fn make_fn_swap_exact_token_for_pt(
    receiver: Address,
    market: Address,
    min_pt_out: &U,
    guess_pt_out: &ApproxParams,
    input: &SimpleTokenInput,
) -> [u8; TOKEN_IN_CALLDATA_LEN] {
    make_token_in_call(
        TokenInFunction::SwapExactTokenForPt,
        receiver,
        market,
        min_pt_out,
        guess_pt_out,
        input,
    )
}

/// Encode a swap from PT to an SY-supported token.
pub fn make_fn_swap_exact_pt_for_token(
    receiver: Address,
    market: Address,
    exact_pt_in: &U,
    output: &SimpleTokenOutput,
) -> [u8; TOKEN_OUT_CALLDATA_LEN] {
    make_token_out_call(
        TokenOutFunction::SwapExactPtForToken,
        receiver,
        market,
        exact_pt_in,
        output,
    )
}

/// Encode a swap from an SY-supported token to YT.
pub fn make_fn_swap_exact_token_for_yt(
    receiver: Address,
    market: Address,
    min_yt_out: &U,
    guess_yt_out: &ApproxParams,
    input: &SimpleTokenInput,
) -> [u8; TOKEN_IN_CALLDATA_LEN] {
    make_token_in_call(
        TokenInFunction::SwapExactTokenForYt,
        receiver,
        market,
        min_yt_out,
        guess_yt_out,
        input,
    )
}

/// Encode a swap from YT to an SY-supported token.
pub fn make_fn_swap_exact_yt_for_token(
    receiver: Address,
    market: Address,
    exact_yt_in: &U,
    output: &SimpleTokenOutput,
) -> [u8; TOKEN_OUT_CALLDATA_LEN] {
    make_token_out_call(
        TokenOutFunction::SwapExactYtForToken,
        receiver,
        market,
        exact_yt_in,
        output,
    )
}

/// Encode adding liquidity from one SY-supported token.
pub fn make_fn_add_liquidity_single_token(
    receiver: Address,
    market: Address,
    min_lp_out: &U,
    guess_pt_received_from_sy: &ApproxParams,
    input: &SimpleTokenInput,
) -> [u8; TOKEN_IN_CALLDATA_LEN] {
    make_token_in_call(
        TokenInFunction::AddLiquiditySingleToken,
        receiver,
        market,
        min_lp_out,
        guess_pt_received_from_sy,
        input,
    )
}

/// Encode removing liquidity into one SY-supported token.
pub fn make_fn_remove_liquidity_single_token(
    receiver: Address,
    market: Address,
    net_lp_to_remove: &U,
    output: &SimpleTokenOutput,
) -> [u8; TOKEN_OUT_CALLDATA_LEN] {
    make_token_out_call(
        TokenOutFunction::RemoveLiquiditySingleToken,
        receiver,
        market,
        net_lp_to_remove,
        output,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address as AlloyAddress, Bytes, U256 as AlloyU256};
    use alloy_sol_types::SolCall;
    use proptest::prelude::*;

    mod alloy_reference {
        use alloy_sol_macro::sol;

        sol! {
            struct ApproxParams {
                uint256 guessMin;
                uint256 guessMax;
                uint256 guessOffchain;
                uint256 maxIteration;
                uint256 eps;
            }

            struct SwapData {
                uint8 swapType;
                address extRouter;
                bytes extCalldata;
                bool needScale;
            }

            struct TokenInput {
                address tokenIn;
                uint256 netTokenIn;
                address tokenMintSy;
                address pendleSwap;
                SwapData swapData;
            }

            struct TokenOutput {
                address tokenOut;
                uint256 minTokenOut;
                address tokenRedeemSy;
                address pendleSwap;
                SwapData swapData;
            }

            struct Order {
                uint256 salt;
                uint256 expiry;
                uint256 nonce;
                uint8 orderType;
                address token;
                address YT;
                address maker;
                address receiver;
                uint256 makingAmount;
                uint256 lnImpliedRate;
                uint256 failSafeRate;
                bytes permit;
            }

            struct FillOrderParams {
                Order order;
                bytes signature;
                uint256 makingAmount;
            }

            struct LimitOrderData {
                address limitRouter;
                uint256 epsSkipMarket;
                FillOrderParams[] normalFills;
                FillOrderParams[] flashFills;
                bytes optData;
            }

            function swapExactTokenForPt(
                address receiver,
                address market,
                uint256 minPtOut,
                ApproxParams calldata guessPtOut,
                TokenInput calldata input,
                LimitOrderData calldata limit
            ) external;

            function swapExactPtForToken(
                address receiver,
                address market,
                uint256 exactPtIn,
                TokenOutput calldata output,
                LimitOrderData calldata limit
            ) external;

            function swapExactTokenForYt(
                address receiver,
                address market,
                uint256 minYtOut,
                ApproxParams calldata guessYtOut,
                TokenInput calldata input,
                LimitOrderData calldata limit
            ) external;

            function swapExactYtForToken(
                address receiver,
                address market,
                uint256 exactYtIn,
                TokenOutput calldata output,
                LimitOrderData calldata limit
            ) external;

            function addLiquiditySingleToken(
                address receiver,
                address market,
                uint256 minLpOut,
                ApproxParams calldata guessPtReceivedFromSy,
                TokenInput calldata input,
                LimitOrderData calldata limit
            ) external;

            function removeLiquiditySingleToken(
                address receiver,
                address market,
                uint256 netLpToRemove,
                TokenOutput calldata output,
                LimitOrderData calldata limit
            ) external;
        }
    }

    fn alloy_u(value: U) -> AlloyU256 {
        AlloyU256::from_be_bytes(*value)
    }

    fn alloy_approx(value: ApproxParams) -> alloy_reference::ApproxParams {
        alloy_reference::ApproxParams {
            guessMin: alloy_u(value.guess_min),
            guessMax: alloy_u(value.guess_max),
            guessOffchain: alloy_u(value.guess_offchain),
            maxIteration: alloy_u(value.max_iteration),
            eps: alloy_u(value.eps),
        }
    }

    fn alloy_swap_data() -> alloy_reference::SwapData {
        alloy_reference::SwapData {
            swapType: 0,
            extRouter: AlloyAddress::ZERO,
            extCalldata: Bytes::new(),
            needScale: false,
        }
    }

    fn alloy_token_input(value: SimpleTokenInput) -> alloy_reference::TokenInput {
        alloy_reference::TokenInput {
            tokenIn: AlloyAddress::from(value.token),
            netTokenIn: alloy_u(value.amount),
            tokenMintSy: AlloyAddress::from(value.token),
            pendleSwap: AlloyAddress::ZERO,
            swapData: alloy_swap_data(),
        }
    }

    fn alloy_token_output(value: SimpleTokenOutput) -> alloy_reference::TokenOutput {
        alloy_reference::TokenOutput {
            tokenOut: AlloyAddress::from(value.token),
            minTokenOut: alloy_u(value.min_amount),
            tokenRedeemSy: AlloyAddress::from(value.token),
            pendleSwap: AlloyAddress::ZERO,
            swapData: alloy_swap_data(),
        }
    }

    fn empty_alloy_limit_order_data() -> alloy_reference::LimitOrderData {
        alloy_reference::LimitOrderData {
            limitRouter: AlloyAddress::ZERO,
            epsSkipMarket: AlloyU256::ZERO,
            normalFills: Vec::new(),
            flashFills: Vec::new(),
            optData: Bytes::new(),
        }
    }

    fn fixture() -> (
        Address,
        Address,
        U,
        ApproxParams,
        SimpleTokenInput,
        SimpleTokenOutput,
    ) {
        (
            [0x11; 20],
            [0x22; 20],
            U::from_u8(7),
            ApproxParams {
                guess_min: U::from_u8(1),
                guess_max: U::from_u8(2),
                guess_offchain: U::from_u8(3),
                max_iteration: U::from_u8(4),
                eps: U::from_u8(5),
            },
            SimpleTokenInput {
                token: [0x33; 20],
                amount: U::from_u8(8),
            },
            SimpleTokenOutput {
                token: [0x44; 20],
                min_amount: U::from_u8(9),
            },
        )
    }

    fn word(encoded: &[u8], index: usize) -> &[u8] {
        &encoded[4 + index * 32..4 + (index + 1) * 32]
    }

    #[test]
    fn token_input_calls_use_fixed_derived_encoding() {
        let (receiver, market, amount, approx, input, _) = fixture();
        let pt = make_fn_swap_exact_token_for_pt(receiver, market, &amount, &approx, &input);
        let yt = make_fn_swap_exact_token_for_yt(receiver, market, &amount, &approx, &input);
        let lp = make_fn_add_liquidity_single_token(receiver, market, &amount, &approx, &input);

        assert_eq!(&pt[..4], &[0xc8, 0x1f, 0x84, 0x7a]);
        assert_eq!(&yt[..4], &[0xed, 0x48, 0x90, 0x7e]);
        assert_eq!(&lp[..4], &[0x12, 0x59, 0x9a, 0xc6]);
        assert_eq!(&pt[4..], &yt[4..]);
        assert_eq!(&pt[4..], &lp[4..]);
        assert_eq!(&word(&pt, 0)[12..], &receiver);
        assert_eq!(&word(&pt, 1)[12..], &market);
        assert_eq!(word(&pt, 2), &amount.0);
        assert_eq!(word(&pt, 8), &U::from_usize(32 * 10).0);
        assert_eq!(word(&pt, 9), &U::from_usize(32 * 20).0);
        assert_eq!(&word(&pt, 10)[12..], &input.token);
        assert_eq!(word(&pt, 11), &input.amount.0);
        assert_eq!(word(&pt, 14), &U::from_usize(32 * 5).0);
        assert_eq!(word(&pt, 17), &U::from_usize(32 * 4).0);
        assert_eq!(word(&pt, 22), &U::from_usize(32 * 5).0);
        assert_eq!(word(&pt, 23), &U::from_usize(32 * 6).0);
        assert_eq!(word(&pt, 24), &U::from_usize(32 * 7).0);
        assert!(word(&pt, 25).iter().all(|byte| *byte == 0));
        assert!(word(&pt, 26).iter().all(|byte| *byte == 0));
        assert!(word(&pt, 27).iter().all(|byte| *byte == 0));
    }

    #[test]
    fn token_output_calls_use_fixed_derived_encoding() {
        let (receiver, market, amount, _, _, output) = fixture();
        let pt = make_fn_swap_exact_pt_for_token(receiver, market, &amount, &output);
        let yt = make_fn_swap_exact_yt_for_token(receiver, market, &amount, &output);
        let lp = make_fn_remove_liquidity_single_token(receiver, market, &amount, &output);

        assert_eq!(&pt[..4], &[0x59, 0x4a, 0x88, 0xcc]);
        assert_eq!(&yt[..4], &[0x05, 0xeb, 0x53, 0x27]);
        assert_eq!(&lp[..4], &[0x60, 0xda, 0x08, 0x60]);
        assert_eq!(&pt[4..], &yt[4..]);
        assert_eq!(&pt[4..], &lp[4..]);
        assert_eq!(&word(&pt, 0)[12..], &receiver);
        assert_eq!(&word(&pt, 1)[12..], &market);
        assert_eq!(word(&pt, 2), &amount.0);
        assert_eq!(word(&pt, 3), &U::from_usize(32 * 5).0);
        assert_eq!(word(&pt, 4), &U::from_usize(32 * 15).0);
        assert_eq!(&word(&pt, 5)[12..], &output.token);
        assert_eq!(word(&pt, 6), &output.min_amount.0);
        assert_eq!(word(&pt, 9), &U::from_usize(32 * 5).0);
        assert_eq!(word(&pt, 12), &U::from_usize(32 * 4).0);
        assert_eq!(word(&pt, 17), &U::from_usize(32 * 5).0);
        assert_eq!(word(&pt, 18), &U::from_usize(32 * 6).0);
        assert_eq!(word(&pt, 19), &U::from_usize(32 * 7).0);
        assert!(word(&pt, 20).iter().all(|byte| *byte == 0));
        assert!(word(&pt, 21).iter().all(|byte| *byte == 0));
        assert!(word(&pt, 22).iter().all(|byte| *byte == 0));
    }

    proptest! {
        #[test]
        fn token_input_calls_match_alloy_sol_encoding(
            receiver in any::<Address>(),
            market in any::<Address>(),
            minimum_out in any::<U>(),
            guess_min in any::<U>(),
            guess_max in any::<U>(),
            guess_offchain in any::<U>(),
            max_iteration in any::<U>(),
            eps in any::<U>(),
            token in any::<Address>(),
            amount in any::<U>(),
        ) {
            let approx = ApproxParams {
                guess_min,
                guess_max,
                guess_offchain,
                max_iteration,
                eps,
            };
            let input = SimpleTokenInput { token, amount };
            let alloy_receiver = AlloyAddress::from(receiver);
            let alloy_market = AlloyAddress::from(market);
            let alloy_minimum_out = alloy_u(minimum_out);

            let expected_pt = alloy_reference::swapExactTokenForPtCall {
                receiver: alloy_receiver,
                market: alloy_market,
                minPtOut: alloy_minimum_out,
                guessPtOut: alloy_approx(approx),
                input: alloy_token_input(input),
                limit: empty_alloy_limit_order_data(),
            }
            .abi_encode();
            let expected_yt = alloy_reference::swapExactTokenForYtCall {
                receiver: alloy_receiver,
                market: alloy_market,
                minYtOut: alloy_minimum_out,
                guessYtOut: alloy_approx(approx),
                input: alloy_token_input(input),
                limit: empty_alloy_limit_order_data(),
            }
            .abi_encode();
            let expected_lp = alloy_reference::addLiquiditySingleTokenCall {
                receiver: alloy_receiver,
                market: alloy_market,
                minLpOut: alloy_minimum_out,
                guessPtReceivedFromSy: alloy_approx(approx),
                input: alloy_token_input(input),
                limit: empty_alloy_limit_order_data(),
            }
            .abi_encode();

            let actual_pt = make_fn_swap_exact_token_for_pt(
                receiver,
                market,
                &minimum_out,
                &approx,
                &input,
            );
            let actual_yt = make_fn_swap_exact_token_for_yt(
                receiver,
                market,
                &minimum_out,
                &approx,
                &input,
            );
            let actual_lp = make_fn_add_liquidity_single_token(
                receiver,
                market,
                &minimum_out,
                &approx,
                &input,
            );

            prop_assert_eq!(actual_pt.as_slice(), expected_pt.as_slice());
            prop_assert_eq!(actual_yt.as_slice(), expected_yt.as_slice());
            prop_assert_eq!(actual_lp.as_slice(), expected_lp.as_slice());
        }

        #[test]
        fn token_output_calls_match_alloy_sol_encoding(
            receiver in any::<Address>(),
            market in any::<Address>(),
            exact_in in any::<U>(),
            token in any::<Address>(),
            min_amount in any::<U>(),
        ) {
            let output = SimpleTokenOutput { token, min_amount };
            let alloy_receiver = AlloyAddress::from(receiver);
            let alloy_market = AlloyAddress::from(market);
            let alloy_exact_in = alloy_u(exact_in);

            let expected_pt = alloy_reference::swapExactPtForTokenCall {
                receiver: alloy_receiver,
                market: alloy_market,
                exactPtIn: alloy_exact_in,
                output: alloy_token_output(output),
                limit: empty_alloy_limit_order_data(),
            }
            .abi_encode();
            let expected_yt = alloy_reference::swapExactYtForTokenCall {
                receiver: alloy_receiver,
                market: alloy_market,
                exactYtIn: alloy_exact_in,
                output: alloy_token_output(output),
                limit: empty_alloy_limit_order_data(),
            }
            .abi_encode();
            let expected_lp = alloy_reference::removeLiquiditySingleTokenCall {
                receiver: alloy_receiver,
                market: alloy_market,
                netLpToRemove: alloy_exact_in,
                output: alloy_token_output(output),
                limit: empty_alloy_limit_order_data(),
            }
            .abi_encode();

            let actual_pt =
                make_fn_swap_exact_pt_for_token(receiver, market, &exact_in, &output);
            let actual_yt =
                make_fn_swap_exact_yt_for_token(receiver, market, &exact_in, &output);
            let actual_lp =
                make_fn_remove_liquidity_single_token(receiver, market, &exact_in, &output);

            prop_assert_eq!(actual_pt.as_slice(), expected_pt.as_slice());
            prop_assert_eq!(actual_yt.as_slice(), expected_yt.as_slice());
            prop_assert_eq!(actual_lp.as_slice(), expected_lp.as_slice());
        }
    }
}
