use super::super::utility::{
    AepBalanceFieldRefs, balance_list_to_json, insert_nep_balance_fields, object_array,
    parse_balance_list, parse_nep_balance_fields, parse_object_array_lossy, required_string,
};
use atipicial_config::ProtocolSettings;
use atipicial_error::{CoreError, CoreResult};
use atipicial_primitives::{UInt160, strip_hex_prefix};
use atipicial_serialization::json::{JObject, JToken};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

/// AEP11 balances for an address matching C# `RpcAep11Balances`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcAep11Balances {
    /// User script hash.
    pub user_script_hash: UInt160,
    /// List of AEP11 asset balances.
    pub balances: Vec<RpcAep11Balance>,
}

impl RpcAep11Balances {
    /// Converts to JSON.
    #[must_use]
    pub fn to_json(&self, protocol_settings: &ProtocolSettings) -> JObject {
        balance_list_to_json(
            &self.balances,
            &self.user_script_hash,
            protocol_settings,
            RpcAep11Balance::to_json,
        )
    }

    /// Creates from JSON.
    pub fn from_json(json: &JObject, protocol_settings: &ProtocolSettings) -> CoreResult<Self> {
        let (balances, user_script_hash) =
            parse_balance_list(json, protocol_settings, RpcAep11Balance::from_json)?;

        Ok(Self {
            user_script_hash,
            balances,
        })
    }
}

/// Individual AEP11 balance per asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcAep11Balance {
    /// Asset hash.
    pub asset_hash: UInt160,
    /// Asset name.
    pub name: String,
    /// Symbol.
    pub symbol: String,
    /// Decimals.
    pub decimals: u8,
    /// Tokens held for this asset.
    pub tokens: Vec<RpcAep11TokenBalance>,
}

impl RpcAep11Balance {
    /// Convert this asset balance to its Atipicial JSON-RPC representation.
    #[must_use]
    pub fn to_json(&self) -> JObject {
        let mut json = JObject::new();
        json.insert(
            "assethash".to_string(),
            JToken::String(self.asset_hash.to_string()),
        );
        json.insert("name".to_string(), JToken::String(self.name.clone()));
        json.insert("symbol".to_string(), JToken::String(self.symbol.clone()));
        json.insert(
            "decimals".to_string(),
            JToken::String(self.decimals.to_string()),
        );
        json.insert(
            "tokens".to_string(),
            object_array(&self.tokens, RpcAep11TokenBalance::to_json),
        );
        json
    }

    /// Parse an asset balance from its Atipicial JSON-RPC representation.
    pub fn from_json(json: &JObject) -> CoreResult<Self> {
        let asset_hash_str =
            required_string(json, "assethash").map_err(|e| CoreError::other(e.to_string()))?;
        let asset_hash = UInt160::parse(&asset_hash_str)
            .map_err(|_| CoreError::other(format!("Invalid asset hash: {asset_hash_str}")))?;

        let name = json
            .get("name")
            .and_then(atipicial_serialization::json::JToken::as_string)
            .unwrap_or_default();
        let symbol = json
            .get("symbol")
            .and_then(atipicial_serialization::json::JToken::as_string)
            .unwrap_or_default();

        let decimals_token = json.get("decimals");
        let decimals = match decimals_token.and_then(atipicial_serialization::json::JToken::as_string) {
            Some(text) => text.parse::<u8>().unwrap_or(0),
            None => decimals_token
                .and_then(atipicial_serialization::json::JToken::as_number)
                .map_or(0, |n| n as u8),
        };

        let tokens = parse_object_array_lossy(json, "tokens", RpcAep11TokenBalance::from_json);

        Ok(Self {
            asset_hash,
            name,
            symbol,
            decimals,
            tokens,
        })
    }
}

/// Balance of a specific AEP11 token id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcAep11TokenBalance {
    /// Token id bytes.
    pub token_id: Vec<u8>,
    /// Amount.
    pub amount: BigInt,
    /// Last updated block.
    pub last_updated_block: u32,
}

impl RpcAep11TokenBalance {
    /// Convert this token balance to its Atipicial JSON-RPC representation.
    #[must_use]
    pub fn to_json(&self) -> JObject {
        let mut json = JObject::new();
        json.insert(
            "tokenid".to_string(),
            JToken::String(hex::encode(&self.token_id)),
        );
        insert_nep_balance_fields(
            &mut json,
            AepBalanceFieldRefs {
                amount: &self.amount,
                last_updated_block: self.last_updated_block,
            },
        );
        json
    }

    /// Parse a token balance from its Atipicial JSON-RPC representation.
    pub fn from_json(json: &JObject) -> CoreResult<Self> {
        let token_id_str =
            required_string(json, "tokenid").map_err(|e| CoreError::other(e.to_string()))?;
        let token_id = hex::decode(strip_hex_prefix(&token_id_str))
            .map_err(|_| CoreError::other(format!("Invalid tokenid: {token_id_str}")))?;

        let fields = parse_nep_balance_fields(json)?;

        Ok(Self {
            token_id,
            amount: fields.amount,
            last_updated_block: fields.last_updated_block,
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/client/models/tokens/rpc_aep11_balances.rs"]
mod tests;
