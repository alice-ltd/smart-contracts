use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{
    mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{
    coins, from_binary, to_binary, BankMsg, Coin, ContractResult, CosmosMsg, Decimal, Reply,
    SubMsg, SubMsgExecutionResponse, WasmMsg,
};
use cosmwasm_std::{DepsMut, OwnedDeps, Response};
use cosmwasm_std::{Env, Uint128};
use cw20::{BalanceResponse, Cw20ExecuteMsg, TokenInfoResponse};
use std::str::FromStr;

use crate::anchor::{MarketCw20HookMsg, MarketExecuteMsg};
use crate::contract::query;
use crate::contract::{execute, reply};
use crate::contract::{instantiate, migrate};
use crate::execute::{DEPOSIT_STABLE_REPLY_ID, REDEEM_STABLE_REPLY_ID};
use crate::msg::InstantiateMsg;
use crate::msg::QueryMsg;
use crate::msg::{ExecuteMsg, MigrateMsg};
use crate::state::Config;

use crate::testing::mock_querier::WasmMockQuerier;

const OK_SUBMSG_RESULT: ContractResult<SubMsgExecutionResponse> =
    ContractResult::Ok(SubMsgExecutionResponse {
        events: vec![],
        data: None,
    });
const OK_DEPOSIT_REPLY: Reply = Reply {
    id: DEPOSIT_STABLE_REPLY_ID,
    result: OK_SUBMSG_RESULT,
};
const OK_REDEEM_REPLY: Reply = Reply {
    id: REDEEM_STABLE_REPLY_ID,
    result: OK_SUBMSG_RESULT,
};

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let mut custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    custom_querier.with_token_balances(&[(
        &"aterra_token_addr".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::zero())],
    )]);

    custom_querier.with_epoch_state(&[(
        &"money_market_addr".to_string(),
        &(Uint256::from(0u64), Decimal256::one()),
    )]);

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

fn instantiate_contract(deps: DepsMut) -> (Response, Env) {
    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        name: String::from("Alice Terra USD"),
        symbol: String::from("aliceUST"),
        decimals: 6,
        stable_denom: String::from("uusd"),
        money_market_addr: String::from("money_market_addr"),
        aterra_token_addr: String::from("aterra_token_addr"),
        redeem_fee_ratio: Decimal256::zero(),
        redeem_fee_cap: Uint128::MAX,
    };
    let env = mock_env();
    let info = mock_info("owner", &[]);

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps, env.clone(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    (res, env)
}

#[test]
fn basic_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let (_res, env) = instantiate_contract(deps.as_mut());

    // Verify token info
    let res = query(deps.as_ref(), env, QueryMsg::TokenInfo {}).unwrap();
    let value: TokenInfoResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::zero(), value.total_supply);
}

#[test]
fn instantiate_redeem_fee_ratio_greater_than_one() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        name: String::from("Alice Terra USD"),
        symbol: String::from("aliceUST"),
        decimals: 6,
        stable_denom: String::from("uusd"),
        money_market_addr: String::from("money_market_addr"),
        aterra_token_addr: String::from("aterra_token_addr"),
        redeem_fee_ratio: Decimal256::from_str("1.1").unwrap(), // greater than 1
        redeem_fee_cap: Uint128::MAX,
    };
    let env = mock_env();
    let info = mock_info("owner", &[]);

    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap_err();
}

#[test]
fn basic_migration() {
    let mut deps = mock_dependencies(&[]);
    let (_res, env) = instantiate_contract(deps.as_mut());

    // update redeem fee ratio and symbol
    migrate(
        deps.as_mut(),
        env.clone(),
        MigrateMsg {
            symbol: Some("newToken".to_string()),
            owner: None,
            money_market_addr: None,
            aterra_token_addr: None,
            redeem_fee_ratio: Some(Decimal256::from_str("0.12345").unwrap()),
            redeem_fee_cap: Some(Uint128::from(50_u128)),
        },
    )
    .unwrap();

    // verify update
    let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config: Config = from_binary(&res).unwrap();
    assert_eq!(
        config.redeem_fee_ratio,
        Decimal256::from_str("0.12345").unwrap()
    );
    assert_eq!(config.redeem_fee_cap, Uint128::from(50_u128));

    let res = query(deps.as_ref(), env, QueryMsg::TokenInfo {}).unwrap();
    let token_info: TokenInfoResponse = from_binary(&res).unwrap();
    assert_eq!(token_info.symbol, "newToken".to_string());
}

