// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import {Test} from "forge-std/Test.sol";

import {IArbFoundry} from "./IArbFoundry.sol";

interface IVrfCallback {
    function initiate() external payable returns (uint256);
    function wasCalled() external view returns (bool);
    function rawFulfillRandomWords(uint256 requestId, uint256[] calldata words) external;
}

// A strict ABI/value fixture, not a replacement for the coordinator's pricing or randomness.
contract MockVrfCoordinator {
    address public requester;
    uint256 public paid;

    function requestRandomWordsInNative(uint32 gasLimit, uint16 confirmations, uint32 words, bytes calldata extraArgs)
        external payable returns (uint256)
    {
        require(gasLimit == 100_000 && confirmations == 2 && words == 5, "bad request parameters");
        require(keccak256(extraArgs) == keccak256(abi.encodeWithSelector(bytes4(keccak256("VRF ExtraArgsV1")), true)), "native payment required");
        require(msg.value == 0.25 ether, "bad forwarded value");
        require(requester == address(0), "duplicate request");
        requester = msg.sender;
        paid = msg.value;
        return 100;
    }

    function fulfill() external {
        require(requester != address(0), "no pending request");
        uint256[] memory words = new uint256[](5);
        for (uint256 i; i < words.length; ++i) words[i] = i + 1;
        IVrfCallback(requester).rawFulfillRandomWords(100, words);
    }
}

contract ChainlinkVrfTest is Test {
    IVrfCallback vrfCallback;
    MockVrfCoordinator constant COORDINATOR = MockVrfCoordinator(0x29576aB8152A09b9DC634804e4aDE73dA1f3a3CC);

    function setUp() public {
        vm.etch(address(COORDINATOR), address(new MockVrfCoordinator()).code);
        vrfCallback = IVrfCallback(IArbFoundry(address(vm)).deployStylusCode(
            "e2e-test/chainlink-vrf-test.wasm"
        ));
    }

    function testCallback() public view {
        assert(!vrfCallback.wasCalled());
    }

    function testInitiate() public {
        vm.deal(address(this), 1 ether);
        assertEq(100, vrfCallback.initiate{value: 0.25 ether}());
        assertEq(COORDINATOR.requester(), address(vrfCallback));
        assertEq(COORDINATOR.paid(), 0.25 ether);
        assertEq(address(COORDINATOR).balance, 0.25 ether);
        assertFalse(vrfCallback.wasCalled());
        COORDINATOR.fulfill();
        assertTrue(vrfCallback.wasCalled());
    }

    function testRejectedRequestBubblesUp() public {
        vm.expectRevert(bytes("bad forwarded value"));
        vrfCallback.initiate();
        assertEq(COORDINATOR.requester(), address(0));
        assertEq(COORDINATOR.paid(), 0);
        assertFalse(vrfCallback.wasCalled());
    }
}
