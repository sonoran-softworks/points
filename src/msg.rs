use std::collections::HashMap;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw721::Cw721ReceiveMsg;
use nois::NoisCallback;

#[cw_serde]
pub enum ExecuteMsg {
    Cw721Receive(Cw721ReceiveMsg),
    ClaimPrize { address: Addr },
    NoisReceive { callback: NoisCallback },
    SetAdmin { address: Addr },
    SetPrizeCost { cost: u64 },
}

#[cw_serde]
pub struct InstantiateMsg {
    pub nois_proxy: String,
    pub prize_cost: u64,
    pub short_description: String,
    pub name: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<(String, String)>)]
    PrizePool {},
    #[returns(u64)]
    PrizeCost {},
    #[returns(HashMap<String, (u64, u64)>)]
    Balances {},
    #[returns(u64)]
    LifetimeBalance { address: Addr },
    #[returns(u64)]
    SpentBalance { address: Addr },
}
