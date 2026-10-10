# SLUG Design Decisions and Verification

Status: decision record for review; this is not an implementation specification
for unapproved consensus details.
Verified against the repository on: 2026-10-09
Scope: design formulation and verification only. No runtime, consensus, genesis,
address, serialization, wire-protocol, or package-identity implementation is
authorized by this document.

Historical status: this document records the pre-implementation review and its
implementation-status tables are not current. Current behavior and remaining
launch limitations are recorded in `SLUG-EMISSION-DESIGN.md`,
`SLUG-TIMING-DESIGN.md`, `SLUG-NETWORK-DECISIONS.md`, and `laporan_terkini.md`.

## Purpose and authority

This record reconciles the SLUG project requirements in `stage1.md` with the
Stage 1 audit, migration matrix, changelog, and the current source code. It
distinguishes:

1. project requirements accepted as design constraints;
2. facts verified in the current Kaspa-derived implementation;
3. design decisions that still require explicit approval; and
4. blockers that make implementation unsafe until those decisions are made.

The project requirements below do not describe current network behavior.
Nothing in this record means the checkout is ready to run a SLUG node.
`stage1.md` remains unchanged.

## 1. Approved project requirements

These are accepted SLUG requirements, not statements that source code already
implements them:

| Requirement | Accepted value / constraint | Implementation status |
|---|---|---|
| Project and native currency | SLUG | Not implemented throughout the codebase. |
| Ticker | SLG | Not implemented in wallet/RPC/UI formatting. |
| Smallest denomination | san | Current source still uses `sompi`. |
| Denomination scale | 100,000,000 san = 1 SLG | Same numeric scale as the inherited unit, but the SLUG unit/API migration is not implemented. |
| Lifetime maximum supply | 28,700,000 SLG = 2,870,000,000,000,000 san | Not enforced by a verified cumulative issuance rule. |
| Genesis issuance | No premine | The future SLUG genesis must allocate no spendable supply; the inherited genesis is not a SLUG genesis and must not be reused. |
| Issuance source | Mining | Supply must arise from consensus-valid mining rewards; the reward schedule and cap logic remain undefined. |
| Mining / proof of work | PoW with adaptive difficulty | Public-network configurations in current source enable PoW; current simnet skips PoW by default. The desired SLUG simnet behavior needs clarification. |
| Main/test/dev target interval | 10 seconds per block | Does not match inherited mainnet/testnet/devnet parameters; see the verified block-rate analysis below. |
| Logical network names | `slug-mainnet`, `slug-testnet-1`, `slug-devnet`, `slug-simnet` | Current IDs and prefixed names are Kaspa names; no mapping is implemented. |
| Planned address prefixes | `slug`, `slugtest`, `slugdev`, `slugsim` | Recorded as planned candidates, not yet as final encoding/API approval. |
| Covenant policy | Retain the current mechanism provisionally, pending compatibility/security review and genesis-policy decision | The inherited mechanism remains in source; its approval for SLUG consensus/genesis is still blocked on review. |

## 2. Technical decisions and facts verified in source

### 2.1 Block-rate semantics and the 10-second requirement

The number in `BlockrateParams::new::<N>()` is an integer **blocks-per-second
(BPS) rate**, not a block interval in seconds. The source chain establishing
this is:

1. `consensus/core/src/config/params.rs::BlockrateParams::new<const BPS: u64>`
   initializes `target_time_per_block` by calling
   `Bps::<BPS>::target_time_per_block()` and also derives GHOSTDAG K, median
   time/difficulty sample rates, parent and mergeset limits, merge/finality/
   pruning depths, and coinbase maturity.
2. `consensus/core/src/config/bps.rs::Bps<BPS>` explicitly models blocks per
   second. `bps()` returns `BPS`, and `target_time_per_block()` returns
   `1000 / BPS` milliseconds per block. The conversion requires `BPS` to divide
   1000 exactly. Its GHOSTDAG lookup is defined only for integer values 1
   through 32.
3. Thus `BlockrateParams::new::<10>()` means **10 blocks per second**, or
   **100 milliseconds per block**. It does not mean 10 seconds per block.
   Mainnet, testnet, and devnet currently use this setting in
   `consensus/core/src/config/params.rs`. Simnet uses the same rate and raises
   its maximum parent count.
