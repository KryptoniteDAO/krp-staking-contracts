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

use basset::common::{QueryTaxWrapper, QueryTaxMsg, TaxRateResponse, TaxCapResponse};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_json, to_json_binary, to_json_vec, AllBalanceResponse, Api, BalanceResponse,
    BankQuery, Coin, ContractResult, CustomQuery, Empty, FullDelegation, OwnedDeps, Querier,
    QuerierResult, QueryRequest, StdError, StdResult, SystemError, SystemResult, Uint128,
    Validator, WasmQuery, Addr, Decimal,
};
use cosmwasm_storage::to_length_prefixed;
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg, TokenInfoResponse};
use serde::de::DeserializeOwned;

use basset::hub::Config;
use basset_sei_validators_registry::registry::ValidatorResponse as RegistryValidator;

pub const VALIDATORS_REGISTRY: &str = "validators_registry";

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


#[derive(Clone, Default)]
pub struct TaxQuerier {
    rate: Decimal,
    caps: HashMap<String, Uint128>,
}

impl TaxQuerier {
    pub fn _new(rate: Decimal, caps: &[(&String, &Uint128)]) -> Self {
        TaxQuerier {
            rate,
            caps: _caps_to_map(caps),
        }
    }
}


pub(crate) fn _caps_to_map(caps: &[(&String, &Uint128)]) -> HashMap<String, Uint128> {
    let mut owner_map: HashMap<String, Uint128> = HashMap::new();
    for (denom, cap) in caps.iter() {
        owner_map.insert(denom.to_string(), **cap);
    }
    owner_map
}

