use crate::{
    block::Block,
    header::{CompressedParents, Header},
    subnets::SUBNETWORK_ID_COINBASE,
    tx::Transaction,
};
use kaspa_hashes::{Hash, ZERO_HASH};
use kaspa_muhash::EMPTY_MUHASH;

#[derive(Clone, Debug)]
pub struct GenesisBlock {
    pub hash: Hash,
    pub version: u16,
    pub hash_merkle_root: Hash,
    pub utxo_commitment: Hash,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
    pub daa_score: u64,
    pub coinbase_payload: &'static [u8],
}

impl GenesisBlock {
    pub fn build_genesis_transactions(&self) -> Vec<Transaction> {
        vec![Transaction::new(0, Vec::new(), Vec::new(), 0, SUBNETWORK_ID_COINBASE, 0, self.coinbase_payload.to_vec())]
    }
}

impl From<&GenesisBlock> for Header {
    fn from(genesis: &GenesisBlock) -> Self {
        Header::new_finalized(
            genesis.version,
            CompressedParents::default(),
            genesis.hash_merkle_root,
            ZERO_HASH,
            genesis.utxo_commitment,
            genesis.timestamp,
            genesis.bits,
            genesis.nonce,
            genesis.daa_score,
            0.into(),
            0,
            ZERO_HASH,
        )
    }
}

impl From<&GenesisBlock> for Block {
    fn from(genesis: &GenesisBlock) -> Self {
        Block::new(genesis.into(), genesis.build_genesis_transactions())
    }
}

impl From<(&Header, &'static [u8])> for GenesisBlock {
    fn from((header, payload): (&Header, &'static [u8])) -> Self {
        Self {
            hash: header.hash,
            version: header.version,
            hash_merkle_root: header.hash_merkle_root,
            utxo_commitment: header.utxo_commitment,
            timestamp: header.timestamp,
            bits: header.bits,
            nonce: header.nonce,
            daa_score: header.daa_score,
            coinbase_payload: payload,
        }
    }
}

const MAINNET_COINBASE_PAYLOAD: &[u8] = &[
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, b's', b'l', b'u', b'g', b'-', b'm', b'a',
    b'i', b'n', b'n', b'e', b't',
];
const TESTNET_COINBASE_PAYLOAD: &[u8] = &[
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, b's', b'l', b'u', b'g', b'-', b't', b'e',
    b's', b't', b'n', b'e', b't', b'-', b'1',
];
const DEVNET_COINBASE_PAYLOAD: &[u8] = &[
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, b's', b'l', b'u', b'g', b'-', b'd', b'e',
    b'v', b'n', b'e', b't',
];
const SIMNET_COINBASE_PAYLOAD: &[u8] = &[
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, b's', b'l', b'u', b'g', b'-', b's', b'i',
    b'm', b'n', b'e', b't',
];

pub const GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x1a, 0x41, 0x10, 0xfe, 0x12, 0xe6, 0xab, 0x96, 0x24, 0x82, 0x38, 0x53, 0xad, 0x80, 0x4c, 0x5d, 0x96, 0xb4, 0xda, 0x2a, 0x48,
        0xdd, 0x3d, 0xdf, 0xc6, 0xdf, 0xe4, 0x39, 0x1c, 0x6d, 0x76, 0x82,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x1c, 0x10, 0x29, 0x17, 0xd7, 0xb7, 0x7b, 0x9b, 0xbb, 0x8b, 0x76, 0x03, 0xa1, 0x9d, 0x98, 0x97, 0xda, 0xa0, 0xe3, 0x8e, 0x18,
        0xdc, 0xe9, 0x83, 0xd7, 0x83, 0xa2, 0x8d, 0x5c, 0x9f, 0x20, 0xe0,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 1_791_504_000_000,
    bits: 0x1f0694e4,
    nonce: 1,
    daa_score: 0,
    coinbase_payload: MAINNET_COINBASE_PAYLOAD,
};

