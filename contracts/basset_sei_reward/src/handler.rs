use crate::{state::{read_config, store_config, read_new_owner, store_new_owner}, error::ContractError};
use cosmwasm_std::{DepsMut, MessageInfo, Response, StdError, Addr};


pub fn update_swap_denom(
    deps: DepsMut,
    info: MessageInfo,
    swap_denom: String,
    is_add: bool,
) -> Result<Response, ContractError> {
    let mut config = read_config(deps.storage)?;
    if config.owner != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Std(StdError::generic_err("Unauthorized")));
    }
    if is_add {
        config.swap_denoms.push(swap_denom.clone());
    } else {
        config.swap_denoms.retain(|x| x != &swap_denom);
    }
    store_config(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_swap_denom")
        .add_attribute("swap_denom", swap_denom.as_str())
        .add_attribute("owner", info.sender))
}

pub fn udpate_config(
    deps: DepsMut,
    info: MessageInfo,
    hub_contract: Option<Addr>,
    reward_denom: Option<String>,
    swap_contract: Option<Addr>,
) -> Result<Response, ContractError> {
    let mut config = read_config(deps.as_ref().storage)?;
    let sender_raw = deps.api.addr_canonicalize(info.sender.as_str())?;

    if sender_raw != config.owner {
        return Err(ContractError::Unauthorized(
            "update_config".to_string(),
            info.sender.to_string(),
        ));
    }

    if let Some(hub_contract) = hub_contract {
        config.hub_contract = deps.api.addr_canonicalize(hub_contract.as_str())?
    }

    if let Some(reward_denom) = reward_denom {
        config.reward_denom = reward_denom;
    }

    if let Some(swap_contract) = swap_contract {
        config.swap_contract = deps.api.addr_canonicalize(swap_contract.as_str())?;
    }

    store_config(deps.storage, &config)?;
    Ok(Response::default())
}



pub fn set_new_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner_addr: Addr,
) -> Result<Response, ContractError> {
    let config = read_config(deps.as_ref().storage)?;
    let mut new_owner = read_new_owner(deps.as_ref().storage)?;
    let sender_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    if sender_raw != config.owner {
        return Err(ContractError::Unauthorized("set_new_owner".to_string(), info.sender.to_string()));
    }
    new_owner.new_owner_addr = deps.api.addr_canonicalize(&new_owner_addr.to_string())?;
    store_new_owner(deps.storage, &new_owner)?;

    Ok(Response::default())
}

pub fn accept_ownership(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let new_owner = read_new_owner(deps.as_ref().storage)?;
    let sender_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    let mut config =  read_config(deps.as_ref().storage)?;
    if sender_raw != new_owner.new_owner_addr {
        return Err(ContractError::Unauthorized("accept_ownership".to_string(), info.sender.to_string()));
    }

    config.owner = new_owner.new_owner_addr;
    store_config(deps.storage, &config)?;

    Ok(Response::default())
}