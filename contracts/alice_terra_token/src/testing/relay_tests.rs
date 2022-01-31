use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_env, mock_info, MockQuerier, MockStorage};
use cosmwasm_std::{
    coins, from_binary, Addr, Coin, ContractResult, DepsMut, Env, OwnedDeps, Reply, Response,
    SubMsgExecutionResponse, Uint128,
};
use cw20::BalanceResponse;

use crate::contract::{execute, instantiate, query, reply};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, RelayNonceResponse};
use crate::testing::mock_bech32_api::MockBech32Api;
use crate::testing::mock_querier::WasmMockQuerier;

const CONTRACT_ADDR: &str = "terra1dzhzukyezv0etz22ud940z7adyv7xgcjkahuun";
const MONEY_MARKET_ADDR: &str = "terra1k82qylhej6lgym9j3w0u4s62pgvyf9c8wypsm7";
const ATERRA_TOKEN_ADDR: &str = "terra15za7p4szgz6ryjuavfsgvyrf4wpu2a5jstydeg";
const ACCOUNT_1: &str = "terra1ra36atnwndtureqcg6gyqumv6ndke6nq5m7ml9";
const ACCOUNT_2: &str = "terra1w8vmc6qqgvy4vz9l8kfzj474c0980jm3lkuzn2";
const ACCOUNT_3: &str = "terra12rusa506gu7f4xaxqucym48arl5q9ltn4ekuw6";

// terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v
// Mnemonic key: notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius
// See `/utils/src/misc/generateRelayUnitTestMsgs.ts`
const ACCOUNT_4: &str = "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v";
const ACCOUNT_4_PUB_KEY: &str =
    "023b33a8524344061b12364cba20fe0a1ab36d4486abf451bb7cebd11ea2241e5b";

const OK_SUBMSG_RESULT: ContractResult<SubMsgExecutionResponse> =
    ContractResult::Ok(SubMsgExecutionResponse {
        events: vec![],
        data: None,
    });
const OK_DEPOSIT_REPLY: Reply = Reply {
    id: 1,
    result: OK_SUBMSG_RESULT,
};

pub fn mock_bech32_env() -> Env {
    let mut env = mock_env();
    env.contract.address = Addr::unchecked(CONTRACT_ADDR);
    env.block.chain_id = "terra-test".to_string();
    env
}

pub fn mock_bech32_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockBech32Api, WasmMockQuerier> {
    let mut custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(CONTRACT_ADDR, contract_balance)]));

    custom_querier.with_token_balances(&[(
        &ATERRA_TOKEN_ADDR.to_string(),
        &[(&CONTRACT_ADDR.to_string(), &Uint128::zero())],
    )]);

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockBech32Api::new(),
        querier: custom_querier,
    }
}

