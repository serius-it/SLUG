# SLUG Stage 1 Repository Audit

Audit date: 2026-10-09
Scope: identity and compatibility inventory only. This audit does not establish
that the current code implements SLUG consensus or is safe to run as a SLUG node.

## Executive summary

The workspace is still a Kaspa-derived Rust implementation, not a migrated SLUG
node. It has 69 Cargo workspace packages, 1,066 Rust source files, and Rusty
Kaspa package, node, network, address, RPC, wallet, and deployment identities.
The current network configuration includes live Kaspa mainnet/testnet DNS
seeders and uses Kaspa network names for the P2P handshake and on-disk state.

Stage 1 changes are deliberately limited to the verified repository metadata
URL and a README that records the approved project identity while warning that
the implementation has not been migrated. No consensus, address, network,
serialization, package, or wire-protocol behavior was changed.

The supplied root `stage1.md` was already untracked when the work began. It was
read as the task specification and has not been edited or removed.

## Architecture overview

The root `Cargo.toml` defines a 69-package Rust workspace. The main execution
path is:

1. `kaspad/` is the full-node binary and assembles node startup, storage,
   consensus, networking, RPC, mining, and indexing.
2. `consensus/` and `consensus/core/` implement DAG consensus, transaction and
   block validation, state, coinbase rewards, and consensus parameters.
3. `protocol/p2p/` and `protocol/flows/` provide protobuf P2P messages,
   handshakes, peer flows, and synchronization.
4. `components/connectionmanager/` handles peer discovery and connections;
   `database/`, `indexes/`, and `consensus/` persist node state.
5. `rpc/` provides gRPC and wRPC clients/servers, API messages, and resolvers.
6. `wallet/`, `crypto/addresses/`, `crypto/txscript/`, and `mining/` provide
   wallet, address, script, and mining functionality. `cli/`, `daemon/`,
   `bridge/`, `wasm/`, and `simpa/` provide end-user tools and bindings.

`cargo metadata --no-deps --format-version 1` reported 69 workspace packages.
The main executable targets include `kaspad`, `kaspa-wallet`, `kaspa-cli`,
`simpa`, and RPC/P2P examples. The root workspace dependency aliases are
`kaspa-*` names and are referenced throughout package manifests and source.

## Identity classification

### Branding-only

- Root `README.md` previously contained only `# SLUG`; it now states the
  approved identity and clearly distinguishes requirements from implementation.
- Root `Cargo.toml`'s inherited `repository` metadata now points to the actual
  configured remote, `https://github.com/serius-it/SLUG`.
- Branding in package descriptions, CLI text, docs, logs, user agents, release
  assets, explorer links, wallet prompts, and WASM artifacts remains largely
  Kaspa-branded. Such strings must be reviewed in context; do not globally
  replace them.

### Package/API naming

- Workspace package names and dependency aliases are predominantly
  `kaspa-*`; the node package and binary are `kaspad`, and the wallet binary is
  `kaspa-wallet`. The authoritative root list is `[workspace.dependencies]` in
  `Cargo.toml`; each package also has its own `Cargo.toml`.
- Relevant entry points include `kaspad/Cargo.toml`, `cli/Cargo.toml`,
  `wallet/native/Cargo.toml`, `wallet/core/Cargo.toml`, `wasm/Cargo.toml`,
  `protocol/p2p/Cargo.toml`, `rpc/grpc/core/Cargo.toml`, and
  `rpc/wrpc/client/Cargo.toml`.
- Renaming crates/binaries is not a safe Stage 1 string-only change: references
  exist in manifests, Rust imports, build commands, CI, scripts, generated
  artifacts, examples, and downstream APIs. Upstream authorship metadata is
  intentionally retained.

### Network isolation

- `consensus/core/src/network.rs` defines the four network types, network IDs,
  RPC/P2P ports, parser, and `kaspa-` textual prefix (`NetworkId::to_prefixed`,
  `NetworkId::from_prefixed`).
- `consensus/core/src/config/params.rs` binds IDs to consensus parameters,
  genesis, and DNS seeds. Mainnet and testnet use Kaspa-operated/third-party
  Kaspa seed hosts; devnet and simnet have empty seeder lists.
- The approved SLUG target interval is 10 seconds on mainnet, testnet, and
  devnet. Current parameters use `BlockrateParams::new::<10>()` for these
  networks; `consensus/core/src/config/bps.rs` defines this as 10 blocks per
  second (100 ms per block). Mainnet/testnet also retain a 1,000 ms
  pre-Crescendo historical interval. These consensus values do not match the
  approved SLUG target interval.
- `protocol/p2p/proto/p2p.proto` has `VersionMessage.network` (field 10).
  `protocol/flows/src/flow_context.rs::initialize_connection` sends the local
  `config.network_name()` and rejects a peer whose network string differs.
  Thus the explicit handshake discriminator currently resolves to the
  Kaspa-prefixed network name; it must become SLUG-specific before operating a
  SLUG network. No separate P2P magic constant was found in the audited P2P
  message/handshake path.
