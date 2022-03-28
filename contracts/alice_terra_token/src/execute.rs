use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    coin, BankMsg, Coin, ContractResult, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError,
    SubMsgExecutionResponse, Uint128,
};
use cw0::{may_pay, must_pay};
use cw20::BalanceResponse;
use cw20_base::contract::{execute_burn, execute_mint, execute_transfer, query_balance};

use crate::anchor::{
    anchor_deposit_stable, anchor_redeem_stable, query_aterra_exchange_rate, query_cw20_balance,
};
use crate::error::ContractError;

use crate::query::query_native_balance;
use crate::state::{
    config_read, pending_deposit_stable_mut, pending_redeem_stable_mut, Config,
    PendingDepositStable, PendingRedeemStable,
};
use crate::utils::{compute_tax, deduct_tax, proto_encode};
use cosmos_sdk_proto::cosmos::authz::v1beta1::MsgExec;
use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;

pub const DEPOSIT_STABLE_REPLY_ID: u64 = 1;
pub const REDEEM_STABLE_REPLY_ID: u64 = 2;

pub fn execute_deposit_stable_authorized(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = config_read(deps.storage).load()?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // use received funds for 2*tax:
    // 1. MsgExec(MsgSend) from user to contract
    // 2. Anchor deposit
    let received_stable_amount = may_pay(&info, config.stable_denom.as_str())?;
    let stable_coin = coin(amount.u128(), config.stable_denom.clone());
    let tax_amount = Uint256::from(2u64) * compute_tax(deps.as_ref(), &stable_coin)?;
    if Uint256::from(received_stable_amount) < tax_amount {
        return Err(ContractError::TaxFundsTooLow {});
    }

    // MsgExec is not a supported CosmosMsg, so we construct a MsgExec(MsgSend) proto manually
    let msg_exec = proto_encode(MsgExec {
        grantee: env.contract.address.to_string(),
        msgs: vec![prost_types::Any {
            type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
            value: proto_encode(MsgSend {
                from_address: recipient.clone(),
                to_address: env.contract.address.to_string(),
                amount: vec![cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                    denom: config.stable_denom,
                    amount: amount.to_string(),
                }],
            })?,
        }],
    })?;

    let deposit_res = deposit_stable(
        deps.branch(),
        env,
        info,
        Some(recipient),
        stable_coin.amount,
    )?;

    Ok(Response::new()
        .add_message(CosmosMsg::Stargate {
            type_url: "/cosmos.authz.v1beta1.MsgExec".to_string(),
            value: msg_exec.into(),
        })
        .add_attributes(deposit_res.attributes)
        .add_submessages(deposit_res.messages))
}

pub fn execute_deposit_stable(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    let config = config_read(deps.storage).load()?;

    // Only accept stable denom coins
    let received_stable_amount = must_pay(&info, config.stable_denom.as_str())?;

    // Deduct tax for Anchor deposit operation
    let stable_amount = deduct_tax(
        deps.as_ref(),
        coin(received_stable_amount.u128(), config.stable_denom),
    )?
    .amount;

    deposit_stable(deps.branch(), env, info, recipient, stable_amount)
}

pub fn deposit_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
    stable_amount: Uint128,
) -> Result<Response, ContractError> {
    let config: Config = config_read(deps.storage).load()?;

    let aterra_balance = query_cw20_balance(
        deps.as_ref(),
        config.aterra_token_addr,
        env.contract.address,
    )?;

    // Save data for reply handler
    let recipient = recipient.unwrap_or_else(|| info.sender.to_string());
    pending_deposit_stable_mut(deps.storage).save(&PendingDepositStable {
        prev_aterra_balance: aterra_balance,
        recipient: deps.api.addr_validate(recipient.as_str())?,
        stable_amount,
    })?;

    let anchor_deposit_res = anchor_deposit_stable(deps, stable_amount, DEPOSIT_STABLE_REPLY_ID)?;
    Ok(Response::new()
        .add_attributes(anchor_deposit_res.attributes)
        .add_submessages(anchor_deposit_res.messages)
        .add_attribute("stable_amount", stable_amount))
}

