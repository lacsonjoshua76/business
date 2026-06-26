#![no_std]

// AgroFlow Escrow Contract
// Buyers lock USDC for a farmer order. A trusted cooperative officer
// confirms delivery, which instantly releases funds to the farmer.
// This removes the 30-45 day middleman payment delay smallholder
// farmers currently face.

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

#[path = "test.rs"]
mod test;

// ---- Storage Keys ----
// Orders are stored by a u64 order_id.
// A separate counter key tracks the next available order_id.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Order(u64),
    NextOrderId,
    CoopOfficer, // address authorized to confirm deliveries
    UsdcToken,   // address of the USDC token contract on this network
}

#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum OrderStatus {
    Locked,
    Released,
    Cancelled,
}

#[contracttype]
#[derive(Clone)]
pub struct Order {
    pub buyer: Address,
    pub farmer: Address,
    pub amount: i128,
    pub status: OrderStatus,
}

#[contract]
pub struct AgroFlowContract;

#[contractimpl]
impl AgroFlowContract {
    /// Initialize the contract once at deploy time.
    /// Sets the USDC token contract address and the cooperative
    /// officer address authorized to confirm deliveries.
    pub fn initialize(env: Env, usdc_token: Address, coop_officer: Address) {
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
        env.storage().instance().set(&DataKey::CoopOfficer, &coop_officer);
        env.storage().instance().set(&DataKey::NextOrderId, &0u64);
    }

    /// Buyer locks USDC into escrow for a specific farmer.
    /// Requires the buyer's signature/auth. Transfers USDC from the
    /// buyer's wallet into this contract's balance, and records an
    /// Order in Locked status.
    /// Returns the new order_id so the buyer/farmer can track it.
    pub fn create_order(env: Env, buyer: Address, farmer: Address, amount: i128) -> u64 {
        buyer.require_auth();
        assert!(amount > 0, "amount must be positive");

        let usdc_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .expect("contract not initialized");

        // Move USDC from buyer to this contract (the escrow vault)
        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&buyer, &env.current_contract_address(), &amount);

        let order_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextOrderId)
            .unwrap_or(0);

        let order = Order {
            buyer,
            farmer,
            amount,
            status: OrderStatus::Locked,
        };

        env.storage().instance().set(&DataKey::Order(order_id), &order);
        env.storage().instance().set(&DataKey::NextOrderId, &(order_id + 1));

        env.events().publish((Symbol::new(&env, "order_created"),), order_id);

        order_id
    }

    /// Cooperative officer confirms physical delivery occurred.
    /// Requires the registered coop officer's signature/auth.
    /// Releases the locked USDC directly to the farmer's wallet
    /// and marks the order Released. This is the step that replaces
    /// the 30-45 day middleman wait with an instant payout.
    pub fn confirm_delivery(env: Env, order_id: u64) {
        let coop_officer: Address = env
            .storage()
            .instance()
            .get(&DataKey::CoopOfficer)
            .expect("contract not initialized");
        coop_officer.require_auth();

        let mut order: Order = env
            .storage()
            .instance()
            .get(&DataKey::Order(order_id))
            .expect("order not found");

        assert!(order.status == OrderStatus::Locked, "order not in Locked status");

        let usdc_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .expect("contract not initialized");

        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&env.current_contract_address(), &order.farmer, &order.amount);

        order.status = OrderStatus::Released;
        env.storage().instance().set(&DataKey::Order(order_id), &order);

        env.events().publish((Symbol::new(&env, "order_released"),), order_id);
    }

    /// Buyer can cancel an order only while it's still Locked
    /// (e.g. farmer never delivered). Refunds the buyer in full.
    pub fn cancel_order(env: Env, order_id: u64) {
        let mut order: Order = env
            .storage()
            .instance()
            .get(&DataKey::Order(order_id))
            .expect("order not found");

        order.buyer.require_auth();
        assert!(order.status == OrderStatus::Locked, "order not in Locked status");

        let usdc_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .expect("contract not initialized");

        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&env.current_contract_address(), &order.buyer, &order.amount);

        order.status = OrderStatus::Cancelled;
        env.storage().instance().set(&DataKey::Order(order_id), &order);

        env.events().publish((Symbol::new(&env, "order_cancelled"),), order_id);
    }

    /// Read-only view of an order's current state. Used by the
    /// frontend / CLI to display status to farmers and buyers.
    pub fn get_order(env: Env, order_id: u64) -> Order {
        env.storage()
            .instance()
            .get(&DataKey::Order(order_id))
            .expect("order not found")
    }
}
