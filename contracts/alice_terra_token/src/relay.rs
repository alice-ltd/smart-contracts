use cosmwasm_std::{
    from_binary, Binary, CanonicalAddr, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use ripemd160::Ripemd160;
use sha2::{Digest, Sha256};

use crate::contract;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, MetaTx};
use crate::state::{config_read, nonces_mut};
use cw20_base::contract::execute_transfer;

pub fn verify_cosmos(
    deps: Deps,
    message: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> StdResult<bool> {
    // Hashing
    let hash = Sha256::digest(message);

    // Verification
    let result = deps
        .api
        .secp256k1_verify(hash.as_ref(), signature, public_key);
    match result {
        Ok(verifies) => Ok(verifies),
        Err(err) => Err(err.into()),
    }
}

pub fn execute_relay(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    meta_tx: Binary,
    signature: Binary,
    public_key: Binary,
) -> Result<Response, ContractError> {
    let signature_verified = verify_cosmos(deps.as_ref(), &meta_tx.0, &signature.0, &public_key.0)?;
    if !signature_verified {
        return Err(ContractError::Unauthorized {});
    }

    let meta_tx: MetaTx = from_binary(&meta_tx)?;
    let canonical_addr = public_key_to_canonical_addr(&public_key);
    let canonical_addr_slice = canonical_addr.as_slice();

    // Check contract address
    if deps.api.addr_validate(meta_tx.contract.as_str())? != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    // Check chain ID
    if meta_tx.chain_id != env.block.chain_id {
        return Err(ContractError::Unauthorized {});
    }

    // Check transaction nonce is greater than previous nonce
    let mut nonces = nonces_mut(deps.storage);
    let prev_nonce = nonces.load(canonical_addr_slice).unwrap_or_default();
    if meta_tx.nonce <= prev_nonce {
        return Err(ContractError::Unauthorized {});
    }
    nonces.save(canonical_addr_slice, &meta_tx.nonce)?;

    match meta_tx.msg {
        // Disallow recursive relay message
        ExecuteMsg::Relay { .. } => Err(ContractError::InvalidRelay {}),
        // Disallow deposit messages
        ExecuteMsg::DepositStable { .. } | ExecuteMsg::DepositStableAuthorized { .. } => {
            Err(ContractError::InvalidRelay {})
        }
        ExecuteMsg::RedeemStable { .. }
        | ExecuteMsg::Transfer { .. }
        | ExecuteMsg::Burn { .. }
        | ExecuteMsg::Send { .. } => {
            let mut as_user_info = info;
            let human_addr = deps.api.addr_humanize(&canonical_addr)?;
            as_user_info.sender = human_addr;

            // Collect tip
            let tip = meta_tx.tip.unwrap_or_default();
            if tip > Uint128::zero() {
                let owner = config_read(deps.storage).load()?.owner;
                execute_transfer(
                    deps.branch(),
                    env.clone(),
                    as_user_info.clone(),
                    owner.to_string(),
                    tip,
                )?;
            }

            // Execute msg as user
            let result = contract::execute(deps, env, as_user_info, meta_tx.msg);
            match result {
                Ok(response) => Ok(response),
                Err(err) => {
                    // if tip > 0, collect tip even when relayed msg errors
                    if tip > Uint128::zero() {
                        Ok(Response::new().add_attribute("error", err.to_string()))
                    } else {
                        Err(err)
                    }
                }
            }
        }
    }
}

fn public_key_to_canonical_addr(public_key: &Binary) -> CanonicalAddr {
    let bech32_addr = Ripemd160::digest(&Sha256::digest(&public_key.0));
    CanonicalAddr(Binary::from(bech32_addr.as_slice()))
}
