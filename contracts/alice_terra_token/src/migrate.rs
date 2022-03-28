use crate::msg::MigrateMsg;
use crate::state::{save_config, Config, CONFIG_KEY};
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Addr, DepsMut, StdResult, Storage, Uint128};
use cosmwasm_storage::{singleton_read, ReadonlySingleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Legacy config object with optional fields
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyConfig {
    /// stablecoin denomination, probably `uusd`
    pub stable_denom: String,
    /// account that collects Anchor & relay fees
    pub owner: Addr,
    /// Anchor Money Market Contract address
    pub money_market_addr: Addr,
    /// Anchor aTerra Token Contract address
    pub aterra_token_addr: Addr,
    /// Redeem fee ratio between 0 and 1
    pub redeem_fee_ratio: Option<Decimal256>,
    /// Redeem fee cap (in stablecoin denom)
    pub redeem_fee_cap: Option<Uint128>,
}

fn legacy_config_read(storage: &dyn Storage) -> ReadonlySingleton<LegacyConfig> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn migrate_config(deps: DepsMut, msg: MigrateMsg) -> StdResult<()> {
    let mut legacy_config = legacy_config_read(deps.storage).load()?;

    if let Some(owner) = msg.owner {
        legacy_config.owner = deps.api.addr_validate(&owner)?;
    }

    if let Some(money_market_addr) = msg.money_market_addr {
        legacy_config.money_market_addr = deps.api.addr_validate(&money_market_addr)?;
    }

    if let Some(aterra_token_addr) = msg.aterra_token_addr {
        legacy_config.aterra_token_addr = deps.api.addr_validate(&aterra_token_addr)?;
    }

    if let Some(redeem_fee_ratio) = msg.redeem_fee_ratio {
        legacy_config.redeem_fee_ratio = Some(redeem_fee_ratio);
    }

    if let Some(redeem_fee_cap) = msg.redeem_fee_cap {
        legacy_config.redeem_fee_cap = Some(redeem_fee_cap);
    }

    save_config(
        deps.storage,
        &Config {
            stable_denom: legacy_config.stable_denom,
            owner: legacy_config.owner,
            money_market_addr: legacy_config.money_market_addr,
            aterra_token_addr: legacy_config.aterra_token_addr,
            redeem_fee_ratio: legacy_config
                .redeem_fee_ratio
                .unwrap_or_else(Decimal256::zero),
            redeem_fee_cap: legacy_config.redeem_fee_cap.unwrap_or_else(Uint128::zero),
        },
    )?;

    Ok(())
}
