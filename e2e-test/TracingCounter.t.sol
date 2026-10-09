// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import {Test} from "forge-std/Test.sol";

import {IArbFoundry} from "./IArbFoundry.sol";

interface ITracingCounter {
    function hello() external returns (uint256);
}

contract SetTransient {
    function set(uint256 x) external {
        uint256 id = uint256(keccak256(abi.encodePacked("bobcat.tracing.counter"))) - 1;
        assembly {
            tstore(id, x)
        }
    }
}

contract TracingCounter is Test {
    ITracingCounter tracingCounter;

    function setUp() public {
        tracingCounter = ITracingCounter(IArbFoundry(address(vm)).deployStylusCode(
            "e2e-test/tracing-counter.wasm"
        ));
    }

    /// forge-config: default.isolate = false
    function testFuzz_counter(uint256 x) public {
        vm.assume(uint256(type(uint32).max) > x);
        SetTransient t = new SetTransient();
        t.set(x);
        vm.etch(address(t), address(tracingCounter).code);
        assertEq(x + 1, ITracingCounter(address(t)).hello());
    }
}
