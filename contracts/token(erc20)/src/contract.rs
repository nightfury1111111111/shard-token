use cosmwasm_std::{
    entry_point, to_binary, to_vec, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Storage, Uint128,
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use std::convert::TryInto;

use crate::error::ContractError;
use crate::msg::{AllowanceResponse, BalanceResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::Constants;

pub const PREFIX_CONFIG: &[u8] = b"config";
pub const PREFIX_BALANCES: &[u8] = b"balances";
pub const PREFIX_ALLOWANCES: &[u8] = b"allowances";

pub const KEY_CONSTANTS: &[u8] = b"constants";
pub const KEY_TOTAL_SUPPLY: &[u8] = b"total_supply";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut total_supply: u128 = 0;
    {
        // Initial balances
        let mut balances_store = PrefixedStorage::new(deps.storage, PREFIX_BALANCES);
        for row in msg.initial_balances {
            let amount_raw = row.amount.u128();
            balances_store.set(row.address.as_str().as_bytes(), &amount_raw.to_be_bytes());
            total_supply += amount_raw;
        }
    }

    // Check name, symbol, decimals
    if !is_valid_name(&msg.name) {
        return Err(ContractError::NameWrongFormat {});
    }
    if !is_valid_symbol(&msg.symbol) {
        return Err(ContractError::TickerWrongSymbolFormat {});
    }
    if msg.decimals > 18 {
        return Err(ContractError::DecimalsExceeded {});
    }

    let mut config_store = PrefixedStorage::new(deps.storage, PREFIX_CONFIG);
    let constants = to_vec(&Constants {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
    })?;
    config_store.set(KEY_CONSTANTS, &constants);
    config_store.set(KEY_TOTAL_SUPPLY, &total_supply.to_be_bytes());

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Approve { spender, amount } => try_approve(deps, env, info, spender, &amount),
        ExecuteMsg::Transfer { recipient, amount } => {
            try_transfer(deps, env, info, recipient, &amount)
        }
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => try_transfer_from(deps, env, info, owner, recipient, &amount),
        ExecuteMsg::Burn { amount } => try_burn(deps, env, info, &amount),
    }
}