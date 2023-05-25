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

use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub hub_contract: String,
    pub bsei_reward_contract: String,
    pub stsei_reward_denom: String,
    pub bsei_reward_denom: String,
    pub lido_fee_address: String,
    pub lido_fee_rate: Decimal,
    pub swap_contract: String,
    pub swap_denoms: Vec<String>,
    pub oracle_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SwapToRewardDenom {
        bsei_total_bonded: Uint128,
        stsei_total_bonded: Uint128,
    },
    UpdateConfig {
        owner: Option<String>,
        hub_contract: Option<String>,
        bsei_reward_contract: Option<String>,
        stsei_reward_denom: Option<String>,
        bsei_reward_denom: Option<String>,
        lido_fee_address: Option<String>,
        lido_fee_rate: Option<Decimal>,
    },
    DispatchRewards {},
    UpdateSwapContract {
        swap_contract: String,
    },
    UpdateSwapDenom {
        swap_denom: String,
        is_add: bool,
    },
    UpdateOracleContract{
        oracle_contract: String,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetBufferedRewards returns the buffered amount of bSei and stSei rewards.
    GetBufferedRewards {},
    // Config returns config
    Config {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetBufferedRewardsResponse {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
