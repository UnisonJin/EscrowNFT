use cosmwasm_std::{Uint128, Timestamp, BlockInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item, MultiIndex, IndexList, Index, IndexedMap};

pub const CONFIG: Item<State> = Item::new("config_state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub admin: String,
    pub denom: String
}


pub trait Order {
    fn expires_at(&self) -> Timestamp;

    fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires_at() <= block.time
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Escrow {
    pub source: String,
    pub recipient: String,
    pub price: Uint128,
    pub expires_at: Timestamp,
    pub collection: String,
    pub token_id: String
}


impl Order for Escrow {
    fn expires_at(&self) -> Timestamp {
        self.expires_at
    }
}

/// Primary key for Escrows: (collection, token_id)
pub type EscrowKey<'a> = (String, String);
/// Convenience Escrow key constructor
pub fn escrow_key<'a>(collection: &'a String, token_id: &'a String) -> EscrowKey<'a> {
    (collection.clone(), token_id.clone())
}

/// Defines indices for accessing Escrows
pub struct EscrowIndicies<'a> {
    pub collection: MultiIndex<'a, String, Escrow, EscrowKey<'a>>,
    pub source: MultiIndex<'a, String, Escrow, EscrowKey<'a>>,
    pub recipient: MultiIndex<'a, String, Escrow, EscrowKey<'a>>,
}

impl<'a> IndexList<Escrow> for EscrowIndicies<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Escrow>> + '_> {
        let v: Vec<&dyn Index<Escrow>> = vec![&self.collection, &self.source, &self.recipient];
        Box::new(v.into_iter())
    }
}

pub fn escrows<'a>() -> IndexedMap<'a, EscrowKey<'a>, Escrow, EscrowIndicies<'a>> {
    let indexes = EscrowIndicies {
        collection: MultiIndex::new(|d: &Escrow| d.collection.clone(), "Escrows", "Escrows__collection"),
        source: MultiIndex::new(|d: &Escrow| d.source.clone(), "Escrows", "Escrows__source"),
        recipient: MultiIndex::new(|d: &Escrow| d.recipient.clone(), "Escrows", "Escrows__recipient"),
    };
    IndexedMap::new("Escrows", indexes)
}


