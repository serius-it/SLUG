//!
//! SLG value formatting and parsing utilities.
//!

use crate::error::Error;
use crate::result::Result;
use kaspa_addresses::Address;
use kaspa_consensus_core::constants::*;
use kaspa_consensus_core::network::NetworkType;
use separator::Separatable;
use workflow_log::style;

pub fn try_slg_str_to_san<S: Into<String>>(s: S) -> Result<Option<u64>> {
    let s: String = s.into();
    let amount = s.trim();
    if amount.is_empty() {
        return Ok(None);
    }

    Ok(Some(str_to_san(amount)?))
}

pub fn try_slg_str_to_san_i64<S: Into<String>>(s: S) -> Result<Option<i64>> {
    let s: String = s.into();
    let amount = s.trim();
    if amount.is_empty() {
        return Ok(None);
    }

    Ok(Some(str_to_san_i64(amount)?))
}

#[inline]
pub fn san_to_slg(san: u64) -> f64 {
    san as f64 / SAN_PER_SLG as f64
}

#[inline]
pub fn slg_to_san(slg: f64) -> u64 {
    (slg * SAN_PER_SLG as f64) as u64
}

#[inline]
pub fn san_to_slg_string(san: u64) -> String {
    let whole = (san / SAN_PER_SLG).separated_string();
    let fraction = format!("{:08}", san % SAN_PER_SLG);
    let fraction = fraction.trim_end_matches('0');
    if fraction.is_empty() { whole } else { format!("{whole}.{fraction}") }
}

#[inline]
pub fn san_to_slg_string_with_trailing_zeroes(san: u64) -> String {
    format!("{}.{:08}", (san / SAN_PER_SLG).separated_string(), san % SAN_PER_SLG)
}

pub fn currency_suffix(network_type: &NetworkType) -> &'static str {
    match network_type {
        NetworkType::Mainnet | NetworkType::Testnet | NetworkType::Simnet | NetworkType::Devnet => "SLG",
    }
}

#[inline]
pub fn san_to_slg_string_with_suffix(san: u64, network_type: &NetworkType) -> String {
    let amount = san_to_slg_string(san);
    let suffix = currency_suffix(network_type);
    format!("{amount} {suffix}")
}

#[inline]
pub fn san_to_slg_string_with_trailing_zeroes_and_suffix(san: u64, network_type: &NetworkType) -> String {
    let amount = san_to_slg_string_with_trailing_zeroes(san);
    let suffix = currency_suffix(network_type);
    format!("{amount} {suffix}")
}

pub fn format_address_colors(address: &Address, range: Option<usize>) -> String {
    let address = address.to_string();

    let parts = address.split(':').collect::<Vec<&str>>();
    let prefix = style(parts[0]).dim();
    let payload = parts[1];
    let range = range.unwrap_or(6);
    let start = range;
    let finish = payload.len() - range;

    let left = &payload[0..start];
    let center = style(&payload[start..finish]).dim();
    let right = &payload[finish..];

    format!("{prefix}:{left}:{center}:{right}")
}

fn str_to_san(amount: &str) -> Result<u64> {
    let (integer, fraction) = amount.split_once('.').unwrap_or((amount, ""));
    if fraction.contains('.') || fraction.len() > 8 || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::custom("SLG amounts must have at most eight decimal places"));
    }

    let integer = integer.parse::<u64>()?;
    let integer = integer.checked_mul(SAN_PER_SLG).ok_or_else(|| Error::custom("SLG amount exceeds the supported range"))?;
    let fraction = if fraction.is_empty() { 0 } else { fraction.parse::<u64>()? * 10u64.pow(8 - fraction.len() as u32) };
    integer.checked_add(fraction).ok_or_else(|| Error::custom("SLG amount exceeds the supported range"))
}

fn str_to_san_i64(amount: &str) -> Result<i64> {
    let (negative, magnitude) = match amount.strip_prefix('-') {
        Some(magnitude) => (true, magnitude),
        None => (false, amount.strip_prefix('+').unwrap_or(amount)),
    };
    let magnitude = str_to_san(magnitude)? as i128;
    let signed = if negative { -magnitude } else { magnitude };
    i64::try_from(signed).map_err(|_| Error::custom("SLG fee exceeds the supported range"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slg_amount_parsing_preserves_single_san_precision() {
        assert_eq!(try_slg_str_to_san("0.00000001").unwrap(), Some(1));
        assert_eq!(try_slg_str_to_san("1.00000001").unwrap(), Some(SAN_PER_SLG + 1));
        assert_eq!(try_slg_str_to_san_i64("-0.00000001").unwrap(), Some(-1));
        assert!(try_slg_str_to_san("0.000000001").is_err());
    }

    #[test]
    fn san_formatting_preserves_amount_at_supply_bound() {
        assert_eq!(san_to_slg_string(1), "0.00000001");
        assert_eq!(san_to_slg_string_with_trailing_zeroes(1), "0.00000001");
        assert_eq!(san_to_slg_string(MAX_SLUG_SUPPLY_SAN), "28,700,000");
    }
}