pub fn handle_reply_deposit_stable(
    mut deps: DepsMut,
    env: Env,
    result: ContractResult<SubMsgExecutionResponse>,
) -> Result<Response, ContractError> {
    let config: Config = config_read(deps.storage).load()?;

    // Retrieve & clear saved data
    let mut pending_deposit_stable = pending_deposit_stable_mut(deps.storage);
    let PendingDepositStable {
        prev_aterra_balance,
        recipient,
        stable_amount: _stable_amount,
    } = pending_deposit_stable.load()?;
    pending_deposit_stable.remove();

    match result {
        ContractResult::Ok(..) => {
            let new_aterra_balance = query_cw20_balance(
                deps.as_ref(),
                config.aterra_token_addr,
                env.contract.address.clone(),
            )?;

            // Difference is the aUST minted in Anchor
            let mint_amount = new_aterra_balance - prev_aterra_balance;

            // call execute_mint as contract self (no one else has permission)
            execute_mint(
                deps.branch(),
                env.clone(),
                MessageInfo {
                    sender: env.contract.address,
                    funds: vec![],
                },
                recipient.to_string(),
                mint_amount,
            )?;

            Ok(Response::new().add_attribute("mint_amount", mint_amount))
        }
        ContractResult::Err(e) => Err(ContractError::Std(StdError::generic_err(e))),
    }
}

pub fn execute_redeem_stable(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    burn_amount: Uint128,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    let config: Config = config_read(deps.storage).load()?;

    let BalanceResponse { balance } = query_balance(deps.as_ref(), info.sender.clone().into())?;
    if burn_amount > balance {
        return Err(ContractError::BalanceTooLow {});
    }

    let recipient = recipient.unwrap_or_else(|| info.sender.to_string());

    // Collect redeem fee
    let fee_amount = if recipient == config.owner {
        Uint128::zero()
    } else {
        let aterra_exchange_rate = query_aterra_exchange_rate(
            deps.as_ref(),
            env.block.height,
            Some(config.money_market_addr.to_string()),
        )?;
        Uint128::min(
            Uint128::from(config.redeem_fee_ratio * Uint256::from(burn_amount)),
            Uint128::from(Uint256::from(config.redeem_fee_cap) / aterra_exchange_rate),
        )
    };
    if fee_amount > Uint128::zero() {
        execute_transfer(
            deps.branch(),
            env.clone(),
            info.clone(),
            config.owner.to_string(),
            fee_amount,
        )?;
    }
    let final_burn_amount = burn_amount - fee_amount;

    let contract_balance =
        query_native_balance(deps.as_ref(), env.contract.address, config.stable_denom)?;

    // Save data for reply handler
    pending_redeem_stable_mut(deps.storage).save(&PendingRedeemStable {
        prev_stable_balance: contract_balance,
        sender: info.sender,
        recipient: deps.api.addr_validate(recipient.as_str())?,
        burn_amount: final_burn_amount,
    })?;

    // Redeem stable submessage
    let anchor_redeem_res =
        anchor_redeem_stable(deps.branch(), final_burn_amount, REDEEM_STABLE_REPLY_ID)?;
    Ok(Response::new()
        .add_submessages(anchor_redeem_res.messages)
        .add_attributes(anchor_redeem_res.attributes)
        .add_attribute("burn_amount", burn_amount)
        .add_attribute("final_burn_amount", final_burn_amount)
        .add_attribute("redeem_fee_amount", fee_amount))
}

pub fn handle_reply_redeem_stable(
    mut deps: DepsMut,
    env: Env,
    result: ContractResult<SubMsgExecutionResponse>,
) -> Result<Response, ContractError> {
    let config: Config = config_read(deps.storage).load()?;

    // Retrieve & clear saved data
    let mut pending_redeem_stable = pending_redeem_stable_mut(deps.storage);
    let PendingRedeemStable {
        prev_stable_balance,
        sender,
        recipient,
        burn_amount,
    } = pending_redeem_stable.load()?;
    pending_redeem_stable.remove();

    match result {
        ContractResult::Ok(..) => {
            let new_stable_balance = query_native_balance(
                deps.as_ref(),
                env.contract.address.clone(),
                config.stable_denom.clone(),
            )?;

            // Difference is the stable amount redeemed from Anchor
            let stable_amount = new_stable_balance - prev_stable_balance;

            execute_burn(
                deps.branch(),
                env,
                MessageInfo {
                    sender,
                    funds: vec![],
                },
                burn_amount,
            )?;

            Ok(Response::new()
                .add_message(CosmosMsg::Bank(BankMsg::Send {
                    to_address: recipient.to_string(),
                    amount: vec![deduct_tax(
                        deps.as_ref(),
                        Coin {
                            denom: config.stable_denom,
                            amount: stable_amount,
                        },
                    )?],
                }))
                .add_attribute("stable_amount", stable_amount))
        }
        ContractResult::Err(e) => Err(ContractError::Std(StdError::generic_err(e))),
    }
}
