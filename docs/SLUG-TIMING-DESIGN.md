# SLUG Timing Design

## Fixed requirement

- All SLUG networks target 10 blocks per second (100 ms per block).
- Preserve Kaspa's consensus, GHOSTDAG, and adaptive difficulty mechanisms. Change only the parameters and conversions needed for the new interval.

## Verified source facts

- `Bps<const BPS: u64>` in `consensus/core/src/config/bps.rs` represents integer blocks per second. `target_time_per_block()` calculates `1000 / BPS` milliseconds and requires an exact division.
- `BlockrateParams::new` derives GHOSTDAG K, parent and mergeset limits, DAA and median-time sampling, merge/finality/pruning depths, and coinbase maturity from BPS.
- Mainnet, testnet, devnet, and simnet use `BlockrateParams::new::<10>()`, giving a target interval of 100 ms and GHOSTDAG K=124. Simnet intentionally allows at least 64 direct parents for benchmarks.
- `bps_history()` also uses 10 BPS from genesis; it no longer applies inherited Kaspa activation scores or a 1 BPS pre-activation period to SLUG.
- `BlockrateParams::new` derives the DAA, timestamp sampling, merge/finality/pruning depths, and coinbase maturity from the same 10 BPS rate.

## Validation

- The consensus parameters expose 100 ms / 10 BPS for every SLUG network.
- The shared `BlockrateParams` derivation is retained, so coupled values follow the existing tested 10 BPS parameter set rather than individually edited constants.
- Mainnet starts from genesis bits `0x1f0694e4`, the compact encoding of `0x694e4 << 224`. Assuming uniformly distributed PoW hashes, this yields an expected 100 ms block interval at 100 kH/s. The inherited maximum permitted target and original Rusty Kaspa DAA are retained so difficulty adjusts normally after launch. This is an initial calibration, not a guarantee of security or observed throughput. Testnet and devnet retain the inherited maximum target for usability.
- Genesis itself is accepted through the upstream genesis-specific PoW bypass. Its bits seed the initial difficulty window; the mainnet genesis nonce is not claimed to satisfy a standalone PoW check.

See [the decision log](./SLUG-DECISION-LOG.md) for remaining launch checks, including operational hashrate and peer deployment validation.
