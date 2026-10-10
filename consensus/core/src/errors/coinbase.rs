use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CoinbaseError {
    #[error("coinbase payload length is {0} while the minimum allowed length is {1}")]
    PayloadLenBelowMin(usize, usize),

    #[error("coinbase payload length is {0} while the maximum allowed length is {1}")]
    PayloadLenAboveMax(usize, usize),

    #[error("coinbase payload script public key length is {0} while the maximum allowed length is {1}")]
    PayloadScriptPublicKeyLenAboveMax(usize, u8),

    #[error(
        "coinbase payload length is {0} bytes but it needs to be at least {1} bytes long in order to accommodate the script public key"
    )]
    PayloadCantContainScriptPublicKey(usize, usize),

    #[error("coinbase subsidy accounting arithmetic overflow")]
    SubsidyAccountingOverflow,

    #[error("coinbase mergeset subsidies exceed the remaining SLUG supply")]
    MergesetSubsidiesExceedSupplyCap,

    #[error("coinbase cumulative subsidy {0} exceeds the SLUG supply cap")]
    CumulativeSubsidyAboveSupplyCap(u64),

    #[error("coinbase cumulative subsidy {0} is less than its block subsidy {1}")]
    CumulativeSubsidyBelowBlockSubsidy(u64, u64),
}

pub type CoinbaseResult<T> = std::result::Result<T, CoinbaseError>;
