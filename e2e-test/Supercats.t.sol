// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import {Test} from "forge-std/Test.sol";
import {MockERC721} from "solady/test/utils/mocks/MockERC721.sol";

import {IArbFoundry} from "./IArbFoundry.sol";

interface IERC721Supercats {
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
    function onERC721Received(address, address, uint256, bytes calldata)
        external
        pure
        returns (bytes4)
    {
        return 0xcafebeef;
    }
}

contract NonERC721Recipient {}

contract SupercatsTest is Test {
    IERC721Supercats internal token;
    MockERC721 internal reference;

    address internal constant OWNER = address(0xA11CE);
    address internal constant TO = address(0xB0B);
    address internal constant SPENDER = address(0xBEEF);
    address internal constant OPERATOR = address(0xCAFE);

    event Transfer(address indexed from, address indexed to, uint256 indexed id);
    event Approval(address indexed owner, address indexed approved, uint256 indexed id);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

    function setUp() public {
        token = IERC721Supercats(
            IArbFoundry(address(vm)).deployStylusCode("e2e-test/supercats.wasm")
        );
        reference = new MockERC721();
    }

    // Supercats has no public mint function. Seed the same logical minted state in its
    // documented Solidity mapping layout, then mint the reference Solady token normally.
    function _mintFixture(uint256 id, address owner) internal {
        bytes32 balanceSlot = keccak256(abi.encode(owner, uint256(0)));
        bytes32 ownerSlot = keccak256(abi.encode(id, uint256(1)));
        vm.store(address(token), balanceSlot, bytes32(uint256(1)));
        vm.store(address(token), ownerSlot, bytes32(uint256(uint160(owner))));
        reference.mint(owner, id);
    }

    function _assertStateEquivalent(uint256 id, address owner0, address owner1) internal view {
        assertEq(token.ownerOf(id), reference.ownerOf(id));
        assertEq(token.balanceOf(owner0), reference.balanceOf(owner0));
        assertEq(token.balanceOf(owner1), reference.balanceOf(owner1));
        assertEq(token.getApproved(id), reference.getApproved(id));
    }

    function _assertCallSuccessEquivalent(address caller, bytes memory callData) internal {
        vm.prank(caller);
        (bool tokenOk,) = address(token).call(callData);
        vm.prank(caller);
        (bool referenceOk,) = address(reference).call(callData);
        assertEq(tokenOk, referenceOk);
    }

    function testMetadata() public view {
        assertEq(token.name(), "Superposition Supercats");
        assertEq(token.symbol(), "SPN CATS");
    }

    function testTokenURI(uint256 id) public {
        _mintFixture(id, OWNER);
        assertEq(token.tokenURI(id), string.concat("https://cats-cdn.superposition.so/", vm.toString(id)));
    }

    function testOwnerAndBalanceEquivalence(uint256 id) public {
        _mintFixture(id, OWNER);
        _assertStateEquivalent(id, OWNER, TO);
    }

    function testApprove(uint256 id) public {
        _mintFixture(id, OWNER);

        vm.expectEmit(true, true, true, true, address(token));
        emit Approval(OWNER, SPENDER, id);
        vm.prank(OWNER);
        token.approve(SPENDER, id);

        vm.prank(OWNER);
        reference.approve(SPENDER, id);
        assertEq(token.getApproved(id), reference.getApproved(id));
    }

    function testApproveByOperator(uint256 id) public {
        _mintFixture(id, OWNER);

        vm.startPrank(OWNER);
        token.setApprovalForAll(OPERATOR, true);
        reference.setApprovalForAll(OPERATOR, true);
        vm.stopPrank();

        vm.prank(OPERATOR);
        token.approve(SPENDER, id);
        vm.prank(OPERATOR);
        reference.approve(SPENDER, id);

        assertEq(token.getApproved(id), reference.getApproved(id));
    }

    function testApproveAll() public {
        vm.expectEmit(true, true, false, true, address(token));
        emit ApprovalForAll(OWNER, OPERATOR, true);
        vm.prank(OWNER);
        token.setApprovalForAll(OPERATOR, true);

        vm.prank(OWNER);
        reference.setApprovalForAll(OPERATOR, true);
        assertEq(
            token.isApprovedForAll(OWNER, OPERATOR),
            reference.isApprovedForAll(OWNER, OPERATOR)
        );

        vm.prank(OWNER);
        token.setApprovalForAll(OPERATOR, false);
        vm.prank(OWNER);
        reference.setApprovalForAll(OPERATOR, false);
        assertEq(
            token.isApprovedForAll(OWNER, OPERATOR),
            reference.isApprovedForAll(OWNER, OPERATOR)
        );
    }

    function testTransferFromSelf(uint256 id) public {
        _mintFixture(id, OWNER);

        vm.expectEmit(true, true, true, true, address(token));
        emit Transfer(OWNER, TO, id);
        vm.prank(OWNER);
        token.transferFrom(OWNER, TO, id);
        vm.prank(OWNER);
        reference.transferFrom(OWNER, TO, id);

        _assertStateEquivalent(id, OWNER, TO);
    }

    function testTransferFromApproved(uint256 id) public {
        _mintFixture(id, OWNER);

        vm.prank(OWNER);
        token.approve(SPENDER, id);
        vm.prank(OWNER);
        reference.approve(SPENDER, id);

        vm.prank(SPENDER);
        token.transferFrom(OWNER, TO, id);
        vm.prank(SPENDER);
        reference.transferFrom(OWNER, TO, id);

        _assertStateEquivalent(id, OWNER, TO);
    }

    function testTransferFromApproveAll(uint256 id) public {
        _mintFixture(id, OWNER);

        vm.prank(OWNER);
        token.setApprovalForAll(OPERATOR, true);
        vm.prank(OWNER);
        reference.setApprovalForAll(OPERATOR, true);

        vm.prank(OPERATOR);
        token.transferFrom(OWNER, TO, id);
        vm.prank(OPERATOR);
        reference.transferFrom(OWNER, TO, id);

        _assertStateEquivalent(id, OWNER, TO);
    }

    function testSafeTransferFromToEOA(uint256 id) public {
        _mintFixture(id, OWNER);

        vm.prank(OWNER);
        token.safeTransferFrom(OWNER, TO, id);
        vm.prank(OWNER);
        reference.safeTransferFrom(OWNER, TO, id);

        _assertStateEquivalent(id, OWNER, TO);
    }

    function testSafeTransferFromToERC721Recipient(uint256 id) public {
        _mintFixture(id, OWNER);
        ERC721Recipient tokenRecipient = new ERC721Recipient();
        ERC721Recipient referenceRecipient = new ERC721Recipient();

        vm.prank(OWNER);
        token.safeTransferFrom(OWNER, address(tokenRecipient), id);
        vm.prank(OWNER);
        reference.safeTransferFrom(OWNER, address(referenceRecipient), id);

        assertEq(tokenRecipient.operator(), referenceRecipient.operator());
        assertEq(tokenRecipient.from(), referenceRecipient.from());
        assertEq(tokenRecipient.id(), referenceRecipient.id());
        assertEq(tokenRecipient.data(), referenceRecipient.data());
        assertEq(token.ownerOf(id), address(tokenRecipient));
        assertEq(reference.ownerOf(id), address(referenceRecipient));
        assertEq(token.balanceOf(OWNER), reference.balanceOf(OWNER));
        assertEq(token.balanceOf(address(tokenRecipient)), 1);
        assertEq(reference.balanceOf(address(referenceRecipient)), 1);
    }

    function testSafeTransferFromToERC721RecipientWithData(uint256 id, bytes memory data) public {
        vm.assume(data.length <= 1000);
        _mintFixture(id, OWNER);
        ERC721Recipient tokenRecipient = new ERC721Recipient();
        ERC721Recipient referenceRecipient = new ERC721Recipient();

        vm.prank(OWNER);
        token.safeTransferFrom(OWNER, address(tokenRecipient), id, data);
        vm.prank(OWNER);
        reference.safeTransferFrom(OWNER, address(referenceRecipient), id, data);

        assertEq(tokenRecipient.operator(), referenceRecipient.operator());
        assertEq(tokenRecipient.from(), referenceRecipient.from());
        assertEq(tokenRecipient.id(), referenceRecipient.id());
        assertEq(tokenRecipient.data(), referenceRecipient.data());
        assertEq(token.balanceOf(OWNER), reference.balanceOf(OWNER));
    }

    function testApproveNonExistentReverts(uint256 id) public {
        _assertCallSuccessEquivalent(
            OWNER, abi.encodeCall(IERC721Supercats.approve, (SPENDER, id))
        );
    }

    function testApproveUnauthorizedReverts(uint256 id) public {
        _mintFixture(id, OWNER);
        _assertCallSuccessEquivalent(
            SPENDER, abi.encodeCall(IERC721Supercats.approve, (TO, id))
        );
    }

    function testTransferFromNotExistentReverts(uint256 id) public {
        _assertCallSuccessEquivalent(
            OWNER, abi.encodeCall(IERC721Supercats.transferFrom, (OWNER, TO, id))
        );
    }

    function testTransferFromWrongFromReverts(uint256 id) public {
        _mintFixture(id, OWNER);
        _assertCallSuccessEquivalent(
            OWNER, abi.encodeCall(IERC721Supercats.transferFrom, (TO, SPENDER, id))
        );
    }

    function testTransferFromToZeroReverts(uint256 id) public {
        _mintFixture(id, OWNER);
        _assertCallSuccessEquivalent(
            OWNER, abi.encodeCall(IERC721Supercats.transferFrom, (OWNER, address(0), id))
        );
    }

    function testTransferFromNotOwner(uint256 id) public {
        _mintFixture(id, OWNER);
        _assertCallSuccessEquivalent(
            SPENDER, abi.encodeCall(IERC721Supercats.transferFrom, (OWNER, TO, id))
        );
    }

    function testSafeTransferFromToNonERC721RecipientReverts(uint256 id) public {
        _mintFixture(id, OWNER);
        address recipient = address(new NonERC721Recipient());
        _assertCallSuccessEquivalent(
            OWNER,
            abi.encodeCall(IERC721Supercats.safeTransferFrom, (OWNER, recipient, id))
        );
        assertEq(token.ownerOf(id), OWNER);
        assertEq(reference.ownerOf(id), OWNER);
    }

    function testSafeTransferFromToWrongReturnDataReverts(uint256 id) public {
        _mintFixture(id, OWNER);
        address recipient = address(new WrongReturnDataERC721Recipient());
        _assertCallSuccessEquivalent(
            OWNER,
            abi.encodeCall(IERC721Supercats.safeTransferFrom, (OWNER, recipient, id))
        );
        assertEq(token.ownerOf(id), OWNER);
        assertEq(reference.ownerOf(id), OWNER);
    }

    function testOwnerOfNonExistent(uint256 id) public {
        (bool tokenOk,) = address(token).staticcall(
            abi.encodeCall(IERC721Supercats.ownerOf, (id))
        );
        (bool referenceOk,) = address(reference).staticcall(
            abi.encodeCall(IERC721Supercats.ownerOf, (id))
        );
        assertEq(tokenOk, referenceOk);
    }

    function testGetApprovedNonExistent(uint256 id) public {
        (bool tokenOk,) = address(token).staticcall(
            abi.encodeCall(IERC721Supercats.getApproved, (id))
        );
        (bool referenceOk,) = address(reference).staticcall(
            abi.encodeCall(IERC721Supercats.getApproved, (id))
        );
        assertEq(tokenOk, referenceOk);
    }

    function testBalanceOfZeroAddressReverts() public {
        (bool tokenOk,) = address(token).staticcall(
            abi.encodeCall(IERC721Supercats.balanceOf, (address(0)))
        );
        (bool referenceOk,) = address(reference).staticcall(
            abi.encodeCall(IERC721Supercats.balanceOf, (address(0)))
        );
        assertEq(tokenOk, referenceOk);
    }
}
