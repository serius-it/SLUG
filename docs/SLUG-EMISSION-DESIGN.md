# SLUG Emission Design

## Fixed requirements

- Lifetime issuance must not exceed 2,870,000,000,000,000 san (28,700,000 SLG).
- There is no premine or spendable genesis allocation. Coins are issued through PoW mining.
- Preserve the inherited mining and reward mechanisms except where a concrete incompatibility with these requirements is demonstrated.
- Use the inherited Rusty Kaspa subsidy table and DAA-score schedule, scaled by 1,000.
- Round scaled rewards down to the nearest whole san using integer division.

## Verified source facts

- `CoinbaseManager` in `consensus/src/processes/coinbase.rs` calculates subsidy from DAA score and an inherited monthly subsidy table, with BPS-dependent scaling and blue/red mergeset reward handling.
- The SLUG networks run at 10 BPS from genesis; monthly table entries retain the upstream per-second conversion (`ceil(table_entry / 10)`) before the SLUG scale is applied.
- `MAX_SAN` in `consensus/core/src/constants.rs` limits an individual transaction amount to the SLUG lifetime supply. It does not track or enforce cumulative issuance, which is checked separately by coinbase consensus.
- Coinbase payloads commit the cumulative mining subsidy in their selected-parent past. A block adds eligible, previously unmerged blue/red subsidies once, then caps its own scheduled subsidy to the remaining supply.
- Mergeset subsidies that together exceed the selected-parent remainder are rejected; fees remain transfers and do not count as issuance.

## Consensus rules now implemented

- `calc_block_subsidy` uses the existing DAA-score/month schedule and divides the result by 1,000 using integer floor division. No reward is added to compensate for rounding.
- At the configured 10 BPS target rate, summing `calc_block_subsidy` across the full pre-deflationary interval and all 426 monthly table intervals yields a scheduled 2,837,620,256,850,000 san (28,376,202.5685 SLG). This is a target-rate schedule projection, not a promise about wall-clock duration or a realized DAG's exact block count. It leaves 32,379,743,150,000 san (323,797.4315 SLG) below the 28,700,000 SLG hard cap; zero-reward table periods and per-block integer truncation do not trigger supplemental issuance.
- Cumulative subsidy is encoded in every coinbase payload, including zero in each genesis payload. Since the value travels with each block, it is reconstructed from the selected-parent coinbase after restart and follows that branch through reorganization.
- The selected parent's cumulative value is combined with subsidies from newly merged DAA-eligible blocks. If this already exceeds the cap, the candidate is invalid. Otherwise, its own reward is `min(scheduled_reward, remaining_supply)`.
- Fees do not count toward issued supply. Red/blue payouts continue through the inherited coinbase rules.

## Implementation boundary

The per-transaction amount limit remains separate from cumulative lifetime issuance, although both use the same san-denominated maximum value. Wallet and RPC monetary fields are named in san, and display formatting uses SLG. The branch-local cap does not guarantee a globally irreversible accounting across a reorganization; it guarantees that every accepted branch's selected-parent past remains at or below the cap. Mainnet readiness also requires reviewing genesis launch difficulty and operational deployment. See [the decision log](./SLUG-DECISION-LOG.md).

Integration tests now exercise two competing branches, a selected-chain switch, merge accounting, and persistence of cumulative subsidy through pruning and database reopen. Wallet and RPC consumers do not parse the coinbase payload to calculate balances; the payload remains opaque outside consensus/mining, while wallets consume validated transactions and UTXO updates.
