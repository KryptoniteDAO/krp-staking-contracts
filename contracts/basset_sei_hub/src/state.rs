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

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    attr,from_json, to_json_vec, Decimal, Order, Response, StdError, StdResult, Storage, Uint128, CanonicalAddr,
};
use cosmwasm_storage::{Bucket, PrefixedStorage, ReadonlyBucket, ReadonlyPrefixedStorage, Singleton, ReadonlySingleton};

use cw_storage_plus::Item;

use basset::hub::{
    Config, CurrentBatch, OldConfig, OldCurrentBatch, OldState, Parameters, State, UnbondHistory,
    UnbondRequest, UnbondType, UnbondWaitEntity,
};

pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");
pub const PARAMETERS: Item<Parameters> = Item::new("\u{0}\u{b}parameteres");
pub const CURRENT_BATCH: Item<CurrentBatch> = Item::new("\u{0}\u{d}current_batch");
pub const STATE: Item<State> = Item::new("\u{0}\u{5}state");

pub const OLD_CONFIG: Item<OldConfig> = Item::new("\u{0}\u{6}config");
pub const OLD_CURRENT_BATCH: Item<OldCurrentBatch> = Item::new("\u{0}\u{d}current_batch");
pub const OLD_STATE: Item<OldState> = Item::new("\u{0}\u{5}state");

pub static OLD_PREFIX_WAIT_MAP: &[u8] = b"wait";
pub static NEW_PREFIX_WAIT_MAP: &[u8] = b"v2_wait";
pub static UNBOND_HISTORY_MAP: &[u8] = b"history_map";
pub static PREFIX_AIRDROP_INFO: &[u8] = b"airedrop_info";
pub static VALIDATORS: &[u8] = b"validators";
pub static KEY_NEWOWNER: &[u8] = b"newowner";


pub const MAX_DEFAULT_RANGE_LIMIT: u32 = 1000;



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NewOwnerAddr {
    pub new_owner_addr: CanonicalAddr, 
}

pub fn store_new_owner(storage: &mut dyn Storage, data: &NewOwnerAddr) -> StdResult<()> {
    Singleton::new(storage, KEY_NEWOWNER).save(data)
}

pub fn read_new_owner(storage: &dyn Storage) -> StdResult<NewOwnerAddr> {
    ReadonlySingleton::new(storage, KEY_NEWOWNER).load()
}

/// Store undelegation wait list per each batch
/// HashMap<user's address, <batch_id, requested_amount>
pub fn store_unbond_wait_list(
    storage: &mut dyn Storage,
    batch_id: u64,
    sender_address: String,
    amount: Uint128,
    unbond_type: UnbondType,
) -> StdResult<()> {
    let batch = to_json_vec(&batch_id)?;
    let addr = to_json_vec(&sender_address)?;
    let mut position_indexer: Bucket<UnbondWaitEntity> =
        Bucket::multilevel(storage, &[NEW_PREFIX_WAIT_MAP, &addr]);
    position_indexer.update(&batch, |asked_already| -> StdResult<UnbondWaitEntity> {
        let mut wl = asked_already.unwrap_or_default();
        match unbond_type {
            UnbondType::BSei => wl.bsei_amount += amount,
            UnbondType::StSei => wl.stsei_amount += amount,
        }
        Ok(wl)
    })?;

    Ok(())
}

/// Remove unbond batch id from user's wait list
pub fn remove_unbond_wait_list(
    storage: &mut dyn Storage,
    batch_id: Vec<u64>,
    sender_address: String,
) -> StdResult<()> {
    let addr = to_json_vec(&sender_address)?;
    let mut position_indexer: Bucket<UnbondWaitEntity> =
        Bucket::multilevel(storage, &[NEW_PREFIX_WAIT_MAP, &addr]);
    for b in batch_id {
        let batch = to_json_vec(&b)?;
        position_indexer.remove(&batch);
    }
    Ok(())
}

pub fn read_unbond_wait_list(
    storage: &dyn Storage,
    batch_id: u64,
    sender_addr: String,
) -> StdResult<UnbondWaitEntity> {
    let vec = to_json_vec(&sender_addr)?;
    let res: ReadonlyBucket<UnbondWaitEntity> =
        ReadonlyBucket::multilevel(storage, &[NEW_PREFIX_WAIT_MAP, &vec]);
    let batch = to_json_vec(&batch_id)?;
    let wl = res.load(&batch)?;
    Ok(wl)
}

pub fn get_unbond_requests(storage: &dyn Storage, sender_addr: String) -> StdResult<UnbondRequest> {
    let vec = to_json_vec(&sender_addr)?;
    let mut requests: UnbondRequest = vec![];
    let res: ReadonlyBucket<UnbondWaitEntity> =
        ReadonlyBucket::multilevel(storage, &[NEW_PREFIX_WAIT_MAP, &vec]);
    for item in res.range(None, None, Order::Ascending) {
        let (k, value) = item?;
        let user_batch: u64 =from_json(&k)?;
        requests.push((user_batch, value.bsei_amount, value.stsei_amount))
    }
    Ok(requests)
}