fn instantiate_bech32_contract(deps: DepsMut) -> (Response, Env) {
    let msg = InstantiateMsg {
        owner: ACCOUNT_1.to_string(),
        name: String::from("Alice Terra USD"),
        symbol: String::from("aliceUST"),
        decimals: 6,
        stable_denom: String::from("uusd"),
        money_market_addr: String::from(MONEY_MARKET_ADDR),
        aterra_token_addr: String::from(ATERRA_TOKEN_ADDR),
        redeem_fee_ratio: Decimal256::zero(),
    };
    let env = mock_bech32_env();
    let info = mock_info(ACCOUNT_1, &[]);

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps, env.clone(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    (res, env)
}

#[test]
fn relayed_transfer() {
    let mut deps = mock_bech32_dependencies(&[]);
    instantiate_bech32_contract(deps.as_mut());

    let sender_pub_key = hex::decode(ACCOUNT_4_PUB_KEY).unwrap();
    let sender_human_addr = ACCOUNT_4.to_string();

    let relay_account_addr = ACCOUNT_2;

    // Deposit 100 UST to Sender as aliceUST
    let env = mock_bech32_env();
    let info = mock_info(relay_account_addr, &coins(100_000_000, "uusd"));
    let initial_deposit = ExecuteMsg::DepositStable {
        recipient: Some(sender_human_addr.clone()),
    };
    execute(deps.as_mut(), env.clone(), info, initial_deposit).unwrap();
    deps.querier.with_token_balances(&[(
        &ATERRA_TOKEN_ADDR.to_string(),
        &[(&CONTRACT_ADDR.to_string(), &Uint128::from(100_000_000_u64))],
    )]);
    reply(deps.as_mut(), env.clone(), OK_DEPOSIT_REPLY).unwrap();

    // Query nonce
    let query_msg = QueryMsg::RelayNonce {
        address: sender_human_addr.clone(),
    };
    let response: RelayNonceResponse =
        from_binary(&query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
    assert_eq!(response.relay_nonce, Uint128::from(0u16));

    let recipient_addr = ACCOUNT_3;

    // Relayed transfer from ACCOUNT_4 to ACCOUNT_3
    // 100 ualiceUST
    let transfer_msg_json = r#"{"contract":"terra1dzhzukyezv0etz22ud940z7adyv7xgcjkahuun","chain_id":"terra-test","nonce":"1","msg":{"transfer":{"recipient":"terra12rusa506gu7f4xaxqucym48arl5q9ltn4ekuw6","amount":"100"}}}"#;
    let transfer_msg: Vec<u8> = transfer_msg_json.into();
    let signature_hex = "cfa7b75202af43e33356e49681ecc9d6e81e5bfb2b7330ef846d2cb9fa0b70980c1307442c6bff270c58b4ebd2949d52800b73302fafdf1c75d350f42ef53f74";
    let signature = hex::decode(signature_hex).unwrap();

    let env = mock_bech32_env();
    let info = mock_info(relay_account_addr, &[]);
    let relay_msg = ExecuteMsg::Relay {
        meta_tx: transfer_msg.into(),
        signature: signature.into(),
        public_key: sender_pub_key.into(),
    };
    let res = execute(deps.as_mut(), env.clone(), info, relay_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Check balance of sender (ACCOUNT_4)
    let query_msg = QueryMsg::Balance {
        address: sender_human_addr.clone(),
    };
    let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let value: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::from(99_999_900_u64), value.balance);

    // Check balance of receiver (ACCOUNT_3)
    let query_msg = QueryMsg::Balance {
        address: recipient_addr.to_string(),
    };
    let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let value: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::from(100_u64), value.balance);

    // Check nonce increased
    let query_msg = QueryMsg::RelayNonce {
        address: sender_human_addr,
    };
    let response: RelayNonceResponse =
        from_binary(&query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
    assert_eq!(response.relay_nonce, Uint128::from(1u16));
}

#[test]
fn relayed_transfer_wrong_signature() {
    let mut deps = mock_bech32_dependencies(&[]);
    let (_res, _env) = instantiate_bech32_contract(deps.as_mut());

    let sender_pub_key = hex::decode(ACCOUNT_4_PUB_KEY).unwrap();
    let sender_human_addr = ACCOUNT_4.to_string();

    let relay_account_addr = ACCOUNT_2;

    // Deposit 100 UST to Sender as aliceUST
    let env = mock_bech32_env();
    let info = mock_info(relay_account_addr, &coins(100_000_000, "uusd"));
    let initial_deposit = ExecuteMsg::DepositStable {
        recipient: Some(sender_human_addr.clone()),
    };
    execute(deps.as_mut(), env.clone(), info, initial_deposit).unwrap();
    deps.querier.with_token_balances(&[(
        &ATERRA_TOKEN_ADDR.to_string(),
        &[(&CONTRACT_ADDR.to_string(), &Uint128::from(100_000_000_u64))],
    )]);
    reply(deps.as_mut(), env.clone(), OK_DEPOSIT_REPLY).unwrap();

    // Query nonce
    let query_msg = QueryMsg::RelayNonce {
        address: sender_human_addr,
    };
    let response: RelayNonceResponse =
        from_binary(&query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
    assert_eq!(response.relay_nonce, Uint128::from(0u16));

    // Relayed transfer from ACCOUNT_4 to ACCOUNT_3
    // 100 ualiceUST
    let transfer_msg_json = r#"{"contract":"terra1dzhzukyezv0etz22ud940z7adyv7xgcjkahuun","chain_id":"terra-test","nonce":"1","msg":{"transfer":{"recipient":"terra12rusa506gu7f4xaxqucym48arl5q9ltn4ekuw6","amount":"100"}}}"#;
    let transfer_msg: Vec<u8> = transfer_msg_json.into();
    // wrong signature
    let signature_hex = "1234565202af43e33356e49681ecc9d6e81e5bfb2b7330ef846d2cb9fa0b70980c1307442c6bff270c58b4ebd2949d52800b73302fafdf1c75d350f42ef53f74";
    let signature = hex::decode(signature_hex).unwrap();

    let env = mock_bech32_env();
    let info = mock_info(relay_account_addr, &[]);
    let relay_msg = ExecuteMsg::Relay {
        meta_tx: transfer_msg.into(),
        signature: signature.into(),
        public_key: sender_pub_key.into(),
    };
    execute(deps.as_mut(), env, info, relay_msg).unwrap_err();
}

#[test]
fn relayed_transfer_wrong_contract() {
    let mut deps = mock_bech32_dependencies(&[]);
    let (_res, _env) = instantiate_bech32_contract(deps.as_mut());

    let sender_pub_key = hex::decode(ACCOUNT_4_PUB_KEY).unwrap();
    let sender_human_addr = ACCOUNT_4.to_string();

    let relay_account_addr = ACCOUNT_2;

    // Deposit 100 UST to Sender as aliceUST
    let env = mock_bech32_env();
    let info = mock_info(relay_account_addr, &coins(100_000_000, "uusd"));
    let initial_deposit = ExecuteMsg::DepositStable {
        recipient: Some(sender_human_addr.clone()),
    };
    execute(deps.as_mut(), env.clone(), info, initial_deposit).unwrap();
    deps.querier.with_token_balances(&[(
        &ATERRA_TOKEN_ADDR.to_string(),
        &[(&CONTRACT_ADDR.to_string(), &Uint128::from(100_000_000_u64))],
    )]);
    reply(deps.as_mut(), env.clone(), OK_DEPOSIT_REPLY).unwrap();

    // Query nonce
    let query_msg = QueryMsg::RelayNonce {
        address: sender_human_addr,
    };
    let response: RelayNonceResponse =
        from_binary(&query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
    assert_eq!(response.relay_nonce, Uint128::from(0u16));

    // Relayed transfer from ACCOUNT_4 to ACCOUNT_3
    // 100 ualiceUST
    // wrong contract address
    let transfer_msg_json = r#"{"contract":"terra1k82qylhej6lgym9j3w0u4s62pgvyf9c8wypsm7","chain_id":"terra-test","nonce":"1","msg":{"transfer":{"recipient":"terra12rusa506gu7f4xaxqucym48arl5q9ltn4ekuw6","amount":"100"}}}"#;
    let transfer_msg: Vec<u8> = transfer_msg_json.into();
    let signature_hex = "eebcfac10848d5497b52f027cd50de1f69886e59826326ed4babe681548193f10a26aebba9a45ec9931cae35e5dffe700fb396e0812f23bb5294596922830a18";
    let signature = hex::decode(signature_hex).unwrap();

    let env = mock_bech32_env();
    let info = mock_info(relay_account_addr, &[]);
    let relay_msg = ExecuteMsg::Relay {
        meta_tx: transfer_msg.into(),
        signature: signature.into(),
        public_key: sender_pub_key.into(),
    };
    execute(deps.as_mut(), env, info, relay_msg).unwrap_err();
}

#[test]
fn relayed_transfer_wrong_chain() {
    let mut deps = mock_bech32_dependencies(&[]);
    let (_res, _env) = instantiate_bech32_contract(deps.as_mut());

    let sender_pub_key = hex::decode(ACCOUNT_4_PUB_KEY).unwrap();
    let sender_human_addr = ACCOUNT_4.to_string();

    let relay_account_addr = ACCOUNT_2;

    // Deposit 100 UST to Sender as aliceUST
    let env = mock_bech32_env();
    let info = mock_info(relay_account_addr, &coins(100_000_000, "uusd"));
    let initial_deposit = ExecuteMsg::DepositStable {
        recipient: Some(sender_human_addr.clone()),
    };
    execute(deps.as_mut(), env.clone(), info, initial_deposit).unwrap();
    deps.querier.with_token_balances(&[(
        &ATERRA_TOKEN_ADDR.to_string(),
        &[(&CONTRACT_ADDR.to_string(), &Uint128::from(100_000_000_u64))],
    )]);
    reply(deps.as_mut(), env.clone(), OK_DEPOSIT_REPLY).unwrap();

    // Query nonce
    let query_msg = QueryMsg::RelayNonce {
        address: sender_human_addr,
    };
    let response: RelayNonceResponse =
        from_binary(&query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
    assert_eq!(response.relay_nonce, Uint128::from(0u16));

    // Relayed transfer from ACCOUNT_4 to ACCOUNT_3
    // 100 ualiceUST
    // wrong chain id
    let transfer_msg_json = r#"{"contract":"terra1dzhzukyezv0etz22ud940z7adyv7xgcjkahuun","chain_id":"terra-test-5","nonce":"1","msg":{"transfer":{"recipient":"terra12rusa506gu7f4xaxqucym48arl5q9ltn4ekuw6","amount":"100"}}}"#;
    let transfer_msg: Vec<u8> = transfer_msg_json.into();
    let signature_hex = "993e2f781c44c203971ef915f78f592f5bb0a26de6123ed54eca57262f5693f5726ad2541d8528fedd79499376af50e5f986f487e13aa9d94d2bcf6032d768f2";
    let signature = hex::decode(signature_hex).unwrap();

    let env = mock_bech32_env();
    let info = mock_info(relay_account_addr, &[]);
    let relay_msg = ExecuteMsg::Relay {
        meta_tx: transfer_msg.into(),
        signature: signature.into(),
        public_key: sender_pub_key.into(),
    };
    execute(deps.as_mut(), env, info, relay_msg).unwrap_err();
}

#[test]
fn relayed_transfer_replay() {
    let mut deps = mock_bech32_dependencies(&[]);
    let (_res, _env) = instantiate_bech32_contract(deps.as_mut());

    let sender_pub_key = hex::decode(ACCOUNT_4_PUB_KEY).unwrap();
    let sender_human_addr = ACCOUNT_4.to_string();

    let relay_account_addr = ACCOUNT_2;

    // Deposit 100 UST to Sender as aliceUST
    let env = mock_bech32_env();
    let info = mock_info(relay_account_addr, &coins(100_000_000, "uusd"));
    let initial_deposit = ExecuteMsg::DepositStable {
        recipient: Some(sender_human_addr),
    };
    execute(deps.as_mut(), env.clone(), info, initial_deposit).unwrap();
    deps.querier.with_token_balances(&[(
        &ATERRA_TOKEN_ADDR.to_string(),
        &[(&CONTRACT_ADDR.to_string(), &Uint128::from(100_000_000_u64))],
    )]);
    reply(deps.as_mut(), env, OK_DEPOSIT_REPLY).unwrap();

    // Relayed transfer from ACCOUNT_4 to ACCOUNT_3
    // 100 ualiceUST
    let transfer_msg_json = r#"{"contract":"terra1dzhzukyezv0etz22ud940z7adyv7xgcjkahuun","chain_id":"terra-test","nonce":"1","msg":{"transfer":{"recipient":"terra12rusa506gu7f4xaxqucym48arl5q9ltn4ekuw6","amount":"100"}}}"#;
    let transfer_msg: Vec<u8> = transfer_msg_json.into();
    let signature_hex = "cfa7b75202af43e33356e49681ecc9d6e81e5bfb2b7330ef846d2cb9fa0b70980c1307442c6bff270c58b4ebd2949d52800b73302fafdf1c75d350f42ef53f74";
    let signature = hex::decode(signature_hex).unwrap();

    let env = mock_bech32_env();
    let info = mock_info(relay_account_addr, &[]);
    let relay_msg = ExecuteMsg::Relay {
        meta_tx: transfer_msg.into(),
        signature: signature.into(),
        public_key: sender_pub_key.into(),
    };
    let res = execute(deps.as_mut(), env, info, relay_msg.clone()).unwrap();
    assert_eq!(0, res.messages.len());

    // Replay of the transfer - expect error
    let env = mock_bech32_env();
    let info = mock_info(relay_account_addr, &[]);
    execute(deps.as_mut(), env, info, relay_msg).unwrap_err();
}

#[test]
fn relayed_transfer_with_tip() {
    let mut deps = mock_bech32_dependencies(&[]);
    let (_res, _env) = instantiate_bech32_contract(deps.as_mut());

    let sender_pub_key = hex::decode(ACCOUNT_4_PUB_KEY).unwrap();
    let sender_human_addr = ACCOUNT_4.to_string();

    let relay_account_addr = ACCOUNT_2;

    // Deposit 100 UST to Sender as aliceUST
    let env = mock_bech32_env();
    let info = mock_info(relay_account_addr, &coins(100_000_000, "uusd"));
    let initial_deposit = ExecuteMsg::DepositStable {
        recipient: Some(sender_human_addr.clone()),
    };
    execute(deps.as_mut(), env.clone(), info, initial_deposit).unwrap();
    deps.querier.with_token_balances(&[(
        &ATERRA_TOKEN_ADDR.to_string(),
        &[(&CONTRACT_ADDR.to_string(), &Uint128::from(100_000_000_u64))],
    )]);
    reply(deps.as_mut(), env.clone(), OK_DEPOSIT_REPLY).unwrap();

    // Query nonce
    let query_msg = QueryMsg::RelayNonce {
        address: sender_human_addr,
    };
    let response: RelayNonceResponse =
        from_binary(&query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
    assert_eq!(response.relay_nonce, Uint128::from(0u16));

    // Relayed transfer from ACCOUNT_4 to ACCOUNT_3
    // 100 ualiceUST
    // 50 ualiceUST tip
    let transfer_msg_json = r#"{"contract":"terra1dzhzukyezv0etz22ud940z7adyv7xgcjkahuun","chain_id":"terra-test","nonce":"1","msg":{"transfer":{"recipient":"terra12rusa506gu7f4xaxqucym48arl5q9ltn4ekuw6","amount":"100"}},"tip":"50"}"#;
    let transfer_msg: Vec<u8> = transfer_msg_json.into();
    let signature_hex = "aed156a023a5de26a8b6322d3e313da26cf704e48b72a13f46a140624efcb53d34b2e894f857acce04c15483e91470b607c8f2ec046c96d96e46e2b02773387c";
    let signature = hex::decode(signature_hex).unwrap();

    let env = mock_bech32_env();
    let info = mock_info(relay_account_addr, &[]);
    let relay_msg = ExecuteMsg::Relay {
        meta_tx: transfer_msg.into(),
        signature: signature.into(),
        public_key: sender_pub_key.into(),
    };
    let res = execute(deps.as_mut(), env.clone(), info, relay_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Check balance of owner for tip
    let query_msg = QueryMsg::Balance {
        address: ACCOUNT_1.to_string(),
    };
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let value: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::from(50_u64), value.balance);
}