4. Mainnet and testnet also set `pre_crescendo_target_time_per_block` to
   `1000` milliseconds. Devnet and simnet use
   `TenBps::target_time_per_block()`, which is 100 milliseconds. This is a
   historical coinbase-emission input as well as a current-rate setting.
5. `consensus/core/src/config/constants.rs` scales block-count-based values
   (including GHOSTDAG timing-related parameters, finality/pruning depths,
   difficulty and median-time sampling, coinbase maturity, and subsidy
   transition scores) from an integer BPS. These values cannot be copied
   unchanged when changing the interval by a factor of 100.
6. `Params::target_time_per_block()` returns the configured interval in
   milliseconds. It is passed into the sampled DAA by
   `consensus/src/consensus/services.rs` and used by
   `consensus/src/processes/difficulty.rs::SampledDifficultyManager`, which
   computes a new target using sampled block timestamps, target interval,
   sample rate, and observed targets. This is an adaptive difficulty path.
   Mining and protocol mining rules also consume target-time parameters.
7. Coinbase emission is coupled to the rate: `Params::bps_history()` computes
   integer rates as `1000 / target_time_per_block`; `CoinbaseManager` uses
   these rates when turning DAA-score progression into elapsed seconds/months
   and when scaling per-block rewards from the legacy table.

Ten seconds per block is 10,000 milliseconds per block, equivalent to 0.1
blocks per second. That fractional rate cannot be represented by the current
`Bps<const BPS: u64>` generic. Substituting `BPS = 0` is invalid: the
`Bps` conversion divides by BPS, the GHOSTDAG lookup has no zero entry, and
other derivations multiply durations by BPS. Changing only
`target_time_per_block` to 10,000 is also invalid: `Params::bps()` and both
entries in `bps_history()` use integer division `1000 / 10000`, yielding zero.
The coinbase manager then uses the rate in divisions, and the rate-derived
block-count parameters would remain those previously generated for 10 BPS.
That would make DAA windows, consensus depths, maturity, and subsidy timing
inconsistent with the intended interval.

**Verified conclusion:** a valid SLUG 10-second setting requires a coherent
parameter model and derivation review, not a one-line change to the const
generic or millisecond interval. The design must explicitly decide how to
represent fractional blocks per second or elapsed seconds per block and how
each derived block-count quantity is rounded and tested. The architecture
choice is not approved here. Any implementation must preserve mathematically
consistent DAA window duration and adaptive-target timing, GHOSTDAG security
assumptions, median-time sampling, maturity/finality/pruning behavior, mining
rate calculations, and subsidy elapsed-time calculations. No emission amount
or new schedule is proposed in this document.

### 2.2 PoW and adaptive difficulty

- `consensus/core/src/config/params.rs` sets `skip_proof_of_work: false` for
  the current mainnet, testnet, and devnet parameter constants; simnet sets it
  to `true`.
- Header pre-GHOSTDAG validation checks proof of work and only accepts a failed
  PoW check when the configured skip flag is enabled. Pruning-proof validation
  consumes the same skip policy.
- `consensus/src/processes/difficulty.rs` contains the sampled difficulty
  manager and recalculates compact target bits from a window of block
  timestamps and targets.

This verifies that the inherited public network configs use PoW and an
adaptive DAA, but it does not certify the DAA's security for a 10-second SLUG
target. Whether local SLUG simnet should skip PoW, and whether any skip option
must be impossible on public SLUG networks, requires approval and explicit
configuration safeguards.

### 2.3 Network IDs, testnet, ports, and peer discovery

- `consensus/core/src/network.rs::NetworkType` has `Mainnet`, `Testnet`,
  `Devnet`, and `Simnet`; `NetworkId` encodes a type and optional `u32`
  suffix. The parser accepts type names with an optional suffix and requires a
  suffix for testnet.
- `NetworkId::to_prefixed()` currently produces `kaspa-{network_id}` and
  `from_prefixed()` accepts only `kaspa-`. This is not any of the approved
  `slug-*` names.
- `Params::from(NetworkId)` in `consensus/core/src/config/params.rs` only has
  a parameter set for testnet suffix 10; another testnet suffix panics. A
  mechanical `slug-testnet-1` to `NetworkId::Testnet { suffix: 1 }` mapping
  therefore does not currently select usable consensus parameters.
