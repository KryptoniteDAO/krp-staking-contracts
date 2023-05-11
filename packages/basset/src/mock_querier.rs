use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, to_binary, AllBalanceResponse, Api, BalanceResponse, BankQuery, CanonicalAddr,
    Coin, ContractResult, Decimal, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError,
    SystemResult, Uint128, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use std::collections::HashMap;

use crate::hub::Config;
use schemars::JsonSchema;
use sei_cosmwasm::SeiQueryWrapper;
use serde::{Deserialize, Serialize};

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

pub struct WasmMockQuerier {
    base: MockQuerier<SeiQueryWrapper>,
    tax_querier: TaxQuerier,
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
            //     panic!("DO NOT ENTER HERE")
            // }
            QueryRequest::Wasm(WasmQuery::Raw {
                contract_addr: _,
                key,
            }) => {
                let prefix_config = to_length_prefixed(b"config").to_vec();
                let api: MockApi = MockApi::default();

                if key.as_slice().to_vec() == prefix_config {
                    let config = Config {
                        creator: api.addr_canonicalize(&String::from("owner1")).unwrap(),
                        reward_dispatcher_contract: Some(
                            api.addr_canonicalize(&String::from("reward_dispatcher"))
                                .unwrap(),
                        ),
                        validators_registry_contract: Some(
                            api.addr_canonicalize(&String::from("validators")).unwrap(),
                        ),
                        bsei_token_contract: Some(
                            api.addr_canonicalize(&String::from("bsei_token")).unwrap(),
                        ),
                        airdrop_registry_contract: Some(
                            api.addr_canonicalize(&String::from("airdrop")).unwrap(),
                        ),
                        stable_contract: Some(
                            api.addr_canonicalize(&String::from("stable")).unwrap(),
                        ),
                        stsei_token_contract: Some(
                            api.addr_canonicalize(&String::from("stsei_token")).unwrap(),
                        ),
                        rewards_contract: Some(
                            api.addr_canonicalize(&String::from("rewards")).unwrap(),
                        ),
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(&config)))
                } else {
                    unimplemented!()
                }
            }
            QueryRequest::Bank(BankQuery::AllBalances { address }) => {
                if address == &String::from("reward") {
                    let mut coins: Vec<Coin> = vec![];
                    let sei = Coin {
                        denom: "usei".to_string(),
                        amount: Uint128::new(1000u128),
                    };
                    coins.push(sei);
                    let krt = Coin {
                        denom: "ukrt".to_string(),
                        amount: Uint128::new(1000u128),
                    };
                    coins.push(krt);
                    let all_balances = AllBalanceResponse { amount: coins };
                    SystemResult::Ok(ContractResult::from(to_binary(&all_balances)))
                } else {
                    unimplemented!()
                }
            }
            QueryRequest::Bank(BankQuery::Balance { address, denom }) => {
                if address == &String::from("reward") && denom == "uusd" {
                    let bank_res = BalanceResponse {
                        amount: Coin {
                            amount: Uint128::new(2000u128),
                            denom: denom.to_string(),
                        },
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(&bank_res)))
                } else {
                    unimplemented!()
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<SeiQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            tax_querier: TaxQuerier::default(),
        }
    }

    // configure the tax mock querier
    pub fn with_tax(&mut self, rate: Decimal, caps: &[(&String, &Uint128)]) {
        self.tax_querier = TaxQuerier::new(rate, caps);
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
    pub mint: Option<MinterData>,
    pub owner: CanonicalAddr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MinterData {
    pub minter: CanonicalAddr,
    /// cap is how many more tokens can be issued by the minter
    pub cap: Option<Uint128>,
}
