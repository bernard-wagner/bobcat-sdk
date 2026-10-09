// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

import {Test} from "forge-std/Test.sol";
import {Vm} from "forge-std/Vm.sol";

import {IArbFoundry} from "./IArbFoundry.sol";

import {IBozo} from "../src/IBozo.sol";

contract ERC20 {
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    event Transfer(address indexed sender, address indexed recipient, uint256 amount);

    constructor() {
        balanceOf[msg.sender] = type(uint256).max;
    }

    function transferFrom(address _from, address _to, uint256 _value) public virtual {
        if (allowance[_from][msg.sender] != type(uint256).max) {
            allowance[_from][msg.sender] -= _value;
        }
        _transfer(_from, _to, _value);
    }

    function _transfer(address _from, address _to, uint256 _value) internal {
        if (_value > balanceOf[_from]) {
            revert("transfer too much");
        }
        unchecked {
            balanceOf[_from] -= _value;
            balanceOf[_to] += _value;
        }
        emit Transfer(_from, _to, _value);
    }

    function transfer(address _to, uint256 _value) external {
        _transfer(msg.sender, _to, _value);
    }

    function approve(address _spender, uint256 _value) external {
        allowance[msg.sender][_spender] = _value;
    }
}

interface IBozoProxyFactory {
    function create(address implementation) external returns (address);
}

contract CallbackToken is ERC20 {
    bool public reentered;
    uint256 public attempts;
    string public rejection;

    function transferFrom(address from, address to, uint256 value) public override {
        ++attempts;
        try IBozo(msg.sender).play(1, address(this), bytes32(0), 0) {
            reentered = true;
        } catch Error(string memory reason) {
            rejection = reason;
        }
        super.transferFrom(from, to, value);
    }
}

contract ConditionalDeposit {
    function deposit(IBozo game, ERC20 token, uint256 amount, uint256 requiredPool) external {
        token.approve(address(game), amount);
        game.play(amount, msg.sender, bytes32(0), game.currentEpoch());
        require(game.poolSize() == requiredPool, "unexpected pool size");
    }
}

