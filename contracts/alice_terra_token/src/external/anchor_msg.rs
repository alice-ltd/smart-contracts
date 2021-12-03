use cosmwasm_bignumber::Uint256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// For interacting with Anchor contracts

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketExecuteMsg {
    DepositStable {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketCw20HookMsg {
    RedeemStable {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketQueryMsg {
    EpochState {
        block_height: Option<u64>,
        distributed_interest: Option<Uint256>,
    },
}
