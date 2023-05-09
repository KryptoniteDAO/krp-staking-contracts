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

use crate::contract::{query_total_bsei_issued, query_total_stsei_issued, slashing};
use crate::math::decimal_division;
use crate::state::{CONFIG, CURRENT_BATCH, PARAMETERS, STATE};
use cosmwasm_std::{
    attr, to_binary, CosmosMsg, DepsMut, Env, Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use std::ops::Mul;

pub fn convert_stsei_bsei(
    mut deps: DepsMut,
    env: Env,
    stsei_amount: Uint128,
    sender: String,
) -> StdResult<Response> {
    let conf = CONFIG.load(deps.storage)?;
    let state = slashing(&mut deps, env)?;
    let params = PARAMETERS.load(deps.storage)?;
    let threshold = params.er_threshold;
    let recovery_fee = params.peg_recovery_fee;

    let stsei_contract = deps.api.addr_humanize(
        &conf
            .stsei_token_contract
            .ok_or_else(|| StdError::generic_err("stsei contract must be registred"))?,
    )?;
    let bsei_contract = deps.api.addr_humanize(
        &conf
            .bsei_token_contract
            .ok_or_else(|| StdError::generic_err("bsei contract must be registred"))?,
    )?;

    let denom_equiv = state.stsei_exchange_rate.mul(stsei_amount);

    let bsei_to_mint = decimal_division(denom_equiv, state.bsei_exchange_rate);
    let current_batch = CURRENT_BATCH.load(deps.storage)?;
    let requested_bsei_with_fee = current_batch.requested_bsei_with_fee;
    let requested_stsei = current_batch.requested_stsei;

    let total_bsei_supply = query_total_bsei_issued(deps.as_ref())?;
    let total_stsei_supply = query_total_stsei_issued(deps.as_ref())?;
    let mut bsei_mint_amount_with_fee = bsei_to_mint;
    if state.bsei_exchange_rate < threshold {
        let max_peg_fee = bsei_to_mint * recovery_fee;
        let required_peg_fee = (total_bsei_supply + bsei_to_mint + requested_bsei_with_fee)
            - (state.total_bond_bsei_amount + denom_equiv);
        let peg_fee = Uint128::min(max_peg_fee, required_peg_fee);
        bsei_mint_amount_with_fee = bsei_to_mint.checked_sub(peg_fee)?;
    }

    STATE.update(deps.storage, |mut prev_state| -> StdResult<_> {
        prev_state.total_bond_bsei_amount += denom_equiv;
        prev_state.total_bond_stsei_amount = prev_state.total_bond_stsei_amount.checked_sub(denom_equiv)
            .map_err(|_| {
                StdError::generic_err(format!(
                    "Decrease amount cannot exceed total stsei bond amount: {}. Trying to reduce: {}",
                    prev_state.total_bond_stsei_amount, denom_equiv,
                ))
            })?;
        prev_state.update_bsei_exchange_rate(
            total_bsei_supply + bsei_mint_amount_with_fee,
            requested_bsei_with_fee,
        );
        prev_state
            .update_stsei_exchange_rate(total_stsei_supply .checked_sub(stsei_amount).map_err(|_| {
                StdError::generic_err(format!(
                    "Decrease amount cannot exceed total stsei supply: {}. Trying to reduce: {}",
                    total_stsei_supply, stsei_amount,
                ))
            })?, requested_stsei);
        Ok(prev_state)
    })?;

    let messages: Vec<CosmosMsg> = vec![
        mint_message(
            bsei_contract.to_string(),
            sender.clone(),
            bsei_mint_amount_with_fee,
        )?,
        burn_message(stsei_contract.to_string(), stsei_amount)?,
    ];

    let res = Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "convert_stsei"),
        attr("from", sender),
        attr("bsei_exchange_rate", state.bsei_exchange_rate.to_string()),
        attr("stsei_exchange_rate", state.stsei_exchange_rate.to_string()),
        attr("stsei_amount", stsei_amount),
        attr("bsei_amount", bsei_mint_amount_with_fee),
    ]);
    Ok(res)
}

