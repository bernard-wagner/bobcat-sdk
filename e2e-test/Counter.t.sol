// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import {Test} from "forge-std/Test.sol";
import {Vm} from "forge-std/Vm.sol";

import {IArbFoundry} from "./IArbFoundry.sol";

interface ICounter {
    function number() external view returns (uint256);
    function setNumber(uint256 value) external;
    function mulNumber(uint256 value) external;
    function addNumber(uint256 value) external;
    function increment() external;
    function addFromMsgValue() external payable;
}

contract CounterTest is Test {
    ICounter counter;

    function setUp() public virtual {
        counter = ICounter(IArbFoundry(address(vm)).deployStylusCode("e2e-test/counter.wasm"));
    }

    function testSetAndGetNumber() public {
        counter.setNumber(10);
        assertEq(counter.number(), 10);
    }

    function testAddNumber() public {
        counter.setNumber(5);
        counter.addNumber(3);
        assertEq(counter.number(), 8);
    }

    function testMulNumber() public {
        counter.setNumber(4);
        counter.mulNumber(3);
        assertEq(counter.number(), 12);
    }

    function testIncrement() public {
        counter.setNumber(7);
        counter.increment();
        assertEq(counter.number(), 8);
    }

    function testAddFromMsgValue() public {
        counter.setNumber(1);
        counter.addFromMsgValue{value: 9 ether}();
        assertEq(counter.number(), 9 ether + 1);
    }
}

// Run the same public behavior checks against the shipped example, not a copy.
contract CounterExampleTest is CounterTest {
    function setUp() public override {
        counter = ICounter(IArbFoundry(address(vm)).deployStylusCode("e2e-test/example-counter.wasm"));
    }

    function testReadDoesNotWrite() public {
        counter.setNumber(42);
        vm.record();
        assertEq(counter.number(), 42);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(counter));
        assertEq(reads.length, 1);
        assertEq(reads[0], bytes32(0));
        assertEq(writes.length, 0);
    }

    function testWrappingAdditionChangesOnlyCounterSlot() public {
        counter.setNumber(type(uint256).max);
        vm.startStateDiffRecording();
        counter.addNumber(2);
        Vm.AccountAccess[] memory diff = vm.stopAndReturnStateDiff();
        assertEq(counter.number(), 1);
        uint256 writes;
        for (uint256 i; i < diff.length; ++i) {
            for (uint256 j; j < diff[i].storageAccesses.length; ++j) {
                Vm.StorageAccess memory access = diff[i].storageAccesses[j];
                if (access.account != address(counter) || !access.isWrite) continue;
                ++writes;
                assertEq(access.slot, bytes32(0));
                assertEq(access.previousValue, bytes32(type(uint256).max));
                assertEq(access.newValue, bytes32(uint256(1)));
                assertFalse(access.reverted);
            }
        }
        assertEq(writes, 1);
    }
}