/// Return all requested unbond amount.
/// This needs to be called after process withdraw rate function.
/// If the batch is released, this will return user's requested
/// amount proportional to withdraw rate.
pub fn get_finished_amount(
    storage: &dyn Storage,
    sender_addr: String,
) -> StdResult<(Uint128, Vec<u64>)> {
    let vec = to_json_vec(&sender_addr)?;
    let mut withdrawable_amount: Uint128 = Uint128::zero();
    let mut deprecated_batches: Vec<u64> = vec![];
    let res: ReadonlyBucket<UnbondWaitEntity> =
        ReadonlyBucket::multilevel(storage, &[NEW_PREFIX_WAIT_MAP, &vec]);
    for item in res.range(None, None, Order::Ascending) {
        let (k, v) = item?;
        let user_batch: u64 =from_json(&k)?;
        let history = read_unbond_history(storage, user_batch);
        if let Ok(h) = history {
            if h.released {
                withdrawable_amount +=
                    v.stsei_amount * h.stsei_withdraw_rate + v.bsei_amount * h.bsei_withdraw_rate;
                deprecated_batches.push(user_batch);
            }
        }
    }
    Ok((withdrawable_amount, deprecated_batches))
}

/// Return the finished amount for all batches that has been before the given block time.
pub fn query_get_finished_amount(
    storage: &dyn Storage,
    sender_addr: String,
    block_time: u64,
) -> StdResult<Uint128> {
    let vec = to_json_vec(&sender_addr)?;
    let mut withdrawable_amount: Uint128 = Uint128::zero();
    let res: ReadonlyBucket<UnbondWaitEntity> =
        ReadonlyBucket::multilevel(storage, &[NEW_PREFIX_WAIT_MAP, &vec]);
    for item in res.range(None, None, Order::Ascending) {
        let (k, v) = item?;
        let user_batch: u64 =from_json(&k)?;
        let history = read_unbond_history(storage, user_batch);
        if let Ok(h) = history {
            if h.time < block_time {
                withdrawable_amount +=
                    v.stsei_amount * h.stsei_withdraw_rate + v.bsei_amount * h.bsei_withdraw_rate;
            }
        }
    }
    Ok(withdrawable_amount)
}

/// Store unbond history map
/// Hashmap<batch_id, <UnbondHistory>>
pub fn store_unbond_history(
    storage: &mut dyn Storage,
    batch_id: u64,
    history: UnbondHistory,
) -> StdResult<()> {
    let vec = batch_id.to_be_bytes().to_vec();
    let value: Vec<u8> = to_json_vec(&history)?;
    PrefixedStorage::new(storage, UNBOND_HISTORY_MAP).set(&vec, &value);
    Ok(())
}

#[allow(clippy::needless_lifetimes)]
pub fn read_unbond_history(storage: &dyn Storage, epoc_id: u64) -> StdResult<UnbondHistory> {
    let vec = epoc_id.to_be_bytes().to_vec();
    let res = ReadonlyPrefixedStorage::new(storage, UNBOND_HISTORY_MAP).get(&vec);
    match res {
        Some(data) =>from_json(&data),
        None => Err(StdError::generic_err(
            "Burn requests not found for the specified time period",
        )),
    }
}

// settings for pagination
const MAX_LIMIT: u32 = 100;
const DEFAULT_LIMIT: u32 = 10;

