use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// account that collects Anchor & relay fees
    pub owner: String,
    /// name of the derivative token
    pub name: String,
    /// symbol / ticker of the derivative token
    pub symbol: String,
    /// decimal places of the derivative token (for UI)
    pub decimals: u8,
    /// stablecoin denom (e.g. uusd)
    pub stable_denom: String,
    /// Anchor Money Market contract address
    pub money_market_addr: String,
    /// Anchor aTerra Token contract address
    pub aterra_token_addr: String,
    /// Redeem fee ratio between 0 and 1, default 0
    pub redeem_fee_ratio: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MetaTx {
    /// contract address
    pub contract: String,
    /// chain ID
    pub chain_id: String,
    /// starts at 1
    pub nonce: Uint128,
    /// must not be ExecuteMsg::Relay
    pub msg: ExecuteMsg,
    /// tip in ualiceUST that user pays, default is 0
    pub tip: Option<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Relay {
        /// MetaTx JSON serialized
        meta_tx: Binary,
        /// Serialized signature. Cosmos format (64 bytes).
        signature: Binary,
        /// Serialized compressed (33 bytes) or uncompressed (65 bytes) public key.
        public_key: Binary,
    },
    /// Use a SendAuthorization to retrieve the stablecoin amount from 'sender'
    /// Only executable by owner
    DepositStableAuthorized {
        sender: String,
        recipient: String,
        amount: Uint128,
    },
    /// MUST be the config stable denomination
    DepositStable {
        /// Default is tx sender
        recipient: Option<String>,
    },
    RedeemStable {
        /// Default is tx sender
        recipient: Option<String>,
        /// Amount in aliceUST
        burn_amount: Uint128,
    },
    /// Implements CW20. Transfer is a base message to move tokens to another account without triggering actions
    Transfer { recipient: String, amount: Uint128 },
    /// Implements CW20. Burn is a base message to destroy tokens forever
    Burn { amount: Uint128 },
    /// Implements CW20.  Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Relay nonce for the given address
    RelayNonce { address: String },
    /// Implements CW20. Returns the current balance of the given address, 0 if unset.
    Balance { address: String },
    /// Implements CW20. Returns metadata on the contract - name, decimals, supply, etc.
    TokenInfo {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    /// symbol / ticker of the derivative token
    pub symbol: Option<String>,
    /// account that collects relay tips & redeem fees
    pub owner: Option<String>,
    /// Anchor Money Market Contract address
    pub money_market_addr: Option<String>,
    /// Anchor aTerra Token Contract address
    pub aterra_token_addr: Option<String>,
    /// Redeem fee ratio between 0 and 1
    pub redeem_fee_ratio: Option<Decimal256>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExchangeRateResponse {
    /// UST/aTerra exchange rate now
    pub aterra_exchange_rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RelayNonceResponse {
    /// Current relay nonce. Add 1 in new tx.
    pub relay_nonce: Uint128,
}
