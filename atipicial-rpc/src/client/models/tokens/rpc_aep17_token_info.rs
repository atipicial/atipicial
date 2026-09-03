use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

/// AEP17 token information matching C# `RpcAep17TokenInfo`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcAep17TokenInfo {
    /// Token name
    pub name: String,

    /// Token symbol
    pub symbol: String,

    /// Number of decimals
    pub decimals: u8,

    /// Total supply
    pub total_supply: BigInt,

    /// Optional balance for a specific address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<BigInt>,

    /// Optional last updated block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated_block: Option<u32>,
}

#[cfg(test)]
#[path = "../../../tests/client/models/tokens/rpc_aep17_token_info.rs"]
mod tests;
