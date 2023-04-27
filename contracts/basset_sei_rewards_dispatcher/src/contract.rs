// Copyright 2021 Lido
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

use cosmwasm_std::{
    attr, to_binary, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128, WasmMsg,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
use basset::hub::ExecuteMsg::UpdateGlobalIndex;


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let conf = Config {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        hub_contract: deps.api.addr_canonicalize(&msg.hub_contract)?,
        bsei_reward_contract: deps.api.addr_canonicalize(&msg.bsei_reward_contract)?,
        bsei_reward_denom: msg.bsei_reward_denom,
        stsei_reward_denom: msg.stsei_reward_denom,
        lido_fee_address: deps.api.addr_canonicalize(&msg.lido_fee_address)?,
        lido_fee_rate: msg.lido_fee_rate,
    };

    CONFIG.save(deps.storage, &conf)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::SwapToRewardDenom {
            bsei_total_bonded: bsei_total_mint_amount,
            stsei_total_bonded: stsei_total_mint_amount,
        } => execute_swap(
            deps,
            env,
            info,
            bsei_total_mint_amount,
            stsei_total_mint_amount,
        ),
        ExecuteMsg::DispatchRewards {} => execute_dispatch_rewards(deps, env, info),
        ExecuteMsg::UpdateConfig {
            owner,
            hub_contract,
            bsei_reward_contract,
            stsei_reward_denom,
            bsei_reward_denom,
            lido_fee_address,
            lido_fee_rate,
        } => execute_update_config(
            deps,
            env,
            info,
            owner,
            hub_contract,
            bsei_reward_contract,
            stsei_reward_denom,
            bsei_reward_denom,
            lido_fee_address,
            lido_fee_rate,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: Option<String>,
    hub_contract: Option<String>,
    bsei_reward_contract: Option<String>,
    stsei_reward_denom: Option<String>,
    bsei_reward_denom: Option<String>,
    lido_fee_address: Option<String>,
    lido_fee_rate: Option<Decimal>,
) -> StdResult<Response> {
    let conf = CONFIG.load(deps.storage)?;
    let sender_raw = deps.api.addr_canonicalize(info.sender.as_str())?;
    if sender_raw != conf.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(o) = owner {
        let owner_raw = deps.api.addr_canonicalize(&o)?;

        CONFIG.update(deps.storage, |mut last_config| -> StdResult<_> {
            last_config.owner = owner_raw;
            Ok(last_config)
        })?;
    }

    if let Some(h) = hub_contract {
        let hub_raw = deps.api.addr_canonicalize(&h)?;

        CONFIG.update(deps.storage, |mut last_config| -> StdResult<_> {
            last_config.hub_contract = hub_raw;
            Ok(last_config)
        })?;
    }

    if let Some(b) = bsei_reward_contract {
        let bsei_raw = deps.api.addr_canonicalize(&b)?;

        CONFIG.update(deps.storage, |mut last_config| -> StdResult<_> {
            last_config.bsei_reward_contract = bsei_raw;
            Ok(last_config)
        })?;
    }

    if let Some(_s) = stsei_reward_denom {
        return Err(StdError::generic_err(
            "updating stSei reward denom is forbidden",
        ));
    }

    if let Some(_b) = bsei_reward_denom {
        return Err(StdError::generic_err(
            "updating bSei reward denom is forbidden",
        ));
    }

    if let Some(r) = lido_fee_rate {
        CONFIG.update(deps.storage, |mut last_config| -> StdResult<_> {
            last_config.lido_fee_rate = r;
            Ok(last_config)
        })?;
    }

    if let Some(a) = lido_fee_address {
        let address_raw = deps.api.addr_canonicalize(&a)?;

        CONFIG.update(deps.storage, |mut last_config| -> StdResult<_> {
            last_config.lido_fee_address = address_raw;
            Ok(last_config)
        })?;
    }

    Ok(Response::default())
}

pub fn execute_swap(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _bsei_total_bonded_amount: Uint128,
    _stsei_total_bonded_amount: Uint128,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let hub_addr = deps.api.addr_humanize(&config.hub_contract)?;

    if info.sender != hub_addr {
        return Err(StdError::generic_err("unauthorized"));
    }

    //     let contr_addr = env.contract.address;
    //     let balance = deps.querier.query_all_balances(contr_addr)?;
    //     let (total_sei_rewards_available, total_ust_rewards_available, mut msgs) =
    //         convert_to_target_denoms(
    //             &deps,
    //             balance.clone(),
    //             config.stsei_reward_denom.clone(),
    //             config.bsei_reward_denom.clone(),
    //         )?;

    //     let (sei_2_ust_rewards_xchg_rate, ust_2_sei_rewards_xchg_rate) = get_exchange_rates(
    //         &deps,
    //         config.stsei_reward_denom.as_str(),
    //         config.bsei_reward_denom.as_str(),
    //     )?;

    //     let (offer_coin, ask_denom) = get_swap_info(
    //         config,
    //         stsei_total_bonded_amount,
    //         bsei_total_bonded_amount,
    //         total_sei_rewards_available,
    //         total_ust_rewards_available,
    //         ust_2_sei_rewards_xchg_rate,
    //         sei_2_ust_rewards_xchg_rate,
    //     )?;

    //     if !offer_coin.amount.is_zero() {
    //         msgs.push(create_swap_msg(offer_coin.clone(), ask_denom.clone()));
    //     }

    let res = Response::new().add_attributes(vec![
        attr("action", "swap"),
        // attr("initial_balance", format!("{:?}", balance)),
        // attr(
        //     "sei_2_ust_rewards_xchg_rate",
        //     sei_2_ust_rewards_xchg_rate.to_string(),
        // ),
        // attr(
        //     "ust_2_sei_rewards_xchg_rate",
        //     ust_2_sei_rewards_xchg_rate.to_string(),
        // ),
        // attr("total_sei_rewards_available", total_sei_rewards_available),
        // attr("total_ust_rewards_available", total_ust_rewards_available),
        // attr("offer_coin_denom", offer_coin.denom),
        // attr("offer_coin_amount", offer_coin.amount),
        // attr("ask_denom", ask_denom),
    ]);

    Ok(res)
}

// #[allow(clippy::needless_collect)]
// pub(crate) fn convert_to_target_denoms(
//     deps: &DepsMut,
//     balance: Vec<Coin>,
//     _denom_to_keep: String,
//     _denom_to_xchg: String,
// ) -> StdResult<(Uint128, Uint128, Vec<CosmosMsg>)> {
//     // let terra_querier = TerraQuerier::new(&deps.querier);
//     let mut total_sei_available: Uint128 = Uint128::zero();
//     let mut total_usd_available: Uint128 = Uint128::zero();

//     let _denoms: Vec<String> = balance.iter().map(|item| item.denom.clone()).collect();
//     // let exchange_rates = query_exchange_rates(deps, denom_to_xchg.clone(), denoms)?;
//     // let known_denoms: Vec<String> = exchange_rates
//     //     .exchange_rates
//     //     .iter()
//     //     .map(|item| item.quote_denom.clone())
//     //     .collect();
//     let mut msgs: Vec<CosmosMsg> = Vec::new();

//     // for coin in balance {
//     //     if !known_denoms.contains(&coin.denom) {
//     //         continue;
//     //     }

//     //     if coin.denom == denom_to_keep {
//     //         total_sei_available += coin.amount;
//     //         continue;
//     //     }

//     //     if coin.denom == denom_to_xchg {
//     //         total_usd_available += coin.amount;
//     //         continue;
//     //     }

//     //     let swap_response: SwapResponse =
//     //         terra_querier.query_swap(coin.clone(), denom_to_xchg.as_str())?;
//     //     total_usd_available += swap_response.receive.amount;

//     //     msgs.push(create_swap_msg(coin, denom_to_xchg.to_string()));
//     // }

//     Ok((total_sei_available, total_usd_available, msgs))
// }

// pub(crate) fn query_exchange_rates(
//     deps: &DepsMut,
//     base_denom: String,
//     quote_denoms: Vec<String>,
// ) -> StdResult<ExchangeRatesResponse> {
//     let querier = TerraQuerier::new(&deps.querier);
//     let res: ExchangeRatesResponse = querier.query_exchange_rates(base_denom, quote_denoms)?;
//     Ok(res)
// }

// pub(crate) fn get_exchange_rates(
//     deps: &DepsMut,
//     denom_a: &str,
//     denom_b: &str,
// ) -> StdResult<(Decimal, Decimal)> {
//     let terra_querier = TerraQuerier::new(&deps.querier);
//     let a_2_b_xchg_rates = terra_querier
//         .query_exchange_rates(denom_a.to_string(), vec![denom_b.to_string()])?
//         .exchange_rates;

//     Ok((
//         a_2_b_xchg_rates[0].exchange_rate,
//         a_2_b_xchg_rates[0]
//             .exchange_rate
//             .inv()
//             .ok_or_else(|| StdError::generic_err("failed to convert exchange rate"))?,
//     ))
// }

// pub(crate) fn get_swap_info(
//     config: Config,
//     stsei_total_bonded_amount: Uint128,
//     bsei_total_bonded_amount: Uint128,
//     total_stsei_rewards_available: Uint128,
//     total_bsei_rewards_available: Uint128,
//     bsei_2_stsei_rewards_xchg_rate: Decimal,
//     stsei_2_bsei_rewards_xchg_rate: Decimal,
// ) -> StdResult<(Coin, String)> {
//     // Total rewards in stsei rewards currency.
//     let total_rewards_in_stsei_rewards = total_stsei_rewards_available
//         + total_bsei_rewards_available.mul(bsei_2_stsei_rewards_xchg_rate);

//     let stsei_share_of_total_rewards = total_rewards_in_stsei_rewards.multiply_ratio(
//         stsei_total_bonded_amount,
//         stsei_total_bonded_amount + bsei_total_bonded_amount,
//     );

//     if total_stsei_rewards_available.gt(&stsei_share_of_total_rewards) {
//         let stsei_rewards_to_sell =
//             total_stsei_rewards_available.checked_sub(stsei_share_of_total_rewards)?;

//         Ok((
//             Coin::new(
//                 stsei_rewards_to_sell.u128(),
//                 config.stsei_reward_denom.as_str(),
//             ),
//             config.bsei_reward_denom,
//         ))
//     } else {
//         let stsei_rewards_to_buy =
//             stsei_share_of_total_rewards.checked_sub(total_stsei_rewards_available)?;
//         let bsei_rewards_to_sell = stsei_rewards_to_buy.mul(stsei_2_bsei_rewards_xchg_rate);

//         Ok((
//             Coin::new(
//                 bsei_rewards_to_sell.u128(),
//                 config.bsei_reward_denom.as_str(),
//             ),
//             config.stsei_reward_denom,
//         ))
//     }
// }

pub fn execute_dispatch_rewards(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    let hub_addr = deps.api.addr_humanize(&config.hub_contract)?;
    if info.sender != hub_addr {
        return Err(StdError::generic_err("unauthorized"));
    }
    let contr_addr = env.contract.address;
    let bsei_reward_addr = deps.api.addr_humanize(&config.bsei_reward_contract)?;
    let bsei_rewards = deps
        .querier
        .query_balance(contr_addr, config.bsei_reward_denom.as_str())?;

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: bsei_reward_addr.to_string(),
        msg: to_binary(&UpdateGlobalIndex {
            airdrop_hooks: None,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "claim_reward"),
        attr("bsei_reward_addr", bsei_reward_addr),
        attr("bsei_rewards", bsei_rewards.to_string()),
    ]))
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::GetBufferedRewards {} => unimplemented!(),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
