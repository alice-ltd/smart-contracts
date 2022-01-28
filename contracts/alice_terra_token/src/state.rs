use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Addr, StdError, StdResult, Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &[u8] = b"config";
pub static PENDING_REDEEM_STABLE_KEY: &[u8] = b"pending_redeem_stable";
pub static PENDING_DEPOSIT_STABLE_KEY: &[u8] = b"pending_deposit_stable";
pub static NONCE_KEY: &[u8] = b"nonce";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// stablecoin denomination, probably `uusd`
    pub stable_denom: String,
    /// account that collects Anchor & relay fees
    pub owner: Addr,
    /// Anchor Money Market Contract address
    pub money_market_addr: Addr,
    /// Anchor aTerra Token Contract address
    pub aterra_token_addr: Addr,
    /// Redeem fee ratio between 0 and 1
    pub redeem_fee_ratio: Decimal256,
}

fn config_mut(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn save_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    if config.redeem_fee_ratio > Decimal256::one() {
        return Err(StdError::generic_err("redeem_fee_ratio must be between 0 and 1").into());
    }

    config_mut(storage).save(&config)?;
    Ok(())
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn nonces_mut(storage: &mut dyn Storage) -> Bucket<Uint128> {
    bucket(storage, NONCE_KEY)
}

pub fn nonces_read(storage: &dyn Storage) -> ReadonlyBucket<Uint128> {
    bucket_read(storage, NONCE_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PendingRedeemStable {
    pub prev_stable_balance: Uint128,
    pub sender: Addr,
    pub recipient: Addr,
    pub burn_amount: Uint128,
}

pub fn pending_redeem_stable_mut(storage: &mut dyn Storage) -> Singleton<PendingRedeemStable> {
    singleton(storage, PENDING_REDEEM_STABLE_KEY)
}

pub fn pending_redeem_stable_read(storage: &dyn Storage) -> ReadonlySingleton<PendingRedeemStable> {
    singleton_read(storage, PENDING_REDEEM_STABLE_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PendingDepositStable {
    pub prev_aterra_balance: Uint128,
    pub recipient: Addr,
    pub stable_amount: Uint128,
}

pub fn pending_deposit_stable_mut(storage: &mut dyn Storage) -> Singleton<PendingDepositStable> {
    singleton(storage, PENDING_DEPOSIT_STABLE_KEY)
}

pub fn pending_deposit_stable_read(
    storage: &dyn Storage,
) -> ReadonlySingleton<PendingDepositStable> {
    singleton_read(storage, PENDING_DEPOSIT_STABLE_KEY)
}
