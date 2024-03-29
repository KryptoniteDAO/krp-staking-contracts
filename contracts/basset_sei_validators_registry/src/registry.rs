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

use cosmwasm_std::StdResult;
use cosmwasm_std::Storage;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::CanonicalAddr;
use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, Map};

pub static CONFIG: Item<Config> = Item::new("config");

pub static REGISTRY: Map<&[u8], Validator> = Map::new("validators_registry");

pub static NEWOWNERADDR: Item<NewOwnerAddr> = Item::new("newowneraddr");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub hub_contract: CanonicalAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Validator {
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValidatorResponse {
    #[serde(default)]
    pub total_delegated: Uint128,

    pub address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NewOwnerAddr {
    pub new_owner_addr: CanonicalAddr, 
}

pub fn store_new_owner(storage: &mut dyn Storage, data: &NewOwnerAddr) -> StdResult<()> {
    NEWOWNERADDR.save(storage, data)?;
    Ok(())
}

pub fn read_new_owner(storage: &dyn Storage) -> StdResult<NewOwnerAddr> {
    NEWOWNERADDR.load(storage)
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NewOwnerResponse {
    pub new_owner: String,
}
