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
use basset_sei_rewards_dispatcher::state::Config as RewardsDispatcherConfig;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, to_binary, Api, Coin, ContractResult, Decimal, Empty, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, WasmQuery,
};

pub const MOCK_OWNER_ADDR: &str = "owner";
pub const MOCK_HUB_CONTRACT_ADDR: &str = "hub";
pub const MOCK_REWARD_DISPATCHER_CONTRACT_ADDR: &str = "reward_dispatcher";
pub const MOCK_REWARDS_CONTRACT_ADDR: &str = "rewards";
pub const MOCK_TOKEN_CONTRACT_ADDR: &str = "token";
pub const MOCK_VALIDATORS_REGISTRY_ADDR: &str = "validators";
pub const MOCK_STSEI_TOKEN_CONTRACT_ADDR: &str = "stsei_token";
pub const MOCK_LIDO_FEE_ADDRESS: &str = "lido_fee";

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
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
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
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg: _,
            }) => {
                if *contract_addr == MOCK_HUB_CONTRACT_ADDR {
                    let config = ConfigResponse {
                        owner: String::from("owner1"),
                        reward_dispatcher_contract: Some(String::from(
                            MOCK_REWARD_DISPATCHER_CONTRACT_ADDR,
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
                } else if contract_addr == MOCK_REWARD_DISPATCHER_CONTRACT_ADDR {
                    let api: MockApi = MockApi::default();

                    let config = RewardsDispatcherConfig {
                        owner: api
                            .addr_canonicalize(&String::from(MOCK_OWNER_ADDR))
                            .unwrap(),
                        hub_contract: api
                            .addr_canonicalize(&String::from(MOCK_HUB_CONTRACT_ADDR))
                            .unwrap(),
                        bsei_reward_contract: api
                            .addr_canonicalize(&String::from(MOCK_REWARDS_CONTRACT_ADDR))
                            .unwrap(),
                        stsei_reward_denom: "usei".to_string(),
                        lido_fee_address: api
                            .addr_canonicalize(&String::from(MOCK_LIDO_FEE_ADDRESS))
                            .unwrap(),
                        lido_fee_rate: Decimal::from_ratio(5u128, 100u128),
                        bsei_reward_denom: "uusd".to_string(),
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
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier { base }
    }
}
