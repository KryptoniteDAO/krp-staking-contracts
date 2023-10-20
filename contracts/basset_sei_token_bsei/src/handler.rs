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

use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128, WasmMsg,
};

use crate::querier::query_reward_contract;
use crate::state::read_hub_contract;
use basset::hub::ExecuteMsg::CheckSlashing;
use basset::reward::ExecuteMsg::{DecreaseBalance, IncreaseBalance};
use cw20_legacy::allowances::{
    execute_burn_from as cw20_burn_from, execute_send_from as cw20_send_from,
    execute_transfer_from as cw20_transfer_from,
};
use cw20_legacy::contract::{
    execute_burn as cw20_burn, execute_mint as cw20_mint, execute_send as cw20_send,
    execute_transfer as cw20_transfer,
};
use cw20_legacy::ContractError;

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let sender = info.sender.clone();
    let reward_contract = query_reward_contract(&deps)?;

    let rcpt_addr = deps.api.addr_validate(&recipient)?;

    let res: Response = cw20_transfer(deps, env, info, recipient, amount)?;
    let messages = vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reward_contract.to_string(),
            msg: to_binary(&DecreaseBalance {
                address: sender.to_string(),
                amount,
            })?,
            funds: vec![],
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reward_contract.to_string(),
            msg: to_binary(&IncreaseBalance {
                address: rcpt_addr.to_string(),
                amount,
            })?,
            funds: vec![],
        })),
    ];
    Ok(Response::new()
        .add_submessages(messages)
        .add_attributes(res.attributes))
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let sender = info.sender.clone();
    let reward_contract = query_reward_contract(&deps)?;
    let hub_contract = deps.api.addr_humanize(&read_hub_contract(deps.storage)?)?;

    if sender != hub_contract {
        return Err(ContractError::Unauthorized {});
    }
    let res: Response = cw20_burn(deps, env, info, amount)?;
    let messages = vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: reward_contract.to_string(),
        msg: to_binary(&DecreaseBalance {
            address: sender.to_string(),
            amount,
        })?,
        funds: vec![],
    }))];
 
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
    let reward_contract = query_reward_contract(&deps)?;

    let res: Response = cw20_mint(deps, env, info, recipient.clone(), amount)?;
    Ok(Response::new()
        .add_submessages(vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reward_contract.to_string(),
            msg: to_binary(&IncreaseBalance {
                address: recipient,
                amount,
            })?,
            funds: vec![],
        }))])
        .add_attributes(res.attributes))
}

pub fn execute_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    let sender = info.sender.clone();
    let reward_contract = query_reward_contract(&deps)?;

    let res: Response = cw20_send(deps, env, info, contract.clone(), amount, msg)?;
    let messages = vec![
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: reward_contract.to_string(),
                msg: to_binary(&DecreaseBalance {
                    address: sender.to_string(),
                    amount,
                })?,
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: reward_contract.to_string(),
                msg: to_binary(&IncreaseBalance {
                    address: contract,
                    amount,
                })?,
                funds: vec![],
            })),
        ],
        res.messages,
    ]
    .concat();

    Ok(Response::new()
        .add_submessages(messages)
        .add_attributes(res.attributes))
}

pub fn execute_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let reward_contract = query_reward_contract(&deps)?;

    let valid_owner = deps.api.addr_validate(owner.as_str())?;

    let res: Response = cw20_transfer_from(deps, env, info, owner, recipient.clone(), amount)?;
    let messages = vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reward_contract.to_string(),
            msg: to_binary(&DecreaseBalance {
                address: valid_owner.to_string(),
                amount,
            })?,
            funds: vec![],
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reward_contract.to_string(),
            msg: to_binary(&IncreaseBalance {
                address: recipient,
                amount,
            })?,
            funds: vec![],
        })),
    ];
    Ok(Response::new()
        .add_submessages(messages)
        .add_attributes(res.attributes))
}

pub fn execute_burn_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let reward_contract = query_reward_contract(&deps)?;
    let hub_contract = deps.api.addr_humanize(&read_hub_contract(deps.storage)?)?;

    let valid_owner = deps.api.addr_validate(owner.as_str())?;

    let res: Response = cw20_burn_from(deps, env, info, owner, amount)?;
    let messages = vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reward_contract.to_string(),
            msg: to_binary(&DecreaseBalance {
                address: valid_owner.to_string(),
                amount,
            })?,
            funds: vec![],
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: hub_contract.to_string(),
            msg: to_binary(&CheckSlashing {})?,
            funds: vec![],
        })),
    ];
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
    let reward_contract = query_reward_contract(&deps)?;

    let valid_owner = deps.api.addr_validate(owner.as_str())?;

    let res: Response = cw20_send_from(deps, env, info, owner, contract.clone(), amount, msg)?;
    let messages = vec![
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: reward_contract.to_string(),
                msg: to_binary(&DecreaseBalance {
                    address: valid_owner.to_string(),
                    amount,
                })?,
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: reward_contract.to_string(),
                msg: to_binary(&IncreaseBalance {
                    address: contract,
                    amount,
                })?,
                funds: vec![],
            })),
        ],
        res.messages,
    ]
    .concat();

    Ok(Response::new()
        .add_submessages(messages)
        .add_attributes(res.attributes))
}
