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

use basset::hub::ExecuteMsg::CheckSlashing;
use cosmwasm_std::{
    to_json_binary, Binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128, WasmMsg,
};
use cw20::Logo;
use cw20_base::allowances::{
    execute_burn_from as cw20_burn_from, execute_send_from as cw20_send_from,
    execute_transfer_from as cw20_transfer_from,
};
use cw20_base::contract::{
    execute_burn as cw20_burn, execute_mint as cw20_mint, execute_send as cw20_send,
    execute_transfer as cw20_transfer, execute_update_marketing as cw20_update_marketing,
    execute_update_minter as cw20_update_minter, execute_upload_logo as cw20_upload_logo,
};
use cw20_base::ContractError;

use crate::state::HUB_CONTRACT;

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    cw20_transfer(deps, env, info, recipient, amount)
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let hub_contract = deps.api.addr_humanize(&HUB_CONTRACT.load(deps.storage)?)?;

    if info.sender != hub_contract {
        return Err(ContractError::Unauthorized {});
    }

    let messages = vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: hub_contract.to_string(),
        msg: to_json_binary(&CheckSlashing {})?,
        funds: vec![],
    }))];
    
    let res = cw20_burn(deps, env, info, amount)?;
    /* send message example
     use cosmwasm_std::{coins, BankMsg};
     let msg = BankMsg::Send { to_address: String::from("you"), amount: coins(1015, "earth") };
     let sub_msg: SubMsg = SubMsg::reply_always(msg, 1234).with_gas_limit(60_000);
     messages.push(sub_msg);
    */
    Ok(Response::new()
        .add_submessages(messages)
        .add_attributes(res.attributes))
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    cw20_mint(deps, env, info, recipient, amount)
}

pub fn execute_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    cw20_send(deps, env, info, contract, amount, msg)
}

pub fn execute_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    cw20_transfer_from(deps, env, info, owner, recipient, amount)
}

pub fn execute_burn_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let hub_contract = deps.api.addr_humanize(&HUB_CONTRACT.load(deps.storage)?)?;

    let res = cw20_burn_from(deps, env, info, owner, amount)?;
    let messages = vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: hub_contract.to_string(),
        msg: to_json_binary(&CheckSlashing {})?,
        funds: vec![],
    }))];
    Ok(Response::new()
        .add_submessages(messages)
        .add_attributes(res.attributes))
}

pub fn execute_send_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    cw20_send_from(deps, env, info, owner, contract, amount, msg)
}

pub fn execute_update_marketing(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    project: Option<String>,
    description: Option<String>,
    marketing: Option<String>,
) -> Result<Response, ContractError> {
    cw20_update_marketing(deps, env, info, project, description, marketing)
}

pub fn execute_upload_logo(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    logo: Logo,
) -> Result<Response, ContractError> {
    cw20_upload_logo(deps, env, info, logo)
}

pub fn execute_update_minter(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    minter: Option<String>,
) -> Result<Response, ContractError> {
    cw20_update_minter(deps, env, info, minter)
}
