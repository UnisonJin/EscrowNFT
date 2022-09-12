#[cfg(test)]
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{ DepsMut, Timestamp, Uint128,to_binary, Env,  CosmosMsg, WasmMsg, Coin, BankMsg};
use cw721::{Cw721ReceiveMsg,Cw721ExecuteMsg};

use crate::contract::{execute, instantiate};
use crate::msg::{ExecuteMsg, InstantiateMsg,  CollectionOffset, EscrowInfoMsg};
use crate::query::{query_state_info, query_escrows_by_source, query_escrows_by_recipient};


fn setup_contract(deps: DepsMut){
   let instantiate_msg = InstantiateMsg {
        admin: "admin".to_string(),
        denom: "ujuno".to_string()
    };
    let info = mock_info("owner", &[]);
    let res = instantiate(deps, mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(res.messages.len(), 0);
}


fn send_nft(
  deps: DepsMut, 
  env: Env, 
  collection: &str, 
  sender: String, 
  token_id: String,
  recipient: String,
  price: Uint128
){
  let sell_msg = EscrowInfoMsg{
    recipient,
    price,
    expiration: Timestamp::from_seconds(env.block.time.seconds() + 300),
};

  let info = mock_info(collection, &[]);
  let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg{
      sender,
      token_id,
      msg:to_binary(&sell_msg).unwrap()
  });
  execute(deps, env, info.clone(), msg).unwrap();
}



#[test]
fn init_contract() {
    let mut deps = mock_dependencies();
    let instantiate_msg = InstantiateMsg {
        admin: "admin".to_string(),
        denom: "ujuno".to_string()
    };
    let info = mock_info("owner", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());
    let state = query_state_info(deps.as_ref()).unwrap();
    assert_eq!(state.admin,"owner".to_string());
}

#[test]
fn send_nft_to_escrow_contract() {
  let mut deps = mock_dependencies();
  let env = mock_env();

  //init contract
  setup_contract(deps.as_mut());

  send_nft(
    deps.as_mut(), 
    env.clone(), 
    "collection1", 
    "source1".to_string(), 
    "Test.1".to_string(), 
    "receiver1".to_string(), 
    Uint128::new(50)
  );

  send_nft(
    deps.as_mut(), 
    env.clone(), 
    "collection2", 
    "source1".to_string(), 
    "Test.2".to_string(), 
    "receiver2".to_string(), 
    Uint128::new(50)
  );

  let escrows_by_source = query_escrows_by_source(deps.as_ref(), "source1".to_string(), None, Some(30)).unwrap();
  println!("{:?}", escrows_by_source);

  
  let escrows_by_source = query_escrows_by_source(deps.as_ref(), "source1".to_string(), Some(CollectionOffset{ collection: "collection1".to_string(), token_id: "Test.1".to_string()}), Some(30)).unwrap();
  println!("{:?}", escrows_by_source);

  let escrows_by_recipient = query_escrows_by_recipient(deps.as_ref(), "receiver1".to_string(), None, Some(30)).unwrap();
  println!("{:?}", escrows_by_recipient)

}


#[test]
fn approve() {
  let mut deps = mock_dependencies();
  let env = mock_env();

  //init contract
  setup_contract(deps.as_mut());

  send_nft(
    deps.as_mut(), 
    env.clone(), 
    "collection1", 
    "source1".to_string(), 
    "Test.1".to_string(), 
    "receiver1".to_string(), 
    Uint128::new(50)
  );

  send_nft(
    deps.as_mut(), 
    env.clone(), 
    "collection2", 
    "source1".to_string(), 
    "Test.2".to_string(), 
    "receiver2".to_string(), 
    Uint128::new(50)
  );

  let info = mock_info("receiver1", &[Coin{denom:"ujuno".to_string(), amount: Uint128::new(50) }]);
  let msg = ExecuteMsg::Approve { collection: "collection1".to_string(), token_id: "Test.1".to_string() };
  let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
  // assert_eq!(res.messages.len(),2);

  assert_eq!(res.messages[0].msg, 
    CosmosMsg::Bank(BankMsg::Send { to_address: "source1".to_string(), amount: vec![Coin{denom:"ujuno".to_string() , amount: Uint128::new(50)}] }) 
  );

  assert_eq!(res.messages[1].msg, 
    CosmosMsg::Wasm(WasmMsg::Execute{ 
      contract_addr: "collection1".to_string(), 
      msg: to_binary(&Cw721ExecuteMsg::TransferNft { recipient: "receiver1".to_string(), token_id: "Test.1".to_string() }).unwrap(), 
      funds: vec![] })
  );

  let escrows_by_source = query_escrows_by_source(deps.as_ref(), "source1".to_string(), None, Some(30)).unwrap();
  println!("{:?}", escrows_by_source);
  
}

