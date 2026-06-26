# AgroFlow

Instant, trustless payments for smallholder farmers — built on Stellar/Soroban.

## Problem
Rosa, a rice farmer in Nueva Ecija, Philippines, sells her harvest through a middleman trader who pays 30–45 days after pickup and deducts an undisclosed "handling fee," forcing her to borrow at high interest just to fund the next planting season.

## Solution
A buyer locks USDC into a Soroban escrow contract when placing an order. Once the cooperative officer confirms delivery on-chain, the contract instantly releases the USDC to the farmer's wallet — no middleman delay, no trust required between buyer and farmer, and fees low enough that even a $40 sack-of-rice payment makes sense.

## Timeline
- **Day 1:** Contract design + `lib.rs` escrow logic
- **Day 2:** Tests, local testnet deployment, CLI demo flow
- **Day 3:** Mobile-first frontend wiring + cooperative officer confirmation UX
- **Day 4:** Polish, demo rehearsal, submission

## Stellar Features Used
- **USDC transfers** — the actual value moved between buyer, contract, and farmer
- **Soroban smart contracts** — escrow logic enforcing the Locked → Released state machine
- **Trustlines** — farmers and buyers establish a USDC trustline to hold/receive funds

## Vision and Purpose
AgroFlow exists to remove the single biggest financial stressor for smallholder farmers: payment delay. By replacing a 30–45 day trust-based wait with an instant, contract-enforced payout, farmers gain predictable cash flow to reinvest in the next planting cycle without resorting to high-interest informal lending. The same escrow pattern generalizes to any cooperative-mediated agricultural supply chain across Southeast Asia.

## Prerequisites
- Rust (`rustup` recommended, stable toolchain)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Soroban CLI v21.0.0 or later: `cargo install --locked soroban-cli`

## How to Build
```bash
soroban contract build
```
This produces the optimized Wasm binary at `target/wasm32-unknown-unknown/release/agroflow.wasm`.

## How to Test
```bash
cargo test
```
Runs the 5-test suite covering the happy path, double-confirmation rejection, post-confirmation state verification, buyer cancellation/refund, and zero-amount rejection.

## How to Deploy to Testnet
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/agroflow.wasm \
  --source <YOUR_ACCOUNT> \
  --network testnet
```

## Sample CLI Invocation (MVP function with dummy arguments)
```bash
# Initialize the contract (run once after deploy)
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <YOUR_ACCOUNT> \
  --network testnet \
  -- initialize \
  --usdc_token <USDC_TOKEN_CONTRACT_ID> \
  --coop_officer <COOP_OFFICER_ADDRESS>

# Buyer creates an order, locking 200 USDC for the farmer
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <BUYER_ACCOUNT> \
  --network testnet \
  -- create_order \
  --buyer <BUYER_ADDRESS> \
  --farmer <FARMER_ADDRESS> \
  --amount 200

# Cooperative officer confirms delivery, releasing funds to the farmer
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <COOP_OFFICER_ACCOUNT> \
  --network testnet \
  -- confirm_delivery \
  --order_id 0
```

## License
MIT

## Deployed Contract

| Field | Value |
|-------|-------|
| Contract ID | `CA7X2VIJYV5DDJZ7DG44AA6RUVVKF6FPDVAFSOCGUSVCSZYPJXNNYJ7T` |
| Network | testnet |
| Explorer | [View on stellar.expert](https://stellar.expert/explorer/testnet/contract/CA7X2VIJYV5DDJZ7DG44AA6RUVVKF6FPDVAFSOCGUSVCSZYPJXNNYJ7T) |
| Deploy Tx | [View transaction](https://stellar.expert/explorer/testnet/tx/bdcae085735424ffac1c308382c11385870e4da25d1a5cd0b24fa9ea1b1b9ce8) |
| Deployed | 2026-06-26 06:49:04 UTC |
| Wallet | freighter (`GBBB…BARE`) |
