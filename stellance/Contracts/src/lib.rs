//! # Stellance Escrow Contract
//!
//! Soroban smart contract for trustless freelance payment escrow on Stellar.
//!
//! ## Purpose
//!
//! This contract holds client funds on-chain while a freelance contract is
//! active. Funds can only leave the contract through one of three paths:
//!
//! - **`release()`** — transfers funds to the freelancer (called when the
//!   client approves delivered work)
//! - **`refund()`** — returns funds to the client (called on mutual
//!   cancellation, or by the platform admin after dispute resolution)
//! - **`dispute()`** — freezes funds pending admin review
//!
//! Neither party can move funds unilaterally. The contract is the only
//! custodian.
//!
//! ## Current Status
//!
//! This file contains a development scaffold that demonstrates the Soroban
//! on-chain event publishing pattern. The full escrow interface (`fund`,
//! `release`, `refund`, `dispute`) is under active development.
//!
//! See [`docs/escrow-flow.md`](../../docs/escrow-flow.md) for the complete
//! specification of the escrow state machine and all transitions.
//! See [`docs/architecture.md`](../../docs/architecture.md) for how this
//! contract fits into the overall system.
//!
//! ## Deployment
//!
//! The contract is compiled to WASM targeting `wasm32-unknown-unknown` and
//! deployed to the Stellar testnet via the Stellar CLI. One contract instance
//! handles all Stellance escrows; each freelance contract maps to a separate
//! storage entry keyed by its off-chain UUID.
//!
//! Build:
//! ```bash
//! cargo build --target wasm32-unknown-unknown --release
//! ```
//!
//! Deploy (once full escrow logic is implemented):
//! ```bash
//! stellar contract deploy \
//!   --wasm target/wasm32-unknown-unknown/release/stellance_contract.wasm \
//!   --source <admin-secret-key> \
//!   --network testnet
//! ```

#![no_std]

use soroban_sdk::{contractimpl, symbol, Env, Symbol};

/// Stellance escrow contract.
///
/// In the full implementation this will hold persistent [`EscrowEntry`]
/// storage for each active freelance contract, keyed by the off-chain
/// contract UUID.
pub struct StellanceContract;

#[contractimpl]
impl StellanceContract {
    /// Emit a ping event. Used in CI to verify the contract builds and
    /// on-chain event publishing works end-to-end.
    ///
    /// In the full escrow contract this will be replaced by `fund()`,
    /// `release()`, `refund()`, and `dispute()`.
    pub fn ping(env: Env) {
        env.events().publish((symbol!("ping"),), ());
    }

    /// Return a greeting symbol. Placeholder for testing contract invocation
    /// from the backend and the Stellar testnet demo.
    pub fn get_greeting(_env: Env) -> Symbol {
        symbol!("Hello from Stellance!")
    }
}
