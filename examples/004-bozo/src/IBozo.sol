// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

interface IBozo {
    function initialise(address admin, address asset) external;

    function play(uint256 amount, address recipient, bytes32 comment, uint256 desiredEpoch)
        external
        returns (uint256 epoch, uint256 deposited);

    function distributeRewards(uint256 epoch, address rewardRecipient, uint256 rngWord)
        external
        returns (uint256 winnerReward);

    function poolSize() external view returns (uint256);
    function poolAsset() external view returns (address);
    function lastBettorAmount() external view returns (uint256);
    function minDeposit() external view returns (uint256);
    function lastBettorAddress() external view returns (uint256);
    function deadline() external view returns (uint256);
    function epochDeadline(uint256) external view returns (uint256);
    function playerCount() external view returns (uint256);
    function ticketCount() external view returns (uint256);
    function currentEpoch() external view returns (uint256);
    function wasEpochCollected(uint256 epoch) external view returns (bool);

    function upgrade(address newImpl) external;

    function changeAdmin(address newAdmin) external;

    function ownerCollectFees() external returns (uint256);

    function daoCollectFees() external returns (uint256);

    event CommentPosted(address indexed poster, bytes32 indexed hash);

    event DepositMade(address indexed recipient, uint256 indexed amount, uint256 indexed currentPool);

    event WinnerChosen(address indexed recipient, uint256 indexed amount, bool indexed isLottery);

    event NewEpoch(uint256 indexed last);
}
