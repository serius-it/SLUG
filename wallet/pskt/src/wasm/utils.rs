use kaspa_consensus_core::constants::*;
use kaspa_consensus_core::network::NetworkType;
use separator::Separatable;

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
