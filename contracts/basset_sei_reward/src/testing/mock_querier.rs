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

use std::collections::HashMap;

use basset::common::{QueryTaxWrapper, TaxRateResponse, QueryTaxMsg, TaxCapResponse};
use basset::hub::ConfigResponse;
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_json, to_json_binary, Coin, ContractResult, OwnedDeps, Querier, QuerierResult, QueryRequest,
    SystemError, SystemResult, WasmQuery, Decimal, Uint128,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    base: MockQuerier<QueryTaxWrapper>,
    tax_querier: TaxQuerier,
    token_querier: TokenQuerier,
    oracle_price_querier: OraclePriceQuerier,
    collateral_querier: CollateralQuerier,
}


#[derive(Clone, Default)]
pub struct TokenQuerier {
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    pub fn new(balances: &[(&String, &[(&String, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

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
pub struct CollateralQuerier {
    collaterals: HashMap<String, Decimal256>,
}

impl CollateralQuerier {
    pub fn new(collaterals: &[(&String, &Decimal256)]) -> Self {
        CollateralQuerier {
            collaterals: collaterals_to_map(collaterals),
        }
    }
}

pub(crate) fn collaterals_to_map(
    collaterals: &[(&String, &Decimal256)],
) -> HashMap<String, Decimal256> {
    let mut collateral_map: HashMap<String, Decimal256> = HashMap::new();
    for (col, max_ltv) in collaterals.iter() {
        collateral_map.insert((*col).clone(), **max_ltv);
    }
    collateral_map
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
/// Query oracle price to oracle contract
    QueryPrice { asset: String },
    Whitelist {
        collateral_token: Option<String>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PriceResponse {
    pub asset: String,
    pub emv_price: Decimal256,
    pub emv_price_raw: i64,
    pub price: Decimal256,
    pub price_raw: i64,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistResponseElem {
    pub name: String,
    pub symbol: String,
    pub max_ltv: Decimal256,
    pub custody_contract: String,
    pub collateral_token: String,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistResponse {
    pub elems: Vec<WhitelistResponseElem>,
}


#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TaxQuerier {
    rate: Decimal,
    // this lets us iterate over all pairs that match the first string
    caps: HashMap<String, Uint128>,
}

impl TaxQuerier {
    pub fn new(rate: Decimal, caps: &[(&String, &Uint128)]) -> Self {
        TaxQuerier {
            rate,
            caps: caps_to_map(caps),
        }
    }
}

pub(crate) fn caps_to_map(caps: &[(&String, &Uint128)]) -> HashMap<String, Uint128> {
    let mut owner_map: HashMap<String, Uint128> = HashMap::new();
    for (denom, cap) in caps.iter() {
        owner_map.insert(denom.to_string(), **cap);
    }
    owner_map
}

#[derive(Clone, Default)]
pub struct OraclePriceQuerier {
    // this lets us iterate over all pairs that match the first string
    oracle_price: HashMap<String, (Decimal256, i64, Decimal256, i64, u64, u64)>,
}

#[allow(clippy::type_complexity)]
impl OraclePriceQuerier {
    pub fn new(oracle_price: &[(&String, &(Decimal256, i64, Decimal256, i64, u64, u64))]) -> Self {
        OraclePriceQuerier {
            oracle_price: oracle_price_to_map(oracle_price),
        }
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn oracle_price_to_map(
    oracle_price: &[(&String, &(Decimal256, i64, Decimal256, i64, u64, u64))],
) -> HashMap< String, (Decimal256, i64, Decimal256, i64, u64, u64)> {
    let mut oracle_price_map: HashMap< String, (Decimal256, i64, Decimal256, i64, u64, u64)> = HashMap::new();
    for (base_quote, oracle_price) in oracle_price.iter() {
        oracle_price_map.insert((*base_quote).clone(), **oracle_price);
    }
    oracle_price_map
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<QueryTaxWrapper> = match from_json(bin_request) {
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
    pub fn handle_query(&self, request: &QueryRequest<QueryTaxWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(QueryTaxWrapper { query_data }) => {
                match query_data {
                    QueryTaxMsg::TaxRate {} => {
                        let res = TaxRateResponse {
                            rate: self.tax_querier.rate,
                        };
                        SystemResult::Ok(ContractResult::from(to_json_binary(&res)))
                    }
                    QueryTaxMsg::TaxCap { denom } => {
                        let cap = self
                            .tax_querier
                            .caps
                            .get(denom)
                            .copied()
                            .unwrap_or_default();
                        let res = TaxCapResponse { cap };
                        SystemResult::Ok(ContractResult::from(to_json_binary(&res)))
                    }
                    _ => todo!(),
                }
            } 
            
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg,
            }) => {
                if *contract_addr == MOCK_HUB_CONTRACT_ADDR {
                    let config = ConfigResponse {
                        owner: String::from("owner1"),
                        update_reward_index_addr: String::from("update_reward_index_addr"),
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
                    SystemResult::Ok(ContractResult::from(to_json_binary(&config)))
                } else {
                    match from_json(msg).unwrap()  {
                        QueryMsg::QueryPrice { asset, } => {
                            match self.oracle_price_querier.oracle_price.get(&(asset)) {
                                Some(v) => {
                                    SystemResult::Ok(ContractResult::from(to_json_binary(&PriceResponse {
                                        asset,
                                        emv_price: v.0, 
                                        emv_price_raw: v.1,
                                        price: v.2,
                                        price_raw: v.3,
                                        last_updated_base: v.4,
                                        last_updated_quote: v.5,
                                    })))
                                }
                                None => SystemResult::Err(SystemError::InvalidRequest {
                                    error: "No oracle price exists".to_string(),
                                    request: msg.as_slice().into(),
                                }),
                            }
                        }
                        QueryMsg::Whitelist {
                            collateral_token,
                            start_after: _,
                            limit: _,
                        } => {
                            match self
                                .collateral_querier
                                .collaterals
                                .get(&collateral_token.unwrap())
                            {
                                Some(v) => {
                                    SystemResult::Ok(ContractResult::from(to_json_binary(&WhitelistResponse {
                                        elems: vec![WhitelistResponseElem {
                                            name: "name".to_string(),
                                            symbol: "symbol".to_string(),
                                            max_ltv: *v,
                                            custody_contract: "custody0000".to_string(),
                                            collateral_token: "token0000".to_string(),
                                        }],
                                    })))
                                }
                                None => SystemResult::Err(SystemError::InvalidRequest {
                                    error: "".to_string(),
                                    request: msg.as_slice().into(),
                                }),
                            }
                        }
                    }
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<QueryTaxWrapper>) -> Self {
        WasmMockQuerier { 
            base, 
            tax_querier: TaxQuerier::default(),
            token_querier: TokenQuerier::default(),
            oracle_price_querier: OraclePriceQuerier::default(),
            collateral_querier:CollateralQuerier::default(),
         }
    }

    // configure the mint whitelist mock basset
    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }
}
