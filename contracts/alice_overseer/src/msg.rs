use crate::state::MigrateTimelock;
use cosmwasm_std::Binary;
use cw0::Duration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub timelock_duration: Duration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Register {
        /// Contract's admin must be this overseer contract
        contract_addr: String,
    },
    /// Initiate migrate timelock for a registered contract
    InitiateMigrate {
        contract_addr: String,
        new_code_id: u64,
        msg: Binary,
    },
    /// Cancel migrate timelock
    CancelMigrate { contract_addr: String },
    /// Execute migrate on registered contract (timelock must be expired)
    Migrate { contract_addr: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub owner: Option<String>,
    pub timelock_duration: Option<Duration>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    RegisteredContracts {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    MigrateTimelock {
        contract_addr: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub timelock_duration: Duration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RegisteredContractsResponse {
    pub contracts: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateTimelockResponse {
    pub timelock: Option<MigrateTimelock>,
}