#[test]
fn basic_deposit() {
    let mut deps = mock_dependencies(&[]);
    instantiate_contract(deps.as_mut());

    // Deposit 100,000,000 uusd
    let env = mock_env();
    let info = mock_info("user1", &coins(100_000_000, "uusd"));
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::DepositStable { recipient: None },
    )
    .unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::reply_always(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "money_market_addr".to_string(),
                funds: coins(100_000_000, "uusd"),
                msg: to_binary(&MarketExecuteMsg::DepositStable {}).unwrap(),
            }),
            1
        )]
    );
    deps.querier.with_token_balances(&[(
        &"aterra_token_addr".to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(100_000_000_u64),
        )],
    )]);

    // Anchor deposit callback
    let res = reply(deps.as_mut(), env.clone(), OK_DEPOSIT_REPLY).unwrap();
    assert_eq!(0, res.messages.len());

    // Check balance is 100,000,000 uusd
    let res = query(
        deps.as_ref(),
        env,
        QueryMsg::Balance {
            address: String::from("user1"),
        },
    )
    .unwrap();
    let value: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::from(100_000_000_u64), value.balance);
}

#[test]
fn deposit_zero() {
    let mut deps = mock_dependencies(&[]);
    instantiate_contract(deps.as_mut());

    // Deposit 0 uusd should error
    let env = mock_env();
    let info = mock_info("user1", &coins(0, "uusd"));
    execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::DepositStable { recipient: None },
    )
    .unwrap_err();
}

#[test]
fn deposit_authorized() {
    let mut deps = mock_dependencies(&[]);
    instantiate_contract(deps.as_mut());

    // Tax rate: 0.3%, cap of 1 UST
    deps.querier.with_tax(
        Decimal::from_str("0.003").unwrap(),
        &[(&"uusd".to_string(), &Uint128::from(1_000_000_u64))],
    );

    // Deposit 100,000,000 uusd
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("owner", &coins(0_600_000, "uusd")),
        ExecuteMsg::DepositStableAuthorized {
            sender: Some("user1".to_string()),
            recipient: "user1".to_string(),
            amount: Uint128::from(100_000_000_u64),
        },
    )
    .unwrap();
    assert_eq!(
        res.messages[1],
        SubMsg::reply_always(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "money_market_addr".to_string(),
                funds: coins(100_000_000, "uusd"),
                msg: to_binary(&MarketExecuteMsg::DepositStable {}).unwrap(),
            }),
            1
        )
    );
    deps.querier.with_token_balances(&[(
        &"aterra_token_addr".to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(100_000_000_u64),
        )],
    )]);

    // Anchor deposit callback
    let res = reply(deps.as_mut(), env.clone(), OK_DEPOSIT_REPLY).unwrap();
    assert_eq!(0, res.messages.len());

    // Check balance is 100,000,000 uusd
    let res = query(
        deps.as_ref(),
        env,
        QueryMsg::Balance {
            address: String::from("user1"),
        },
    )
    .unwrap();
    let value: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::from(100_000_000_u64), value.balance);
}

#[test]
fn deposit_authorized_tax_funds_too_low() {
    let mut deps = mock_dependencies(&[]);
    instantiate_contract(deps.as_mut());

    // Tax rate: 0.3%, cap of 1 UST
    deps.querier.with_tax(
        Decimal::from_str("0.003").unwrap(),
        &[(&"uusd".to_string(), &Uint128::from(1_000_000_u64))],
    );

    // Deposit 100,000,000 uusd
    // Send 0.599 UST funds, not enough for tax
    let env = mock_env();
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("owner", &coins(0_599_000, "uusd")),
        ExecuteMsg::DepositStableAuthorized {
            sender: Some("user1".to_string()),
            recipient: "user1".to_string(),
            amount: Uint128::from(100_000_000_u64),
        },
    )
    .unwrap_err();
}

