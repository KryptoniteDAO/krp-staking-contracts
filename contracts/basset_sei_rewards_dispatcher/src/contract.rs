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

use crate::handler::{update_oracle_contract, update_swap_contract, update_swap_denom};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{read_config, store_config, Config, CONFIG, read_new_owner, store_new_owner};
use basset::dispatcher::ConfigResponse;
use basset::hub::ExecuteMsg::{BondRewards, UpdateGlobalIndex,};
use basset::oracle_pyth::QueryMsg as PythOracleQueryMsg;
use basset::swap_ext::{Asset, AssetInfo, SimulationResponse, SwapExecteMsg, SwapQueryMsg};
use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, Fraction,
    MessageInfo, QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use std::ops::Mul;

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
        krp_keeper_address: deps.api.addr_canonicalize(&msg.krp_keeper_address)?,
        krp_keeper_rate: msg.krp_keeper_rate,
        swap_contract: deps.api.addr_canonicalize(&msg.swap_contract)?,
        swap_denoms: msg.swap_denoms,
        oracle_contract: deps.api.addr_canonicalize(&msg.oracle_contract)?,
    };
    
    if msg.krp_keeper_rate > Decimal::one() {
        return Err(StdError::generic_err("keeper rate can not be greater than 1."));
    }


    store_config(deps.storage, &conf)?;
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
            hub_contract,
            bsei_reward_contract,
            stsei_reward_denom,
            bsei_reward_denom,
            krp_keeper_address,
            krp_keeper_rate,
        } => execute_update_config(
            deps,
            env,
            info,
            hub_contract,
            bsei_reward_contract,
            stsei_reward_denom,
            bsei_reward_denom,
            krp_keeper_address,
            krp_keeper_rate,
        ),
        ExecuteMsg::SetOwner { new_owner_addr } => {
            let api = deps.api;
            set_new_owner(deps, info, api.addr_validate(&new_owner_addr)?)
        }
        ExecuteMsg::AcceptOwnership {} => accept_ownership(deps, info),
        ExecuteMsg::UpdateSwapContract { swap_contract } => {
            update_swap_contract(deps, info, swap_contract)
        }
        ExecuteMsg::UpdateSwapDenom { swap_denom, is_add } => {
            update_swap_denom(deps, info, swap_denom, is_add)
        }
        ExecuteMsg::UpdateOracleContract { oracle_contract } => {
            update_oracle_contract(deps, info, oracle_contract)
        }
    }
}

pub fn set_new_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner_addr: Addr,
) -> StdResult<Response> {
    let config = read_config(deps.as_ref().storage)?;
    let mut new_owner = read_new_owner(deps.as_ref().storage)?;
    let sender_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    if sender_raw != config.owner {
        return Err(StdError::generic_err("Unauthorized call set_new_owner function"));
    }
    new_owner.new_owner_addr = deps.api.addr_canonicalize(&new_owner_addr.to_string())?;
    store_new_owner(deps.storage, &new_owner)?;

    Ok(Response::default())
}

pub fn accept_ownership(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let new_owner = read_new_owner(deps.as_ref().storage)?;
    let sender_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    let mut config = read_config(deps.as_ref().storage)?;
    if sender_raw != new_owner.new_owner_addr {
        return Err(StdError::generic_err("Unauthorized call set_new_owner function"));
    }

    config.owner = new_owner.new_owner_addr;
    store_config(deps.storage, &config)?;

    Ok(Response::default())
}

