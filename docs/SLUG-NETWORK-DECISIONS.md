# SLUG Network Decisions

## Fixed requirements

- Network IDs: `slug-mainnet`, `slug-testnet-1`, `slug-devnet`, and `slug-simnet`.
- SLUG must fail closed across network mismatches and must not automatically use Kaspa seeders, resolvers, bootstrap endpoints, or network configuration.
- Separate default application data, logs, database, configuration, and wallet namespaces from Kaspa.
- Preserve address and network protocol isolation across all SLUG networks.

## Initial SLUG values

These values are initial SLUG protocol choices, not Kaspa compatibility aliases:

| Network | Address prefix | P2P | gRPC | wRPC Borsh | wRPC JSON |
|---|---|---:|---:|---:|---:|
| `slug-mainnet` | `slug` | 19111 | 19110 | 19112 | 19113 |
| `slug-testnet-1` | `slugtest` | 19211 | 19210 | 19212 | 19213 |
| `slug-devnet` | `slugdev` | 19311 | 19310 | 19312 | 19313 |
| `slug-simnet` | `slugsim` | 19411 | 19410 | 19412 | 19413 |

Each genesis block is tagged with its full SLUG network ID in its coinbase payload, has a zero subsidy, no transaction outputs, and an empty UTXO commitment. The initial timestamp is `2026-10-09T00:00:00Z`. Testnet, devnet, and simnet use compact target bits `0x207fffff`; mainnet starts with `0x1f0694e4`, calibrated for an expected 100 ms block at 100 kH/s. The inherited maximum permitted target and Rusty Kaspa DAA are retained so difficulty can adjust normally after launch. Per-network nonces and hashes are in `consensus/core/src/config/genesis.rs`. Mainnet genesis uses the inherited genesis PoW bypass; public launch still requires real hash-rate and security review.

## Verified source facts

- `NetworkId` in `consensus/core/src/network.rs` formats network names as `slug-*`; its internal `NetworkType` model supports mainnet, testnet with a suffix, devnet, and simnet.
- All consensus parameter sets have empty DNS seeder lists.
- The node stores logs and database state under the SLUG application directory and prefixed network name.
- `crypto/addresses/src/lib.rs` defines SLUG-only address prefixes; old Kaspa prefixes are rejected.
- The wRPC client requires explicit resolver configuration; it does not bundle Kaspa public resolver endpoints.
- Resolver default lookup fails explicitly when no trusted SLUG resolver is configured. CLI connection likewise rejects the old `public` setting unless an explicit endpoint is supplied.
- SLUG uses a separate default port block and does not reuse the inherited Kaspa ports. Peer handshake identity is the `slug-*` network name.

## Decisions still required

- Confirm external firewall and deployment availability for the initial port block before public rollout; local source inspection cannot establish production firewall availability.
- Define whether SLUG will operate DNS seeders; if so, identify SLUG-owned domains. Until then, nodes must not use automatic seeders or resolver fallback.
- Use distinct SLUG-tagged genesis blocks, with no spendable genesis allocation or inherited Kaspa genesis hash.
- Determine wallet namespace and any import/migration behavior without reading Kaspa wallet state by default.
- Review binary/package names and remaining application metadata for accidental Kaspa network selection.

## Implementation boundary

Use the initial SLUG address prefixes and port block above; remove inherited automatic Kaspa discovery and isolate the default data root. Genesis must have distinct SLUG-tagged values and no spendable allocation. Emission and timing remain gated on a separate design and validation. See [the decision log](./SLUG-DECISION-LOG.md).
