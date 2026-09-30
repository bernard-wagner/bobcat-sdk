
# Bozo

Bozo is an end to end application example of bobcat-sdk. It's deployed on Farcaster and on
Arbitrum One. Users deposit amounts until a timeout has been reached, and the last
depositor receives the entire pool. It's similar to Yeet on Berachain. Bozo was made using
`bobcat-new`. Alex (Bayge) wrote the contracts, and Ivan (IvanSN_) made the frontend
using Figma and it's code generator. The frontend was almost entirely modified by Codex,
after using Rainbow and Wagmi as the wallet-interacting package.

It was scaffolded using `bobcat-new` at commit `2cd66f9a519f375fbfb31e52625399abcd160497`, at
1761199872.

## Claiming rewards

User reward distribution is performed by anyone, using a `distributeRewards()` function. A
tiny amount of the pool is set aside for this distribution. If a single user made a
deposit or the amount deposited cumulatively is less than the amount needed to pay for a
Chainlink VRF call, then the contract will simply reimburse every user.

## Pseudocode

You can read a reference implementation at `src/reference.py`.

## Deployment

The deployment implementation is hosted at `0x8a2de40f965ec4ece00a0e9ff3556b98c842b310`.
The proxy is hosted at `0xd2c5cdeceaa85e9e44edcc63b207febaab8dcaf4`. Both of these contracts
are on Arbitrum One. The token in use is USND
(`0x4ecf61a6c2fab8a047ceb3b3b263b401763e9d49`), an algorithmic stablecoin by Nerite.

The permit relayer is deployed at `0x503a8DD317791aF592B1C84B19eD20a6134ea02d`.

These deployments are on the Arbitrum One mainnet network.

## DISCLAIMER

This was made for fun and to show off bobcat-sdk! Do not participate if you can afford to
lose.
