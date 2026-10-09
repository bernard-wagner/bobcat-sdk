// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import {Test} from "forge-std/Test.sol";
import {MockERC721} from "../forge-libs/solady/test/utils/mocks/MockERC721.sol";

import {IArbFoundry} from "./IArbFoundry.sol";

interface IERC721Supercats {
    function mint(address owner, uint256 id) external;
    function name() external view returns (string memory);
    function symbol() external view returns (string memory);
    function tokenURI(uint256 id) external view returns (string memory);
    function balanceOf(address owner) external view returns (uint256);
    function ownerOf(uint256 id) external view returns (address);
    function getApproved(uint256 id) external view returns (address);
    function isApprovedForAll(address owner, address operator) external view returns (bool);
    function approve(address account, uint256 id) external;
    function setApprovalForAll(address operator, bool approved) external;
    function transferFrom(address from, address to, uint256 id) external;
    function safeTransferFrom(address from, address to, uint256 id) external;
    function safeTransferFrom(address from, address to, uint256 id, bytes calldata data) external;
    function supportsInterface(bytes4 interfaceId) external view returns (bool);
}

contract ERC721Recipient {
    address public operator;
    address public from;
    uint256 public id;
    bytes public data;

    function onERC721Received(address operator_, address from_, uint256 id_, bytes calldata data_)
        external
        returns (bytes4)
    {
        operator = operator_;
        from = from_;
        id = id_;
        data = data_;
        return this.onERC721Received.selector;
    }
}

contract WrongReturnDataERC721Recipient {
    function onERC721Received(address, address, uint256, bytes calldata) external pure returns (bytes4) {
        return 0xcafebeef;
    }
}

contract NonERC721Recipient {}

contract RevertingERC721Recipient {
    error ReceiverReverted(uint256 value);

    function onERC721Received(address, address, uint256, bytes calldata) external pure returns (bytes4) {
        revert ReceiverReverted(42);
    }
}

contract ShortReturnERC721Recipient {
    function onERC721Received(address, address, uint256, bytes calldata) external pure returns (bytes4) {
        assembly {
            mstore(0, shl(224, 0x150b7a02))
            return(0, 4)
        }
    }
}

contract LongReturnERC721Recipient {
    function onERC721Received(address, address, uint256, bytes calldata) external pure returns (bytes4) {
        assembly {
            mstore(0, shl(224, 0x150b7a02))
            mstore(32, 0x1234)
            return(0, 64)
        }
    }
}

contract LargeReturnERC721Recipient {
    function onERC721Received(address, address, uint256, bytes calldata) external pure returns (bytes4) {
        assembly {
            mstore(0, shl(224, 0x150b7a02))
            return(0, 0x4000)
        }
    }
}

contract ReentrantERC721Recipient {
    IERC721Supercats internal immutable token;
    address internal immutable finalRecipient;

    constructor(IERC721Supercats token_, address finalRecipient_) {
        token = token_;
        finalRecipient = finalRecipient_;
    }

    function onERC721Received(address, address, uint256 id, bytes calldata) external returns (bytes4) {
        require(msg.sender == address(token), "wrong token");
        require(token.ownerOf(id) == address(this), "stale owner");
        token.transferFrom(address(this), finalRecipient, id);
        return this.onERC721Received.selector;
    }
}

