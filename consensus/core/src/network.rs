//!
//! # Network Types
//!
//! This module implements [`NetworkType`] (such as `mainnet`, `testnet`, `devnet`, and `simnet`)
//! and [`NetworkId`] that combines a network type with an optional numerical suffix.
//!
//! The suffix is used to differentiate between multiple networks of the same type and is used
//! explicitly with `testnet` networks, allowing declaration of testnet versions such as
//! `testnet-1`.
//!

#![allow(non_snake_case)]

use borsh::{BorshDeserialize, BorshSerialize};
use kaspa_addresses::Prefix;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use wasm_bindgen::convert::TryFromJsValue;
use wasm_bindgen::prelude::*;
use workflow_wasm::prelude::*;

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone)]
pub enum NetworkTypeError {
    #[error("Invalid network type: {0}")]
    InvalidNetworkType(String),
}

/// @category Consensus
#[derive(Clone, Copy, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
#[wasm_bindgen]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Devnet,
    Simnet,
}

impl NetworkType {
    pub fn default_rpc_port(&self) -> u16 {
        match self {
            NetworkType::Mainnet => 19110,
            NetworkType::Testnet => 19210,
            NetworkType::Devnet => 19310,
            NetworkType::Simnet => 19410,
        }
    }

    pub fn default_borsh_rpc_port(&self) -> u16 {
        match self {
            NetworkType::Mainnet => 19112,
            NetworkType::Testnet => 19212,
            NetworkType::Devnet => 19312,
            NetworkType::Simnet => 19412,
        }
    }

    pub fn default_json_rpc_port(&self) -> u16 {
        match self {
            NetworkType::Mainnet => 19113,
            NetworkType::Testnet => 19213,
            NetworkType::Devnet => 19313,
            NetworkType::Simnet => 19413,
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        static NETWORK_TYPES: [NetworkType; 4] =
            [NetworkType::Mainnet, NetworkType::Testnet, NetworkType::Devnet, NetworkType::Simnet];
        NETWORK_TYPES.iter().copied()
    }
}

impl TryFrom<Prefix> for NetworkType {
    type Error = NetworkTypeError;
    fn try_from(prefix: Prefix) -> Result<Self, Self::Error> {
        match prefix {
            Prefix::Mainnet => Ok(NetworkType::Mainnet),
            Prefix::Testnet => Ok(NetworkType::Testnet),
            Prefix::Simnet => Ok(NetworkType::Simnet),
            Prefix::Devnet => Ok(NetworkType::Devnet),
            #[allow(unreachable_patterns)]
            #[cfg(test)]
            _ => Err(NetworkTypeError::InvalidNetworkType(prefix.to_string())),
        }
    }
}

impl From<NetworkType> for Prefix {
    fn from(network_type: NetworkType) -> Self {
        match network_type {
            NetworkType::Mainnet => Prefix::Mainnet,
            NetworkType::Testnet => Prefix::Testnet,
            NetworkType::Devnet => Prefix::Devnet,
            NetworkType::Simnet => Prefix::Simnet,
        }
    }
}

impl FromStr for NetworkType {
    type Err = NetworkTypeError;
    fn from_str(network_type: &str) -> Result<Self, Self::Err> {
        match network_type.to_lowercase().as_str() {
            "mainnet" => Ok(NetworkType::Mainnet),
            "testnet" => Ok(NetworkType::Testnet),
            "simnet" => Ok(NetworkType::Simnet),
            "devnet" => Ok(NetworkType::Devnet),
            _ => Err(NetworkTypeError::InvalidNetworkType(network_type.to_string())),
        }
    }
}

impl Display for NetworkType {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NetworkType::Mainnet => "mainnet",
            NetworkType::Testnet => "testnet",
            NetworkType::Simnet => "simnet",
            NetworkType::Devnet => "devnet",
        };
        f.write_str(s)
    }
}

