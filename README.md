# SLUG

SLUG is an independent blockchain project being developed from this Kaspa-derived
Rust node and wallet codebase.

## Project identity

- Native currency: SLUG (ticker: SLG)
- Smallest denomination: san (100,000,000 san = 1 SLG)
- Maximum supply: 28,700,000 SLG (2,870,000,000,000,000 san)
- Target block interval: 100 ms (10 blocks per second)
- Network names: `slug-mainnet`, `slug-testnet-1`, `slug-devnet`, and
  `slug-simnet`
- Genesis has no premine or spendable allocation.

The source implements SLUG consensus parameters, genesis, issuance cap,
addresses, and isolated network identity. Monetary amounts are represented in
san and wallet/CLI displays use SLG. Mainnet launch readiness still depends on
real-world hash-rate/security review and production network deployment checks.

See [the current project report](laporan_terkini.md) for verified behavior,
tests, and outstanding launch checks. The Stage 1 audit and migration matrix
are historical records.
