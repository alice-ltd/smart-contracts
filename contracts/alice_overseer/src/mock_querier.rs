use std::collections::HashMap;

use cosmwasm_std::testing::MockQuerier;
use cosmwasm_std::{
    from_slice, to_binary, ContractResult, Querier, QuerierResult, QueryRequest, SystemError,
    SystemResult,
};
use terra_cosmwasm::{ContractInfoResponse, TerraQuery, TerraQueryWrapper, TerraRoute};

// Mock querier for contract info
// Reference: https://github.com/Anchor-Protocol/money-market-contracts/blob/61918b1f348d1ee2cc8271ce79ff9d4486bd0174/contracts/overseer/src/testing/mock_querier.rs

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    contract_info_querier: ContractInfoQuerier,
}

#[derive(Clone, Default)]
pub struct ContractInfoQuerier {
    // {contract_addr: ContractInfoResponse}
    contract_info_map: HashMap<String, ContractInfoResponse>,
}

impl ContractInfoQuerier {
    pub fn new(contract_info: &[(&String, &ContractInfoResponse)]) -> Self {
        ContractInfoQuerier {
            contract_info_map: contract_info_to_map(contract_info),
        }
    }
}

pub(crate) fn contract_info_to_map(
    epoch_state: &[(&String, &ContractInfoResponse)],
) -> HashMap<String, ContractInfoResponse> {
    let mut contract_info_map: HashMap<String, ContractInfoResponse> = HashMap::new();
    for (market_contract, epoch_state) in epoch_state.iter() {
        contract_info_map.insert((*market_contract).clone(), (*epoch_state).clone());
    }
    contract_info_map
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
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(TerraQueryWrapper { route, query_data }) => {
                if &TerraRoute::Wasm == route {
                    match query_data {
                        TerraQuery::ContractInfo { contract_address } => match self
                            .contract_info_querier
                            .contract_info_map
                            .get(contract_address)
                        {
                            Some(v) => SystemResult::Ok(ContractResult::from(to_binary(&v))),
                            None => SystemResult::Err(SystemError::InvalidRequest {
                                error: "No contract info exists".to_string(),
                                request: contract_address.as_bytes().into(),
                            }),
                        },
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            contract_info_querier: ContractInfoQuerier::default(),
        }
    }

    #[allow(dead_code)]
    pub fn with_contract_info(&mut self, contract_info: &[(&String, &ContractInfoResponse)]) {
        self.contract_info_querier = ContractInfoQuerier::new(contract_info);
    }
}
