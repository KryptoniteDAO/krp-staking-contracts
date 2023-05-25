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

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::global::{execute_swap, execute_update_global_index};
use crate::state::{read_config, read_state, store_config, store_state, Config, State};
use crate::user::{
    execute_claim_rewards, execute_decrease_balance, execute_increase_balance,
    query_accrued_rewards, query_holder, query_holders,
};
use cosmwasm_std::{
    to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use basset::reward::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StateResponse,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let conf = Config {
        owner: deps.api.addr_canonicalize(&info.sender.to_string())?,
        hub_contract: deps.api.addr_canonicalize(&msg.hub_contract)?,
        reward_denom: msg.reward_denom,
        swap_contract: deps.api.addr_canonicalize(&msg.swap_contract)?,
        swap_denoms: msg.swap_denoms,
    };

    store_config(deps.storage, &conf)?;
    store_state(
        deps.storage,
        &State {
            global_index: Decimal::zero(),
            total_balance: Uint128::zero(),
            prev_reward_balance: Uint128::zero(),
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::ClaimRewards { recipient } => execute_claim_rewards(deps, env, info, recipient),
        ExecuteMsg::SwapToRewardDenom {} => execute_swap(deps, env, info),
        ExecuteMsg::UpdateGlobalIndex {} => execute_update_global_index(deps, env, info),
        ExecuteMsg::IncreaseBalance { address, amount } => {
            execute_increase_balance(deps, env, info, address, amount)
        }
        ExecuteMsg::DecreaseBalance { address, amount } => {
            execute_decrease_balance(deps, env, info, address, amount)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::AccruedRewards { address } => to_binary(&query_accrued_rewards(deps, address)?),
        QueryMsg::Holder { address } => to_binary(&query_holder(deps, address)?),
        QueryMsg::Holders { start_after, limit } => {
            to_binary(&query_holders(deps, start_after, limit)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        hub_contract: deps.api.addr_humanize(&config.hub_contract)?.to_string(),
        reward_denom: config.reward_denom,
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
    })
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state: State = read_state(deps.storage)?;
    Ok(StateResponse {
        global_index: state.global_index,
        total_balance: state.total_balance,
        prev_reward_balance: state.prev_reward_balance,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