#[allow(clippy::too_many_arguments)]
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    hub_contract: Option<String>,
    bsei_reward_contract: Option<String>,
    stsei_reward_denom: Option<String>,
    bsei_reward_denom: Option<String>,
    krp_keeper_address: Option<String>,
    krp_keeper_rate: Option<Decimal>,
) -> StdResult<Response> {
    let conf = read_config(deps.storage)?;
    let sender_raw = deps.api.addr_canonicalize(info.sender.as_str())?;
    if sender_raw != conf.owner {
        return Err(StdError::generic_err("unauthorized"));
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
        CONFIG.update(deps.storage, |mut last_config| -> StdResult<_> {
            last_config.bsei_reward_denom = _b;
            Ok(last_config)
        })?;
    }

    if let Some(r) = krp_keeper_rate {
        
        if r > Decimal::one() {
            return Err(StdError::generic_err("keeper rate can not be greater than 1."));
        }
    
        CONFIG.update(deps.storage, |mut last_config| -> StdResult<_> {
            last_config.krp_keeper_rate = r;
            Ok(last_config)
        })?;
    }

    if let Some(a) = krp_keeper_address {
        let address_raw = deps.api.addr_canonicalize(&a)?;

        CONFIG.update(deps.storage, |mut last_config| -> StdResult<_> {
            last_config.krp_keeper_address = address_raw;
            Ok(last_config)
        })?;
    }

    Ok(Response::default())
}

pub fn execute_swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bsei_total_bonded_amount: Uint128,
    stsei_total_bonded_amount: Uint128,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    let hub_addr = deps.api.addr_humanize(&config.hub_contract)?;
    let swap_addr = deps.api.addr_humanize(&config.swap_contract)?;
    let oracle_addr = deps.api.addr_humanize(&config.oracle_contract)?;

    if info.sender != hub_addr {
        return Err(StdError::generic_err("unauthorized"));
    }

    let contr_addr = env.contract.address;
    let balance = deps.querier.query_all_balances(contr_addr)?;

    let (total_sei_rewards_available, total_ust_rewards_available, mut msgs) =
        convert_to_target_denoms(
            &deps,
            balance.clone(),
            config.clone(),
            config.stsei_reward_denom.clone(),
            config.bsei_reward_denom.clone(),
            None,
        )?;

    let (sei_2_ust_rewards_xchg_rate, ust_2_sei_rewards_xchg_rate) = get_exchange_rates(
        &deps,
        &oracle_addr,
        config.stsei_reward_denom.as_str(),
        config.bsei_reward_denom.as_str(),
    )?;

    let (offer_coin, ask_denom) = get_swap_info(
        config,
        stsei_total_bonded_amount,
        bsei_total_bonded_amount,
        total_sei_rewards_available,
        total_ust_rewards_available,
        ust_2_sei_rewards_xchg_rate,
        sei_2_ust_rewards_xchg_rate,
    )?;

    if !offer_coin.amount.is_zero() {
        let msg = create_swap_msg(
            offer_coin.clone(),
            ask_denom.clone(),
            swap_addr.clone().to_string(),
            None,
        )?;
        msgs.push(msg);
    }

    let res = Response::new().add_messages(msgs).add_attributes(vec![
        attr("action", "swap"),
        attr("initial_balance", format!("{:?}", balance)),
        attr(
            "sei_2_ust_rewards_xchg_rate",
            sei_2_ust_rewards_xchg_rate.to_string(),
        ),
        attr(
            "ust_2_sei_rewards_xchg_rate",
            ust_2_sei_rewards_xchg_rate.to_string(),
        ),
        attr("total_sei_rewards_available", total_sei_rewards_available),
        attr("total_ust_rewards_available", total_ust_rewards_available),
        attr("offer_coin_denom", offer_coin.denom),
        attr("offer_coin_amount", offer_coin.amount),
        attr("ask_denom", ask_denom),
    ]);

    Ok(res)
}