- Current default ports in `consensus/core/src/network.rs` are:

  | Existing network | P2P | gRPC | wRPC Borsh | wRPC JSON |
  |---|---:|---:|---:|---:|
  | Mainnet | 16111 | 16110 | 17110 | 18110 |
  | Testnet suffix 10 | 16211 | 16210 | 17210 | 18210 |
  | Testnet suffix 12 | 16311 | 16210 | 17210 | 18210 |
  | Other testnet suffix | 16411 | 16210 | 17210 | 18210 |
  | Simnet | 16511 | 16510 | 17510 | 18510 |
  | Devnet | 16611 | 16610 | 17610 | 18610 |

- `consensus/core/src/config/params.rs` contains live Kaspa mainnet and
  testnet DNS seeders; devnet and simnet seed lists are empty.
  `components/connectionmanager/src/lib.rs` uses the configured seeder list
  and default P2P port to discover peers.
- `protocol/p2p/proto/p2p.proto::VersionMessage.network` is a string. The
  flow in `protocol/flows/src/flow_context.rs::initialize_connection` sends
  the local network name and rejects a peer whose name differs. Current
  `config.network_name()` derives from the Kaspa-prefixed `NetworkId`.
  No distinct P2P magic value was identified in this path; that does not
  certify all external protocol compatibility.
- The resolver default is not empty: `rpc/wrpc/client/Resolvers.toml` disables
  its explicit localhost record but enables the `kaspa.stream` groups by
  default (an omitted `enable` means enabled). The resolver builds a path with
  `/v{version}/kaspa/{network_id}/{tls}/wrpc/{encoding}` in
  `rpc/wrpc/client/src/resolver.rs`.

**Verified conclusion:** neither network IDs nor peer discovery are isolated
for SLUG. Do not enable SLUG DNS/resolver defaults until SLUG-operated endpoint
ownership and network segregation are established. Testnet suffix mapping and
all default ports require explicit decisions; none is inferred here.

### 2.4 Database, logs, and wallet state

- `kaspad/src/daemon.rs::get_app_dir()` defaults to `~/.rusty-kaspa` on
  non-Windows systems and `rusty-kaspa` below the Windows local data directory.
  The node data directory is then nested under
  `network.to_prefixed()/datadir`; logs default under
  `network.to_prefixed()/logs`. Since the prefix is currently `kaspa-`, these
  locations do not provide SLUG-specific isolation.
- The wallet SDK defaults its local storage directory to `~/.kaspa`, with
  default wallet/settings names `kaspa`, in
  `wallet/core/src/storage/local/mod.rs`. `wallet/core/src/compat/gen0.rs`
  explicitly checks historical Kaspa wallet locations, the `kaspa.kpk` name,
  and the `kaspa-wallet` storage key. This is a legacy key/wallet import path,
  distinct from node chain-state storage.
- A SLUG-specific node directory alone is insufficient unless resolved paths,
  explicit `--appdir`/logdir overrides, reset/migration behavior, and all
  database and index stores have been checked. A legacy wallet import policy
  must not imply importing Kaspa chain state or treating old addresses as SLUG
  addresses.

### 2.5 Money, rewards, and the meaning of the supply cap

- `consensus/core/src/constants.rs::SOMPI_PER_KASPA` is 100,000,000. The
  approved SLUG scale has the same numeric ratio, but all current names and
  displayed denomination behavior are Kaspa-specific.
- `MAX_SOMPI` is `29_000_000_000 * SOMPI_PER_KASPA`. The constant is consumed
  by transaction input/output range checks, including
  `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs`
  and `tx_validation_in_utxo_context.rs`, plus associated error messages.
  It bounds amounts within transactions; it is **not** a lifetime issuance
  counter or supply-cap enforcement mechanism.
- `consensus/src/processes/coinbase.rs::CoinbaseManager::calc_block_subsidy`
  computes a reward from consensus parameters and the inherited 426-entry
  `SUBSIDY_BY_MONTH_TABLE`. `expected_coinbase_transaction` forms rewards for
  mergeset blue blocks and aggregated red-block rewards. This is inherited
  reward behavior, not an approved SLUG schedule.
- The inherited genesis transaction has no spendable outputs, but its payload
  and static commitments belong to Kaspa. That fact does not create a SLUG
  genesis decision or validate future supply accounting.

