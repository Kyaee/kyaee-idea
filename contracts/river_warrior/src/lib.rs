#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    token::Client as TokenClient,
    Address, Env, Symbol,
};

#[contracttype]
pub enum DataKey {
    /// Address allowed to trigger disbursements
    Admin,
    /// USDC (or reward token) contract address
    Token,
    /// Reward per verified submission (stroops)
    BountyAmount,
    /// Running total paid out
    TotalDisbursed,
    /// Tracks if an address already claimed in the current temporary TTL window
    Claimed(Address),
}

#[contract]
pub struct RiverWarriorContract;

#[contractimpl]
impl RiverWarriorContract {
    /// Called once after deploy. Sets admin, token, and bounty amount.
    pub fn initialize(env: Env, admin: Address, token: Address, bounty_amount: i128) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::BountyAmount, &bounty_amount);
        env.storage().instance().set(&DataKey::TotalDisbursed, &0_i128);
    }

    /// Admin authorizes this call; transfers `bounty_amount` from this contract to `collector`.
    pub fn disburse_reward(env: Env, collector: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let claim_key = DataKey::Claimed(collector.clone());
        if env.storage().temporary().has(&claim_key) {
            panic!("already claimed this period");
        }

        let amount: i128 = env.storage().instance().get(&DataKey::BountyAmount).unwrap();
        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();

        TokenClient::new(&env, &token_addr).transfer(
            &env.current_contract_address(),
            &collector,
            &amount,
        );

        env.storage().temporary().set(&claim_key, &true);
        env.storage().temporary().extend_ttl(&claim_key, 17280, 17280);

        let total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalDisbursed)
            .unwrap();
        env.storage()
            .instance()
            .set(&DataKey::TotalDisbursed, &(total + amount));

        env.events().publish(
            (Symbol::new(&env, "reward_disbursed"), collector.clone()),
            amount,
        );
    }

    pub fn get_bounty(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::BountyAmount).unwrap()
    }

    pub fn get_total_disbursed(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalDisbursed).unwrap()
    }

    pub fn set_bounty(env: Env, new_amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::BountyAmount, &new_amount);
    }
}

#[cfg(test)]
mod test;