#[allow(clippy::needless_collect)]
pub(crate) fn convert_to_target_denoms(
    deps: &DepsMut,
    balance: Vec<Coin>,
    config: Config,
    denom_to_keep: String,
    denom_to_xchg: String,
    reward_addr: Option<String>,
) -> StdResult<(Uint128, Uint128, Vec<CosmosMsg>)> {
    let mut total_sei_available: Uint128 = Uint128::zero();
    let mut total_usd_available: Uint128 = Uint128::zero();

    let _denoms: Vec<String> = balance.iter().map(|item| item.denom.clone()).collect();

    let known_denoms = config.swap_denoms;
    let swap_contract = deps.api.addr_humanize(&config.swap_contract)?;

    let mut msgs: Vec<CosmosMsg> = Vec::new();

    for coin in balance {
        if !known_denoms.contains(&coin.denom) {
            continue;
        }

        if coin.denom == denom_to_keep {
            total_sei_available += coin.amount;
            continue;
        }

        if coin.denom == denom_to_xchg {
            total_usd_available += coin.amount;
            continue;
        }

        if !coin.amount.is_zero() {
            let simulation_response = query_swap_simulation(
                &deps,
                swap_contract.to_string(),
                coin.clone(),
                denom_to_xchg.clone().to_string(),
            )?;

            total_usd_available += simulation_response.return_amount;
            let msg = create_swap_msg(
                coin,
                denom_to_xchg.clone(),
                swap_contract.clone().to_string(),
                reward_addr.clone(),
            )?;
            msgs.push(msg);
        }
    }

    Ok((total_sei_available, total_usd_available, msgs))
}

pub(crate) fn query_swap_simulation(
    deps: &DepsMut,
    contract_addr: String,
    offer_coin: Coin,
    ask_denom: String,
) -> StdResult<SimulationResponse> {
    let querier = &deps.querier;
    let asset_infos = [
        AssetInfo::NativeToken {
            denom: offer_coin.denom.clone(),
        },
        AssetInfo::NativeToken {
            denom: ask_denom.clone(),
        },
    ];
    let offer_asset: Asset = Asset {
        info: AssetInfo::NativeToken {
            denom: offer_coin.denom.clone(),
        },
        amount: offer_coin.amount,
    };
    let simulation_response = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.clone(),
        msg: to_binary(&SwapQueryMsg::QuerySimulation {
            asset_infos,
            offer_asset,
        })?,
    }))?;

    Ok(simulation_response)
}

pub(crate) fn create_swap_msg(
    coin: Coin,
    reward_denom: String,
    swap_addr: String,
    reward_addr: Option<String>,
) -> StdResult<CosmosMsg> {
    let swap_msg = SwapExecteMsg::SwapDenom {
        from_coin: coin.clone(),
        target_denom: reward_denom,
        to_address: reward_addr,
    };
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: swap_addr,
        msg: to_binary(&swap_msg).unwrap(),
        funds: vec![coin.clone()],
    });
    Ok(msg)
}

pub(crate) fn get_exchange_rates(
    deps: &DepsMut,
    oracle_addr: &Addr,
    denom_a: &str,
    denom_b: &str,
) -> StdResult<(Decimal, Decimal)> {
    let querier = &deps.querier;
    let a_2_b_xchg_rate: Decimal = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: oracle_addr.to_string(),
        msg: to_binary(&PythOracleQueryMsg::QueryExchangeRateByAssetLabel {
            base_label: denom_a.to_string(),
            quote_label: denom_b.to_string(),
        })?,
    }))?;

    Ok((
        a_2_b_xchg_rate.clone(),
        a_2_b_xchg_rate
            .inv()
            .ok_or_else(|| StdError::generic_err("failed to convert exchange rate"))?,
    ))
}

