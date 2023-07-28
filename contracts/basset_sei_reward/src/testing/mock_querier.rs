// Copyright 2021 Anchor Protocol. Modified by Lido
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use basset::hub::ConfigResponse;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, to_binary, Coin, ContractResult, OwnedDeps, Querier, QuerierResult, QueryRequest,
    SystemError, SystemResult, WasmQuery,
};
use sei_cosmwasm::SeiQueryWrapper;

pub const MOCK_HUB_CONTRACT_ADDR: &str = "hub";
pub const MOCK_REWARDS_DISPATCHER_ADDR: &str = "rewards_dispatcher";
pub const MOCK_TOKEN_CONTRACT_ADDR: &str = "token";
pub const MOCK_VALIDATORS_REGISTRY_ADDR: &str = "validators";
pub const MOCK_STSEI_TOKEN_CONTRACT_ADDR: &str = "stsei_token";
pub const MOCK_SWAP_CONTRACT_ADDR: &str = "swap";

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: Default::default(),
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<SeiQueryWrapper>,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<SeiQueryWrapper> = match from_slice(bin_request) {
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
    pub fn handle_query(&self, request: &QueryRequest<SeiQueryWrapper>) -> QuerierResult {
        match &request {
            // QueryRequest::Custom(SeiQueryWrapper { route, query_data }) => {
            // if &SeiRoute::Treasury == route {
            //     match query_data {
            //         SeiQuery::TaxRate {} => {
            //             let res = TaxRateResponse {
            //                 rate: Decimal::percent(1),
            //             };
            //             SystemResult::Ok(ContractResult::from(to_binary(&res)))
            //         }
            //         SeiQuery::TaxCap { denom: _ } => {
            //             let cap = Uint128::new(1000000u128);
            //             let res = TaxCapResponse { cap };
            //             SystemResult::Ok(ContractResult::from(to_binary(&res)))
            //         }
            //         _ => panic!("DO NOT ENTER HERE"),
            //     }
            // } else
            // if &SeiRoute::Oracle == route {
            // match query_data {
            //     SeiQuery::ExchangeRates {
            //         base_denom,
            //         quote_denoms,
            //     } => {
            //         if quote_denoms.iter().any(|item| item == &"mnt".to_string()) {
            //             return SystemResult::Err(SystemError::Unknown {});
            //         }
            //         SystemResult::Ok(ContractResult::from(to_binary(
            //             &ExchangeRatesResponse {
            //                 base_denom: base_denom.to_string(),
            //                 exchange_rates: vec![ExchangeRateItem {
            //                     quote_denom: quote_denoms[0].to_string(),
            //                     exchange_rate: Decimal::from_str("22.1").unwrap(),
            //                 }],
            //             },
            //         )))
            //     }
            //     _ => panic!("DO NOT ENTER HERE"),
            // }
            // } else {
            //     panic!("DO NOT ENTER HERE")
            // }
            // }
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg: _,
            }) => {
                if *contract_addr == MOCK_HUB_CONTRACT_ADDR {
                    let config = ConfigResponse {
                        owner: String::from("owner1"),
                        reward_dispatcher_contract: Some(String::from(
                            MOCK_REWARDS_DISPATCHER_ADDR,
                        )),
                        validators_registry_contract: Some(String::from(
                            MOCK_VALIDATORS_REGISTRY_ADDR,
                        )),
                        bsei_token_contract: Some(String::from(MOCK_TOKEN_CONTRACT_ADDR)),
                        airdrop_registry_contract: Some(String::from("airdrop")),
                        stsei_token_contract: Some(String::from(MOCK_STSEI_TOKEN_CONTRACT_ADDR)),

                        token_contract: Some(String::from(MOCK_TOKEN_CONTRACT_ADDR)),
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(&config)))
                } else {
                    unimplemented!()
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<SeiQueryWrapper>) -> Self {
        WasmMockQuerier { base }
    }
}