To ensure cumulative issuance never exceeds 2,870,000,000,000,000 san, a future
consensus design must decide and specify all of the following before code is
written:

1. **What counts as issued:** define whether the counter is gross subsidy
   authorized by consensus, subsidy actually claimable/paid, or another
   precise measure. Fees are transfers of already issued san and must be
   accounted separately from newly minted subsidy. Define treatment of
   genesis allocations, burns, invalidated/orphaned blocks, and any future
   non-mining issuance; the approved no-premine/mining-only requirements rule
   out a spendable genesis allocation and unapproved issuance sources.
2. **Where issuance is counted in a DAG:** define deterministic branch-local
   accounting and how the selected/virtual chain, blue/red classification,
   mergeset rewards, reorganizations, and competing valid blocks avoid counting
   a reward twice or allowing two branches each to spend the same remaining
   issuance budget. The accounting state must be consensus-derived and
   reproducible during initial sync and pruning/recovery.
3. **How the cap affects each reward:** specify that no valid accepted history
   can authorize cumulative subsidy above the cap, including the final partial
   reward, rounding, parallel block production, and any aggregate coinbase
   output. Determine whether the last reward is reduced to the exact remaining
   san or issuance stops before it; do not assume a schedule can be clipped
   safely without specifying its consensus consequences.
4. **How validation enforces it:** identify the state transition and
   independent block/coinbase validation checks that reject an over-cap
   reward, define arithmetic/overflow behavior, and guarantee identical
   acceptance for every conforming node. Transaction-value bounds remain
   separate checks.
5. **How it is proven:** test genesis issuance equals zero, accumulated
   issuance at every reward boundary, just-below/equal/above cap, partial last
   reward or stop behavior, fees not increasing issuance, DAG blue/red/reorg
   cases, restart/initial-sync/pruning equivalence, and total issuance
   invariants. The approved maximum must be asserted against a mathematically
   derived schedule and state transition, not a renamed constant.

These are required specification questions, not a proposed emission schedule.
No reward amounts, halving cadence, duration, curve, or transition height is
specified or inferred here.

### 2.6 Addresses, RPC, and compatibility

- Current address prefixes in `crypto/addresses/src/lib.rs::Prefix` are
  `kaspa`, `kaspatest`, `kaspasim`, and `kaspadev`. The SLUG strings remain
  planned prefix candidates; no change to address encoding or parsing is
  verified or authorized.
- P2P and gRPC protobuf definitions use `protowire`; P2P protobuf declares a
  Kaspa Go package. wRPC resolver path includes a literal `kaspa` namespace.
  These are protocol/API compatibility surfaces, not branding-only strings.
- Wallet Rust names, wasm exports, RPC amount fields (including `sompi`
  spellings), storage formats, endpoints, and old wallet-import support each
  need a compatibility policy. A change in unit label must not silently change
  the serialized integer value or interpretation.
- Public Kaspa resolver endpoints, DNS seeders, explorer links, default ports,
  and network data locations cannot be repurposed for SLUG by changing only a
  name in UI or config.

## 3. Decisions not final; approval required

No approval is recorded in the supplied Stage 1 audit/changelog for the
following implementation choices. The fixed project requirements in Section 1
remain constraints while these choices are discussed.

### Consensus and issuance

- The full emission schedule: initial mining reward, schedule shape and
  duration, transitions, rounding rules, endpoint behavior, and how rewards
  account for the 10-second block interval.
- Exact lifetime-cap accounting semantics and the deterministic branch/DAG
  enforcement model described in Section 2.5.
- Whether all SLUG networks use active PoW, or simnet retains a
  non-production/testing-only PoW skip. Public network configurations must
  enforce PoW if PoW is approved there.
- The revised block-rate parameter representation and derivation policy needed
  to support 0.1 blocks/second while preserving adaptive difficulty and
  consistent timing/security parameters. Decide rational-rate representation,
  time-domain quantities, and rounding for block-count quantities; no single
  code architecture is selected here.
- The consensus security review of inherited GHOSTDAG, difficulty, median time,
  finality, pruning, coinbase maturity, and mining pool/stratum assumptions at
  the 10-second target.
- Genesis construction procedure, initial difficulty/target, timestamp, nonce,
  commitment derivation and review. Only the no-premine requirement is fixed;
  no genesis value is approved.
