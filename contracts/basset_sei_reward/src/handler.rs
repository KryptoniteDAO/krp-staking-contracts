use cosmwasm_std::{DepsMut, MessageInfo, Response, StdError, StdResult};
use crate::state::{read_config, store_config};

pub fn update_swap_contract(
    deps: DepsMut,
    info: MessageInfo,
    swap_contract: String,
) -> StdResult<Response> {
    let mut config = read_config(deps.storage)?;
    if config.owner != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(StdError::generic_err("Unauthorized"));
    }
    config.swap_contract = deps.api.addr_canonicalize(swap_contract.as_str())?;
    store_config(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_swap_contract")
        .add_attribute("swap_contract", swap_contract)
        .add_attribute("owner", info.sender))
}

pub fn update_swap_denom(
    deps: DepsMut,
    info: MessageInfo,
    swap_denom: String,
    is_add: bool,
)-> StdResult<Response> {
    let mut config = read_config(deps.storage)?;
    if config.owner != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(StdError::generic_err("Unauthorized"));
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
