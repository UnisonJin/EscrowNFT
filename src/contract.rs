use cosmwasm_std::{
    entry_point, to_binary, from_binary, Coin, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128, CosmosMsg, WasmMsg, BankMsg, Storage
};

use cw2::set_contract_version;
use cw721::{Cw721ReceiveMsg, Cw721ExecuteMsg};

use crate::msg::{ ExecuteMsg, InstantiateMsg,  EscrowInfoMsg};
use crate::state::{ escrows, escrow_key,Order, Escrow, State, CONFIG };
use crate::error::ContractError;


const CONTRACT_NAME: &str = "Escrow Contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let state = State {
        admin: msg.admin.clone(),
        denom: msg.denom
    };
    CONFIG.save(deps.storage,&state)?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", msg.admin))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ReceiveNft(
            msg
        ) =>execute_receive_nft(
            deps,
            env,
            info,
            msg
        ),
        ExecuteMsg::WithdrawNft {
            collection ,
            token_id 
        } => execute_withdraw(
            deps,
            env,
            info,
            collection,
            token_id
        ),
        ExecuteMsg::Approve { 
            collection, 
            token_id 
        } => execute_approve(
            deps,
            env,
            info,
            collection,
            token_id
        ),
        ExecuteMsg::ChangeConfig { 
            state 
        } => execute_change_config(
            deps,
            env,
            info,
            state
        )
            
 }
}


fn execute_receive_nft(
    deps: DepsMut,
    env:Env,
    info: MessageInfo,
    rcv_msg: Cw721ReceiveMsg,
)-> Result<Response, ContractError> {
    
    let msg:EscrowInfoMsg = from_binary(&rcv_msg.msg)?;
    let collection = info.sender.to_string();
    let token_id = rcv_msg.token_id.clone();

    //validation check
    deps.api.addr_validate(&msg.recipient)?;
    if msg.price == Uint128::zero() {
        return Err(ContractError::NotEnoughFunds {  })
    }

    //Save escrow information
    let escrow = Escrow {
        source: rcv_msg.sender.clone(),
        recipient: msg.recipient,
        price: msg.price,
        expires_at: msg.expiration,
        collection,
        token_id,
    };

    //check if this escrow is expired because of the wrong setting
    if escrow.is_expired(&env.block){
        return Err(ContractError::EscrowExpired {  })
    }

    store_escrow(deps.storage, &escrow)?;

    Ok(Response::new()
        .add_attribute("action", "Send nfts to the arbiter contract")
        .add_attribute("token_id", rcv_msg.token_id)
        .add_attribute("source_user", rcv_msg.sender))
}


fn execute_withdraw(
    deps: DepsMut,
    env:Env,
    info: MessageInfo,
    collection: String,
    token_id: String
)-> Result<Response, ContractError> {
    
    //validation check
    deps.api.addr_validate(&collection)?;
    nonpayable(&info)?;
    
    let sender = info.sender.to_string();
    
    //load escrow
    let escrow_key = escrow_key(&collection.clone(), &token_id.clone());
    let escrow = escrows().may_load(deps.storage, escrow_key.clone())?;
    
    match escrow {
        Some(escrow) => {
            //User can withdraw after the escrow is expired
            if !escrow.is_expired(&env.block){
                return Err(ContractError::EscrowNotExpired {  })
            }
            if sender != escrow.source{
                return Err(ContractError::Unauthorized {  } )
            }
            //remove current escrow 
            remove_escrow(deps.storage, &escrow)?;
        },
        None => {
            return Err(ContractError::NoEscrow{} );
        }
    }

    let cw721_transfer_msg = WasmMsg::Execute { 
        contract_addr: collection.clone(), 
        msg: to_binary(&Cw721ExecuteMsg::TransferNft { recipient: sender.clone(), token_id: token_id.clone() })? , 
        funds: vec![] 
    };

    let message: CosmosMsg = CosmosMsg::Wasm(cw721_transfer_msg);

    Ok(Response::new()
        .add_attribute("action", "Withdraw NFT")
        .add_attribute("token_id", token_id)
        .add_attribute("collection", collection)
        .add_attribute("source", sender)
        .add_message(message)
      )
}



