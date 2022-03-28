use cosmwasm_bignumber::{Decimal256, Uint256};
use std::collections::HashMap;

use crate::anchor::{MarketEpochStateResponse, MarketQueryMsg};
use cosmwasm_std::testing::MockQuerier;
use cosmwasm_std::{
    from_binary, from_slice, to_binary, ContractResult, Decimal, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use cw20::{BalanceResponse, Cw20QueryMsg};
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};

// Mock querier for Anchor money market contract and aTerra token contract queries
// Reference: https://github.com/Anchor-Protocol/money-market-contracts/blob/61918b1f348d1ee2cc8271ce79ff9d4486bd0174/contracts/overseer/src/testing/mock_querier.rs

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    tax_querier: TaxQuerier,
    epoch_state_querier: EpochStateQuerier,
    token_querier: TokenQuerier,
}

#[derive(Clone, Default)]
pub struct TaxQuerier {
    rate: Decimal,
    // this lets us iterate over all pairs that match the first string
    caps: HashMap<String, Uint128>,
}

#[allow(dead_code)]
impl TaxQuerier {
    pub fn new(rate: Decimal, caps: &[(&String, &Uint128)]) -> Self {
        TaxQuerier {
            rate,
            caps: caps_to_map(caps),
        }
    }
}

#[allow(dead_code)]
pub(crate) fn caps_to_map(caps: &[(&String, &Uint128)]) -> HashMap<String, Uint128> {
    let mut owner_map: HashMap<String, Uint128> = HashMap::new();
    for (denom, cap) in caps.iter() {
        owner_map.insert(denom.to_string(), **cap);
    }
    owner_map
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    // {contract_addr: {address: balance)}
    balances: HashMap<String, HashMap<String, Uint128>>,
}

#[allow(dead_code)]
impl TokenQuerier {
    pub fn new(balances: &[(&String, &[(&String, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

#[allow(dead_code)]
pub(crate) fn balances_to_map(
    balances: &[(&String, &[(&String, &Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut balances_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (contract_addr, balances) in balances.iter() {
        let mut contract_balances_map: HashMap<String, Uint128> = HashMap::new();
        for (addr, balance) in balances.iter() {
            contract_balances_map.insert(addr.to_string(), **balance);
        }

        balances_map.insert(contract_addr.to_string(), contract_balances_map);
    }
    balances_map
}

#[derive(Clone, Default)]
pub struct EpochStateQuerier {
    // this lets us iterate over all pairs that match the first string
    // {contract_addr: (aterra_supply, exchange_rate)}
    epoch_state: HashMap<String, (Uint256, Decimal256)>,
}

impl EpochStateQuerier {
    pub fn new(epoch_state: &[(&String, &(Uint256, Decimal256))]) -> Self {
        EpochStateQuerier {
            epoch_state: epoch_state_to_map(epoch_state),
        }
    }
}

pub(crate) fn epoch_state_to_map(
    epoch_state: &[(&String, &(Uint256, Decimal256))],
) -> HashMap<String, (Uint256, Decimal256)> {
    let mut epoch_state_map: HashMap<String, (Uint256, Decimal256)> = HashMap::new();
    for (market_contract, epoch_state) in epoch_state.iter() {
        epoch_state_map.insert((*market_contract).clone(), **epoch_state);
    }
    epoch_state_map
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                });
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(TerraQueryWrapper { route, query_data }) => {
                if &TerraRoute::Treasury == route {
                    match query_data {
                        TerraQuery::TaxRate {} => {
                            let res = TaxRateResponse {
                                rate: self.tax_querier.rate,
                            };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        TerraQuery::TaxCap { denom } => {
                            let cap = self
                                .tax_querier
                                .caps
                                .get(denom)
                                .copied()
                                .unwrap_or_default();
                            let res = TaxCapResponse { cap };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => match from_binary(msg) {
                Ok(MarketQueryMsg::EpochState {
                    block_height: _,
                    distributed_interest: _,
                }) => match self.epoch_state_querier.epoch_state.get(contract_addr) {
                    Some(v) => SystemResult::Ok(ContractResult::from(to_binary(
                        &MarketEpochStateResponse {
                            aterra_supply: v.0,
                            exchange_rate: v.1,
                        },
                    ))),
                    None => SystemResult::Err(SystemError::InvalidRequest {
                        error: "No epoch state exists".to_string(),
                        request: msg.as_slice().into(),
                    }),
                },
                _ => match from_binary(msg) {
                    Ok(Cw20QueryMsg::Balance { address }) => {
                        match self.token_querier.balances.get(contract_addr) {
                            Some(balances) => match balances.get(address.as_str()) {
                                Some(balance) => SystemResult::Ok(ContractResult::from(to_binary(
                                    &BalanceResponse { balance: *balance },
                                ))),
                                None => SystemResult::Err(SystemError::InvalidRequest {
                                    error: "Balance not found".to_string(),
                                    request: msg.clone(),
                                }),
                            },
                            None => SystemResult::Err(SystemError::InvalidRequest {
                                error: format!(
                                    "No balance info exists for the contract {}",
                                    contract_addr
                                ),
                                request: msg.clone(),
                            }),
                        }
                    }
                    _ => panic!("Unsupported Wasm query"),
                },
            },
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            tax_querier: TaxQuerier::default(),
            epoch_state_querier: EpochStateQuerier::default(),
            token_querier: TokenQuerier::default(),
        }
    }

    pub fn with_base(&mut self, base: MockQuerier<TerraQueryWrapper>) {
        self.base = base;
    }

    #[allow(dead_code)]
    pub fn with_tax(&mut self, rate: Decimal, caps: &[(&String, &Uint128)]) {
        self.tax_querier = TaxQuerier::new(rate, caps);
    }

    pub fn with_epoch_state(&mut self, epoch_state: &[(&String, &(Uint256, Decimal256))]) {
        self.epoch_state_querier = EpochStateQuerier::new(epoch_state);
    }

    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }
}