impl TryFrom<&NetworkTypeT> for NetworkType {
    type Error = NetworkTypeError;
    fn try_from(value: &NetworkTypeT) -> Result<Self, Self::Error> {
        if let Ok(network_id) = NetworkId::try_cast_from(value) {
            Ok(network_id.network_type())
        } else if let Some(network_type) = value.as_string() {
            Self::from_str(&network_type)
        } else if let Ok(network_type) = NetworkType::try_from_js_value(JsValue::from(value)) {
            Ok(network_type)
        } else {
            Err(NetworkTypeError::InvalidNetworkType(format!("{value:?}")))
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "Network", typescript_type = "NetworkType | NetworkId | string")]
    #[derive(Debug)]
    pub type NetworkTypeT;
}

impl TryFrom<&NetworkTypeT> for Prefix {
    type Error = NetworkIdError;
    fn try_from(value: &NetworkTypeT) -> Result<Self, Self::Error> {
        Ok(NetworkType::try_from(value)?.into())
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum NetworkIdError {
    #[error("Invalid network name prefix: {0}. The expected prefix is 'slug'.")]
    InvalidPrefix(String),

    #[error(transparent)]
    InvalidNetworkType(#[from] NetworkTypeError),

    #[error("Invalid network suffix: {0}. Only 32 bits unsigned integer (u32) are supported.")]
    InvalidSuffix(String),

    #[error("Unexpected extra token: {0}.")]
    UnexpectedExtraToken(String),

    #[error("Missing network suffix: '{0}'")]
    MissingNetworkSuffix(String),

    #[error("Network suffix required for network type: '{0}'")]
    NetworkSuffixRequired(String),

    #[error("Invalid network id: '{0}'")]
    InvalidNetworkId(String),

    #[error(transparent)]
    Wasm(#[from] workflow_wasm::error::Error),
}

impl From<NetworkIdError> for JsValue {
    fn from(err: NetworkIdError) -> Self {
        JsValue::from_str(&err.to_string())
    }
}

///
/// NetworkId is a unique identifier for a SLUG network instance.
/// It is composed of a network type and an optional suffix.
///
/// @category Consensus
///
#[derive(Clone, Copy, Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq, Hash, Ord, PartialOrd, CastFromJs)]
#[wasm_bindgen(inspectable)]
pub struct NetworkId {
    #[wasm_bindgen(js_name = "type")]
    pub network_type: NetworkType,
    #[wasm_bindgen(js_name = "suffix")]
    pub suffix: Option<u32>,
}

impl NetworkId {
    pub const fn new(network_type: NetworkType) -> Self {
        if !matches!(network_type, NetworkType::Mainnet | NetworkType::Devnet | NetworkType::Simnet) {
            panic!("network suffix required for this network type");
        }

        Self { network_type, suffix: None }
    }

    pub fn try_new(network_type: NetworkType) -> Result<Self, NetworkIdError> {
        if !matches!(network_type, NetworkType::Mainnet | NetworkType::Devnet | NetworkType::Simnet) {
            return Err(NetworkIdError::NetworkSuffixRequired(network_type.to_string()));
        }

        Ok(Self { network_type, suffix: None })
    }

    pub const fn with_suffix(network_type: NetworkType, suffix: u32) -> Self {
        Self { network_type, suffix: Some(suffix) }
    }

    pub fn network_type(&self) -> NetworkType {
        self.network_type
    }

    pub fn is_mainnet(&self) -> bool {
        self.network_type == NetworkType::Mainnet
    }

    pub fn suffix(&self) -> Option<u32> {
        self.suffix
    }

    pub fn default_p2p_port(&self) -> u16 {
        match self.network_type {
            NetworkType::Mainnet => 19111,
            NetworkType::Testnet => 19211,
            NetworkType::Devnet => 19311,
            NetworkType::Simnet => 19411,
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        static NETWORK_IDS: [NetworkId; 4] = [
            NetworkId::new(NetworkType::Mainnet),
            NetworkId::with_suffix(NetworkType::Testnet, 1),
            NetworkId::new(NetworkType::Devnet),
            NetworkId::new(NetworkType::Simnet),
        ];
        NETWORK_IDS.iter().copied()
    }

    /// Returns a textual description of the network prefixed with `slug-`
    pub fn to_prefixed(&self) -> String {
        format!("slug-{}", self)
    }

    pub fn from_prefixed(prefixed: &str) -> Result<Self, NetworkIdError> {
        if let Some(stripped) = prefixed.strip_prefix("slug-") {
            Self::from_str(stripped)
        } else {
            Err(NetworkIdError::InvalidPrefix(prefixed.to_string()))
        }
    }
}

impl Deref for NetworkId {
    type Target = NetworkType;

    fn deref(&self) -> &Self::Target {
        &self.network_type
    }
}

impl TryFrom<NetworkType> for NetworkId {
    type Error = NetworkIdError;
    fn try_from(value: NetworkType) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<NetworkId> for Prefix {
    fn from(net: NetworkId) -> Self {
        (*net).into()
    }
}

impl From<NetworkId> for NetworkType {
    fn from(net: NetworkId) -> Self {
        *net
    }
}

impl FromStr for NetworkId {
    type Err = NetworkIdError;
    fn from_str(network_name: &str) -> Result<Self, Self::Err> {
        let network_name = network_name.strip_prefix("slug-").unwrap_or(network_name);
        let mut parts = network_name.split('-').fuse();
        let network_type = NetworkType::from_str(parts.next().unwrap_or_default())?;
        let suffix = parts.next().map(|x| u32::from_str(x).map_err(|_| NetworkIdError::InvalidSuffix(x.to_string()))).transpose()?;
        // Disallow testnet network without suffix.
        // Lack of suffix makes it impossible to distinguish between
        // multiple testnet networks
        if !matches!(network_type, NetworkType::Mainnet | NetworkType::Devnet | NetworkType::Simnet) && suffix.is_none() {
            return Err(NetworkIdError::MissingNetworkSuffix(network_name.to_string()));
        }
        match parts.next() {
            Some(extra_token) => Err(NetworkIdError::UnexpectedExtraToken(extra_token.to_string())),
            None => Ok(Self { network_type, suffix }),
        }
    }
}

impl Display for NetworkId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(suffix) = self.suffix { write!(f, "{}-{}", self.network_type, suffix) } else { write!(f, "{}", self.network_type) }
    }
}

impl Serialize for NetworkId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct NetworkIdVisitor;

impl de::Visitor<'_> for NetworkIdVisitor {
    type Value = NetworkId;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string containing network_type and optional suffix separated by a '-'")
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        NetworkId::from_str(value).map_err(|err| de::Error::custom(err.to_string()))
    }
}

impl<'de> Deserialize<'de> for NetworkId {
    fn deserialize<D>(deserializer: D) -> Result<NetworkId, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(NetworkIdVisitor)
    }
}

#[wasm_bindgen]
impl NetworkId {
    #[wasm_bindgen(constructor)]
    pub fn ctor(value: &JsValue) -> Result<NetworkId, NetworkIdError> {
        Ok(NetworkId::try_cast_from(value)?.into_owned())
    }

