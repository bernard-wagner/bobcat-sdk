
# Examples

This is a tiny collection of bobcat-sdk examples, including some tests to spot codesize
regressions between releases. Examples that mirror the reference versions must remain
functionally identical. The bobcat-sdk code works without an allocator.

For a more complete illustration of what the SDK offers, check the end-to-end tests. Build
each example from its own project directory.

Note that stylus-sdk accidentally bundles the std with its built code.

## End-to-end behavior tests

The repository's `./tests.sh` builds the actual Counter and Camelot example sources
and tests them from `e2e-test`. Counter tests cover public reads, arithmetic,
payable increments and storage changes. Camelot tests use deterministic ERC20 and
router fixtures to check calldata, token transfers, approvals, swap output and
failure rollback without depending on a live liquidity pool. The allocation-free
Camelot example aborts on failed calls; it does not forward callee revert data.

The script also runs Bozo's existing `004-bozo/test/Bozo.t.sol` suite with call
isolation both disabled and enabled. These tests exercise deposits, fees, tickets,
deadlines, epoch transitions, token callbacks and upgrades through a Bobcat-created
EIP-1967 proxy. Event and state-diff assertions accompany checks of the game's
public results and token balances, including rollback of a conditional deposit.

Run the suites using `arbos-forge` with Stylus event and storage inspection support.
The build uses the shared `e2e-test/wasm-post.sh` conversion step; generated WASM
artifacts do not need to be committed.

## Codesize comparison, with wasm-opt turned on with Stylus.toml (in bytes)

|    Name   | stylus-sdk (0.10.9) | bobcat-sdk |                          Description                           |
|-----------|--------------------|-------------|----------------------------------------------------------------|
| Counter   | 18015              | 6975        | A simple counter app that does basic manipulation of storage.  |
| Muldiv    | 21623              | 7201        | A muldiv implementation, compared to the version in 9lives.    |
| Camelot   | 40940              | 4626        | Acts as an intermediary for Camelot swapping using its router. |
| Bozo      | N/A                | 26320       | The contract code powering a dumb app for bidding on things.   |


|    Name   | stylus-sdk (0.9.0) | bobcat-sdk |                          Description                           |
|-----------|--------------------|------------|----------------------------------------------------------------|
| Supercats | 89413              | 22265      | NFT contract code for Superposition Supercats on Arbitrum One. |

The Supercats example reference is a copy and paste from the OpenZeppelin NFT wizard. The
code was compiled with `cargo stylus build`. The OZ SDK for this only supports Stylus
0.9.0 (that I'm aware of.)
