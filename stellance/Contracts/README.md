# Stellance — Soroban Smart Contracts

Soroban (Stellar) smart contracts for Stellance escrow logic.

> **Status:** scaffolded — escrow contract in active development.  
> The current code demonstrates the on-chain event pattern; full escrow logic (fund, release, refund, dispute) is tracked in GitHub issues.

## What Goes Here

| Contract | Purpose |
|----------|---------|
| `escrow` | Locks funds when a contract is created; releases to freelancer on approval or refunds the client on dispute resolution |

The escrow contract is the core of Stellance's trustless payment flow:

```
Client wallet ──fund()──► Escrow contract ──release()──► Freelancer wallet
                                            ──refund()───► Client wallet
                                           (admin only on dispute)
```

## Structure

```
Contracts/
├── src/
│   └── lib.rs              # Contract entry point
├── tests/
│   └── contract_test.rs    # Unit tests
├── Cargo.toml              # Crate metadata and Soroban SDK dependency
└── rust-toolchain.toml     # Pinned Rust version for reproducible builds
```

## Prerequisites

- Rust (version pinned in `rust-toolchain.toml`)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Stellar CLI (optional, for local deployment): [install guide](https://developers.stellar.org/docs/tools/developer-tools/cli/install-stellar-cli)

## Build

```bash
cd stellance/Contracts
cargo build --target wasm32-unknown-unknown --release
# Output: target/wasm32-unknown-unknown/release/stellance_contract.wasm
```

## Test

```bash
cargo test
```

## Deploy to Testnet (once escrow logic is complete)

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellance_contract.wasm \
  --source <your-secret-key> \
  --network testnet
```

## Resources

- [Soroban documentation](https://developers.stellar.org/docs/build/smart-contracts/overview)
- [soroban-sdk crate](https://docs.rs/soroban-sdk/latest/soroban_sdk/)
- [Stellar testnet faucet (Friendbot)](https://laboratory.stellar.org/#account-creator?network=test)