/// Return all unbond_history from UnbondHistory map
#[allow(clippy::needless_lifetimes)]
pub fn all_unbond_history(
    storage: &dyn Storage,
    start: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<UnbondHistory>> {
    let vec = convert(start);

    let lim = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let res: StdResult<Vec<UnbondHistory>> =
        ReadonlyPrefixedStorage::new(storage, UNBOND_HISTORY_MAP)
            .range(vec.as_deref(), None, Order::Ascending)
            .take(lim)
            .map(|item| {
                let history: StdResult<UnbondHistory> =from_json(&item.1);
                history
            })
            .collect();
    res
}

fn convert(start_after: Option<u64>) -> Option<Vec<u8>> {
    start_after.map(|idx| {
        let mut v = idx.to_be_bytes().to_vec();
        v.push(1);
        v
    })
}

pub fn read_validators(storage: &dyn Storage) -> StdResult<Vec<String>> {
    let res = ReadonlyPrefixedStorage::new(storage, VALIDATORS);
    let validators: StdResult<Vec<String>> = res
        .range(None, None, Order::Ascending)
        .map(|item| {
            let (key, _) = item;
            let sender: StdResult<String> =from_json(&key);
            sender
        })
        .collect();
    validators
}

pub fn remove_whitelisted_validators_store(storage: &mut dyn Storage) -> StdResult<()> {
    let mut res = PrefixedStorage::new(storage, VALIDATORS);
    let items = res
        .range(None, None, Order::Ascending)
        .collect::<Vec<(Vec<u8>, Vec<u8>)>>();
    for (key, _) in items {
        res.remove(&key)
    }
    Ok(())
}

type OldUnbondWaitList = (Vec<u8>, Uint128);

pub fn read_old_unbond_wait_lists(
    storage: &mut dyn Storage,
    limit: Option<u32>,
) -> StdResult<Vec<StdResult<OldUnbondWaitList>>> {
    let reader: ReadonlyBucket<Uint128> =
        ReadonlyBucket::multilevel(storage, &[OLD_PREFIX_WAIT_MAP]);
    Ok(reader
        .range(None, None, Order::Ascending)
        .take(limit.unwrap_or(MAX_DEFAULT_RANGE_LIMIT) as usize)
        .collect::<Vec<StdResult<OldUnbondWaitList>>>())
}

// migrate_unbond_wait_lists moves the old values (Uint128) in OLD_PREFIX_WAIT_MAP storage to UnbondWaitEntity
// in NEW_PREFIX_WAIT_MAP and deletes the old entries.
pub fn migrate_unbond_wait_lists(
    storage: &mut dyn Storage,
    limit: Option<u32>,
) -> StdResult<Response> {
    let (removed_keys, num_migrated_entries) = {
        let old_unbond_wait_list_entries = read_old_unbond_wait_lists(storage, limit)?;
        if old_unbond_wait_list_entries.is_empty() {
            return Ok(Response::new().add_attributes(vec![
                attr("action", "migrate_unbond_wait_lists"),
                attr("num_migrated_entries", "0"),
            ]));
        }

        let mut num_migrated_entries: u32 = 0;
        let mut new_unbond_wait_list: Bucket<UnbondWaitEntity> =
            Bucket::multilevel(storage, &[NEW_PREFIX_WAIT_MAP]);
        let mut removed_keys: Vec<Vec<u8>> = vec![];

        for res in old_unbond_wait_list_entries {
            let (key, amount) = res?;
            let unbond_wait_entity = UnbondWaitEntity {
                bsei_amount: amount,
                stsei_amount: Uint128::zero(),
            };
            new_unbond_wait_list.save(&key, &unbond_wait_entity)?;
            removed_keys.push(key);
            num_migrated_entries += 1;
        }

        (removed_keys, num_migrated_entries)
    };

    let mut old_unbond_wait_list: Bucket<Uint128> =
        Bucket::multilevel(storage, &[OLD_PREFIX_WAIT_MAP]);
    for key in removed_keys {
        old_unbond_wait_list.remove(&key);
    }

    // unpause contract if we've migrated all unbond wait lists
    let old_unbond_wait_list_entries = read_old_unbond_wait_lists(storage, Some(1u32))?;
    if old_unbond_wait_list_entries.is_empty() {
        let mut params: Parameters = PARAMETERS.load(storage)?;
        params.paused = Some(false);
        PARAMETERS.save(storage, &params)?;
    }

    Ok(Response::new().add_attributes(vec![
        attr("action", "migrate_unbond_wait_lists"),
        attr("num_migrated_entries", num_migrated_entries.to_string()),
    ]))
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OldUnbondHistory {
    pub batch_id: u64,
    pub time: u64,
    pub amount: Uint128,
    pub applied_exchange_rate: Decimal,
    pub withdraw_rate: Decimal,
    pub released: bool,
}

pub fn migrate_unbond_history(storage: &mut dyn Storage) -> StdResult<()> {
    let unbond_history: StdResult<Vec<UnbondHistory>> =
        ReadonlyPrefixedStorage::new(storage, UNBOND_HISTORY_MAP)
            .range(None, None, Order::Ascending)
            .map(|item| {
                let old_history: OldUnbondHistory = match from_json(&item.1) {
                    Ok(h) => h,
                    Err(e) => return Err(e),
                };
                let new_history = UnbondHistory {
                    batch_id: old_history.batch_id,
                    time: old_history.time,
                    bsei_amount: old_history.amount,
                    bsei_applied_exchange_rate: old_history.applied_exchange_rate,
                    bsei_withdraw_rate: old_history.withdraw_rate,
                    stsei_amount: Uint128::zero(),
                    stsei_applied_exchange_rate: Decimal::one(),
                    stsei_withdraw_rate: Decimal::one(),
                    released: old_history.released,
                };
                Ok(new_history)
            })
            .collect();

    for history in unbond_history? {
        store_unbond_history(storage, history.batch_id, history)?;
    }
    Ok(())
}