    #[wasm_bindgen(getter, js_name = "id")]
    pub fn js_id(&self) -> String {
        self.to_string()
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn js_to_string(&self) -> String {
        self.to_string()
    }

    #[wasm_bindgen(js_name = "addressPrefix")]
    pub fn js_address_prefix(&self) -> String {
        Prefix::from(self.network_type).to_string()
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "NetworkId | string")]
    pub type NetworkIdT;
}

impl TryFrom<&JsValue> for NetworkId {
    type Error = NetworkIdError;
    fn try_from(value: &JsValue) -> Result<Self, Self::Error> {
        Self::try_owned_from(value)
    }
}

impl TryFrom<JsValue> for NetworkId {
    type Error = NetworkIdError;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        Self::try_owned_from(value)
    }
}

impl TryCastFromJs for NetworkId {
    type Error = NetworkIdError;
    fn try_cast_from<'a, R>(value: &'a R) -> Result<Cast<'a, Self>, Self::Error>
    where
        R: AsRef<JsValue> + 'a,
    {
        Self::resolve(value, || {
            if let Some(network_id) = value.as_ref().as_string() {
                Ok(NetworkId::from_str(&network_id)?)
            } else {
                Err(NetworkIdError::InvalidNetworkId(format!("{:?}", value.as_ref())))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_id_parse_roundtrip() {
        for nt in NetworkType::iter() {
            if matches!(nt, NetworkType::Mainnet | NetworkType::Devnet | NetworkType::Simnet) {
                let ni = NetworkId::try_from(nt).expect("failed to create network id");
                assert_eq!(nt, *NetworkId::from_str(ni.to_string().as_str()).unwrap());
                assert_eq!(ni, NetworkId::from_str(ni.to_string().as_str()).unwrap());
            }
            let nis = NetworkId::with_suffix(nt, 1);
            assert_eq!(nt, *NetworkId::from_str(nis.to_string().as_str()).unwrap());
            assert_eq!(nis, NetworkId::from_str(nis.to_string().as_str()).unwrap());

            assert_eq!(nis, NetworkId::from_str(nis.to_string().as_str()).unwrap());
        }
    }

    #[test]
    fn test_network_id_prefixed_roundtrip() {
        let networks = [
            (NetworkId::new(NetworkType::Mainnet), "slug-mainnet"),
            (NetworkId::with_suffix(NetworkType::Testnet, 1), "slug-testnet-1"),
            (NetworkId::new(NetworkType::Devnet), "slug-devnet"),
            (NetworkId::new(NetworkType::Simnet), "slug-simnet"),
        ];

        for (network_id, expected) in networks {
            assert_eq!(network_id.to_prefixed(), expected);
            assert_eq!(NetworkId::from_prefixed(expected).unwrap(), network_id);
        }

        assert!(NetworkId::from_prefixed("kaspa-mainnet").is_err());
    }

    #[test]
    fn slug_networks_use_dedicated_service_ports() {
        let networks = [
            (NetworkId::new(NetworkType::Mainnet), 19111, 19110, 19112, 19113),
            (NetworkId::with_suffix(NetworkType::Testnet, 1), 19211, 19210, 19212, 19213),
            (NetworkId::new(NetworkType::Devnet), 19311, 19310, 19312, 19313),
            (NetworkId::new(NetworkType::Simnet), 19411, 19410, 19412, 19413),
        ];

        for (network, p2p, rpc, borsh, json) in networks {
            assert_eq!(network.default_p2p_port(), p2p);
            assert_eq!(network.network_type().default_rpc_port(), rpc);
            assert_eq!(network.network_type().default_borsh_rpc_port(), borsh);
            assert_eq!(network.network_type().default_json_rpc_port(), json);
        }
    }

    #[test]
    fn test_network_id_parse() {
        struct Test {
            name: &'static str,
            expr: &'static str,
            expected: Result<NetworkId, NetworkIdError>,
        }

        let tests = vec![
            Test { name: "Valid mainnet", expr: "mainnet", expected: Ok(NetworkId::new(NetworkType::Mainnet)) },
            Test { name: "Valid prefixed mainnet", expr: "slug-mainnet", expected: Ok(NetworkId::new(NetworkType::Mainnet)) },
            Test {
                name: "Valid prefixed testnet",
                expr: "slug-testnet-1",
                expected: Ok(NetworkId::with_suffix(NetworkType::Testnet, 1)),
            },
            Test { name: "Valid testnet", expr: "testnet-88", expected: Ok(NetworkId::with_suffix(NetworkType::Testnet, 88)) },
            Test { name: "Missing network", expr: "", expected: Err(NetworkTypeError::InvalidNetworkType("".to_string()).into()) },
            Test {
                name: "Invalid network",
                expr: "gamenet",
                expected: Err(NetworkTypeError::InvalidNetworkType("gamenet".to_string()).into()),
            },
            Test { name: "Invalid suffix", expr: "testnet-x", expected: Err(NetworkIdError::InvalidSuffix("x".to_string())) },
            Test {
                name: "Unexpected extra token",
                expr: "testnet-10-x",
                expected: Err(NetworkIdError::UnexpectedExtraToken("x".to_string())),
            },
        ];

        for test in tests {
            let Test { name, expr, expected } = test;
            match NetworkId::from_str(expr) {
                Ok(nid) => assert_eq!(nid, expected.unwrap(), "{}: unexpected result", name),
                Err(err) => assert_eq!(err.to_string(), expected.unwrap_err().to_string(), "{}: unexpected error", name),
            }
        }
    }
}
