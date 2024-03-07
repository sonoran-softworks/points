use std::collections::HashMap;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, POINTS_STATE};
#[cfg(not(feature = "library"))]
use crate::state::{NOIS_PROXY, RND_OUTCOME};
use cosmwasm_std::{
    ensure, ensure_eq, entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw721::Cw721ExecuteMsg::TransferNft;
use cw721::Cw721ReceiveMsg;
use nois::NoisCallback;
const CONTRACT_NAME: &str = "crates.io:points";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    POINTS_STATE.save(
        deps.storage,
        &State {
            admin: info.sender.to_string(),
            symbol: "POINTS".to_string(),
            short_description: msg.short_description,
            balances: HashMap::new(),
            locked: false,
            prize_pool: vec![],
            prize_cost: msg.prize_cost,
            whitelist: vec![],
        },
    )?;

    let nois_proxy_addr = deps
        .api
        .addr_validate(&msg.nois_proxy)
        .map_err(|_| ContractError::InvalidProxyAddress)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    NOIS_PROXY.save(deps.storage, &nois_proxy_addr)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Cw721Receive(msg) => receive_nft(deps, env, info, msg),
        ExecuteMsg::ClaimPrize { address } => claim_prize(deps, env, info, address),
        ExecuteMsg::NoisReceive { callback } => execute_receive(deps, env, info, callback),
        ExecuteMsg::SetAdmin { address } => set_admin(deps, env, info, address),
        ExecuteMsg::SetPrizeCost { cost } => set_prize_cost(deps, env, info, cost),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::PrizePool {} => to_json_binary(&get_prize_pool(deps)?),
        QueryMsg::PrizeCost {} => to_json_binary(&get_prize_cost(deps)?),
        QueryMsg::LifetimeBalance { address } => {
            to_json_binary(&get_balance(deps, address, "lifetime")?)
        }
        QueryMsg::SpentBalance { address } => to_json_binary(&get_balance(deps, address, "spent")?),
        QueryMsg::Balances {} => to_json_binary(&get_balances(deps)?),
    }
}

fn get_balances(deps: Deps) -> StdResult<Vec<(String, (u64, u64))>> {
    let state = POINTS_STATE.load(deps.storage)?;
    let mut balances = vec![];
    for (address, points) in state.balances.iter() {
        balances.push((
            address.to_string(),
            (points.lifetime_balance, points.spent_balance),
        ));
    }
    Ok(balances)
}

fn get_balance(deps: Deps, address: Addr, balance_type: &str) -> StdResult<u64> {
    let state = POINTS_STATE.load(deps.storage)?;
    let balance = state.balances.get(&address).unwrap();
    match balance_type {
        "lifetime" => Ok(balance.lifetime_balance),
        "spent" => Ok(balance.spent_balance),
        _ => panic!("Invalid balance type"),
    }
}

fn get_prize_cost(deps: Deps) -> StdResult<u64> {
    let state = POINTS_STATE.load(deps.storage)?;
    Ok(state.prize_cost)
}

fn get_prize_pool(deps: Deps) -> StdResult<Vec<(String, String)>> {
    let state = POINTS_STATE.load(deps.storage)?;
    Ok(state.prize_pool)
}

fn receive_nft(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let mut state = POINTS_STATE.load(deps.storage)?;
    state.prize_pool.push((msg.sender, msg.token_id));
    POINTS_STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

fn claim_prize(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    address: Addr,
) -> Result<Response, ContractError> {
    let mut state = POINTS_STATE.load(deps.storage)?;
    let address = deps
        .api
        .addr_validate(&address.into_string())
        .map_err(|_| ContractError::InvalidProxyAddress)?;

    // make sure address has a balance
    let balance = state
        .balances
        .get(&address)
        .ok_or(ContractError::Unauthorized {})?;
    ensure!(balance.lifetime_balance > 0, ContractError::Unauthorized {});

    // get the first randomness from RND_OUTCOME where the bool is false
    let outcome = RND_OUTCOME
        .range(deps.storage, None, None, Order::Ascending)
        .next()
        .ok_or(ContractError::NoRandomnessAvailable {})?;

    let prize = state.claim_prize(address.clone(), outcome.unwrap().1 .0);

    let send_msg = TransferNft {
        token_id: prize.1,
        recipient: address.into_string(),
    };
    let wasm_msg = WasmMsg::Execute {
        contract_addr: prize.0,
        msg: to_json_binary(&send_msg)?,
        funds: vec![],
    };

    POINTS_STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_message(wasm_msg)
        .add_attribute("method", "claim_prize"))
}

pub fn execute_receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    callback: NoisCallback,
) -> Result<Response, ContractError> {
    let proxy = NOIS_PROXY.load(deps.storage)?;
    ensure_eq!(info.sender, proxy, ContractError::UnauthorizedReceive {});

    let NoisCallback {
        job_id, randomness, ..
    } = callback;

    let randomness: [u8; 32] = randomness
        .to_array()
        .map_err(|_| ContractError::InvalidRandomness {})?;

    let response = match RND_OUTCOME.may_load(deps.storage, &job_id)? {
        None => Response::default(),
        Some(_randomness) => return Err(ContractError::JobIdAlreadyPresent {}),
    };
    RND_OUTCOME.save(deps.storage, &job_id, &(randomness, false))?;

    Ok(response)
}

fn set_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: Addr,
) -> Result<Response, ContractError> {
    let mut state = POINTS_STATE.load(deps.storage)?;
    ensure_eq!(info.sender, state.admin, ContractError::Unauthorized {});
    state.admin = address.into_string();
    POINTS_STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "set_admin"))
}

fn set_prize_cost(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cost: u64,
) -> Result<Response, ContractError> {
    let mut state = POINTS_STATE.load(deps.storage)?;
    ensure_eq!(info.sender, state.admin, ContractError::Unauthorized {});
    state.prize_cost = cost;
    POINTS_STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "set_prize_cost")
        .add_attribute("cost", cost.to_string()))
}

#[cfg(test)]
mod tests {
    // initialize
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw721::Cw721ReceiveMsg;
    use nois::NoisCallback;
    use std::str::FromStr;

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {
            nois_proxy: "nois_proxy".to_string(),
            prize_cost: 10,
            short_description: "short_description".to_string(),
            name: "name".to_string(),
        };
        let info = mock_info("creator", &[]);
        instantiate(deps, mock_env(), info, msg).unwrap();
    }
    
}
