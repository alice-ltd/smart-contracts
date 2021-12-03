use cosmwasm_std::{Addr, Binary, Storage};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use cw0::{Duration, Expiration};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &[u8] = b"config";
pub static CONTRACTS_KEY: &[u8] = b"contracts";
pub static TIMELOCKS_KEY: &[u8] = b"timelocks";
pub static MIGRATE_KEY: &[u8] = b"migrate_timelocks";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub timelock_duration: Duration,
}

pub fn config_mut(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn contracts_mut(storage: &mut dyn Storage) -> Bucket<bool> {
    bucket(storage, CONTRACTS_KEY)
}

pub fn contracts_read(storage: &dyn Storage) -> ReadonlyBucket<bool> {
    bucket_read(storage, CONTRACTS_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateTimelock {
    pub expiration: Expiration,
    pub new_code_id: u64,
    pub msg: Binary,
}

pub fn migrate_timelocks_mut(storage: &mut dyn Storage) -> Bucket<MigrateTimelock> {
    Bucket::multilevel(storage, &[TIMELOCKS_KEY, MIGRATE_KEY])
}

pub fn migrate_timelocks_read(storage: &dyn Storage) -> ReadonlyBucket<MigrateTimelock> {
    ReadonlyBucket::multilevel(storage, &[TIMELOCKS_KEY, MIGRATE_KEY])
}
