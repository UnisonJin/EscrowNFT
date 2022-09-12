use crate::msg::{ EscrowsCountResponse,  EscrowResponse, EscrowsResponse, QueryMsg, CollectionOffset };
use crate::state::{  State, CONFIG, escrows, escrow_key };
use cosmwasm_std::{entry_point, to_binary, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;


#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStateInfo {} => to_binary(&query_state_info(deps)?),
        QueryMsg::Escrow {
            collection,
            token_id,
        } => to_binary(&query_escrow(deps, collection, token_id)?),
        QueryMsg::Escrows {
            collection,
            start_after,
            limit,
        } => to_binary(&query_escrows(
            deps,
            collection,
            start_after,
            limit,
        )?),
        QueryMsg::ReverseEscrows{
            collection,
            start_before,
            limit,
        } => to_binary(&reverse_query_escrows(
            deps,
            collection,
            start_before,
            limit,
        )?),
        QueryMsg::EscrowsBySource {
            source,
            start_after,
            limit,
        } => to_binary(&query_escrows_by_source(
            deps,
            source,
            start_after,
            limit,
        )?),
        QueryMsg::EscrowsByRecipient {
            recipient,
            start_after,
            limit,
        } => to_binary(&query_escrows_by_recipient(
            deps,
            recipient,
            start_after,
            limit,
        )?),
        QueryMsg::EscrowsCount { collection } => {
            to_binary(&query_escrows_count(deps, collection)?)
        },
     
    }
}

pub fn query_state_info(deps:Deps) -> StdResult<State>{
    let state =  CONFIG.load(deps.storage)?;
    Ok(state)
}

pub fn query_escrow(deps: Deps, collection: String, token_id: String) -> StdResult<EscrowResponse> {
    let escrow = escrows().may_load(deps.storage, escrow_key(&collection, &token_id))?;

    Ok(EscrowResponse { escrow })
}


pub fn query_escrows(
    deps: Deps,
    collection: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<EscrowsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let escrows = escrows()
        .idx
        .collection
        .prefix(collection.clone())
        .range(
            deps.storage,
            Some(Bound::exclusive((
                collection,
                start_after.unwrap_or_default(),
            ))),
            None,
            Order::Ascending,
        )
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(EscrowsResponse { escrows })
}

pub fn reverse_query_escrows(
    deps: Deps,
    collection: String,
    start_before: Option<String>,
    limit: Option<u32>,
) -> StdResult<EscrowsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let escrows = escrows()
        .idx
        .collection
        .prefix(collection.clone())
        .range(
            deps.storage,
            None,
            Some(Bound::exclusive((
                collection,
                start_before.unwrap_or_default(),
            ))),
            Order::Descending,
        )
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(EscrowsResponse { escrows })
}

pub fn query_escrows_count(deps: Deps, collection: String) -> StdResult<EscrowsCountResponse> {
    let count = escrows()
        .idx
        .collection
        .prefix(collection)
        .keys_raw(deps.storage, None, None, Order::Ascending)
        .count() as u32;

    Ok(EscrowsCountResponse { count })
}

pub fn query_escrows_by_source(
    deps: Deps,
    source: String,
    start_after: Option<CollectionOffset>,
    limit: Option<u32>,
) -> StdResult<EscrowsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = if let Some(start) = start_after {
        deps.api.addr_validate(&start.collection)?;
        let collection = start.collection;
        Some(Bound::exclusive(escrow_key(&collection, &start.token_id)))
    } else {
        None
    };

    let escrows = escrows()
        .idx
        .source
        .prefix(source)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(EscrowsResponse { escrows })
}


pub fn query_escrows_by_recipient(
    deps: Deps,
    recipient: String,
    start_after: Option<CollectionOffset>,
    limit: Option<u32>,
) -> StdResult<EscrowsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = if let Some(start) = start_after {
        deps.api.addr_validate(&start.collection)?;
        let collection = start.collection;
        Some(Bound::exclusive(escrow_key(&collection, &start.token_id)))
    } else {
        None
    };

    let escrows = escrows()
        .idx
        .recipient
        .prefix(recipient)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(EscrowsResponse { escrows })
}