pub fn convert_bsei_stsei(
    mut deps: DepsMut,
    env: Env,
    bsei_amount: Uint128,
    sender: String,
) -> StdResult<Response> {
    let conf = CONFIG.load(deps.storage)?;
    let state = slashing(&mut deps, env)?;
    let stsei_contract = deps.api.addr_humanize(
        &conf
            .stsei_token_contract
            .ok_or_else(|| StdError::generic_err("stsei contract must be registred"))?,
    )?;
    let bsei_contract = deps.api.addr_humanize(
        &conf
            .bsei_token_contract
            .ok_or_else(|| StdError::generic_err("bsei contract must be registred"))?,
    )?;

    let params = PARAMETERS.load(deps.storage)?;
    let threshold = params.er_threshold;
    let recovery_fee = params.peg_recovery_fee;

    let current_batch = CURRENT_BATCH.load(deps.storage)?;
    let requested_bsei_with_fee = current_batch.requested_bsei_with_fee;
    let requested_stsei_with_fee = current_batch.requested_stsei;

    let total_bsei_supply = query_total_bsei_issued(deps.as_ref())?;
    let total_stsei_supply = query_total_stsei_issued(deps.as_ref())?;

    // Apply peg recovery fee
    let bsei_amount_with_fee: Uint128;
    if state.bsei_exchange_rate < threshold {
        let max_peg_fee = bsei_amount * recovery_fee;
        let required_peg_fee = (total_bsei_supply + current_batch.requested_bsei_with_fee)
            .checked_sub(state.total_bond_bsei_amount)?;
        let peg_fee = Uint128::min(max_peg_fee, required_peg_fee);
        bsei_amount_with_fee = bsei_amount.checked_sub(peg_fee)?;
    } else {
        bsei_amount_with_fee = bsei_amount;
    }

    let denom_equiv = state.bsei_exchange_rate.mul(bsei_amount_with_fee);

    let stsei_to_mint = decimal_division(denom_equiv, state.stsei_exchange_rate);

    STATE.update(deps.storage, |mut prev_state| -> StdResult<_> {
        prev_state.total_bond_bsei_amount = prev_state.total_bond_bsei_amount.checked_sub(denom_equiv)
            .map_err(|_| {
                StdError::generic_err(format!(
                    "Decrease amount cannot exceed total bsei bond amount: {}. Trying to reduce: {}",
                    prev_state.total_bond_bsei_amount, denom_equiv,
                ))
            })?;
        prev_state.total_bond_stsei_amount += denom_equiv;
        prev_state.update_bsei_exchange_rate(
            total_bsei_supply.checked_sub(bsei_amount).map_err(|_| {
                StdError::generic_err(format!(
                    "Decrease amount cannot exceed total bsei supply: {}. Trying to reduce: {}",
                    total_bsei_supply, bsei_amount,
                ))
            })?,
            requested_bsei_with_fee,
        );
        prev_state.update_stsei_exchange_rate(
            total_stsei_supply + stsei_to_mint,
            requested_stsei_with_fee,
        );
        Ok(prev_state)
    })?;

    let messages: Vec<CosmosMsg> = vec![
        mint_message(stsei_contract.to_string(), sender.clone(), stsei_to_mint)?,
        burn_message(bsei_contract.to_string(), bsei_amount)?,
    ];

    let res = Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "convert_stsei"),
        attr("from", sender),
        attr("bsei_exchange_rate", state.bsei_exchange_rate.to_string()),
        attr("stsei_exchange_rate", state.stsei_exchange_rate.to_string()),
        attr("bsei_amount", bsei_amount),
        attr("stsei_amount", stsei_to_mint),
    ]);
    Ok(res)
}

fn mint_message(contract: String, recipient: String, amount: Uint128) -> StdResult<CosmosMsg> {
    let mint_msg = Cw20ExecuteMsg::Mint { recipient, amount };
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract,
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    }))
}

fn burn_message(contract: String, amount: Uint128) -> StdResult<CosmosMsg> {
    let burn_msg = Cw20ExecuteMsg::Burn { amount };
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract,
        msg: to_binary(&burn_msg)?,
        funds: vec![],
    }))
}