- `kaspad/src/daemon.rs::get_app_dir` uses `rusty-kaspa`/`.rusty-kaspa` as the
  default application directory. The database and logs are further separated
  by `NetworkId::to_prefixed()` (for example, `kaspa-mainnet`) and the database
  subdirectory `datadir`. These names would collide with the source chain if
  reused.
- `kaspad/src/args.rs` defines network flags and `KASPAD_*` environment
  variables. Explicit peer options bypass DNS seeding. Network-specific
  database initialization and genesis handling are in `kaspad/src/daemon.rs`
  and `consensus/`.
- Current numeric defaults are:

  | Network | P2P | gRPC | wRPC Borsh | wRPC JSON |
  |---|---:|---:|---:|---:|
  | Mainnet | 16111 | 16110 | 17110 | 18110 |
  | Testnet suffix 10 | 16211 | 16210 | 17210 | 18210 |
  | Testnet suffix 12 | 16311 | 16210 | 17210 | 18210 |
  | Other testnet suffix | 16411 | 16210 | 17210 | 18210 |
  | Simnet | 16511 | 16510 | 17510 | 18510 |
  | Devnet | 16611 | 16610 | 17610 | 18610 |

  Values are defined in `consensus/core/src/network.rs`; the testnet P2P port
  varies by suffix while its RPC ports are shared.
- The current testnet parameter set is `testnet-10`; `Params::from(NetworkId)`
  panics for other testnet suffixes. The approved logical test network is
  `slug-testnet-1`; this mismatch must be resolved rather than silently mapped.
- wRPC resolver paths use `/v{version}/kaspa/{network_id}/{tls}/wrpc/{encoding}`
  in `rpc/wrpc/client/src/resolver.rs::make_url`. gRPC/P2P protobuf packages,
  service names, and generated bindings are protocol/API surfaces, not safe
  branding-only text.
- Isolation must be completed before any SLUG node is run: use a distinct
  handshake/network name, no Kaspa DNS seeds or resolvers, approved unique
  ports, distinct default data/log paths, and SLUG-specific parameters and
  genesis. Confirm every consumer, including direct-peer and wallet/RPC paths.

### Consensus-critical

- `consensus/core/src/config/params.rs::Params` and the `*_PARAMS` constants
  bind network IDs, genesis, target timing, difficulty windows, pruning,
  block limits, and coinbase parameters.
- `consensus/core/src/config/genesis.rs` defines static mainnet, testnet,
  simnet, and devnet genesis fields. `GenesisBlock::build_genesis_transactions`
  constructs the genesis coinbase; conversion to `Header` installs the
  configured merkle root, UTXO commitment, timestamp, bits, nonce, and DAA
  score. `test_genesis_hashes` checks the constructed block's merkle root and
  block hash against the configured values.
- `consensus/src/processes/coinbase.rs::CoinbaseManager` calculates subsidies
  from network parameters and a 426-entry monthly table. Transaction output
  range enforcement is in `consensus/src/processes/transaction_validator/`.
- Block/transaction hashing, merkle computation, UTXO commitment, difficulty,
  and state validation are consensus rules. Key locations include
  `consensus/core/src/hashing/`, `consensus/core/src/merkle.rs`,
  `consensus/core/src/muhash.rs`, `consensus/src/processes/difficulty.rs`,
  `consensus/src/pipeline/`, and `consensus/src/processes/transaction_validator/`.
- No consensus parameter, emission schedule, reward, hash, genesis value, or
  validation behavior was changed in Stage 1.

### Serialization, protocol, and external ecosystem compatibility

- Address prefixes and parsing/formatting live in `crypto/addresses/src/lib.rs`
  (`Prefix`, `Prefix::as_str`, `TryFrom<&str>`) and are consumed by
  `consensus/core/src/network.rs` and wallet/wasm APIs. The implemented values
  are `kaspa`, `kaspatest`, `kaspasim`, and `kaspadev`, not the approved
  candidates `slug`, `slugtest`, `slugdev`, and `slugsim`. Changing a prefix
  affects address strings, APIs, fixtures, and downstream wallets; the
  address implementation was not changed.
- `protocol/p2p/proto/*.proto` declares package `protowire`, Go package
  `github.com/kaspanet/kaspad/protowire`, and the P2P service/messages.
  `rpc/grpc/core/proto/*.proto` also declares `protowire`. Renaming these or
  changing serialized fields requires an explicit compatibility/versioning
  plan.
- wRPC's `kaspa` URL segment, RPC message names, wallet serialized formats,
  Rust crate names, JavaScript/WASM exports, and existing integration fixtures
  are external compatibility surfaces. For example, wallet compatibility code
  reads legacy Kaspa wallet files in `wallet/core/src/compat/gen0.rs`.