#[test]
fn basic_redeem() {
    let mut deps = mock_dependencies(&[]);
    instantiate_contract(deps.as_mut());

    // Deposit 100,000,000 uusd
    let env = mock_env();
    let info = mock_info("user1", &coins(100_000_000, "uusd"));
    execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::DepositStable { recipient: None },
    )
    .unwrap();
    deps.querier.with_token_balances(&[(
        &"aterra_token_addr".to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(100_000_000_u64),
        )],
    )]);
    reply(deps.as_mut(), env, OK_DEPOSIT_REPLY).unwrap();

    // Redeem 100,000,000 ualiceUST
    let env = mock_env();
    let info = mock_info("user1", &[]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::RedeemStable {
            recipient: None,
            burn_amount: Uint128::from(100_000_000_u64),
        },
    )
    .unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::reply_always(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "aterra_token_addr".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: "money_market_addr".to_string(),
                    amount: Uint128::from(100_000_000_u64),
                    msg: to_binary(&MarketCw20HookMsg::RedeemStable {}).unwrap(),
                })
                .unwrap(),
            }),
            2
        )]
    );
    deps.querier.with_base(MockQuerier::new(&[(
        MOCK_CONTRACT_ADDR,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100_000_000_u64),
        }],
    )]));

    // Anchor redeem callback
    let res = reply(deps.as_mut(), env.clone(), OK_REDEEM_REPLY).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "user1".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(100_000_000u128),
            }],
        }))]
    );

    // Check balance is 0
    let res = query(
        deps.as_ref(),
        env,
        QueryMsg::Balance {
            address: String::from("user1"),
        },
    )
    .unwrap();
    let value: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::zero(), value.balance);
}

#[test]
fn redeem_after_interest() {
    let mut deps = mock_dependencies(&[]);
    instantiate_contract(deps.as_mut());

    // Deposit 100,000,000 uusd -> 100,000,000 uaUST
    let mut env = mock_env();
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("user1", &coins(100_000_000, "uusd")),
        ExecuteMsg::DepositStable { recipient: None },
    )
    .unwrap();
    deps.querier.with_token_balances(&[(
        &"aterra_token_addr".to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(100_000_000_u64),
        )],
    )]);
    reply(deps.as_mut(), env.clone(), OK_DEPOSIT_REPLY).unwrap();
    deps.querier.with_base(MockQuerier::new(&[(
        MOCK_CONTRACT_ADDR,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(1_000_000_000_u64),
        }],
    )]));

    // 420 blocks later:
    // UST/aUST exchange rate: say 1.00001592106642
    // balance: 100,000,000 uaUST = ~100,001,592 uusd
    env.block.height += 420;

    // Redeem 100,000,000 ualiceUST
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("user1", &[]),
        ExecuteMsg::RedeemStable {
            recipient: None,
            burn_amount: Uint128::from(100_000_000_u64),
        },
    )
    .unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::reply_always(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "aterra_token_addr".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: "money_market_addr".to_string(),
                    amount: Uint128::from(100_000_000_u64),
                    msg: to_binary(&MarketCw20HookMsg::RedeemStable {}).unwrap(),
                })
                .unwrap(),
            }),
            2
        )]
    );
    deps.querier.with_base(MockQuerier::new(&[(
        MOCK_CONTRACT_ADDR,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(1_100_001_592_u64),
        }],
    )]));

    // Anchor redeem callback
    let res = reply(deps.as_mut(), env.clone(), OK_REDEEM_REPLY).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "user1".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(100_001_592_u64),
            }],
        }))]
    );

    // Check balance is 0
    let res = query(
        deps.as_ref(),
        env,
        QueryMsg::Balance {
            address: String::from("user1"),
        },
    )
    .unwrap();
    let value: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::zero(), value.balance);
}

