use cosmwasm_std::{Binary, CanonicalAddr, Coin, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(PartialEq)]
pub enum BondType {
    BSei,
    StSei,
    BondRewards,
}

pub type UnbondRequest = Vec<(u64, Uint128, Uint128)>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub epoch_period: u64,
    pub underlying_coin_denom: String,
    pub unbonding_period: u64,
    pub peg_recovery_fee: Decimal,
    pub er_threshold: Decimal,
    pub reward_denom: String,
    pub update_reward_index_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct State {
    pub bsei_exchange_rate: Decimal,
    pub stsei_exchange_rate: Decimal,
    pub total_bond_bsei_amount: Uint128,
    pub total_bond_stsei_amount: Uint128,
    pub last_index_modification: u64,
    pub prev_hub_balance: Uint128,
    pub last_unbonded_time: u64,
    pub last_processed_batch: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct OldState {
    pub exchange_rate: Decimal,
    pub total_bond_amount: Uint128,
    pub last_index_modification: u64,
    pub prev_hub_balance: Uint128,
    pub actual_unbonded_amount: Uint128,
    pub last_unbonded_time: u64,
    pub last_processed_batch: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub creator: CanonicalAddr,
    pub update_reward_index_addr: CanonicalAddr,
    pub reward_dispatcher_contract: Option<CanonicalAddr>,
    pub validators_registry_contract: Option<CanonicalAddr>,
    pub bsei_token_contract: Option<CanonicalAddr>,
    pub stsei_token_contract: Option<CanonicalAddr>,
    pub airdrop_registry_contract: Option<CanonicalAddr>,
    pub rewards_contract: Option<CanonicalAddr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OldConfig {
    pub creator: CanonicalAddr,
    pub reward_contract: Option<CanonicalAddr>,
    pub token_contract: Option<CanonicalAddr>,
    pub airdrop_registry_contract: Option<CanonicalAddr>,
}

impl State {
    pub fn update_bsei_exchange_rate(
        &mut self,
        total_issued: Uint128,
        requested_with_fee: Uint128,
    ) {
        let actual_supply = total_issued + requested_with_fee;
        if self.total_bond_bsei_amount.is_zero() || actual_supply.is_zero() {
            self.bsei_exchange_rate = Decimal::one()
        } else {
            self.bsei_exchange_rate =
                Decimal::from_ratio(self.total_bond_bsei_amount, actual_supply);
        }
    }

    pub fn update_stsei_exchange_rate(&mut self, total_issued: Uint128, requested: Uint128) {
        let actual_supply = total_issued + requested;
        if self.total_bond_stsei_amount.is_zero() || actual_supply.is_zero() {
            self.stsei_exchange_rate = Decimal::one()
        } else {
            self.stsei_exchange_rate =
                Decimal::from_ratio(self.total_bond_stsei_amount, actual_supply);
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ////////////////////
    /// Owner's operations
    ////////////////////

    /// Set the owener
    UpdateConfig {
        rewards_dispatcher_contract: Option<String>,
        validators_registry_contract: Option<String>,
        bsei_token_contract: Option<String>,
        stsei_token_contract: Option<String>,
        airdrop_registry_contract: Option<String>,
        rewards_contract: Option<String>,
        update_reward_index_addr: Option<String>,
    },

    /// update the parameters that is needed for the contract
    UpdateParams {
        epoch_period: Option<u64>,
        unbonding_period: Option<u64>,
        peg_recovery_fee: Option<Decimal>,
        er_threshold: Option<Decimal>,
        paused: Option<bool>,
        reward_denom: Option<String>,
    },

    SetOwner {
        new_owner_addr: String,
    },

    AcceptOwnership {
    },

    ////////////////////
    /// User's operations
    ////////////////////

    /// Receives `amount` in underlying coin denom from sender.
    /// Delegate `amount` equally between validators from the registry.
    /// Issue `amount` / exchange_rate for the user.
    Bond {},

    BondForStSei {},

    BondRewards {},

    /// Update global index
    UpdateGlobalIndex {
        airdrop_hooks: Option<Vec<Binary>>,
    },

    /// Send back unbonded coin to the user
    WithdrawUnbonded {},

    /// Check whether the slashing has happened or not
    CheckSlashing {},

    ////////////////////
    /// bAsset's operations
    ///////////////////

    /// Receive interface for send token.
    /// Unbond the underlying coin denom.
    /// Burn the received basset token.
    Receive(Cw20ReceiveMsg),

    ////////////////////
    /// internal operations
    ///////////////////
    ClaimAirdrop {
        airdrop_token_contract: String,
        // Contract address of MIR Cw20 Token
        airdrop_contract: String,
        // Contract address of MIR Airdrop
        airdrop_swap_contract: String,
        // E.g. Contract address of MIR <> UST Terraswap Pair
        claim_msg: Binary,
        // Base64-encoded JSON of MIRAirdropHandleMsg::Claim
        swap_msg: Binary, // Base64-encoded string of JSON of PairHandleMsg::Swap
    },

    /// Swaps claimed airdrop tokens to UST through Terraswap & sends resulting UST to bsei Reward contract
    SwapHook {
        airdrop_token_contract: String,
        // E.g. contract address of MIR Token
        airdrop_swap_contract: String,
        // E.g. Contract address of MIR <> UST Terraswap Pair
        swap_msg: Binary, // E.g. Base64-encoded JSON of PairHandleMsg::Swap
    },

    RedelegateProxy {
        // delegator is automatically set to address of the calling contract
        src_validator: String,
        redelegations: Vec<(String, Coin)>, //(dst_validator, amount)
    },

    // MigrateUnbondWaitList migrates a limited amount of old waitlist entries to
    // the new state.
    MigrateUnbondWaitList {
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Unbond {},
    Convert {},
    // UpdateGlobalIndex {
    //     airdrop_hooks: Option<Vec<Binary>>,
    // },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Parameters {
    pub epoch_period: u64,
    pub underlying_coin_denom: String,
    pub unbonding_period: u64,
    pub peg_recovery_fee: Decimal,
    pub er_threshold: Decimal,
    pub reward_denom: String,
    pub paused: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentBatch {
    pub id: u64,
    pub requested_bsei_with_fee: Uint128,
    pub requested_stsei: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OldCurrentBatch {
    pub id: u64,
    pub requested_with_fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnbondHistory {
    pub batch_id: u64,
    pub time: u64,
    pub bsei_amount: Uint128,
    pub bsei_applied_exchange_rate: Decimal,
    pub bsei_withdraw_rate: Decimal,

    pub stsei_amount: Uint128,
    pub stsei_applied_exchange_rate: Decimal,
    pub stsei_withdraw_rate: Decimal,

    pub released: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnbondHistoryResponse {
    pub batch_id: u64,
    pub time: u64,
    pub bsei_amount: Uint128,
    pub bsei_applied_exchange_rate: Decimal,
    pub bsei_withdraw_rate: Decimal,

    pub stsei_amount: Uint128,
    pub stsei_applied_exchange_rate: Decimal,
    pub stsei_withdraw_rate: Decimal,

    pub released: bool,

    // #[deprecated]
    pub amount: Uint128,
    // #[deprecated]
    pub applied_exchange_rate: Decimal,
    // #[deprecated]
    pub withdraw_rate: Decimal,
}

#[derive(JsonSchema, Serialize, Deserialize, Default)]
pub struct UnbondWaitEntity {
    pub bsei_amount: Uint128,
    pub stsei_amount: Uint128,
}

pub enum UnbondType {
    BSei,
    StSei,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub bsei_exchange_rate: Decimal,
    pub stsei_exchange_rate: Decimal,
    pub total_bond_bsei_amount: Uint128,
    pub total_bond_stsei_amount: Uint128,
    pub last_index_modification: u64,
    pub prev_hub_balance: Uint128,
    pub last_unbonded_time: u64,
    pub last_processed_batch: u64,

    // #[deprecated]
    pub total_bond_amount: Uint128,
    // #[deprecated]
    pub exchange_rate: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub update_reward_index_addr: String,
    pub reward_dispatcher_contract: Option<String>,
    pub validators_registry_contract: Option<String>,
    pub bsei_token_contract: Option<String>,
    pub stsei_token_contract: Option<String>,
    pub airdrop_registry_contract: Option<String>,

    // #[deprecated]
    pub token_contract: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentBatchResponse {
    pub id: u64,
    pub requested_bsei_with_fee: Uint128,
    pub requested_stsei: Uint128,

    // #[deprecated]
    pub requested_with_fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawableUnbondedResponse {
    pub withdrawable: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnbondRequestsResponse {
    pub address: String,
    pub requests: UnbondRequest,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllHistoryResponse {
    pub history: Vec<UnbondHistoryResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub reward_dispatcher_contract: String,
    pub validators_registry_contract: String,
    pub stsei_token_contract: String,
    pub rewards_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    CurrentBatch {},
    WithdrawableUnbonded {
        address: String,
    },
    Parameters {},
    UnbondRequests {
        address: String,
    },
    AllHistory {
        start_from: Option<u64>,
        limit: Option<u32>,
    },
    NewOwner {},
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NewOwnerResponse {
    pub new_owner: String,
}