- CI and release wiring are under `.github/workflows/ci.yaml` and
  `.github/workflows/deploy.yaml`; the latter builds and packages `kaspad`,
  `kaspa-wallet`, and `stratum-bridge` with `rusty-kaspa-*` release archives.
  Build scripts include `protocol/p2p/build.rs`, `rpc/grpc/core/build.rs`,
  `crypto/hashes/build.rs`, `crypto/smt/build.rs`, and WASM scripts under
  `wasm/`, `wallet/wasm/`, and `rpc/wrpc/wasm/`.

## Monetary-unit and supply audit

`consensus/core/src/constants.rs` currently defines
`SOMPI_PER_KASPA = 100_000_000` and
`MAX_SOMPI = 29_000_000_000 * SOMPI_PER_KASPA`. The former is also used to
scale `STORAGE_MASS_PARAMETER`. `MAX_SOMPI` is used by transaction input/output
range validation and error reporting; it is a per-transaction amount bound,
not proof of a lifetime supply cap.

The emission/reward dependency chain is:

1. `consensus/core/src/config/params.rs::Params` supplies the pre-deflationary
   base subsidy, deflationary DAA score, BPS history, and activation settings.
2. `consensus/src/processes/coinbase.rs::CoinbaseManager::calc_block_subsidy`
   derives a block subsidy from those settings and the 426-month
   `SUBSIDY_BY_MONTH_TABLE`.
3. Coinbase rewards are generated and checked through consensus transaction
   processing and `consensus/src/processes/transaction_validator/`.
4. Wallet, RPC, CLI, mining, tests, and WASM code parse/display values in
   `sompi` and `KAS` denominations. These API/unit names are used widely and
   must migrate coherently after consensus rules are approved.

The source inventory search for `sompi`, `SOMPI_PER_KASPA`, and `MAX_SOMPI`
found uses in the following Rust files:

- Consensus and mining: `consensus/core/src/constants.rs`,
  `consensus/core/src/errors/tx.rs`, `consensus/core/src/mass/mod.rs`,
  `consensus/src/processes/coinbase.rs`,
  `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs`,
  `consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs`,
  `mining/src/feerate/mod.rs`, `mining/src/manager_tests.rs`,
  `mining/src/mempool/check_transaction_limits.rs`,
  `mining/src/mempool/check_transaction_standard.rs`,
  `mining/src/mempool/config.rs`, `mining/src/mempool/model/frontier.rs`,
  `mining/src/template_limits_tests.rs`, `mining/src/testutils/coinbase_mock.rs`,
  `crypto/txscript/src/opcodes/mod.rs`, and `crypto/txscript/src/standard.rs`.
- RPC, bridge, CLI, and tools: `rpc/core/src/model/feerate_estimate.rs`,
  `rpc/core/src/model/message.rs`, `rpc/core/src/model/tests.rs`,
  `rpc/core/src/wasm/message.rs`, `rpc/grpc/core/src/convert/message.rs`,
  `rpc/service/src/service.rs`, `bridge/src/prom.rs`, `rothschild/src/main.rs`,
  `cli/src/extensions/transaction.rs`, `cli/src/modules/account.rs`,
  `cli/src/modules/estimate.rs`, `cli/src/modules/pskb.rs`,
  `cli/src/modules/rpc.rs`, `cli/src/modules/send.rs`,
  `cli/src/modules/transfer.rs`, `cli/src/utils.rs`.
- Wallet and integration coverage: `wallet/core/src/account/mod.rs`,
  `wallet/core/src/account/pskb.rs`, `wallet/core/src/api/message.rs`,
  `wallet/core/src/prelude.rs`, `wallet/core/src/storage/transaction/record.rs`,
  `wallet/core/src/tx/fees.rs`, `wallet/core/src/tx/generator/generator.rs`,
  `wallet/core/src/tx/generator/summary.rs`, `wallet/core/src/tx/generator/test.rs`,
  `wallet/core/src/tx/mass.rs`, `wallet/core/src/tx/payment.rs`,
  `wallet/core/src/utils.rs`, `wallet/core/src/utxo/balance.rs`,
  `wallet/core/src/utxo/context.rs`, `wallet/core/src/utxo/test.rs`,
  `wallet/core/src/wallet/api.rs`, `wallet/core/src/wasm/api/message.rs`,
  `wallet/core/src/wasm/tx/generator/generator.rs`,
  `wallet/core/src/wasm/utils.rs`, `wallet/pskt/src/bundle.rs`,
  `wallet/pskt/src/output.rs`, `wallet/pskt/src/wasm/bundle.rs`,
  `wallet/pskt/src/wasm/utils.rs`, `testing/integration/src/common/args.rs`,
  `testing/integration/src/common/fee.rs`,
  `testing/integration/src/consensus_integration_tests.rs`,
  `testing/integration/src/mempool_benchmarks.rs`,
  `testing/integration/src/rpc_perf_benchmarks.rs`,
  `testing/integration/src/rpc_tests.rs`.

