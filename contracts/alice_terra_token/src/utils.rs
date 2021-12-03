use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Coin, Deps, StdResult};

use crate::error::ContractError;

use terra_cosmwasm::TerraQuerier;

pub fn compute_tax(deps: Deps, coin: &Coin) -> StdResult<Uint256> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let tax_rate = Decimal256::from((terra_querier.query_tax_rate()?).rate);
    let tax_cap = Uint256::from((terra_querier.query_tax_cap(coin.denom.to_string())?).cap);
    let amount = Uint256::from(coin.amount);
    Ok(std::cmp::min(amount * tax_rate, tax_cap))
}

// https://github.com/Anchor-Protocol/money-market-contracts/blob/230ccf7f41fb04fff66536c48a9d397225813544/packages/moneymarket/src/querier.rs#L70
fn compute_deducted_tax(deps: Deps, coin: &Coin) -> StdResult<Uint256> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let tax_rate = Decimal256::from((terra_querier.query_tax_rate()?).rate);
    let tax_cap = Uint256::from((terra_querier.query_tax_cap(coin.denom.to_string())?).cap);
    let amount = Uint256::from(coin.amount);
    Ok(std::cmp::min(
        amount * Decimal256::one() - amount / (Decimal256::one() + tax_rate),
        tax_cap,
    ))
}

pub fn deduct_tax(deps: Deps, coin: Coin) -> StdResult<Coin> {
    let tax_amount = compute_deducted_tax(deps, &coin)?;
    Ok(Coin {
        denom: coin.denom,
        amount: (Uint256::from(coin.amount) - tax_amount).into(),
    })
}

pub fn proto_encode<M: prost::Message>(msg: M) -> Result<Vec<u8>, ContractError> {
    let mut msg_buf: Vec<u8> = vec![];
    msg.encode(&mut msg_buf)
        .map_err(|_e| ContractError::ProtoEncodeError {})?;
    Ok(msg_buf)
}