contract Bozo is Test {
    address alex = 0x6221A9c005F6e47EB398fD867784CacfDcFFF4E7;
    address erik = 0xdd50872400Fb1dA43FFfA87Be38b85AA79DFa0ae;

    event Transfer(address indexed sender, address indexed recipient, uint256 amount);

    event CommentPosted(address indexed poster, bytes32 indexed hash);
    event PointsCollected(address indexed player, uint256 indexed points);
    event DepositMade(address indexed recipient, uint256 indexed amount, uint256 indexed currentPool);
    event NewEpoch(uint256 indexed epoch);
    event Upgraded(address indexed implementation);

    function assertWrite(
        Vm.AccountAccess[] memory diff,
        address account,
        bytes32 slot,
        uint256 beforeValue,
        uint256 afterValue,
        bool reverted
    ) internal pure {
        uint256 count;
        for (uint256 i; i < diff.length; ++i) {
            for (uint256 j; j < diff[i].storageAccesses.length; ++j) {
                Vm.StorageAccess memory access = diff[i].storageAccesses[j];
                if (access.account != account || access.slot != slot || !access.isWrite) continue;
                ++count;
                assertEq(access.previousValue, bytes32(beforeValue));
                assertEq(access.newValue, bytes32(afterValue));
                assertEq(access.reverted, reverted);
            }
        }
        assertEq(count, 1, "expected one persisted update to the game field");
    }

    // Bozo's storage macro uses the Solidity mapping layout: key, then field index.
    function epochSlot(uint256 field, uint256 epoch) internal pure returns (bytes32) {
        return keccak256(abi.encode(epoch, field));
    }

    function testDepositAccountsForFeesTicketsDeadlineAndEvents() public {
        (IBozo game, ERC20 token) = createGame(1000, 2000);
        vm.warp(100);
        bytes32 comment = keccak256("hello bozo");
        vm.expectEmit(true, true, false, true, address(game));
        emit CommentPosted(alex, comment);
        vm.expectEmit(true, true, false, true, address(token));
        emit Transfer(alex, address(game), 1000);
        vm.expectEmit(true, true, false, true, address(game));
        emit PointsCollected(alex, 1805);
        vm.expectEmit(true, true, true, true, address(game));
        emit DepositMade(alex, 950, 950);
        vm.recordLogs();
        vm.startStateDiffRecording();
        vm.prank(alex);
        (uint256 epoch, uint256 deposited) = game.play(1000, alex, comment, 0);
        Vm.AccountAccess[] memory diff = vm.stopAndReturnStateDiff();
        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(epoch, 0);
        assertEq(deposited, 950);
        assertEq(game.poolAsset(), address(token));
        assertEq(game.poolSize(), 950);
        assertEq(game.lastBettorAddress(), uint256(uint160(alex)));
        assertEq(game.lastBettorAmount(), 950);
        assertEq(game.playerCount(), 1);
        assertEq(game.ticketCount(), 1805);
        assertEq(game.deadline(), 2500);
        assertEq(token.balanceOf(alex), 0);
        assertEq(token.balanceOf(address(game)), 1000);
        assertEq(logs.length, 4);
        assertEq(logs[0].emitter, address(game));
        assertEq(logs[0].topics[0], keccak256("CommentPosted(address,bytes32)"));
        assertEq(logs[1].emitter, address(token));
        assertEq(logs[1].data, abi.encode(uint256(1000)));
        assertEq(logs[2].topics[0], keccak256("PointsCollected(address,uint256)"));
        assertEq(logs[3].topics[0], keccak256("DepositMade(address,uint256,uint256)"));
        assertWrite(diff, address(game), bytes32(uint256(2)), 0, 30, false);
        assertWrite(diff, address(game), bytes32(uint256(3)), 0, 20, false);
        assertWrite(diff, address(game), epochSlot(7, 0), 0, 950, false);
        assertWrite(diff, address(game), epochSlot(4, 0), 0, 2500, false);
        assertEq(game.ownerCollectFees(), 30);
        assertEq(game.daoCollectFees(), 20);
        assertEq(token.balanceOf(alex), 50);
        assertEq(token.balanceOf(address(game)), game.poolSize());
        assertEq(game.ownerCollectFees(), 0);
        assertEq(game.daoCollectFees(), 0);
    }

    function testRepeatPlayerAccumulatesTicketsWithoutDuplicateRegistration() public {
        (IBozo game,) = createGame(3000, 0);
        vm.warp(100);
        vm.startPrank(alex);
        game.play(1000, alex, bytes32(0), 0);
        game.play(2000, alex, bytes32(0), 0);
        vm.stopPrank();
        assertEq(game.poolSize(), 2850);
        assertEq(game.lastBettorAmount(), 1900);
        assertEq(game.playerCount(), 1);
        assertEq(game.ticketCount(), 5415);
        vm.record();
        assertEq(game.poolSize(), 2850);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(game));
        assertEq(writes.length, 0);
        bool readPool;
        for (uint256 i; i < reads.length; ++i) {
            if (reads[i] == epochSlot(7, 0)) readPool = true;
        }
        assertTrue(readPool);
    }

    function testRejectedDepositRestoresTokenAndGameState() public {
        (IBozo game, ERC20 token) = createGame(1000, 1000);
        vm.warp(100);
        vm.prank(alex);
        game.play(1000, alex, bytes32(0), 0);
        vm.startStateDiffRecording();
        vm.prank(erik);
        (bool ok,) = address(game).call(abi.encodeCall(game.play, (1, erik, bytes32(0), 0)));
        Vm.AccountAccess[] memory diff = vm.stopAndReturnStateDiff();
        assertFalse(ok);
        assertEq(token.balanceOf(erik), 1000);
        assertEq(token.balanceOf(address(game)), 1000);
        assertEq(game.poolSize(), 950);
        assertEq(game.lastBettorAddress(), uint256(uint160(alex)));
        assertEq(game.playerCount(), 1);
        assertEq(game.ticketCount(), 1805);
        assertEq(game.deadline(), 2500);
        uint256 tokenWrites;
        for (uint256 i; i < diff.length; ++i) {
            for (uint256 j; j < diff[i].storageAccesses.length; ++j) {
                Vm.StorageAccess memory access = diff[i].storageAccesses[j];
                if (access.account != address(token) || !access.isWrite) continue;
                ++tokenWrites;
                assertTrue(access.reverted);
            }
        }
        assertGt(tokenWrites, 0, "token transfer must be rolled back after the game rejects it");
    }

    function testConditionalTransactionRollsBackFlushedGameWrites() public {
        (IBozo game, ERC20 token) = createGame(0, 0);
        ConditionalDeposit batch = new ConditionalDeposit();
        token.transfer(address(batch), 1000);
        vm.warp(100);
        vm.startStateDiffRecording();
        // Observe the actual reverted call; expectRevert changes its outcome
        // before the state-diff inspector records the reverted flags.
        (bool ok, bytes memory reason) = address(batch).call(abi.encodeCall(batch.deposit, (game, token, 1000, 999)));
        Vm.AccountAccess[] memory diff = vm.stopAndReturnStateDiff();
        assertFalse(ok);
        assertEq(reason, abi.encodeWithSignature("Error(string)", "unexpected pool size"));
        assertEq(game.poolSize(), 0);
        assertEq(game.playerCount(), 0);
        assertEq(token.balanceOf(address(batch)), 1000);
        assertEq(token.balanceOf(address(game)), 0);
        assertWrite(diff, address(game), epochSlot(7, 0), 0, 950, true);
        assertWrite(diff, address(game), bytes32(uint256(2)), 0, 30, true);
        // The failed transaction must not prevent a subsequent valid deposit.
        batch.deposit(game, token, 1000, 950);
        assertEq(game.poolSize(), 950);
    }

    function testExpiredGameStartsIndependentEpoch() public {
        (IBozo game, ERC20 token) = createGame(1000, 2000);
        vm.warp(100);
        vm.prank(alex);
        game.play(1000, alex, bytes32(0), 0);
        vm.warp(2501);
        vm.expectEmit(true, false, false, true, address(game));
        emit NewEpoch(1);
        vm.prank(erik);
        (uint256 epoch, uint256 deposited) = game.play(2000, erik, bytes32(0), 1);
        assertEq(epoch, 1);
        assertEq(deposited, 1900);
        assertEq(game.currentEpoch(), 1);
        assertEq(game.poolSize(), 1900);
        assertEq(game.epochDeadline(0), 2500);
        assertEq(game.deadline(), 4901);
        assertEq(game.playerCount(), 1);
        assertEq(token.balanceOf(address(game)), 3000);
        assertEq(vm.load(address(game), epochSlot(7, 0)), bytes32(uint256(950)));
    }

    function testTokenCallbackCannotReenterPlay() public {
        IBozo game = IBozo(IArbFoundry(address(vm)).deployStylusCode("bozo.wasm"));
        CallbackToken token = new CallbackToken();
        game.initialise(alex, address(token));
        token.approve(address(game), 3000);
        vm.warp(100);
        game.play(1000, address(this), bytes32(0), 0);
        assertFalse(token.reentered());
        assertTrue(vm.contains(token.rejection(), "reentrancy is not allowed"));
        assertEq(token.attempts(), 1);
        assertEq(game.poolSize(), 950);
        game.play(2000, address(this), bytes32(0), 0);
        assertFalse(token.reentered());
        assertTrue(vm.contains(token.rejection(), "reentrancy is not allowed"));
        assertEq(token.attempts(), 2);
        assertEq(game.poolSize(), 2850);
        assertEq(game.playerCount(), 1);
    }

    function testSdkProxyKeepsGameStateAndEventsAcrossUpgrade() public {
        IBozo impl = IBozo(IArbFoundry(address(vm)).deployStylusCode("bozo.wasm"));
        IBozoProxyFactory factory =
            IBozoProxyFactory(IArbFoundry(address(vm)).deployStylusCode("../../e2e-test/eip1967-proxy.wasm"));
        IBozo proxy = IBozo(factory.create(address(impl)));
        ERC20 token = new ERC20();
        proxy.initialise(address(this), address(token));
        token.approve(address(proxy), 3000);
        vm.warp(100);
        vm.record();
        vm.expectEmit(true, true, true, true, address(proxy));
        emit DepositMade(address(this), 950, 950);
        proxy.play(1000, address(this), bytes32(0), 0);
        (, bytes32[] memory proxyWrites) = vm.accesses(address(proxy));
        (, bytes32[] memory implWrites) = vm.accesses(address(impl));
        assertGt(proxyWrites.length, 0);
        assertEq(implWrites.length, 0);
        assertEq(proxy.poolSize(), 950);
        assertEq(impl.poolSize(), 0);
        IBozo replacement = IBozo(IArbFoundry(address(vm)).deployStylusCode("bozo.wasm"));
        vm.expectEmit(true, false, false, true, address(proxy));
        emit Upgraded(address(replacement));
        proxy.upgrade(address(replacement));
        proxy.play(2000, address(this), bytes32(0), 0);
        assertEq(proxy.poolSize(), 2850);
        assertEq(proxy.playerCount(), 1);
        assertEq(replacement.poolSize(), 0);
        bytes32 implSlot = bytes32(uint256(keccak256("eip1967.proxy.implementation")) - 1);
        assertEq(vm.load(address(proxy), implSlot), bytes32(uint256(uint160(address(replacement)))));
    }

    function createGame(uint256 alexBal, uint256 erikBal) internal returns (IBozo a, ERC20 token) {
        a = IBozo(IArbFoundry(address(vm)).deployStylusCode("bozo.wasm"));
        token = new ERC20();
        token.transfer(alex, alexBal);
        token.transfer(erik, erikBal);
        a.initialise(alex, address(token));
        vm.prank(alex);
        token.approve(address(a), type(uint256).max);
        vm.prank(erik);
        token.approve(address(a), type(uint256).max);
    }

    function test_fuzzGame(uint256 alexDeposit) external {
        vm.assume(alexDeposit > 100);
        vm.assume(1e50 > alexDeposit);
        uint256 erikDeposit = alexDeposit + ((alexDeposit * 3) / 10);
        uint256 pool = alexDeposit + erikDeposit;
        (IBozo a, ERC20 token) = createGame(alexDeposit, erikDeposit);
        vm.warp(405611000);
        vm.prank(alex);
        a.play(alexDeposit, alex, 0, 0);
        assertEq(0, token.balanceOf(alex));
        assertEq(0, a.currentEpoch());
        vm.warp(405611745);
        vm.prank(erik);
        a.play(erikDeposit, erik, 0, 0);
        assertEq(0, token.balanceOf(erik));
        assertEq(block.timestamp + 2400, a.deadline());
        vm.warp(405910536);
        vm.prank(alex);
        a.distributeRewards(0, alex, 123);
        vm.assumeNoRevert();
        uint256 exp = pool - ((pool * 5) / 100);
        vm.assertApproxEqAbsDecimal(exp, token.balanceOf(alex) + token.balanceOf(erik), 1000, 6);
    }
}
