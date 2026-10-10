use crate::error::Error;
use crate::result::Result;
use std::fmt::Display;

pub fn try_parse_required_nonzero_slg_as_san_u64<S: ToString + Display>(slg_amount: Option<S>) -> Result<u64> {
    if let Some(slg_amount) = slg_amount {
        let san_amount = kaspa_wallet_core::utils::try_slg_str_to_san(slg_amount.to_string())?
            .ok_or_else(|| Error::custom("SLG amount is required"))?;
        if san_amount == 0 {
            Err(Error::custom(format!("Required SLG amount must not be zero: '{slg_amount}'")))
        } else {
            Ok(san_amount)
        }
    } else {
        Err(Error::custom("Missing SLG amount"))
    }
}

pub fn try_parse_required_slg_as_san_u64<S: ToString + Display>(slg_amount: Option<S>) -> Result<u64> {
    if let Some(slg_amount) = slg_amount {
        kaspa_wallet_core::utils::try_slg_str_to_san(slg_amount.to_string())?.ok_or_else(|| Error::custom("SLG amount is required"))
    } else {
        Err(Error::custom("Missing SLG amount"))
    }
}

pub fn try_parse_optional_slg_as_san_i64<S: ToString + Display>(slg_amount: Option<S>) -> Result<Option<i64>> {
    if let Some(slg_amount) = slg_amount {
        Ok(kaspa_wallet_core::utils::try_slg_str_to_san_i64(slg_amount.to_string())?)
    } else {
        Ok(None)
    }
}
