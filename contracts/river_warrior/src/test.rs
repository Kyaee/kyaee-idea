use super::{RiverWarriorContract, RiverWarriorContractClient};
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

fn setup() -> (
    Env,
    Address,
    Address,
    Address,
    RiverWarriorContractClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let collector = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(admin.clone());
    let token_sa = StellarAssetClient::new(&env, &token_id);
    let contract_id = env.register_contract(None, RiverWarriorContract);
    let client = RiverWarriorContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_id, &10_000_000_i128);
    token_sa.mint(&contract_id, &1_000_000_000_i128);
    (env, admin, collector, token_id, client)
}

#[test]
fn test_disburse_reward_happy_path() {
    let (env, _admin, collector, token_id, client) = setup();
    let token = TokenClient::new(&env, &token_id);
    let before = token.balance(&collector);
    client.disburse_reward(&collector);
    let after = token.balance(&collector);
    assert_eq!(
        after - before,
        10_000_000_i128,
        "Collector should receive exactly 1 USDC (7 decimals test asset uses standard 7?)"
    );
}

#[test]
#[should_panic(expected = "already claimed this period")]
fn test_double_claim_rejected() {
    let (_env, _admin, collector, _token_id, client) = setup();
    client.disburse_reward(&collector);
    client.disburse_reward(&collector);
}

#[test]
fn test_total_disbursed_state() {
    let (_env, _admin, collector, _token_id, client) = setup();
    assert_eq!(client.get_total_disbursed(), 0_i128);
    client.disburse_reward(&collector);
    assert_eq!(
        client.get_total_disbursed(),
        10_000_000_i128,
        "Total disbursed must reflect the payment"
    );
}

#[test]
#[should_panic]
fn test_unauthorized_disburse() {
    let env2 = Env::default();
    let fake_admin = Address::generate(&env2);
    let token_id2 = env2.register_stellar_asset_contract(fake_admin.clone());
    let token_sa2 = StellarAssetClient::new(&env2, &token_id2);
    let contract_id2 = env2.register_contract(None, RiverWarriorContract);
    let client2 = RiverWarriorContractClient::new(&env2, &contract_id2);
    client2.initialize(&fake_admin, &token_id2, &10_000_000_i128);
    token_sa2.mint(&contract_id2, &1_000_000_000_i128);
    let random = Address::generate(&env2);
    client2.disburse_reward(&random);
}

#[test]
fn test_set_bounty_and_verify() {
    let (env, _admin, collector, token_id, client) = setup();
    let token = TokenClient::new(&env, &token_id);
    client.set_bounty(&20_000_000_i128);
    assert_eq!(client.get_bounty(), 20_000_000_i128);
    let before = token.balance(&collector);
    client.disburse_reward(&collector);
    let after = token.balance(&collector);
    assert_eq!(
        after - before,
        20_000_000_i128,
        "Payout must use the updated bounty amount"
    );
}
