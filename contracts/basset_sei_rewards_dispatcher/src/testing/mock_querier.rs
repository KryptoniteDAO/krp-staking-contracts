// Copyright 2021 Lido
//
// Licensedicensed under the Apache License, Version 2.0 (the "License");
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

use basset::oracle_pyth::QueryMsg as PythOracleQueryMsg;
use basset::swap_ext::{AssetInfo, SimulationResponse, SwapQueryMsg};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary,from_json, to_json_binary, Coin, ContractResult, Decimal, Empty, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, Uint128, WasmQuery,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MOCK_HUB_CONTRACT_ADDR: &str = "hub";
pub const MOCK_BSEI_REWARD_CONTRACT_ADDR: &str = "rewards";
pub const MOCK_KRP_KEEPER_CONTRACT_ADDR: &str = "krp_keeper";
pub const MOCK_SWAP_CONTRACT_ADDR: &str = "swap";
pub const MOCK_ORACLE_CONTRACT_ADDR: &str = "oracle";
pub const BTOKEN_REWARD_DENOM: &str = "kusd";
pub const STTOKEN_REWARD_DENOM: &str = "usei";

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
    base: MockQuerier<Empty>,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<Empty> = match from_json(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return QuerierResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                });
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        let (sei_denom, usd_denom) = ("usei", "kusd");
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                if *contract_addr == MOCK_SWAP_CONTRACT_ADDR {
                    match from_binary(msg).unwrap() {
                        SwapQueryMsg::QuerySimulation {
                            asset_infos,
                            offer_asset,
                        } => {
                            // 1usei = 32kusd
                            // 1usdr = 2kusd
                            // 1kusd = 0.03125usei
                            if asset_infos.starts_with(&[AssetInfo::NativeToken {
                                denom: "usei".to_string(),
                            }]) && asset_infos.ends_with(&[AssetInfo::NativeToken {
                                denom: "kusd".to_string(),
                            }]) {
                                let simulation_response = SimulationResponse {
                                    return_amount: Uint128::from(offer_asset.amount.u128() * 32),
                                    spread_amount: Default::default(),
                                    commission_amount: Default::default(),
                                };
                                QuerierResult::Ok(ContractResult::from(to_json_binary(
                                    &simulation_response,
                                )))
                            } else if asset_infos.starts_with(&[AssetInfo::NativeToken {
                                denom: "usdr".to_string(),
                            }]) && asset_infos.ends_with(&[AssetInfo::NativeToken {
                                denom: "kusd".to_string(),
                            }]) {
                                let simulation_response = SimulationResponse {
                                    return_amount: Uint128::from(offer_asset.amount.u128() * 2),
                                    spread_amount: Default::default(),
                                    commission_amount: Default::default(),
                                };
                                QuerierResult::Ok(ContractResult::from(to_json_binary(
                                    &simulation_response,
                                )))
                            } else if asset_infos.starts_with(&[AssetInfo::NativeToken {
                                denom: "kusd".to_string(),
                            }]) && asset_infos.ends_with(&[AssetInfo::NativeToken {
                                denom: "usei".to_string(),
                            }]) {
                                let simulation_response = SimulationResponse {
                                    return_amount: Uint128::from(offer_asset.amount.u128() / 32),
                                    spread_amount: Default::default(),
                                    commission_amount: Default::default(),
                                };
                                QuerierResult::Ok(ContractResult::from(to_json_binary(
                                    &simulation_response,
                                )))
                            } else {
                                panic!("UNSUPPORTED");
                            }
                        }
                        SwapQueryMsg::QueryReverseSimulation {
                            asset_infos: _,
                            ask_asset: _,
                        } => {
                            panic!("UNSUPPORTED");
                        }
                        SwapQueryMsg::QueryCumulativePrices { asset_infos: _ } => {
                            panic!("UNSUPPORTED");
                        }
                    }
                } else if *contract_addr == MOCK_ORACLE_CONTRACT_ADDR {
                    match from_binary(msg).unwrap() {
                        PythOracleQueryMsg::QueryExchangeRateByAssetLabel {
                            base_label,
                            quote_label,
                        } => {
                            // 1usei = 32kusd
                            if base_label == sei_denom && quote_label == usd_denom {
                                let rates =
                                    Decimal::from_ratio(Uint128::from(32u64), Uint128::from(1u64));
                                QuerierResult::Ok(ContractResult::from(to_json_binary(&rates)))
                            } else {
                                panic!("UNSUPPORTED DENOM: {}", base_label);
                            }
                        }
                    }
                } else {
                    unimplemented!()
                }
            }
            QueryRequest::Wasm(WasmQuery::Raw {
                contract_addr: _,
                key: _,
            }) => unimplemented!(),
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier { base }
    }
}

/// ExchangeRatesResponse is data format returned from OracleRequest::ExchangeRates query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExchangeRatesResponse {
    pub base_denom: String,
    pub exchange_rates: Vec<ExchangeRateItem>,
}

/// ExchangeRateItem is data format returned from OracleRequest::ExchangeRates query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExchangeRateItem {
    pub quote_denom: String,
    pub exchange_rate: Decimal,
}
