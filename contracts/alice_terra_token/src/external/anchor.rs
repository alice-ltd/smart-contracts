use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Deps, DepsMut, QueryRequest, Response, StdResult, SubMsg,
    Uint128, WasmMsg, WasmQuery,
};

use crate::state::config_read;
use cw20::Cw20ExecuteMsg;

pub use crate::external::anchor_msg::{MarketCw20HookMsg, MarketExecuteMsg, MarketQueryMsg};

pub fn query_cw20_balance(deps: Deps, cw20_addr: Addr, addr: Addr) -> StdResult<Uint128> {
    let balance_response = deps
        .querier
        .query::<cw20::BalanceResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: cw20_addr.to_string(),
            msg: to_binary(&cw20::Cw20QueryMsg::Balance {
                address: addr.to_string(),
            })?,
        }))?;
    Ok(balance_response.balance)
}

/// Returns response with submessage to deposit stable_amount into Anchor.
/// Warning: does not account for Terra tax.
pub fn anchor_deposit_stable(
    deps: DepsMut,
    stable_amount: Uint128,
    reply_id: u64,
) -> StdResult<Response> {
    // deposit `amount` uusd into Anchor by sending a `DepositStable`
    // message to Money Market contract

    // see https://github.com/Anchor-Protocol/anchor-earn/blob/master/src/fabricators/market-deposit-stable.ts

    let config = config_read(deps.storage).load()?;

    let money_market_addr: String = config.money_market_addr.to_string();

    Ok(Response::new()
        .add_submessage(SubMsg::reply_always(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: money_market_addr,
                funds: vec![Coin {
                    denom: config.stable_denom,
                    amount: stable_amount,
                }],
                msg: to_binary(&MarketExecuteMsg::DepositStable {})?,
            }),
            reply_id,
        ))
        .add_attribute("anchor_deposit_amount", stable_amount))
}

/// Returns response with submessage to redeem aterra_amount from Anchor.
pub fn anchor_redeem_stable(
    deps: DepsMut,
    aterra_amount: Uint128,
    reply_id: u64,
) -> StdResult<Response> {
    // withdraw `amount` from Anchor by sending a `Send` message
    // to the Anchor aTerra contract with the following format

    // see https://github.com/Anchor-Protocol/anchor-earn/blob/master/src/fabricators/market-redeem-stable.ts

    let (money_market_addr, aterra_token_addr) = {
        let config = config_read(deps.storage).load()?;
        (
            config.money_market_addr.to_string(),
            config.aterra_token_addr.to_string(),
        )
    };

    Ok(Response::new()
        .add_submessage(SubMsg::reply_always(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: aterra_token_addr,
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: money_market_addr,
                    amount: aterra_amount,
                    msg: to_binary(&MarketCw20HookMsg::RedeemStable {})?,
                })?,
            }),
            reply_id,
        ))
        .add_attribute("anchor_redeem_amount", aterra_amount))
}