- Covenant compatibility/security review and the policy for any genesis
  covenant bindings. The mechanism is provisional, not finally approved for
  SLUG launch.

### Network and operational isolation

- Whether `slug-testnet-1` means suffix 1 in the current conceptual ID model,
  or needs an independent logical name/ID encoding. In either case it needs an
  explicit consensus-parameter mapping and parser/handshake tests.
- Unique SLUG defaults for P2P, gRPC, wRPC Borsh, and wRPC JSON; testnet
  suffix/port rules and collision policy.
- SLUG DNS seeders, resolver ownership and domains, bootstrap procedure, and
  behavior while no public seeders exist. No Kaspa endpoint may be used as a
  fallback.
- Exact SLUG node application/data/log directories, wallet storage namespace,
  overrides, startup guardrails, state reset and migration behavior, and
  operating-system-specific paths.
- Whether old Kaspa wallets may be imported by a user-initiated, clearly
  separated key-recovery operation. It must not reuse Kaspa network identity,
  addresses, or chain state as SLUG data.
- Rules for direct peers, public RPC selection, and explicit endpoints so
  wrong-network connections fail closed.

### API, compatibility, and project ownership

- Final address prefix approval and any legacy-prefix behavior. The listed
  prefix candidates are not yet an implementation/encoding approval.
- RPC/gRPC/wRPC namespace, protobuf/Go package names, wire/API versioning,
  amount field semantics, resolver URLs, and whether existing clients are
  supported.
- Crate/binary names, CLI flags, environment variables, wallet labels,
  JS/WASM/npm exports, release artifacts, and any compatibility aliases.
- Approved SLUG explorer/service URLs and ownership of external infrastructure.
- Publishing and upstream attribution/license policy. Existing author and
  license metadata are preserved; this document does not approve removing or
  changing them.

## 4. Blockers: implementation is unsafe to proceed

The following are hard gates, not merely cleanup tasks:

1. **No network isolation design is implemented.** Current IDs, P2P
   handshake names, main/test DNS seeders, public resolver domains, data
   directories, and ports identify or overlap the Kaspa implementation.
   Until the approved SLUG identity, endpoint, port, and storage policies are
   defined and tested fail-closed, do not operate this source as a SLUG node.
2. **The approved testnet name does not map to the available testnet
   parameters.** `slug-testnet-1` cannot safely be treated as the existing
   suffix-10 testnet; current `Params::from` panics on unsupported suffixes.
3. **The 10-second target cannot be represented by the current BPS generic.**
   A `new::<10>()` setting means 100 ms, while a direct 10,000-ms override
   yields zero from integer `Params::bps()` and `bps_history()`. Do not modify
   the generic or one parameter in isolation. Resolve and review the coherent
   timing/difficulty/GHOSTDAG/subsidy parameter model first.
4. **There is no approved issuance schedule or verified lifetime-cap
   accounting rule.** `MAX_SOMPI` is only a per-transaction amount bound.
   Changing it, or changing the denomination name, cannot establish the
   lifetime maximum. No consensus reward/cap implementation is safe before
   precise DAG accounting and cap-boundary rules are approved.
5. **There is no SLUG genesis specification.** Reusing existing Kaspa constants
   or inventing hash, merkle root, UTXO commitment, timestamp, nonce, or target
   would define an unapproved chain. Genesis work remains blocked.
6. **Address and protocol compatibility is undecided.** Prefix, wire/API
   namespace, serialized-unit, client, and wallet compatibility changes can
   strand users or allow cross-network ambiguity. Do not alter those formats
   without full consumer tracing and explicit approval.
7. **Covenant/genesis policy review is outstanding.** Keep the current code
   untouched; do not assign final SLUG covenant IDs or use it to infer genesis
   policy before review.

## 5. Stage gate and review request

This document completes only the requested design-decision formulation and
source verification. Implementation must remain paused until this decision
record has been reviewed and the pending choices—especially the fractional
block-rate model, supply accounting, testnet mapping, isolation values, genesis
policy, and compatibility scope—are explicitly approved. A later
implementation task must cite those approvals and define its own test vectors.

The only changes made for this task are this documentation file. The existing
Stage 1 files and the untracked `stage1.md` task file are preserved.
