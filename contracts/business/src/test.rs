#[cfg(test)]
mod tests {

// AgroFlow contract test suite.
// Exactly 5 tests covering happy path, edge case failure,
// state verification, cancellation, and double-confirm protection.

use crate::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, Address, Env};

// Helper: deploys a mock USDC token contract and returns
// (token_address, token_admin_client) so tests can mint balances.
fn create_token_contract(env: &Env, admin: &Address) -> (Address, token::StellarAssetClient) {
    let contract_address = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let admin_client = token::StellarAssetClient::new(env, &contract_address);
    (contract_address, admin_client)
}

fn setup() -> (
    Env,
    AgroFlowContractClient<'static>,
    Address, // buyer
    Address, // farmer
    Address, // coop officer
    token::Client<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let farmer = Address::generate(&env);
    let coop_officer = Address::generate(&env);

    let (usdc_address, usdc_admin) = create_token_contract(&env, &admin);
    let usdc_client = token::Client::new(&env, &usdc_address);

    // Mint 1000 USDC (in stroops, 7 decimals assumed -> use raw units for test simplicity)
    usdc_admin.mint(&buyer, &1000);

    let contract_id = env.register(AgroFlowContract, ());
    let client = AgroFlowContractClient::new(&env, &contract_id);
    client.initialize(&usdc_address, &coop_officer);

    (env, client, buyer, farmer, coop_officer, usdc_client)
}

#[test]
fn test_happy_path_full_escrow_flow() {
    // Test 1: MVP transaction executes successfully end-to-end.
    let (_, client, buyer, farmer, _coop, usdc) = setup();

    let order_id = client.create_order(&buyer, &farmer, &200);
    assert_eq!(usdc.balance(&farmer), 0);

    client.confirm_delivery(&order_id);

    assert_eq!(usdc.balance(&farmer), 200);
    assert_eq!(usdc.balance(&buyer), 800);
}

#[test]
#[should_panic(expected = "order not in Locked status")]
fn test_cannot_confirm_already_released_order() {
    // Test 2: Edge case - confirming a non-Locked (already released) order fails.
    let (_, client, buyer, farmer, _coop, _usdc) = setup();

    let order_id = client.create_order(&buyer, &farmer, &150);
    client.confirm_delivery(&order_id);
    // Second confirmation attempt on the same order must panic.
    client.confirm_delivery(&order_id);
}

#[test]
fn test_state_reflects_released_status_after_confirmation() {
    // Test 3: Storage state verification after MVP transaction.
    let (_, client, buyer, farmer, _coop, _usdc) = setup();

    let order_id = client.create_order(&buyer, &farmer, &300);
    let order_before = client.get_order(&order_id);
    assert_eq!(order_before.status, OrderStatus::Locked);
    assert_eq!(order_before.amount, 300);

    client.confirm_delivery(&order_id);

    let order_after = client.get_order(&order_id);
    assert_eq!(order_after.status, OrderStatus::Released);
    assert_eq!(order_after.farmer, farmer);
    assert_eq!(order_after.buyer, buyer);
}

#[test]
fn test_buyer_can_cancel_locked_order_and_get_refund() {
    // Test 4: Buyer cancels before delivery confirmation; funds return to buyer.
    let (_, client, buyer, farmer, _coop, usdc) = setup();

    let order_id = client.create_order(&buyer, &farmer, &400);
    assert_eq!(usdc.balance(&buyer), 600);

    client.cancel_order(&order_id);

    assert_eq!(usdc.balance(&buyer), 1000);
    let order = client.get_order(&order_id);
    assert_eq!(order.status, OrderStatus::Cancelled);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_create_order_rejects_zero_amount() {
    // Test 5: Edge case - zero/invalid amount must be rejected at creation.
    let (_, client, buyer, farmer, _coop, _usdc) = setup();
    client.create_order(&buyer, &farmer, &0);
}
}