// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import {Test} from "forge-std/Test.sol";
import {Vm, VmSafe} from "forge-std/Vm.sol";
import {IArbFoundry} from "./IArbFoundry.sol";

interface ICamelotExample {
    function makeSwap(address tokenIn, address tokenOut, uint256 amountIn, uint256 minimumOut)
        external
        returns (uint256);
}

contract SwapToken {
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    function mint(address to, uint256 value) external {
        balanceOf[to] += value;
    }

    function approve(address to, uint256 value) external returns (bool) {
        allowance[msg.sender][to] = value;
        emit Approval(msg.sender, to, value);
        return true;
    }

    function transfer(address to, uint256 value) external returns (bool) {
        move(msg.sender, to, value);
        return true;
    }

    function transferFrom(address from, address to, uint256 value) external returns (bool) {
        require(allowance[from][msg.sender] >= value, "insufficient allowance");
        allowance[from][msg.sender] -= value;
        move(from, to, value);
        return true;
    }

    function move(address from, address to, uint256 value) internal {
        require(balanceOf[from] >= value, "insufficient balance");
        balanceOf[from] -= value;
        balanceOf[to] += value;
        emit Transfer(from, to, value);
    }
}

// A deterministic ABI/token-flow fixture, not a model of AMM pricing.
contract QuoteRouter {
    struct ExactInputSingleParams {
        address tokenIn;
        address tokenOut;
        address recipient;
        uint256 deadline;
        uint256 amountIn;
        uint256 amountOutMinimum;
        uint160 limitSqrtPrice;
    }
    error Slippage(uint256 output, uint256 minimum);

    function exactInputSingle(ExactInputSingleParams calldata params) external returns (uint256 output) {
        require(params.deadline == block.timestamp + 1, "wrong deadline");
        require(params.limitSqrtPrice == type(uint160).max, "wrong price limit");
        output = params.amountIn * 2;
        if (output < params.amountOutMinimum) revert Slippage(output, params.amountOutMinimum);
        require(SwapToken(params.tokenIn).transferFrom(msg.sender, address(this), params.amountIn));
        require(SwapToken(params.tokenOut).transfer(params.recipient, output));
    }
}

contract CamelotExampleTest is Test {
    address constant ROUTER = 0x6221A9c005F6e47EB398fD867784CacfDcFFF4E7;
    ICamelotExample swapper;
    SwapToken tokenIn;
    SwapToken tokenOut;

    function setUp() public {
        swapper = ICamelotExample(IArbFoundry(address(vm)).deployStylusCode("e2e-test/example-camelot.wasm"));
        vm.etch(ROUTER, address(new QuoteRouter()).code);
        tokenIn = new SwapToken();
        tokenOut = new SwapToken();
        tokenIn.mint(address(this), 1000);
        tokenOut.mint(ROUTER, 2000);
        tokenIn.approve(address(swapper), 1000);
    }

    function testSwapUsesTokenBindingsAndReturnsRouterOutput() public {
        vm.expectCall(address(tokenIn), abi.encodeCall(tokenIn.transferFrom, (address(this), address(swapper), 1000)));
        vm.expectCall(address(tokenIn), abi.encodeCall(tokenIn.approve, (ROUTER, 1000)));
        vm.recordLogs();
        assertEq(swapper.makeSwap(address(tokenIn), address(tokenOut), 1000, 1900), 2000);
        assertEq(tokenIn.balanceOf(address(this)), 0);
        assertEq(tokenIn.balanceOf(address(swapper)), 0);
        assertEq(tokenIn.balanceOf(ROUTER), 1000);
        assertEq(tokenIn.allowance(address(swapper), ROUTER), 0);
        assertEq(tokenOut.balanceOf(address(this)), 2000);
        assertEq(tokenOut.balanceOf(address(swapper)), 0);
        assertEq(tokenOut.balanceOf(ROUTER), 0);
        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs.length, 4);
        assertEq(logs[0].emitter, address(tokenIn));
        assertEq(logs[0].topics[0], keccak256("Transfer(address,address,uint256)"));
        assertEq(logs[1].topics[0], keccak256("Approval(address,address,uint256)"));
        assertEq(logs[1].topics[1], bytes32(uint256(uint160(address(swapper)))));
        assertEq(logs[1].topics[2], bytes32(uint256(uint160(ROUTER))));
        assertEq(logs[1].data, abi.encode(uint256(1000)));
        assertEq(logs[3].emitter, address(tokenOut));
        assertEq(logs[3].topics[2], bytes32(uint256(uint160(address(this)))));
        assertEq(logs[3].data, abi.encode(uint256(2000)));
    }

    function testSlippageAbortsAndRollsBackTransferAndApproval() public {
        vm.expectCall(ROUTER, abi.encodeCall(QuoteRouter.exactInputSingle, (QuoteRouter.ExactInputSingleParams({
            tokenIn: address(tokenIn),
            tokenOut: address(tokenOut),
            recipient: address(this),
            deadline: block.timestamp + 1,
            amountIn: 1000,
            amountOutMinimum: 2001,
            limitSqrtPrice: type(uint160).max
        }))));
        vm.startStateDiffRecording();
        (bool ok, bytes memory reason) =
            address(swapper).call(abi.encodeCall(swapper.makeSwap, (address(tokenIn), address(tokenOut), 1000, 2001)));
        Vm.AccountAccess[] memory diff = vm.stopAndReturnStateDiff();
        assertFalse(ok);
        // The allocation-free example aborts rather than copying router revert data.
        assertEq(reason.length, 0);
        assertEq(tokenIn.balanceOf(address(this)), 1000);
        assertEq(tokenIn.balanceOf(address(swapper)), 0);
        assertEq(tokenIn.balanceOf(ROUTER), 0);
        assertEq(tokenIn.allowance(address(this), address(swapper)), 1000);
        assertEq(tokenIn.allowance(address(swapper), ROUTER), 0);
        assertEq(tokenOut.balanceOf(ROUTER), 2000);
        uint256 writes;
        for (uint256 i; i < diff.length; ++i) {
            for (uint256 j; j < diff[i].storageAccesses.length; ++j) {
                Vm.StorageAccess memory access = diff[i].storageAccesses[j];
                if (access.account != address(tokenIn) || !access.isWrite) continue;
                ++writes;
                assertTrue(access.reverted);
            }
        }
        assertGt(writes, 0, "the input transfer must have executed before the router rejected it");
    }

    function testTokenFailureAbortsBeforeRouterCall() public {
        tokenIn.approve(address(swapper), 0);
        vm.expectCall(address(tokenIn), abi.encodeCall(tokenIn.transferFrom, (address(this), address(swapper), 1000)));
        vm.startStateDiffRecording();
        (bool ok, bytes memory reason) =
            address(swapper).call(abi.encodeCall(swapper.makeSwap, (address(tokenIn), address(tokenOut), 1000, 0)));
        Vm.AccountAccess[] memory diff = vm.stopAndReturnStateDiff();
        assertFalse(ok);
        assertEq(reason.length, 0);
        for (uint256 i; i < diff.length; ++i) {
            assertFalse(diff[i].kind == VmSafe.AccountAccessKind.Call && diff[i].account == ROUTER,
                "router must not be called after a failed input transfer");
        }
        assertEq(tokenIn.balanceOf(address(this)), 1000);
        assertEq(tokenIn.balanceOf(address(swapper)), 0);
        assertEq(tokenOut.balanceOf(ROUTER), 2000);
    }
}