contract SupercatsTest is Test {
    IERC721Supercats internal token;
    MockERC721 internal referenceToken;

    address internal constant OWNER = address(0xA11CE);
    address internal constant TO = address(0xB0B);
    address internal constant SPENDER = address(0xBEEF);
    address internal constant OPERATOR = address(0xCAFE);

    function setUp() public {
        token = IERC721Supercats(IArbFoundry(address(vm)).deployStylusCode("e2e-test/supercats.wasm"));
        referenceToken = new MockERC721();
    }

    function _mintFixture(uint256 id, address owner) internal {
        token.mint(owner, id);
        referenceToken.mint(owner, id);
    }

    function _assertStateEquivalent(uint256 id, address owner0, address owner1) internal view {
        assertEq(token.ownerOf(id), referenceToken.ownerOf(id));
        assertEq(token.balanceOf(owner0), referenceToken.balanceOf(owner0));
        assertEq(token.balanceOf(owner1), referenceToken.balanceOf(owner1));
        assertEq(token.getApproved(id), referenceToken.getApproved(id));
    }

    function _assertCallReverts(address caller, bytes memory callData) internal {
        vm.prank(caller);
        (bool tokenOk,) = address(token).call(callData);
        assertFalse(tokenOk);
    }

    function testMetadata() public view {
        assertEq(token.name(), "Superposition Supercats");
        assertEq(token.symbol(), "SPN CATS");
    }

    function testSupportsInterface(bytes4 interfaceId) public view {
        assertEq(token.supportsInterface(interfaceId), referenceToken.supportsInterface(interfaceId));
    }

    function testSupportedInterfaces() public view {
        assertTrue(token.supportsInterface(0x01ffc9a7));
        assertTrue(token.supportsInterface(0x80ac58cd));
        assertTrue(token.supportsInterface(0x5b5e139f));
        assertFalse(token.supportsInterface(0xffffffff));
    }

    function testSupportsInterfaceRejectsDirtyPadding() public view {
        bytes memory callData = abi.encodePacked(
            IERC721Supercats.supportsInterface.selector, bytes4(0x01ffc9a7), bytes28(type(uint224).max)
        );
        (bool tokenOk,) = address(token).staticcall(callData);
        assertFalse(tokenOk);
    }

    function testTokenURI(uint256 id) public {
        _mintFixture(id, OWNER);
        string memory expected = string.concat("https://cats-cdn.superposition.so/", vm.toString(id));
        assertEq(token.tokenURI(id), expected);

        (bool ok, bytes memory returnData) = address(token).staticcall(abi.encodeCall(IERC721Supercats.tokenURI, (id)));
        assertTrue(ok);
        assertEq(returnData, abi.encode(expected));
    }

    function testOwnerAndBalanceEquivalence(uint256 id) public {
        _mintFixture(id, OWNER);
        _assertStateEquivalent(id, OWNER, TO);
    }

    function testApprove(uint256 id) public {
        _mintFixture(id, OWNER);

        vm.prank(OWNER);
        token.approve(SPENDER, id);

        vm.prank(OWNER);
        referenceToken.approve(SPENDER, id);
        assertEq(token.getApproved(id), referenceToken.getApproved(id));
    }

    function testApproveByOperator(uint256 id) public {
        _mintFixture(id, OWNER);

        vm.startPrank(OWNER);
        token.setApprovalForAll(OPERATOR, true);
        referenceToken.setApprovalForAll(OPERATOR, true);
        vm.stopPrank();

        vm.prank(OPERATOR);
        token.approve(SPENDER, id);
        vm.prank(OPERATOR);
        referenceToken.approve(SPENDER, id);

        assertEq(token.getApproved(id), referenceToken.getApproved(id));
    }

    function testApproveOwner(uint256 id) public {
        _mintFixture(id, OWNER);

        vm.prank(OWNER);
        token.approve(OWNER, id);
        vm.prank(OWNER);
        referenceToken.approve(OWNER, id);

        assertEq(token.getApproved(id), referenceToken.getApproved(id));
    }

    function testApproveAll() public {
        vm.prank(OWNER);
        token.setApprovalForAll(OPERATOR, true);

        vm.prank(OWNER);
        referenceToken.setApprovalForAll(OPERATOR, true);
        assertEq(token.isApprovedForAll(OWNER, OPERATOR), referenceToken.isApprovedForAll(OWNER, OPERATOR));

        vm.prank(OWNER);
        token.setApprovalForAll(OPERATOR, false);
        vm.prank(OWNER);
        referenceToken.setApprovalForAll(OPERATOR, false);
        assertEq(token.isApprovedForAll(OWNER, OPERATOR), referenceToken.isApprovedForAll(OWNER, OPERATOR));
    }

    function testApproveAllSelf() public {
        vm.prank(OWNER);
        token.setApprovalForAll(OWNER, true);
        vm.prank(OWNER);
        referenceToken.setApprovalForAll(OWNER, true);

        assertEq(token.isApprovedForAll(OWNER, OWNER), referenceToken.isApprovedForAll(OWNER, OWNER));
    }

    function testTransferFromSelf(uint256 id) public {
        _mintFixture(id, OWNER);

        vm.prank(OWNER);
        token.transferFrom(OWNER, TO, id);
        vm.prank(OWNER);
        referenceToken.transferFrom(OWNER, TO, id);

        _assertStateEquivalent(id, OWNER, TO);
    }

    function testTransferFromApproved(uint256 id) public {
        _mintFixture(id, OWNER);

        vm.prank(OWNER);
        token.approve(SPENDER, id);
        vm.prank(OWNER);
        referenceToken.approve(SPENDER, id);

        vm.prank(SPENDER);
        token.transferFrom(OWNER, TO, id);
        vm.prank(SPENDER);
        referenceToken.transferFrom(OWNER, TO, id);

        _assertStateEquivalent(id, OWNER, TO);
    }

    function testTransferFromApproveAll(uint256 id) public {
        _mintFixture(id, OWNER);

        vm.prank(OWNER);
        token.setApprovalForAll(OPERATOR, true);
        vm.prank(OWNER);
        referenceToken.setApprovalForAll(OPERATOR, true);

        vm.prank(OPERATOR);
        token.transferFrom(OWNER, TO, id);
        vm.prank(OPERATOR);
        referenceToken.transferFrom(OWNER, TO, id);

        _assertStateEquivalent(id, OWNER, TO);
    }

    function testSafeTransferFromToEOA(uint256 id) public {
        _mintFixture(id, OWNER);

        vm.prank(OWNER);
        token.safeTransferFrom(OWNER, TO, id);
        vm.prank(OWNER);
        referenceToken.safeTransferFrom(OWNER, TO, id);

        _assertStateEquivalent(id, OWNER, TO);
    }

    function testSafeTransferFromToERC721Recipient(uint256 id) public {
        _mintFixture(id, OWNER);
        ERC721Recipient tokenRecipient = new ERC721Recipient();
        ERC721Recipient referenceTokenRecipient = new ERC721Recipient();

        vm.prank(OWNER);
        token.safeTransferFrom(OWNER, address(tokenRecipient), id);
        vm.prank(OWNER);
        referenceToken.safeTransferFrom(OWNER, address(referenceTokenRecipient), id);

        assertEq(tokenRecipient.operator(), referenceTokenRecipient.operator());
        assertEq(tokenRecipient.from(), referenceTokenRecipient.from());
        assertEq(tokenRecipient.id(), referenceTokenRecipient.id());
        assertEq(tokenRecipient.data(), referenceTokenRecipient.data());
        assertEq(token.ownerOf(id), address(tokenRecipient));
        assertEq(referenceToken.ownerOf(id), address(referenceTokenRecipient));
        assertEq(token.balanceOf(OWNER), referenceToken.balanceOf(OWNER));
        assertEq(token.balanceOf(address(tokenRecipient)), 1);
        assertEq(referenceToken.balanceOf(address(referenceTokenRecipient)), 1);
    }

    function testSafeTransferFromToERC721RecipientWithData(uint256 id, bytes memory data) public {
        vm.assume(data.length <= 1000);
        _mintFixture(id, OWNER);
        ERC721Recipient tokenRecipient = new ERC721Recipient();
        ERC721Recipient referenceTokenRecipient = new ERC721Recipient();

        vm.prank(OWNER);
        token.safeTransferFrom(OWNER, address(tokenRecipient), id, data);
        vm.prank(OWNER);
        referenceToken.safeTransferFrom(OWNER, address(referenceTokenRecipient), id, data);

        assertEq(tokenRecipient.operator(), referenceTokenRecipient.operator());
        assertEq(tokenRecipient.from(), referenceTokenRecipient.from());
        assertEq(tokenRecipient.id(), referenceTokenRecipient.id());
        assertEq(tokenRecipient.data(), referenceTokenRecipient.data());
        assertEq(token.balanceOf(OWNER), referenceToken.balanceOf(OWNER));
    }

    function testApproveNonExistentReverts(uint256 id) public {
        _assertCallReverts(OWNER, abi.encodeCall(IERC721Supercats.approve, (SPENDER, id)));
    }

    function testApproveUnauthorizedReverts(uint256 id) public {
        _mintFixture(id, OWNER);
        _assertCallReverts(SPENDER, abi.encodeCall(IERC721Supercats.approve, (TO, id)));
    }

    function testTransferFromNotExistentReverts(uint256 id) public {
        _assertCallReverts(OWNER, abi.encodeCall(IERC721Supercats.transferFrom, (OWNER, TO, id)));
    }

    function testTransferFromWrongFromReverts(uint256 id) public {
        _mintFixture(id, OWNER);
        _assertCallReverts(OWNER, abi.encodeCall(IERC721Supercats.transferFrom, (TO, SPENDER, id)));
    }

    function testTransferFromToZeroReverts(uint256 id) public {
        _mintFixture(id, OWNER);
        _assertCallReverts(OWNER, abi.encodeCall(IERC721Supercats.transferFrom, (OWNER, address(0), id)));
    }

    function testTransferFromNotOwner(uint256 id) public {
        _mintFixture(id, OWNER);
        _assertCallReverts(SPENDER, abi.encodeCall(IERC721Supercats.transferFrom, (OWNER, TO, id)));
    }

    function testSafeTransferFromToNonERC721RecipientReverts(uint256 id) public {
        _mintFixture(id, OWNER);
        address recipient = address(new NonERC721Recipient());
        _assertCallReverts(
            OWNER, abi.encodeWithSignature("safeTransferFrom(address,address,uint256)", OWNER, recipient, id)
        );
        assertEq(token.ownerOf(id), OWNER);
        assertEq(referenceToken.ownerOf(id), OWNER);
    }

    function testSafeTransferFromToWrongReturnDataReverts(uint256 id) public {
        _mintFixture(id, OWNER);
        address recipient = address(new WrongReturnDataERC721Recipient());
        _assertCallReverts(
            OWNER, abi.encodeWithSignature("safeTransferFrom(address,address,uint256)", OWNER, recipient, id)
        );
        assertEq(token.ownerOf(id), OWNER);
        assertEq(referenceToken.ownerOf(id), OWNER);
    }

    function testSafeTransferRejectsShortReturn(uint256 id) public {
        _mintFixture(id, OWNER);
        address recipient = address(new ShortReturnERC721Recipient());
        _assertCallReverts(
            OWNER, abi.encodeWithSignature("safeTransferFrom(address,address,uint256)", OWNER, recipient, id)
        );
        assertEq(token.ownerOf(id), OWNER);
        assertEq(referenceToken.ownerOf(id), OWNER);
    }

    function testSafeTransferExposesStateAndAllowsReentrancy(uint256 id) public {
        _mintFixture(id, OWNER);
        ReentrantERC721Recipient tokenRecipient = new ReentrantERC721Recipient(token, TO);
        ReentrantERC721Recipient referenceRecipient =
            new ReentrantERC721Recipient(IERC721Supercats(address(referenceToken)), TO);

        vm.prank(OWNER);
        token.safeTransferFrom(OWNER, address(tokenRecipient), id);
        vm.prank(OWNER);
        referenceToken.safeTransferFrom(OWNER, address(referenceRecipient), id);

        _assertStateEquivalent(id, OWNER, TO);
        assertEq(token.balanceOf(address(tokenRecipient)), referenceToken.balanceOf(address(referenceRecipient)));
    }

    function testOwnerOfNonExistent(uint256 id) public {
        (bool tokenOk,) = address(token).staticcall(abi.encodeCall(IERC721Supercats.ownerOf, (id)));
        assertFalse(tokenOk);
    }

    function testGetApprovedNonExistent(uint256 id) public {
        (bool tokenOk,) = address(token).staticcall(abi.encodeCall(IERC721Supercats.getApproved, (id)));
        assertFalse(tokenOk);
    }

    function testTokenURINonExistent(uint256 id) public {
        (bool tokenOk,) = address(token).staticcall(abi.encodeCall(IERC721Supercats.tokenURI, (id)));
        assertFalse(tokenOk);
    }

    function testBalanceOfZeroAddressReverts() public {
        (bool tokenOk,) = address(token).staticcall(abi.encodeCall(IERC721Supercats.balanceOf, (address(0))));
        assertFalse(tokenOk);
    }
}
