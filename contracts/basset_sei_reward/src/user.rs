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

use crate::error::ContractError;
use crate::querier::query_token_contract_address;
use crate::state::{
    read_config, read_holder, read_holders, read_state, store_holder, store_state, Config, Holder,
    State,
};
use basset::reward::{AccruedRewardsResponse, HolderResponse, HoldersResponse};

use cosmwasm_std::{
    attr, BankMsg, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};

use crate::math::{
    decimal_multiplication_in_256, decimal_subtraction_in_256, decimal_summation_in_256,
};
use basset::deduct_tax;

pub fn execute_claim_rewards(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    let holder_addr = info.sender;
    let holder_addr_raw = deps.api.addr_canonicalize(holder_addr.as_str())?;
    let recipient = match recipient {
        Some(value) => deps.api.addr_validate(value.as_str())?,
        None => holder_addr.clone(),
    };

    let mut holder: Holder = read_holder(deps.storage, &holder_addr_raw)?;
    let mut state: State = read_state(deps.storage)?;
    let config: Config = read_config(deps.storage)?;

    let reward_with_decimals =
        calculate_decimal_rewards(state.global_index, holder.index, holder.balance);

    let all_reward_with_decimals =
        decimal_summation_in_256(reward_with_decimals, holder.pending_rewards);
    let rewards = all_reward_with_decimals * Uint128::new(1);
    let decimals = all_reward_with_decimals - Decimal::from_ratio(rewards, Uint128::new(1));

    if rewards.is_zero() {
        return Err(ContractError::Std(StdError::generic_err("No rewards have accrued yet")));
    }

    let new_balance = (state.prev_reward_balance.checked_sub(rewards))?;
    state.prev_reward_balance = new_balance;
    store_state(deps.storage, &state)?;

    holder.pending_rewards = decimals;
    holder.index = state.global_index;
    store_holder(deps.storage, &holder_addr_raw, &holder)?;

    let bank_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![deduct_tax(
            &deps.querier,
            Coin {
                denom: config.reward_denom,
                amount: rewards,
            },
        )?],
    });

    let res = Response::new()
        .add_attributes(vec![
            attr("action", "claim_reward"),
            attr("holder_address", holder_addr),
            attr("rewards", rewards),
        ])
        .add_message(bank_msg);

    Ok(res)
}

pub fn execute_increase_balance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = read_config(deps.storage)?;
    let owner_human = deps.api.addr_humanize(&config.hub_contract)?;
    let address_raw = deps.api.addr_canonicalize(&address)?;
    let sender = info.sender;

    let token_address = deps
        .api
        .addr_humanize(&query_token_contract_address(deps.as_ref(), owner_human)?)?;

    // Check sender is token contract
    if sender != token_address {
        return Err(ContractError::Std(StdError::generic_err("unauthorized")));
    }

    let mut state: State = read_state(deps.storage)?;
    let mut holder: Holder = read_holder(deps.storage, &address_raw)?;

    // get decimals
    let rewards = calculate_decimal_rewards(state.global_index, holder.index, holder.balance);

    holder.index = state.global_index;
    holder.pending_rewards = decimal_summation_in_256(rewards, holder.pending_rewards);
    holder.balance += amount;
    state.total_balance += amount;

    store_holder(deps.storage, &address_raw, &holder)?;
    store_state(deps.storage, &state)?;

    let attributes = vec![
        attr("action", "increase_balance"),
        attr("holder_address", address),
        attr("amount", amount),
    ];

    let res = Response::new().add_attributes(attributes);
    Ok(res)
}

pub fn execute_decrease_balance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = read_config(deps.storage)?;
    let hub_contract = deps.api.addr_humanize(&config.hub_contract)?;
    let address_raw = deps.api.addr_canonicalize(&address)?;

    // Check sender is token contract
    if query_token_contract_address(deps.as_ref(), hub_contract)?
        != deps.api.addr_canonicalize(info.sender.as_str())?
    {
        return Err(ContractError::Std(StdError::generic_err("unauthorized")));
    }

    let mut state: State = read_state(deps.storage)?;
    let mut holder: Holder = read_holder(deps.storage, &address_raw)?;
    if holder.balance < amount {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Decrease amount cannot exceed user balance: {}",
            holder.balance
        ))));
    }

    let rewards = calculate_decimal_rewards(state.global_index, holder.index, holder.balance);

    holder.index = state.global_index;
    holder.pending_rewards = decimal_summation_in_256(rewards, holder.pending_rewards);
    holder.balance = (holder.balance.checked_sub(amount))?;
    state.total_balance = (state.total_balance.checked_sub(amount))?;

    store_holder(deps.storage, &address_raw, &holder)?;
    store_state(deps.storage, &state)?;

    let attributes = vec![
        attr("action", "decrease_balance"),
        attr("holder_address", address),
        attr("amount", amount),
    ];

    let res = Response::new().add_attributes(attributes);

    Ok(res)
}

pub fn query_accrued_rewards(deps: Deps, address: String) -> StdResult<AccruedRewardsResponse> {
    let global_index = read_state(deps.storage)?.global_index;

    let holder: Holder = read_holder(deps.storage, &deps.api.addr_canonicalize(&address)?)?;
    let reward_with_decimals =
        calculate_decimal_rewards(global_index, holder.index, holder.balance);
    let all_reward_with_decimals =
        decimal_summation_in_256(reward_with_decimals, holder.pending_rewards);

    let rewards = all_reward_with_decimals * Uint128::new(1);

    Ok(AccruedRewardsResponse { rewards })
}

pub fn query_holder(deps: Deps, address: String) -> StdResult<HolderResponse> {
    let holder: Holder = read_holder(deps.storage, &deps.api.addr_canonicalize(&address)?)?;
    Ok(HolderResponse {
        address,
        balance: holder.balance,
        index: holder.index,
        pending_rewards: holder.pending_rewards,
    })
}

pub fn query_holders(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<HoldersResponse> {
    let start_after = if let Some(start_after) = start_after {
        Some(deps.api.addr_validate(&start_after)?)
    } else {
        None
    };

    let holders: Vec<HolderResponse> = read_holders(deps, start_after, limit)?;

    Ok(HoldersResponse { holders })
}

// calculate the reward based on the sender's index and the global index.
fn calculate_decimal_rewards(
    global_index: Decimal,
    user_index: Decimal,
    user_balance: Uint128,
) -> Decimal {
    let decimal_balance = Decimal::from_ratio(user_balance, Uint128::new(1));
    decimal_multiplication_in_256(
        decimal_subtraction_in_256(global_index, user_index),
        decimal_balance,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn proper_calculate_rewards() {
        let global_index = Decimal::from_ratio(Uint128::new(9), Uint128::new(100));
        let user_index = Decimal::zero();
        let user_balance = Uint128::new(1000);
        let reward = calculate_decimal_rewards(global_index, user_index, user_balance);
        assert_eq!(reward.to_string(), "90");
    }

    #[test]
    pub fn proper_get_decimals() {
        let global_index = Decimal::from_ratio(Uint128::new(9999999), Uint128::new(100000000));
        let user_index = Decimal::zero();
        let user_balance = Uint128::new(10);
        let decimal_reward = calculate_decimal_rewards(global_index, user_index, user_balance);

        let rewards = decimal_reward * Uint128::new(1);
        let decimals = decimal_reward - Decimal::from_ratio(rewards, Uint128::new(1));

        assert_eq!(decimals.to_string(), "0.9999999");
    }
}
