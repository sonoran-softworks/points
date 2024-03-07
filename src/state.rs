use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use nois::ints_in_range;
use std::collections::HashMap;

pub const NOIS_PROXY: Item<Addr> = Item::new("nois_proxy");
pub const RND_OUTCOME: Map<&str, ([u8; 32], bool)> = Map::new("rnd_outcome");

#[cw_serde]
pub struct Points {
    pub lifetime_balance: u64,
    pub spent_balance: u64,
}

#[cw_serde]
pub struct State {
    pub admin: String,
    pub symbol: String,
    pub short_description: String,
    pub balances: HashMap<Addr, Points>,
    pub locked: bool,
    pub prize_pool: Vec<(String, String)>,
    pub prize_cost: u64,
    pub whitelist: Vec<Addr>,
}

impl State {
    pub fn add_points(&mut self, address: Addr, points: u64) {
        let balance = self.balances.entry(address).or_insert(Points {
            lifetime_balance: 0,
            spent_balance: 0,
        });
        balance.lifetime_balance += points;
    }
    pub fn spend_points(&mut self, address: Addr, points: u64) {
        let balance = self.balances.get_mut(&address).unwrap();
        assert!(balance.lifetime_balance - balance.spent_balance >= points);
        balance.spent_balance += points;
    }
    pub fn claim_prize(&mut self, address: Addr, randomness: [u8; 32]) -> (String, String) {
        // error if the prize pool is epmty
        assert!(!self.prize_pool.is_empty());
        // spend points from address
        self.spend_points(address.clone(), self.prize_cost);
        // get a prize from the prize pool
        // turn the randomness into a number from 0 to the length of the prize pool

        let num = ints_in_range(randomness, 1, 0, self.prize_pool.len() as u8);

        // get the index of the prize
        let prize = self.prize_pool.remove(num[0] as usize);
        prize
    }
}

pub const POINTS_BALANCES: Item<HashMap<String, u64>> = Item::new("balances");
pub const POINTS_STATE: Item<State> = Item::new("state");