pub struct WasmMockQuerier {
    base: MockQuerier<QueryTaxWrapper>,
    token_querier: TokenQuerier,
    balance_querier: BalanceQuerier,
    tax_querier: TaxQuerier,
    validators: Vec<RegistryValidator>,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<QueryTaxWrapper> = match from_json(bin_request) {
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
                      _ => panic!("DO NOT ENTER HERE"),
                    }
                } 
            
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                if contract_addr == VALIDATORS_REGISTRY {
                    let mut validators = self.validators.clone();
                    validators.sort_by(|v1, v2| v1.total_delegated.cmp(&v2.total_delegated));
                    return SystemResult::Ok(ContractResult::from(to_json_binary(&validators)));
                }
                match from_json(msg).unwrap() {
                    Cw20QueryMsg::TokenInfo {} => {
                        let balances: &HashMap<String, Uint128> =
                            match self.token_querier.balances.get(contract_addr) {
                                Some(balances) => balances,
                                None => {
                                    return SystemResult::Err(SystemError::InvalidRequest {
                                        error: format!(
                                            "No balance info exists for the contract {}",
                                            contract_addr
                                        ),
                                        request: msg.as_slice().into(),
                                    });
                                }
                            };
                        let mut total_supply = Uint128::zero();

                        for balance in balances {
                            total_supply += *balance.1;
                        }
                        // let token_inf: TokenInfo = TokenInfo {
                        //     name: "bsei".to_string(),
                        //     symbol: "BSEI".to_string(),
                        //     decimals: 6,
                        //     total_supply,
                        //     mint: Some(MinterData {
                        //         minter: Addr::unchecked(MOCK_CONTRACT_ADDR),
                        //         cap: None,
                        //     }),
                        // };
                        let token_inf: TokenInfoResponse = TokenInfoResponse {
                            name: "bsei".to_string(),
                            symbol: "BSEI".to_string(),
                            decimals: 6,
                            total_supply,
                        };

                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&token_inf).unwrap()))
                    }
                    Cw20QueryMsg::Balance { address } => {
                        let balances: &HashMap<String, Uint128> =
                            match self.token_querier.balances.get(contract_addr) {
                                Some(balances) => balances,
                                None => {
                                    return SystemResult::Err(SystemError::InvalidRequest {
                                        error: format!(
                                            "No balance info exists for the contract {}",
                                            contract_addr
                                        ),
                                        request: msg.as_slice().into(),
                                    });
                                }
                            };

                        let balance = match balances.get(&address) {
                            Some(v) => *v,
                            None => {
                                return SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&Cw20BalanceResponse {
                                        balance: Uint128::zero(),
                                    })
                                    .unwrap(),
                                ));
                            }
                        };

                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&Cw20BalanceResponse { balance }).unwrap(),
                        ))
                    }
                    _ => panic!("DO NOT ENTER HERE"),
                }
            }
            QueryRequest::Wasm(WasmQuery::Raw {
                contract_addr: _,
                key,
            }) => {
                let prefix_config = to_length_prefixed(b"config").to_vec();
                let api: MockApi = MockApi::default();

                if key.as_slice().to_vec() == prefix_config {
                    let config = Config {
                        creator: api.addr_canonicalize(&String::from("owner1")).unwrap(),
                        update_reward_index_addr: api.addr_canonicalize(&String::from("update_reward_index_addr")).unwrap(),
                        reward_dispatcher_contract: Some(
                            api.addr_canonicalize(&String::from("reward_dispatcher"))
                                .unwrap(),
                        ),
                        bsei_token_contract: Some(
                            api.addr_canonicalize(&String::from("token")).unwrap(),
                        ),
                        validators_registry_contract: Some(
                            api.addr_canonicalize(&String::from("validators")).unwrap(),
                        ),
                        stsei_token_contract: Some(
                            api.addr_canonicalize(&String::from("stsei_token")).unwrap(),
                        ),
                        airdrop_registry_contract: Some(
                            api.addr_canonicalize(&String::from("airdrop")).unwrap(),
                        ),
                        rewards_contract: Some(
                            api.addr_canonicalize(&String::from("rewards")).unwrap(),
                        ),
                    };
                    QuerierResult::Ok(ContractResult::from(to_json_binary(
                        &to_json_binary(&config).unwrap(),
                    )))
                } else {
                    unimplemented!()
                }
            }
            QueryRequest::Bank(BankQuery::AllBalances { address }) => {
                if address == &String::from("reward") {
                    let mut coins: Vec<Coin> = vec![];
                    let sei = Coin {
                        denom: "usei".to_string(),
                        amount: Uint128::from(1000u128),
                    };
                    coins.push(sei);
                    let krt = Coin {
                        denom: "ukrt".to_string(),
                        amount: Uint128::from(1000u128),
                    };
                    coins.push(krt);
                    let usd = Coin {
                        denom: "uusd".to_string(),
                        amount: Uint128::from(1000u128),
                    };
                    coins.push(usd);
                    let all_balances = AllBalanceResponse { amount: coins };
                    QuerierResult::Ok(ContractResult::from(to_json_binary(&all_balances)))
                } else {
                    unimplemented!()
                }
            }
            QueryRequest::Bank(BankQuery::Balance { address, denom }) => {
                if address == &String::from(MOCK_CONTRACT_ADDR) && denom == "usei" {
                    match self
                        .balance_querier
                        .balances
                        .get(&String::from(MOCK_CONTRACT_ADDR))
                    {
                        Some(coin) => {
                            QuerierResult::Ok(ContractResult::from(to_json_binary(&BalanceResponse {
                                amount: Coin {
                                    denom: coin.denom.clone(),
                                    amount: coin.amount,
                                },
                            })))
                        }
                        None => QuerierResult::Err(SystemError::InvalidRequest {
                            error: "balance not found".to_string(),
                            request: Default::default(),
                        }),
                    }
                } else if address == &String::from("reward") && denom == "uusd" {
                    let bank_res = BalanceResponse {
                        amount: Coin {
                            amount: Uint128::from(2000u128),
                            denom: denom.to_string(),
                        },
                    };
                    QuerierResult::Ok(ContractResult::from(to_json_binary(&bank_res)))
                } else {
                    unimplemented!()
                }
            }
            _ => self.base.handle_query(request),
        }
    }
    pub fn update_staking(
        &mut self,
        denom: &str,
        validators: &[Validator],
        delegations: &[FullDelegation],
    ) {
        self.base.update_staking(denom, validators, delegations);
    }

    pub fn query<T: DeserializeOwned>(&self, request: &QueryRequest<Empty>) -> StdResult<T> {
        self.custom_query(request)
    }

    /// Makes the query and parses the response. Also handles custom queries,
    /// so you need to specify the custom query type in the function parameters.
    /// If you are no using a custom query, just use `query` for easier interface.
    ///
    /// Any error (System Error, Error or called contract, or Parse Error) are flattened into
    /// one level. Only use this if you don't need to check the SystemError
    /// eg. If you don't differentiate between contract missing and contract returned error
    pub fn custom_query<C: CustomQuery, U: DeserializeOwned>(
        &self,
        request: &QueryRequest<C>,
    ) -> StdResult<U> {
        let raw = to_json_vec(request).map_err(|serialize_err| {
            StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
        })?;
        match self.raw_query(&raw) {
            SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
                "Querier system error: {}",
                system_err
            ))),
            SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(
                format!("Querier contract error: {}", contract_err),
            )),
            SystemResult::Ok(ContractResult::Ok(value)) => from_json(&value),
        }
    }
}

#[derive(Clone, Default)]
pub struct BalanceQuerier {
    balances: HashMap<String, Coin>,
}

impl BalanceQuerier {
    pub fn new(balances: &[(String, Coin)]) -> Self {
        BalanceQuerier {
            balances: native_balances_to_map(balances),
        }
    }
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

pub(crate) fn native_balances_to_map(balances: &[(String, Coin)]) -> HashMap<String, Coin> {
    let mut balances_map: HashMap<String, Coin> = HashMap::new();
    for (contract_addr, balances) in balances.iter() {
        let coin = Coin {
            denom: balances.clone().denom,
            amount: balances.clone().amount,
        };
        balances_map.insert(String::from(contract_addr), coin);
    }
    balances_map
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

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<QueryTaxWrapper>) -> Self {
        WasmMockQuerier {
            base,
            tax_querier: TaxQuerier::default(),
            token_querier: TokenQuerier::default(),
            balance_querier: BalanceQuerier::default(),
            validators: vec![],
        }
    }

    pub fn with_native_balances(&mut self, balances: &[(String, Coin)]) {
        self.balance_querier = BalanceQuerier::new(balances);
    }

    // configure the mint whitelist mock basset
    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }

    pub fn add_validator(&mut self, validator: RegistryValidator) {
        self.validators.push(validator);
    }
}
