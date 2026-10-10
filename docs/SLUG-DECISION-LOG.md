# SLUG Decision Log

This log separates fixed project requirements from verified inherited behavior and unresolved decisions. A recommendation or source default is not user approval.

## Fixed project requirements

- Project: SLUG; ticker: SLG; smallest unit: san; 100,000,000 san = 1 SLG.
- Lifetime maximum issuance: 28,700,000 SLG = 2,870,000,000,000,000 san.
- No premine or spendable genesis allocation; issuance through PoW mining.
- Target block interval: 100 ms (10 BPS) on every SLUG network.
- Network IDs: `slug-mainnet`, `slug-testnet-1`, `slug-devnet`, `slug-simnet`.
- Address prefixes: `slug`, `slugtest`, `slugdev`, and `slugsim`.
- Default ports use the SLUG-only 19110-19413 ranges documented in [network decisions](./SLUG-NETWORK-DECISIONS.md).
- Preserve Kaspa consensus, GHOSTDAG, PoW, block production, DAA, transactions, covenants, and other functioning core mechanisms unless a concrete technical incompatibility requires a change.
- Keep upstream licensing and attribution.

## Verified inherited behavior

- Consensus timing and several derived parameters use integer BPS; public networks are currently configured for 10 BPS (100 ms per block).
- Subsidy calculation uses the inherited BPS-adjusted monthly table and DAA score, scaled by 1,000 with integer floor division. Coinbase consensus enforces a branch-local cumulative subsidy cap.
- Wallet, CLI, and RPC amounts use `san`, with `SLG` as the displayed currency. `MAX_SAN` bounds a transaction amount to the lifetime supply; cumulative issuance is separately enforced by coinbase consensus.
- Earlier source defaults used Kaspa network IDs, address prefixes, genesis constants, discovery settings, and storage names; current SLUG identity work replaces the network-facing values.
- These repository facts describe current code only; they do not approve retaining the associated behavior for SLUG.

## Approved policy

- The fixed project requirements and initial SLUG network values in the network decisions document are authorized for implementation.
- Genesis blocks use network-specific `slug-*` coinbase tags, a zero subsidy, no outputs, and an empty UTXO commitment. The timestamp is `2026-10-09T00:00:00Z`; testnet, devnet, and simnet use compact target bits `0x207fffff`. Per-network nonces and hashes are fixed in source.
- DNS seeders and resolver fallback remain disabled until SLUG-owned endpoints are explicitly configured.
- SLUG uses the upstream subsidy schedule from genesis, with the prescribed 1:1,000 reward scale and branch-local cap accounting. Coinbase payloads carry cumulative issued subsidy; newly merged eligible blue/red subsidies count once, fees are excluded, and the candidate subsidy is truncated to remaining supply.
- Every network uses 10 BPS from genesis; inherited Kaspa block-rate activation heights are not applied.
- Mainnet starts with genesis compact target `0x1f0694e4`, calibrated to an expected 100 ms interval at 100 kH/s under a uniform-hash model. It retains the inherited maximum permitted target and Rusty Kaspa DAA so difficulty can adjust normally after launch. Its genesis block follows the upstream genesis PoW bypass; this nonce is not represented as a standalone valid PoW solution.

## Remaining launch decisions

| Area | Required decision or work | Safe boundary |
|---|---|---|
| Emission | Validate branch-local cap accounting through long DAG/reorg/pruning/recovery integration tests and confirm wallet/indexer support for cumulative-subsidy payloads. | The rule caps each accepted branch; it is not an irreversible global counter across reorgs. |
| Network deployment | Confirm firewall availability for the selected port block and configure SLUG-owned bootstrap endpoints before public peer discovery is enabled. | Keep DNS seeding and resolver fallback disabled. |
| Genesis launch security | Measure expected and observed mainnet hash rate and reassess the target before public mining. | The initial target is calibrated for 100 kH/s, not proof of an adequate security budget or network availability. |
| Wallet and metadata | Define wallet namespace/import policy and review binary/package and application metadata. | Do not open or migrate Kaspa wallet state by default. |

## Traceability

- [Timing design](./SLUG-TIMING-DESIGN.md)
- [Emission design](./SLUG-EMISSION-DESIGN.md)
- [Network decisions](./SLUG-NETWORK-DECISIONS.md)
- [Stage 1 audit](./SLUG-STAGE1-AUDIT.md) and [migration matrix](./SLUG-MIGRATION-MATRIX.md) record earlier repository observations; they are not approvals.