#[test]
fn redeem_after_interest_balance_too_low() {
    let mut deps = mock_dependencies(&[]);
    instantiate_contract(deps.as_mut());

    // Deposit 100,000,000 uusd -> 100,000,000 uaUST
    let mut env = mock_env();
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("user1", &coins(100_000_000, "uusd")),
        ExecuteMsg::DepositStable { recipient: None },
    )
    .unwrap();

    // 420 blocks later:
    // UST/aUST exchange rate: say 1.00001592106642
    // balance: 100,000,000 ualiceUST = ~100,001,592 uusd
    env.block.height += 420;

    // Redeem should error: only 100,000,000 ualiceUST available
    let _res = execute(
        deps.as_mut(),
        env,
        mock_info("user1", &[]),
        ExecuteMsg::RedeemStable {
            recipient: None,
            burn_amount: Uint128::from(100_000_001_u64),
        },
    )
    .unwrap_err();
}

#[test]
fn redeem_after_interest_with_fee() {
    let mut deps = mock_dependencies(&[]);

    // Instantiate contract with 0.5% fee
    let instantiate_msg = InstantiateMsg {
        owner: "owner".to_string(),
        name: String::from("Alice Terra USD"),
        symbol: String::from("aliceUST"),
        decimals: 6,
        stable_denom: String::from("uusd"),
        money_market_addr: String::from("money_market_addr"),
        aterra_token_addr: String::from("aterra_token_addr"),
        redeem_fee_ratio: Decimal256::from_str("0.005").unwrap(),
        redeem_fee_cap: Uint128::MAX,
    };
    let env = mock_env();
    let info = mock_info("owner", &[]);
    instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

    // Deposit 100,000,000 uusd -> 100,000,000 uaUST
    let mut env = mock_env();
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("user1", &coins(100_000_000, "uusd")),
        ExecuteMsg::DepositStable { recipient: None },
    )
    .unwrap();
    deps.querier.with_token_balances(&[(
        &"aterra_token_addr".to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(100_000_000_u64),
        )],
    )]);
    reply(deps.as_mut(), env.clone(), OK_DEPOSIT_REPLY).unwrap();
    deps.querier.with_base(MockQuerier::new(&[(
        MOCK_CONTRACT_ADDR,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(1_000_000_000_u64),
        }],
    )]));

    // 420 blocks later:
    // UST/aUST exchange rate: say 1.00001592106642
    // balance: 100,000,000 uaUST = ~100,001,592 uusd
    env.block.height += 420;

    // Redeem 100,000,000 ualiceUST
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("user1", &[]),
        ExecuteMsg::RedeemStable {
            recipient: None,
            burn_amount: Uint128::from(100_000_000_u64),
        },
    )
    .unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::reply_always(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "aterra_token_addr".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: "money_market_addr".to_string(),
                    amount: Uint128::from(99_500_000_u64),
                    msg: to_binary(&MarketCw20HookMsg::RedeemStable {}).unwrap(),
                })
                .unwrap(),
            }),
            2
        )]
    );
    deps.querier.with_base(MockQuerier::new(&[(
        MOCK_CONTRACT_ADDR,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(1_000_000_000_u64 + 99_501_584_u64),
        }],
    )]));

    // Anchor redeem callback
    let res = reply(deps.as_mut(), env.clone(), OK_REDEEM_REPLY).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "user1".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(99_501_584_u64),
            }],
        }))]
    );

    // Check balance of user is 0
    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Balance {
            address: String::from("user1"),
        },
    )
    .unwrap();
    let value: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::zero(), value.balance);

    // Check balance of owner for redeem fee
    let query_msg = QueryMsg::Balance {
        address: "owner".to_string(),
    };
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let value: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(Uint128::from(500_000_u64), value.balance);
}
