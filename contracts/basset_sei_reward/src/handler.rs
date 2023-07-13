use crate::{state::{read_config, store_config}, error::ContractError};
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
    owner_addr: Option<Addr>,
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

    if let Some(owner_addr) = owner_addr {
        config.owner = deps.api.addr_canonicalize(owner_addr.as_str())?
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
