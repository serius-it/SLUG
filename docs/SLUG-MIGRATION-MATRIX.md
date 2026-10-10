# SLUG Stage 1 Migration Matrix

Status labels are intentionally limited to `NOT_STARTED`, `INVESTIGATED`,
`IMPLEMENTED`, `TESTED`, `BLOCKED`, and `REQUIRES_APPROVAL`. `TESTED` is only
used where the relevant command was run successfully.

| File/module | Required change | Risk | Dependencies | Status |
|---|---|---|---|---|
| `README.md` | State approved SLUG identity and make the un-migrated status explicit. | Low; documentation only. | Approved project identity. | IMPLEMENTED |
| `Cargo.toml` workspace repository metadata | Point inherited repository metadata at the configured `serius-it/SLUG` remote. Preserve upstream authorship. | Low; package metadata, not code/API. | Verify the configured remote; done. | TESTED |
| `Cargo.toml` workspace members/dependencies | Coordinate any `kaspa-*` crate/dependency renaming across all packages. | High; compile-time/public package API, downstream breakage. | Complete package graph, builds, CI, release and consumer plan. | INVESTIGATED |
| `kaspad/Cargo.toml`, `cli/Cargo.toml`, `wallet/native/Cargo.toml` | Decide new daemon, CLI, and wallet package/binary names and update every consumer together. | High; binary names, scripts, user workflows. | Package/API approval; no compatibility strategy yet. | REQUIRES_APPROVAL |
| `consensus/core/src/network.rs` | Add SLUG network identity parsing, display/prefix mapping, network IDs, and approved port defaults. | Critical for isolation; identifier values are also serialized and consumed broadly. | Testnet suffix decision, port allocation, wire compatibility. | REQUIRES_APPROVAL |
| `consensus/core/src/config/params.rs`, `consensus/core/src/config/bps.rs` | Define only approved SLUG parameter sets, genesis bindings, the approved 10-second target interval for mainnet/testnet/devnet, difficulty, and empty/SLUG-owned seed lists. | Consensus/network critical. | Approved network map, seed infrastructure, monetary and genesis design. | BLOCKED |
| `protocol/p2p/proto/p2p.proto`, `protocol/flows/src/flow_context.rs` | Ensure SLUG-only network name is sent and enforced; decide whether additional wire identity/versioning is required. | Critical; peer isolation/protocol compatibility. | Approved unique network marker and protocol compatibility plan. | INVESTIGATED |
| `components/connectionmanager/src/lib.rs`, `kaspad/src/daemon.rs` | Prevent Kaspa DNS discovery and isolate peer settings, database, and logs for SLUG. | Critical; accidental connection/state reuse. | SLUG-owned discovery, unique identity and paths. | INVESTIGATED |
| `kaspad/src/args.rs` | Decide SLUG flags and environment-variable names while retaining/retiring KASPAD names intentionally. | Medium/high; operational API and scripts. | CLI/API compatibility decision. | REQUIRES_APPROVAL |
| `crypto/addresses/src/lib.rs` | Implement approved SLUG prefixes consistently in address formatting, parsing, serialization, and tests. | High; address compatibility and funds UX. | Prefix approval, exhaustive encode/decode vectors, client coordination. | REQUIRES_APPROVAL |
| `consensus/core/src/config/genesis.rs` | Create new SLUG genesis values and prove all derived hash/root/commitment/target/state values; do not reuse current constants. | Consensus-critical; irreversible chain identity. | Approved consensus parameters, timestamp/nonce process, independent review. | BLOCKED |
| `consensus/core/src/constants.rs` | Replace Kaspa denomination/transaction-bound symbols only as part of the approved SLUG monetary design; keep storage-mass units consistent. | Consensus-critical; could alter transaction acceptance/mass. | Complete emission and transaction range design. | INVESTIGATED |
| `consensus/src/processes/coinbase.rs`, `consensus/core/src/config/params.rs` | Implement approved SLUG rewards and prove the lifetime cap through consensus tests. | Consensus-critical; issuance/hard cap. | Emission schedule and cap semantics approval. | BLOCKED |
| `consensus/src/processes/transaction_validator/` | Update transaction range enforcement only as required by approved monetary rules; add boundary tests. | Consensus-critical; transaction acceptance. | Monetary design; coinbase/reward implementation. | BLOCKED |
| `wallet/core/src/utils.rs`, `wallet/core/src/wasm/utils.rs`, `wallet/pskt/src/wasm/utils.rs` | Migrate denomination conversion/formatting and user-facing SLG suffixes without changing encoded amounts. | Medium/high; incorrect amount display or API break. | Consensus unit decision and RPC/API plan. | INVESTIGATED |
| `rpc/core/`, `rpc/grpc/core/proto/`, `rpc/wrpc/client/src/resolver.rs` | Decide API namespace, RPC units, generated protobuf package, resolver path, and external endpoint strategy. | High; serialized/wire/client compatibility. | API governance, endpoint ownership, versioning. | REQUIRES_APPROVAL |
| `protocol/p2p/proto/`, `rpc/grpc/core/proto/`, build scripts | Review `protowire`, Go package, field numbers/types, and generated code before changing protocol names. | High; serialization/protocol compatibility. | Protocol versioning and downstream client plan. | INVESTIGATED |
| `consensus/core/src/hashing/covenant_id.rs`, `crypto/hashes/src/hashers.rs`, `consensus/core/src/tx.rs` | Preserve the current covenant implementation for now; review it before SLUG genesis/covenant policy is set. | Consensus/security critical. | Genesis covenant design and compatibility/security review. | INVESTIGATED |
| `kaspad/src/daemon.rs`, `core/src/log/consts.rs`, database/index initialization | Introduce isolated SLUG app-data, chain-state, and log locations; never silently open Kaspa state. | High; state contamination/data loss. | Approved paths, startup safeguards, migration/import policy. | INVESTIGATED |
| `wallet/core/src/compat/gen0.rs`, wallet storage | Decide whether importing legacy Kaspa wallets is supported, explicitly and safely. | High; key/user-fund handling and branding. | Wallet compatibility/security review. | REQUIRES_APPROVAL |
| `daemon/`, `mining/`, `protocol/mining/`, `bridge/` | Trace and migrate mining configuration/stratum interfaces only after chain reward and network IDs are fixed. | High; mining on/advertising wrong chain. | Approved SLUG network, rewards, ports, miner API. | INVESTIGATED |
| `cli/src/matchers.rs`, wallet and resolver defaults | Replace Kaspa explorer links and external service defaults with approved SLUG endpoints; no guessed URLs. | Medium; external endpoint integrity. | Service ownership and URLs. | REQUIRES_APPROVAL |
| `.github/workflows/ci.yaml`, `.github/workflows/deploy.yaml`, shell scripts | Coordinate binary/package/archive/WASM artifact names after the package naming decision. | Medium; release and CI breakage. | Approved names and output compatibility. | INVESTIGATED |
| `wasm/`, `wallet/wasm/`, `rpc/wrpc/wasm/` | Migrate exported package, JS symbols, TypeScript docs, npm metadata, examples, and generated API bindings consistently. | High; public API/serialized data compatibility. | Package/API and RPC/address decisions. | INVESTIGATED |
| `testing/integration/`, consensus/network tests | Add cross-network connection/state isolation, supply cap, genesis, address, and compatibility tests for each approved change. | High; validation coverage. | Implementations and approved test vectors. | NOT_STARTED |
| `docs/SLUG-STAGE1-AUDIT.md` | Record architecture, categorized findings, network/money/genesis/covenant audit, decisions, and migration order. | Low; documentation. | Repository inspection. | IMPLEMENTED |
| `docs/SLUG-MIGRATION-MATRIX.md` | Maintain this matrix and statuses as each approved stage lands. | Low; documentation. | Stage decisions and verified tests. | IMPLEMENTED |
| `docs/SLUG-STAGE1-CHANGELOG.md` | Record Stage 1 changes, preserved constraints, and actual validation results. | Low; documentation. | Commands actually executed. | IMPLEMENTED |

## Dependency order

Resolve approval/blocker rows before changing consensus parameters, addresses,
or wire protocols. Network isolation and no-Kaspa discovery/data paths precede
any operation of a SLUG node. The emission design precedes supply-limit code;
approved network/consensus parameters precede genesis creation; genesis and
covenant review precede production wallet/RPC exposure. Package and branding
renames follow those behavioral decisions so they cannot disguise incomplete
network separation.
