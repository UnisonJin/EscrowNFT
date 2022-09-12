use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{ Timestamp, Uint128};
use cw721::Cw721ReceiveMsg;

use crate::state::{State, Escrow};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
  pub admin: String,
  pub denom: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ReceiveNft(Cw721ReceiveMsg),
    WithdrawNft{
        collection: String,
        token_id: String
    },
    Approve{
        collection: String,
        token_id: String
    },
    ChangeConfig{
        state: State
    }
    
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns a human-readable representation of the arbiter.
    GetStateInfo {},
 
    /// Get the current ask for specific NFT
    /// Return type: `CurrentAskResponse`
    Escrow{collection:String, token_id:String},
    /// Get all escrows for a collection
    /// Return type: `EscrowsResponse`
    // start_after is token_id
    Escrows {
        collection: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Get all escrows for a collection in reverse
    /// Return type: `EscrowsResponse`
    ReverseEscrows {
        collection: String,
        start_before: Option<String>,
        limit: Option<u32>,
    },
      /// Count of all escrows
    /// Return type: `EscrowsCountResponse`
    EscrowsCount { collection: String },
    /// Get all asks by source
    /// Return type: `EscrowsResponse`
    EscrowsBySource {
        source: String,
        start_after: Option<CollectionOffset>,
        limit: Option<u32>,
    },
    EscrowsByRecipient{
        recipient: String,
        start_after: Option<CollectionOffset>,
        limit: Option<u32>,
    }
}

/// Offset for collection pagination
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionOffset {
    pub collection: String,
    pub token_id: String,
}

/// Escrow infos
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EscrowInfoMsg {
    pub recipient: String,
    pub price: Uint128,
    pub expiration: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EscrowResponse {  pub escrow: Option<Escrow> }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EscrowsResponse { pub escrows: Vec<Escrow> }


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EscrowsCountResponse { pub count: u32 }