The intended SLUG scale is also 100,000,000 san per SLG, but matching scale
alone does not make a rename safe. The approved maximum of 28,700,000 SLG
corresponds to 2,870,000,000,000,000 san. Neither changing `MAX_SOMPI` nor
renaming unit symbols would implement or prove that hard cap. No monetary
consensus logic was changed.

## Genesis and covenant audit

`consensus/core/src/config/genesis.rs` statically supplies each existing
network's genesis hash, merkle root, UTXO commitment, timestamp, difficulty
bits, nonce, DAA score, and coinbase payload. The genesis coinbase transaction
has no outputs in `build_genesis_transactions`; the existing genesis payload
contains a subsidy field, but it is not a SLUG genesis and its configured
hash, root, commitment, and parameters must not be reused as SLUG values.
`test_genesis_hashes` validates the generated block hash and transaction
merkle root for the four configured Kaspa networks. It does not calculate or
approve a SLUG genesis, supply schedule, initial target, or UTXO commitment.
Consumers include `consensus/core/src/config/params.rs`, consensus state
initialization (`consensus/src/model/stores/virtual_state.rs`), pruning and
parents processing, and IBD/pruning-proof validation in `protocol/flows/`.

The covenant ID function is
`consensus/core/src/hashing/covenant_id.rs::covenant_id`. It hashes the
authorizing input's previous outpoint transaction ID and index, the count of
authorized outputs, and for each output its index, value, script-public-key
version, and script bytes. It uses the domain-separated
`CovenantID` hasher declared in `crypto/hashes/src/hashers.rs`. The genesis
covenant grouping and binding process is
`consensus/core/src/tx.rs::Transaction::populate_genesis_covenants`; tests in
that file cover grouping/validation. This mechanism and its serialized
representation were left unchanged; no final SLUG covenant IDs were invented.

## Unresolved decisions

1. Approve SLUG's full emission schedule, exact cap enforcement rules, reward
   transitions, and tests proving total issuance never exceeds the approved
   supply.
2. Decide how `slug-testnet-1` maps to the current suffix-based ID model, which
   currently only has matching parameters for testnet suffix 10.
3. Approve unique P2P, gRPC, wRPC Borsh, and wRPC JSON ports, the P2P network
   name, and any protocol-version/magic requirements.
4. Provision SLUG-owned DNS seeders and resolver endpoints; until then all
   SLUG seed lists/resolver defaults must be empty or explicitly supplied by
   the operator.
5. Approve address prefixes and the compatibility/migration behavior for
   existing address parsers and clients.
6. Decide crate, binary, CLI, environment-variable, wallet, JS/WASM, and
   release naming and whether downstream APIs require compatibility aliases.
7. Approve separate app-data, database, and log locations and whether any
   existing wallet/database import is supported. Do not reuse or auto-migrate
   Kaspa chain state.
8. Specify a no-premine SLUG genesis only after chain parameters are approved;
   no hash, commitment, timestamp, nonce, or target is inferred here.
9. Review covenant compatibility/security and the genesis-covenant policy
   before creating SLUG genesis data.
10. Decide RPC namespace and protobuf/Go package compatibility, resolver
    behavior, explorer links, and external service ownership.
11. Confirm the project's upstream attribution/license and package publishing
    strategy. Existing attribution is preserved in this stage.

## Recommended implementation order

1. Approve the unresolved network, monetary, genesis, API, and ownership
   decisions above.
2. Implement chain/network isolation first: unique network identity, peer
   handshake enforcement, no Kaspa seeders/resolvers, ports, and distinct data
   paths; test that wrong-network peers and state cannot be reused.
3. Design the SLUG emission schedule and enforce/test the complete lifetime
   supply bound in consensus.
4. Generate a no-premine genesis from approved parameters and test every
   derived hash, merkle root, UTXO commitment, target, and initial state.
5. Review and test covenant behavior against the genesis and transaction
   design.
6. Migrate address prefixes and APIs with end-to-end encode/decode vectors;
   migrate RPC/protobuf/wRPC namespaces with compatibility tests.
7. Rename packages, binaries, CLI/environment names, wallets, docs, scripts,
   workflows, and release artifacts as a coordinated packaging change.
8. Run integration tests and an independent consensus/network review before
   declaring any network usable.

## Stage 1 validation record

Commands and observed results are recorded in
`docs/SLUG-STAGE1-CHANGELOG.md`. No consensus or runtime identity tests were
run because Stage 1 made no runtime protocol or consensus changes.
