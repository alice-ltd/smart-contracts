use cosmwasm_std::{Addr, BalanceResponse, BankQuery, Deps, QueryRequest, StdResult, Uint128};

use crate::msg::RelayNonceResponse;
use crate::state::nonces_read;

pub fn query_relay_nonce(deps: Deps, address: String) -> StdResult<RelayNonceResponse> {
    let canonical_addr = deps.api.addr_canonicalize(&address)?;
    let nonce = nonces_read(deps.storage)
        .may_load(canonical_addr.as_slice())?
        .unwrap_or_default();
    Ok(RelayNonceResponse { relay_nonce: nonce })
}

pub fn query_native_balance(deps: Deps, account_addr: Addr, denom: String) -> StdResult<Uint128> {
    let balance: BalanceResponse = deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address: account_addr.to_string(),
        denom,
    }))?;
    Ok(balance.amount.amount)
}