fn execute_approve(
    deps: DepsMut,
    env:Env,
    info: MessageInfo,
    collection: String,
    token_id: String
)-> Result<Response, ContractError> {
    
    //validation check
    deps.api.addr_validate(&collection)?;

    let sender = info.sender.to_string();
    
    //load escrow
    let escrow_key = escrow_key(&collection.clone(), &token_id.clone());
    let escrow = escrows().may_load(deps.storage, escrow_key.clone())?;
    let state = CONFIG.load(deps.storage)?;

    let mut messages :Vec<CosmosMsg> = Vec::new();

    match escrow {
        Some(escrow) => {
            //User can withdraw after the escrow is expired
            if escrow.is_expired(&env.block){
                return Err(ContractError::EscrowExpired {  })
            }
            //Check if the sent money is the same as the list price of escrow
            fund_check(&state, &info, &escrow)?;
          
            if sender != escrow.recipient {
                return Err(ContractError::Unauthorized {  } )
            }
            //remove current escrow 
            remove_escrow(deps.storage, &escrow)?;

            let transfer_coin_msg = BankMsg::Send { 
                to_address: escrow.source, 
                amount:  vec![Coin{denom: state.denom, amount: escrow.price}]
            };
            messages.push(CosmosMsg::Bank(transfer_coin_msg) );
        },
        None => {
            return Err(ContractError::NoEscrow{} );
        }
    }

    let cw721_transfer_msg = WasmMsg::Execute { 
        contract_addr: collection.clone(), 
        msg: to_binary(&Cw721ExecuteMsg::TransferNft { recipient: sender.clone(), token_id: token_id.clone() })? , 
        funds: vec![] 
    };

    messages.push(CosmosMsg::Wasm(cw721_transfer_msg));

    Ok(Response::new()
        .add_attribute("action", "Withdraw NFT")
        .add_attribute("token_id", token_id)
        .add_attribute("collection", collection)
        .add_attribute("source", sender)
        .add_messages(messages)
      )
}

fn execute_change_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    state: State
) -> Result<Response, ContractError> {
    only_owner(&state, &info)?;

    CONFIG.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "chanage config")
      )

}



fn store_escrow(store: &mut dyn Storage, escrow: &Escrow) -> StdResult<()> {
    escrows().save(store, escrow_key(&escrow.collection, &escrow.token_id), escrow)
}


fn remove_escrow(store: &mut dyn Storage, escrow: &Escrow) -> StdResult<()> {
    escrows().remove(store, escrow_key(&escrow.collection, &escrow.token_id))
}

fn fund_check(state: &State, info: &MessageInfo, escrow: &Escrow) -> Result<(), ContractError>  {

    let sent_denom = info.funds[0].denom.clone();
    let sent_amount = info.funds[0].amount;
    
    if info.funds.len() != 1 {
        return Err(ContractError::OnlyOneCoinAvailable{});
    } else{
        if sent_denom != state.denom {
            return Err(ContractError::NotExpectedDenom { denom: sent_denom });
        } else{
            if sent_amount != escrow.price{
                return Err(ContractError::NotEnoughFunds {  })
            }
            else{
                Ok(())
            }
        }
    }
    
}

fn nonpayable(info: &MessageInfo) -> Result<(), ContractError> {
    if info.funds.len() > 0 {
        return Err(ContractError::NonPayable{} )
    }
    else{
        Ok(())
    }
}

fn only_owner(state: &State, info: &MessageInfo) -> Result<(), ContractError> {
    if info.sender.to_string() != state.admin {
        return Err(ContractError::Unauthorized {  } )
    }
    else{
        Ok(())
    }
}