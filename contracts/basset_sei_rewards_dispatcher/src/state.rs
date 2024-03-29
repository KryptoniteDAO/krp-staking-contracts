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

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Decimal, StdResult, Storage};

use cw_storage_plus::Item;

pub static CONFIG: Item<Config> = Item::new("config");
pub static NEWOWNERADDR: Item<NewOwnerAddr> = Item::new("newowneraddr");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub hub_contract: CanonicalAddr,
    pub bsei_reward_contract: CanonicalAddr,
    pub stsei_reward_denom: String,
    pub bsei_reward_denom: String,
    pub krp_keeper_address: CanonicalAddr,
    pub krp_keeper_rate: Decimal,
    pub swap_contract: CanonicalAddr,
    pub swap_denoms: Vec<String>,
    pub oracle_contract: CanonicalAddr,
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


pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)?;
    Ok(())
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

