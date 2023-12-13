use cosmwasm_std::Decimal;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub hub_contract: String,
    pub bsei_reward_contract: String,
    pub stsei_reward_denom: String,
    pub bsei_reward_denom: String,
    pub krp_keeper_address: String,
    pub krp_keeper_rate: Decimal,
    pub swap_contract: String,
    pub swap_denoms: Vec<String>,
    pub oracle_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NewOwnerResponse {
    pub new_owner: String,
}
