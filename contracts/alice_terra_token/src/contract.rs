use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20_base::contract::{execute_send, execute_transfer, query_balance, query_token_info};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};

use crate::error::ContractError;
use crate::execute::{
    execute_deposit_stable, execute_deposit_stable_authorized, execute_redeem_stable,
    handle_reply_deposit_stable, handle_reply_redeem_stable, DEPOSIT_STABLE_REPLY_ID,
    REDEEM_STABLE_REPLY_ID,
};
use crate::migrate::migrate_config;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::query_relay_nonce;
use crate::relay::execute_relay;
use crate::state::{config_read, save_config, Config};

const CONTRACT_NAME: &str = "crates.io:alice-terra-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: Uint128::zero(),
        // set self as minter, so we can properly handle mint and burn
        mint: Some(MinterData {
            minter: env.contract.address,
            cap: None,
        }),
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    // initialize CW2
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // contract configuration information
    save_config(
        deps.storage,
        &Config {
            stable_denom: msg.stable_denom,
            owner: deps.api.addr_validate(&msg.owner)?,
            money_market_addr: deps.api.addr_validate(&msg.money_market_addr)?,
            aterra_token_addr: deps.api.addr_validate(&msg.aterra_token_addr)?,
            redeem_fee_ratio: msg.redeem_fee_ratio,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Relay {
            meta_tx,
            signature,
            public_key,
        } => execute_relay(deps, env, info, meta_tx, signature, public_key),
        ExecuteMsg::DepositStableAuthorized {
            recipient, amount, ..
        } => execute_deposit_stable_authorized(deps, env, info, recipient, amount),
        ExecuteMsg::DepositStable { recipient } => {
            execute_deposit_stable(deps, env, info, recipient)
        }
        ExecuteMsg::RedeemStable {
            burn_amount,
            recipient,
        } => execute_redeem_stable(deps, env, info, burn_amount, recipient),
        // inherited from cw20-base
        ExecuteMsg::Transfer { recipient, amount } => {
            Ok(execute_transfer(deps, env, info, recipient, amount)?)
        }
        ExecuteMsg::Burn { amount } => execute_redeem_stable(
            deps,
            env,
            info.clone(),
            amount,
            Some(info.sender.to_string()),
        ),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => Ok(execute_send(deps, env, info, contract, amount, msg)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match (reply.id, reply.result) {
        (REDEEM_STABLE_REPLY_ID, result) => handle_reply_redeem_stable(deps, env, result),
        (DEPOSIT_STABLE_REPLY_ID, result) => handle_reply_deposit_stable(deps, env, result),
        _ => Err(StdError::generic_err("invalid reply id or result").into()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::RelayNonce { address } => to_binary(&query_relay_nonce(deps, address)?),
        QueryMsg::Config {} => to_binary(&config_read(deps.storage).load()?),
        // inherited from cw20-base
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
    if let Some(symbol) = msg.symbol.clone() {
        TOKEN_INFO.update(deps.storage, |mut token_info| -> StdResult<_> {
            token_info.symbol = symbol;
            Ok(token_info)
        })?;
    }

    // update CW2 version info
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    migrate_config(deps, msg)?;

    Ok(Response::default())
}
