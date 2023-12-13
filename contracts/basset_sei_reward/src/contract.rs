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

use crate::error::ContractError;
use crate::global::{execute_swap, execute_update_global_index};
use crate::state::{read_config, read_state, store_config, store_state, Config, State, NewOwnerAddr, store_new_owner, read_new_owner};
use crate::user::{
    execute_claim_rewards, execute_decrease_balance, execute_increase_balance,
    query_accrued_rewards, query_holder, query_holders,
};
use cosmwasm_std::{
    to_json_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use crate::handler::{udpate_config, update_swap_denom, set_new_owner, accept_ownership};
use basset::reward::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StateResponse, NewOwnerResponse,
};

use basset::handle::optional_addr_validate;

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

    store_new_owner(
        deps.storage,
        &NewOwnerAddr {
            new_owner_addr:  deps.api.addr_canonicalize(&info.sender.to_string())?,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ClaimRewards { recipient } => execute_claim_rewards(deps, env, info, recipient),
        ExecuteMsg::UpdateConfig {
            hub_contract,
            reward_denom,
            swap_contract,
        } => {
            let api = deps.api;
            udpate_config(
                deps,
                info,
                optional_addr_validate(api, hub_contract)?,
                reward_denom,
                optional_addr_validate(api, swap_contract)?,
            )
        }
        ExecuteMsg::SetOwner { new_owner_addr } => {
            let api = deps.api;
            set_new_owner(deps, info, api.addr_validate(&new_owner_addr)?)
        }
        ExecuteMsg::AcceptOwnership {} => accept_ownership(deps, info),
	ExecuteMsg::SwapToRewardDenom {} => execute_swap(deps, env, info),
        ExecuteMsg::UpdateGlobalIndex {} => execute_update_global_index(deps, env, info),
        ExecuteMsg::IncreaseBalance { address, amount } => {
            execute_increase_balance(deps, env, info, address, amount)
        }
        ExecuteMsg::DecreaseBalance { address, amount } => {
            execute_decrease_balance(deps, env, info, address, amount)
        }
        ExecuteMsg::UpdateSwapDenom { swap_denom, is_add } => {
            update_swap_denom(deps, info, swap_denom, is_add)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::State {} => to_json_binary(&query_state(deps)?),
        QueryMsg::AccruedRewards { address } => to_json_binary(&query_accrued_rewards(deps, address)?),
        QueryMsg::Holder { address } => to_json_binary(&query_holder(deps, address)?),
        QueryMsg::Holders { start_after, limit } => {
            to_json_binary(&query_holders(deps, start_after, limit)?)
        }
        QueryMsg::NewOwner {} => to_json_binary(&query_new_owner(deps)?),
    }
}

fn query_new_owner(deps: Deps) -> StdResult<NewOwnerResponse> {
    let new_owner = read_new_owner(deps.storage)?;
    Ok(NewOwnerResponse {
        new_owner: deps
            .api
            .addr_humanize(&new_owner.new_owner_addr)?
            .to_string(),
    })
}


fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        hub_contract: deps.api.addr_humanize(&config.hub_contract)?.to_string(),
        reward_denom: config.reward_denom,
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        swap_contract: deps.api.addr_humanize(&config.swap_contract)?.to_string(),
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
