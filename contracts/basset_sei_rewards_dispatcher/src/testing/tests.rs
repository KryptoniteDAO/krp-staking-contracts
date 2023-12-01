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

//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests as follows:
//! 1. Copy them over verbatim
//! 2. Then change
//!      let mut deps = mock_dependencies(&[]);
//!    to
//!      let mut deps = mock_instance(WASM, &[]);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(&deps, ...) you must replace it with query(deps.as_mut(), ...)

use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{coins, Api, Coin, Decimal, StdError, Uint128};

use crate::contract::{execute, get_swap_info, instantiate};
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::CONFIG;
use crate::testing::mock_querier::{
    mock_dependencies, BTOKEN_REWARD_DENOM, MOCK_BSEI_REWARD_CONTRACT_ADDR, MOCK_HUB_CONTRACT_ADDR,
    MOCK_KRP_KEEPER_CONTRACT_ADDR, MOCK_ORACLE_CONTRACT_ADDR, MOCK_SWAP_CONTRACT_ADDR,
    STTOKEN_REWARD_DENOM,
};

fn default_init() -> InstantiateMsg {
    InstantiateMsg {
        hub_contract: MOCK_HUB_CONTRACT_ADDR.to_string(),
        bsei_reward_contract: MOCK_BSEI_REWARD_CONTRACT_ADDR.to_string(),
        bsei_reward_denom: BTOKEN_REWARD_DENOM.to_string(),
        stsei_reward_denom: STTOKEN_REWARD_DENOM.to_string(),
        krp_keeper_address: MOCK_KRP_KEEPER_CONTRACT_ADDR.to_string(),
        krp_keeper_rate: Decimal::from_ratio(Uint128::from(5u64), Uint128::from(100u64)),
        swap_contract: MOCK_SWAP_CONTRACT_ADDR.to_string(),
        swap_denoms: vec!["usei".to_string(), "kusd".to_string(), "usdr".to_string()],
        oracle_contract: MOCK_ORACLE_CONTRACT_ADDR.to_string(),
    }
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = default_init();
    let info = mock_info("creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn test_swap_to_reward_denom() {
    struct TestCase {
        rewards_balance: Vec<Coin>,
        stsei_total_bonded: Uint128,
        bsei_total_bonded: Uint128,
        expected_total_sei_rewards_available: String,
        expected_total_ust_rewards_available: String,
        expected_offer_coin_denom: String,
        expected_offer_coin_amount: String,
        expected_ask_denom: String,
    }

    let test_cases: Vec<TestCase> = vec![
        TestCase {
            rewards_balance: vec![
                Coin::new(200, "usei"),
                Coin::new(300, "kusd"),
                Coin::new(500, "usdr"),
                Coin::new(100, "mnt"),
            ],
            stsei_total_bonded: Uint128::from(1u128),
            bsei_total_bonded: Uint128::from(2u128),
            expected_total_sei_rewards_available: "200".to_string(),
            expected_total_ust_rewards_available: "1300".to_string(),
            expected_offer_coin_denom: "usei".to_string(),
            expected_offer_coin_amount: "120".to_string(),
            expected_ask_denom: "kusd".to_string(),
        },
        TestCase {
            rewards_balance: vec![
                Coin::new(200, "usei"),
                Coin::new(300, "kusd"),
                Coin::new(500, "usdr"),
                Coin::new(100, "mnt"),
            ],
            stsei_total_bonded: Uint128::from(2u128),
            bsei_total_bonded: Uint128::from(2u128),
            expected_total_sei_rewards_available: "200".to_string(),
            expected_total_ust_rewards_available: "1300".to_string(),
            expected_offer_coin_denom: "usei".to_string(),
            expected_offer_coin_amount: "80".to_string(),
            expected_ask_denom: "kusd".to_string(),
        },
        TestCase {
            rewards_balance: vec![
                Coin::new(200, "usei"),
                Coin::new(300, "kusd"),
                Coin::new(500, "usdr"),
                Coin::new(100, "mnt"),
            ],
            stsei_total_bonded: Uint128::from(2u128),
            bsei_total_bonded: Uint128::from(1u128),
            expected_total_sei_rewards_available: "200".to_string(),
            expected_total_ust_rewards_available: "1300".to_string(),
            expected_offer_coin_denom: "usei".to_string(),
            expected_offer_coin_amount: "40".to_string(),
            expected_ask_denom: "kusd".to_string(),
        },
        TestCase {
            rewards_balance: vec![
                Coin::new(0, "usei"),
                Coin::new(300, "kusd"),
                Coin::new(500, "usdr"),
                Coin::new(100, "mnt"),
            ],
            stsei_total_bonded: Uint128::from(2u128),
            bsei_total_bonded: Uint128::from(2u128),
            expected_total_sei_rewards_available: "0".to_string(),
            expected_total_ust_rewards_available: "1300".to_string(),
            expected_offer_coin_denom: "kusd".to_string(),
            expected_offer_coin_amount: "640".to_string(),
            expected_ask_denom: "usei".to_string(),
        },
    ];

    for test_case in test_cases {
        let mut deps = mock_dependencies(&test_case.rewards_balance);

        let msg = default_init();
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        // println!(" ", query_config(deps.as_ref()).unwrap());

        let info = mock_info(String::from(MOCK_HUB_CONTRACT_ADDR).as_str(), &[]);
        let msg = ExecuteMsg::SwapToRewardDenom {
            stsei_total_bonded: test_case.stsei_total_bonded,
            bsei_total_bonded: test_case.bsei_total_bonded,
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        for attr in res.attributes {
            if attr.key == *"total_sei_rewards_available" {
                assert_eq!(attr.value, test_case.expected_total_sei_rewards_available)
            }
            if attr.key == *"total_ust_rewards_available" {
                assert_eq!(attr.value, test_case.expected_total_ust_rewards_available)
            }
            if attr.key == *"offer_coin_denom" {
                assert_eq!(attr.value, test_case.expected_offer_coin_denom)
            }
            if attr.key == *"offer_coin_amount" {
                assert_eq!(attr.value, test_case.expected_offer_coin_amount)
            }
            if attr.key == *"ask_denom" {
                assert_eq!(attr.value, test_case.expected_ask_denom)
            }
        }
    }
}

#[test]
fn test_dispatch_rewards() {
    let mut deps = mock_dependencies(&[
        Coin::new(200, "usei"),
        Coin::new(300, "kusd"),
        Coin::new(20, "usdr"),
    ]);

    let msg = default_init();
    let info = mock_info("creator", &[]);

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let info = mock_info(String::from(MOCK_HUB_CONTRACT_ADDR).as_str(), &[]);
    let msg = ExecuteMsg::DispatchRewards {};

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(3, res.messages.len());

    for attr in res.attributes {
        if attr.key == "stsei_rewards" {
            assert_eq!("190usei", attr.value)
        }
        if attr.key == "bsei_rewards" {
            assert_eq!("300kusd", attr.value)
        }
        if attr.key == "lido_stsei_fee" {
            assert_eq!("10usei", attr.value)
        }
        if attr.key == "lido_bsei_fee" {
            assert_eq!("14kusd", attr.value)
        }
    }
}

#[test]
fn test_dispatch_rewards_zero_krp_keeper_rate() {
    let mut deps = mock_dependencies(&[
        Coin::new(200, "usei"),
        Coin::new(300, "kusd"),
        Coin::new(20, "usdr"),
    ]);

    let msg = InstantiateMsg {
        hub_contract: MOCK_HUB_CONTRACT_ADDR.to_string(),
        bsei_reward_contract: String::from(MOCK_BSEI_REWARD_CONTRACT_ADDR),
        bsei_reward_denom: BTOKEN_REWARD_DENOM.to_string(),
        stsei_reward_denom: STTOKEN_REWARD_DENOM.to_string(),
        krp_keeper_address: String::from(MOCK_KRP_KEEPER_CONTRACT_ADDR),
        krp_keeper_rate: Decimal::zero(),
        swap_contract: String::from(MOCK_SWAP_CONTRACT_ADDR),
        swap_denoms: vec![],
        oracle_contract: String::from(MOCK_ORACLE_CONTRACT_ADDR),
    };
    let info = mock_info("creator", &[]);

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let info = mock_info(String::from(MOCK_HUB_CONTRACT_ADDR).as_str(), &[]);
    let msg = ExecuteMsg::DispatchRewards {};

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(3, res.messages.len());

    for attr in res.attributes {
        if attr.key == "stsei_rewards" {
            assert_eq!("200usei", attr.value)
        }
        if attr.key == "bsei_rewards" {
            assert_eq!("300kusd", attr.value)
        }
    }
}

#[test]
fn test_get_swap_info() {
    let mut deps = mock_dependencies(&[]);

    let msg = default_init();
    let info = mock_info("creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let config = CONFIG.load(&deps.storage).unwrap();

    let stsei_total_bond_amount = Uint128::from(2u64);
    let bsei_total_bond_amount = Uint128::from(2u64);
    let total_sei_rewards_available = Uint128::from(20u64);
    let total_ust_rewards_available = Uint128::from(20u64);
    let bsei_2_stsei_rewards_xchg_rate =
        Decimal::from_ratio(Uint128::from(1u64), Uint128::from(1u64));
    let stsei_2_bsei_rewards_xchg_rate =
        Decimal::from_ratio(Uint128::from(1u64), Uint128::from(1u64));
    let (offer_coin, _) = get_swap_info(
        config.clone(),
        stsei_total_bond_amount,
        bsei_total_bond_amount,
        total_sei_rewards_available,
        total_ust_rewards_available,
        bsei_2_stsei_rewards_xchg_rate,
        stsei_2_bsei_rewards_xchg_rate,
    )
    .unwrap();
    assert_eq!(offer_coin.denom, config.bsei_reward_denom);
    assert_eq!(offer_coin.amount, Uint128::zero());

    let stsei_total_bond_amount = Uint128::from(2u64);
    let bsei_total_bond_amount = Uint128::from(2u64);
    let total_sei_rewards_available = Uint128::from(20u64);
    let total_ust_rewards_available = Uint128::from(20u64);
    let bsei_2_stsei_rewards_xchg_rate =
        Decimal::from_ratio(Uint128::from(15u64), Uint128::from(10u64));
    let stsei_2_bsei_rewards_xchg_rate =
        Decimal::from_ratio(Uint128::from(10u64), Uint128::from(15u64));
    let (offer_coin, _) = get_swap_info(
        config.clone(),
        stsei_total_bond_amount,
        bsei_total_bond_amount,
        total_sei_rewards_available,
        total_ust_rewards_available,
        bsei_2_stsei_rewards_xchg_rate,
        stsei_2_bsei_rewards_xchg_rate,
    )
    .unwrap();
    assert_eq!(offer_coin.denom, config.bsei_reward_denom);
    assert_eq!(offer_coin.amount, Uint128::from(3u64));

    let stsei_total_bond_amount = Uint128::from(2u64);
    let bsei_total_bond_amount = Uint128::from(2u64);
    let total_sei_rewards_available = Uint128::from(20u64);
    let total_ust_rewards_available = Uint128::from(20u64);
    let bsei_2_stsei_rewards_xchg_rate =
        Decimal::from_ratio(Uint128::from(75u64), Uint128::from(100u64));
    let stsei_2_bsei_rewards_xchg_rate =
        Decimal::from_ratio(Uint128::from(100u64), Uint128::from(75u64));
    let (offer_coin, _) = get_swap_info(
        config.clone(),
        stsei_total_bond_amount,
        bsei_total_bond_amount,
        total_sei_rewards_available,
        total_ust_rewards_available,
        bsei_2_stsei_rewards_xchg_rate,
        stsei_2_bsei_rewards_xchg_rate,
    )
    .unwrap();
    assert_eq!(offer_coin.denom, config.stsei_reward_denom);
    assert_eq!(offer_coin.amount, Uint128::from(3u64));
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);

    let owner = String::from("creator");
    let msg = default_init();
    let info = mock_info(&owner, &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    //check call from invalid owner
    let invalid_owner = String::from("invalid_owner");
    let update_config_msg = ExecuteMsg::UpdateConfig {
        // owner: Some(String::from("some_addr")),
        hub_contract: None,
        bsei_reward_contract: None,
        stsei_reward_denom: None,
        bsei_reward_denom: None,
        krp_keeper_address: None,
        krp_keeper_rate: None,
    };
    let info = mock_info(&invalid_owner, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, update_config_msg);
    assert_eq!(res.unwrap_err(), StdError::generic_err("unauthorized"));

    // change owner
    let new_owner = String::from("new_owner");
    let update_config_msg = ExecuteMsg::UpdateConfig {
        // owner: Some(new_owner.clone()),
        hub_contract: None,
        bsei_reward_contract: None,
        stsei_reward_denom: None,
        bsei_reward_denom: None,
        krp_keeper_address: None,
        krp_keeper_rate: None,
    };
    let info = mock_info(&owner, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, update_config_msg);
    assert!(res.is_ok());

    let config = CONFIG.load(&deps.storage).unwrap();
    let new_owner_raw = deps.api.addr_canonicalize(&new_owner).unwrap();
    assert_eq!(new_owner_raw, config.owner);

    // change hub_contract
    let update_config_msg = ExecuteMsg::UpdateConfig {
        // owner: None,
        hub_contract: Some(String::from("some_address")),
        bsei_reward_contract: None,
        stsei_reward_denom: None,
        bsei_reward_denom: None,
        krp_keeper_address: None,
        krp_keeper_rate: None,
    };
    let info = mock_info(&new_owner, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, update_config_msg);
    assert!(res.is_ok());

    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(
        deps.api
            .addr_canonicalize(&String::from("some_address"))
            .unwrap(),
        config.hub_contract
    );

    // change bsei_reward_contract
    let update_config_msg = ExecuteMsg::UpdateConfig {
        // owner: None,
        hub_contract: None,
        bsei_reward_contract: Some(String::from("some_address")),
        stsei_reward_denom: None,
        bsei_reward_denom: None,
        krp_keeper_address: None,
        krp_keeper_rate: None,
    };
    let info = mock_info(&new_owner, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, update_config_msg);
    assert!(res.is_ok());

    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(
        deps.api
            .addr_canonicalize(&String::from("some_address"))
            .unwrap(),
        config.bsei_reward_contract
    );

    // change stsei_reward_denom
    let update_config_msg = ExecuteMsg::UpdateConfig {
        // owner: None,
        hub_contract: None,
        bsei_reward_contract: None,
        stsei_reward_denom: Some(String::from("new_denom")),
        bsei_reward_denom: None,
        krp_keeper_address: None,
        krp_keeper_rate: None,
    };
    let info = mock_info(&new_owner, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, update_config_msg);
    assert!(res.is_err());
    assert_eq!(
        Some(StdError::generic_err(
            "updating stSei reward denom is forbidden"
        )),
        res.err()
    );

    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(String::from("usei"), config.stsei_reward_denom);

    // change bsei_reward_denom
    let update_config_msg = ExecuteMsg::UpdateConfig {
        // owner: None,
        hub_contract: None,
        bsei_reward_contract: None,
        stsei_reward_denom: None,
        bsei_reward_denom: Some(String::from("new_denom")),
        krp_keeper_address: None,
        krp_keeper_rate: None,
    };
    let info = mock_info(&new_owner, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, update_config_msg);
    assert!(res.is_err());
    assert_eq!(
        Some(StdError::generic_err(
            "updating bSei reward denom is forbidden"
        )),
        res.err()
    );

    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(String::from("kusd"), config.bsei_reward_denom);

    // change krp_keeper_address
    let update_config_msg = ExecuteMsg::UpdateConfig {
        // owner: None,
        hub_contract: None,
        bsei_reward_contract: None,
        stsei_reward_denom: None,
        bsei_reward_denom: None,
        krp_keeper_address: Some(String::from("some_address")),
        krp_keeper_rate: None,
    };
    let info = mock_info(&new_owner, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, update_config_msg);
    assert!(res.is_ok());

    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(
        deps.api
            .addr_canonicalize(&String::from("some_address"))
            .unwrap(),
        config.krp_keeper_address
    );

    // change krp_keeper_rate
    let update_config_msg = ExecuteMsg::UpdateConfig {
        // owner: None,
        hub_contract: None,
        bsei_reward_contract: None,
        stsei_reward_denom: None,
        bsei_reward_denom: None,
        krp_keeper_address: None,
        krp_keeper_rate: Some(Decimal::one()),
    };
    let info = mock_info(&new_owner, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, update_config_msg);
    assert!(res.is_ok());

    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(Decimal::one(), config.krp_keeper_rate);
}