pub(crate) fn get_swap_info(
    config: Config,
    stsei_total_bonded_amount: Uint128,
    bsei_total_bonded_amount: Uint128,
    total_stsei_rewards_available: Uint128,
    total_bsei_rewards_available: Uint128,
    bsei_2_stsei_rewards_xchg_rate: Decimal,
    stsei_2_bsei_rewards_xchg_rate: Decimal,
) -> StdResult<(Coin, String)> {
    // Total rewards in stsei rewards currency.
    let total_rewards_in_stsei_rewards = total_stsei_rewards_available
        + total_bsei_rewards_available.mul(bsei_2_stsei_rewards_xchg_rate);

    let stsei_share_of_total_rewards = total_rewards_in_stsei_rewards.multiply_ratio(
        stsei_total_bonded_amount,
        stsei_total_bonded_amount + bsei_total_bonded_amount,
    );

    if total_stsei_rewards_available.gt(&stsei_share_of_total_rewards) {
        let stsei_rewards_to_sell =
            total_stsei_rewards_available.checked_sub(stsei_share_of_total_rewards)?;

        Ok((
            Coin::new(
                stsei_rewards_to_sell.u128(),
                config.stsei_reward_denom.as_str(),
            ),
            config.bsei_reward_denom,
        ))
    } else {
        let stsei_rewards_to_buy =
            stsei_share_of_total_rewards.checked_sub(total_stsei_rewards_available)?;
        let bsei_rewards_to_sell = stsei_rewards_to_buy.mul(stsei_2_bsei_rewards_xchg_rate);

        Ok((
            Coin::new(
                bsei_rewards_to_sell.u128(),
                config.bsei_reward_denom.as_str(),
            ),
            config.stsei_reward_denom,
        ))
    }
}

pub fn execute_dispatch_rewards(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let config = read_config(deps.storage)?;

    let hub_addr = deps.api.addr_humanize(&config.hub_contract)?;
    if info.sender != hub_addr {
        return Err(StdError::generic_err("unauthorized"));
    }

    let contr_addr = env.contract.address;
    let stsei_rewards = deps
        .querier
        .query_balance(contr_addr.clone(), config.stsei_reward_denom.as_str())?;


    let bsei_reward_addr = deps.api.addr_humanize(&config.bsei_reward_contract)?;
    let bsei_rewards = deps
        .querier
        .query_balance(contr_addr, config.bsei_reward_denom.as_str())?;

    let mut messages: Vec<CosmosMsg> = vec![];
    if !bsei_rewards.amount.is_zero() {
        let keeper_rewards = bsei_rewards.amount * config.krp_keeper_rate;

        messages.push(
            BankMsg::Send {
                to_address: deps
                    .api
                    .addr_humanize(&config.krp_keeper_address)?
                    .to_string(),
                amount: vec![Coin {
                    denom: config.bsei_reward_denom.clone(),
                    amount: keeper_rewards,
                }],
            }
            .into(),
        );

        messages.push(
            BankMsg::Send {
                to_address: bsei_reward_addr.to_string(),
                amount: vec![Coin {
                    denom: config.bsei_reward_denom.clone(),
                    amount: bsei_rewards.amount - keeper_rewards,
                }],
            }
            .into(),
        );
    }

    if !stsei_rewards.amount.is_zero() {
        let keeper_rewards = stsei_rewards.amount * config.krp_keeper_rate;
   
        messages.push(
            BankMsg::Send {
                to_address: deps
                    .api
                    .addr_humanize(&config.krp_keeper_address)?
                    .to_string(),
                amount: vec![Coin {
                    denom: config.stsei_reward_denom.clone(),
                    amount: keeper_rewards,
                }],
            }
            .into(),
        );

        let rebond_rewards = stsei_rewards.amount.checked_sub(keeper_rewards)?;
        if !rebond_rewards.is_zero() {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: hub_addr.to_string(),
                msg: to_binary(&BondRewards {}).unwrap(),
                funds: vec![Coin {
                    denom: config.stsei_reward_denom.clone(),
                    amount: rebond_rewards,
                }],
            }));
        }
    }

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

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        hub_contract: deps.api.addr_humanize(&config.hub_contract)?.to_string(),
        bsei_reward_contract: deps
            .api
            .addr_humanize(&config.bsei_reward_contract)?
            .to_string(),
        stsei_reward_denom: config.stsei_reward_denom,
        bsei_reward_denom: config.bsei_reward_denom,
        krp_keeper_address: deps
            .api
            .addr_humanize(&config.krp_keeper_address)?
            .to_string(),
        krp_keeper_rate: config.krp_keeper_rate,
        swap_contract: deps.api.addr_humanize(&config.swap_contract)?.to_string(),
        swap_denoms: config.swap_denoms,
        oracle_contract: deps.api.addr_humanize(&config.oracle_contract)?.to_string(),
    })
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