pub const TESTNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x76, 0xc6, 0x3c, 0x78, 0x81, 0x53, 0x22, 0x4d, 0xa0, 0x01, 0xf4, 0xc5, 0x1d, 0x30, 0x16, 0x11, 0x5a, 0x97, 0x6e, 0xa9, 0x03,
        0x79, 0xa4, 0x77, 0x54, 0x10, 0xad, 0xfe, 0x80, 0x88, 0xd2, 0xa6,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x92, 0x5f, 0x4e, 0xd9, 0x1c, 0xcb, 0x37, 0x64, 0x44, 0x75, 0x4c, 0x2b, 0xfc, 0x67, 0xe0, 0x90, 0x31, 0x51, 0x3b, 0xbd, 0x72,
        0x64, 0xef, 0x25, 0x81, 0x6b, 0xf5, 0xda, 0x71, 0x6e, 0xad, 0xf8,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 1_791_504_000_000,
    bits: 0x207fffff,
    nonce: 4,
    daa_score: 0,
    coinbase_payload: TESTNET_COINBASE_PAYLOAD,
};

pub const DEVNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0xfc, 0x89, 0x20, 0xdf, 0xc8, 0x47, 0xe1, 0x4f, 0x80, 0x3a, 0x10, 0x22, 0xb2, 0x78, 0x12, 0x37, 0x26, 0xa7, 0x5e, 0x02, 0xef,
        0x2b, 0x56, 0x16, 0x0d, 0x40, 0xae, 0xf4, 0x2c, 0x6c, 0xeb, 0x7d,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x3e, 0x4f, 0x9b, 0xda, 0x2e, 0xc7, 0xe4, 0x17, 0xf1, 0x20, 0x26, 0xbf, 0x8f, 0x84, 0x62, 0xc6, 0x8b, 0x60, 0xc9, 0xbd, 0x14,
        0xa0, 0xeb, 0xb9, 0xac, 0xd6, 0xd5, 0x90, 0x54, 0xb6, 0x33, 0x13,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 1_791_504_000_000,
    bits: 0x207fffff,
    nonce: 0,
    daa_score: 0,
    coinbase_payload: DEVNET_COINBASE_PAYLOAD,
};

pub const SIMNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0xcf, 0x4e, 0xdd, 0xea, 0x31, 0x2e, 0x76, 0x7b, 0x57, 0xd8, 0x9e, 0xe5, 0x6a, 0x13, 0x3b, 0x35, 0x0e, 0x54, 0x9e, 0xef, 0x78,
        0x0e, 0xb7, 0x89, 0xb0, 0x5e, 0xc5, 0xeb, 0x41, 0x42, 0x94, 0x39,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0xca, 0xb7, 0xe0, 0x8b, 0x2b, 0xce, 0x7e, 0x09, 0x10, 0x99, 0x0d, 0x0a, 0x30, 0x19, 0x9b, 0x66, 0xe0, 0xd8, 0x5e, 0xd3, 0xf0,
        0xcb, 0xba, 0x50, 0xb5, 0xc5, 0x24, 0x3f, 0x94, 0x23, 0xc6, 0xfe,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 1_791_504_000_000,
    bits: 0x207fffff,
    nonce: 0,
    daa_score: 0,
    coinbase_payload: SIMNET_COINBASE_PAYLOAD,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::calc_hash_merkle_root;

    #[test]
    fn slug_genesis_blocks_are_unique_and_valid() {
        let mut hashes = Vec::new();
        for (network, genesis) in [
            ("slug-mainnet", GENESIS),
            ("slug-testnet-1", TESTNET_GENESIS),
            ("slug-devnet", DEVNET_GENESIS),
            ("slug-simnet", SIMNET_GENESIS),
        ] {
            let transactions = genesis.build_genesis_transactions();
            let block: Block = (&genesis).into();
            assert_eq!(calc_hash_merkle_root(transactions.iter()), genesis.hash_merkle_root, "{network} Merkle root");
            assert_eq!(block.hash(), genesis.hash, "{network} block hash");
            assert_eq!(genesis.utxo_commitment, EMPTY_MUHASH, "{network} UTXO commitment");
            assert!(transactions.iter().all(|tx| tx.outputs.is_empty()), "{network} genesis must not allocate spendable outputs");
            assert!(genesis.coinbase_payload[..24].iter().all(|byte| *byte == 0), "{network} genesis subsidy must be zero");
            assert!(genesis.coinbase_payload.ends_with(network.as_bytes()), "{network} payload tag");
            assert!(!hashes.contains(&genesis.hash), "{network} genesis hash must be unique");
            hashes.push(genesis.hash);
        }
    }
}
