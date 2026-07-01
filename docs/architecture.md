# Stellance — System Architecture

## Table of Contents

1. [Overview](#overview)
2. [Component Map](#component-map)
3. [Layer-by-Layer Breakdown](#layer-by-layer-breakdown)
   - [Frontend](#frontend-nextjs)
   - [Backend API](#backend-nestjs-api)
   - [Database](#database-postgresql--prisma)
   - [Stellar Network & Horizon](#stellar-network--horizon)
   - [Soroban Smart Contracts](#soroban-smart-contracts)
4. [Data Flow: End-to-End Payment](#data-flow-end-to-end-payment)
5. [Auth Architecture](#auth-architecture)
6. [Key Design Decisions](#key-design-decisions)
7. [What Is Not Yet Built](#what-is-not-yet-built)

---

## Overview

Stellance is a three-tier application sitting on top of the Stellar blockchain. The frontend and backend are conventional web-app layers; the Stellar network provides the payment rail and the Soroban contract layer provides trustless escrow logic — funds held in the contract cannot be released unilaterally by either party.

```
┌──────────────────────────────────────────────────────────────────────┐
│  Browser                                                             │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │  Next.js 16 (App Router)                                       │  │
│  │  Tailwind CSS  ·  stellar-sdk  ·  Freighter wallet extension   │  │
│  └────────────────────────────────┬───────────────────────────────┘  │
└─────────────────────────────────── │ ─────────────────────────────────┘
                             HTTPS / REST
┌─────────────────────────────────── │ ─────────────────────────────────┐
│  Backend (NestJS 11)               │                                  │
│  ┌─────────────────────────────────▼───────────────────────────────┐  │
│  │  API Gateway (global prefix /api)                               │  │
│  │  Helmet · CORS · ValidationPipe · Swagger                       │  │
│  ├─────────────┬──────────────┬────────────────┬───────────────────┤  │
│  │  AuthModule │  UsersModule │  (Jobs module) │  (EscrowModule)   │  │
│  │  JWT + RT   │  /users/me   │  planned       │  planned          │  │
│  └──────┬──────┴──────┬───────┴────────────────┴──────────┬────────┘  │
│         │             │                                    │           │
│  ┌──────▼─────────────▼──────────────┐      ┌─────────────▼────────┐  │
│  │  Prisma 7 (ORM)                   │      │  @stellar/stellar-sdk │  │
│  │  PostgreSQL                       │      │  Horizon REST API     │  │
│  └───────────────────────────────────┘      └─────────────┬────────┘  │
└────────────────────────────────────────────────────────── │ ──────────┘
                                                            │ XDR / HTTPS
┌────────────────────────────────────────────────────────── │ ──────────┐
│  Stellar Network                                          │           │
│  ┌────────────────────────────────────────────────────────▼────────┐  │
│  │  Horizon API (testnet / mainnet)                                │  │
│  │  Transaction submission · Account queries · Event streaming     │  │
│  ├─────────────────────────────────────────────────────────────────┤  │
│  │  Soroban Runtime                                                │  │
│  │  ┌─────────────────────────────────────────────────────────┐   │  │
│  │  │  EscrowContract (WASM)                                  │   │  │
│  │  │  fund(contract_id, client, freelancer, amount, token)   │   │  │
│  │  │  release(contract_id, caller)                           │   │  │
│  │  │  refund(contract_id, caller)                            │   │  │
│  │  │  dispute(contract_id, caller)                           │   │  │
│  │  └─────────────────────────────────────────────────────────┘   │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────┘
```

---

## Component Map

| Component | Path | Technology | Status |
|-----------|------|-----------|--------|
| Frontend app | `stellance/frontend/` | Next.js 16, Tailwind, stellar-sdk | Landing page + demo ✅ |
| Backend API | `stellance/backend/` | NestJS 11, Prisma 7, PostgreSQL | Auth + users ✅ |
| Soroban contracts | `stellance/Contracts/` | Rust, soroban-sdk | Scaffold 🚧 |
| CI | `.github/workflows/ci.yml` | GitHub Actions | Lint, test, WASM build ✅ |

---

## Layer-by-Layer Breakdown

### Frontend (Next.js)

```
stellance/frontend/
├── app/
│   ├── layout.tsx          # Root layout — fonts, global CSS
│   ├── page.tsx            # Marketing landing page
│   ├── globals.css         # Tailwind base styles
│   └── demo/
│       └── page.tsx        # Interactive Stellar testnet demo
└── public/
    └── logo.png, free.jpeg # Static assets
```

**What the frontend does today:**
- Renders a marketing landing page explaining the product
- Provides a live testnet demo at `/demo`: generates a Stellar keypair, funds it via Friendbot, submits a real 1 XLM transaction on testnet using `stellar-sdk`

**What it will do:**
- Marketplace UI: job listings, job detail, apply flow
- Client dashboard: post job, create contract, fund escrow
- Freelancer dashboard: active contracts, milestone submissions
- Freighter wallet connection for signing Soroban transactions

The frontend communicates with the backend via `NEXT_PUBLIC_API_URL`. Stellar transactions that require a user signature are built by the backend and signed in the browser via Freighter (the user's private key never leaves the browser).

---

### Backend (NestJS API)

```
stellance/backend/src/
├── main.ts                 # App bootstrap
├── app.module.ts           # Root module
├── auth/                   # Auth module (complete)
│   ├── auth.controller.ts  # POST /register, /login, /refresh, /logout, /logout-all
│   ├── auth.service.ts     # Token issuance, rotation, revocation
│   ├── strategies/         # Passport local + JWT strategies
│   ├── guards/             # JwtAuthGuard (global default), LocalAuthGuard
│   └── dto/                # RegisterDto, LoginDto (class-validator)
├── users/                  # Users module (partial)
│   ├── users.controller.ts # GET /users/me
│   └── users.service.ts    # findOneByEmail, findOneById, create
├── prisma/                 # Database wrapper
│   └── prisma.service.ts   # PrismaClient singleton with lifecycle hooks
└── test-utils/
    └── prisma.mock.ts      # In-memory Prisma mock for unit tests
```

All routes are prefixed `/api`. The `JwtAuthGuard` is applied globally; routes that need no auth are decorated with `@Public()`.

**Middleware stack applied at bootstrap (`main.ts`):**

1. `helmet()` — sets secure HTTP headers
2. `cookieParser()` — parses cookies for refresh token transport
3. `ValidationPipe({ whitelist: true, forbidNonWhitelisted: true })` — strips and rejects unknown fields on all DTOs
4. CORS restricted to `FRONTEND_URL`
5. Swagger at `/docs` (auto-generated from decorators)

---

### Database (PostgreSQL + Prisma)

Schema lives at `stellance/backend/prisma/schema.prisma`. Six models:

```
User ──────────┬── jobs (as client)
               ├── contractsAsClient
               ├── contractsAsFreelancer
               └── refreshTokens

Job ────────────── contract (one-to-one)

Contract ──────┬── milestones
               └── payments

Milestone ─────── payment (one-to-one, optional)

Payment ────────── (belongs to Contract, optionally to Milestone)

RefreshToken ───── user (belongs to)
```

Key fields that link to Stellar:
- `User.stellarPublicKey` — the user's Stellar account address (unique, indexed)
- `Contract.escrowTxHash` — hash of the Stellar transaction that funded the escrow
- `Payment.stellarTxHash` — hash of the Stellar transaction that released funds

These hashes allow any payment to be independently verified on Stellar's blockchain explorer ([testnet](https://stellar.expert/explorer/testnet) · [mainnet](https://stellar.expert/explorer/public)).

---

### Stellar Network & Horizon

Stellance uses the Stellar network for:

1. **Payment settlement** — XLM or Stellar assets sent peer-to-peer in 3–5 seconds
2. **Escrow custody** — funds held in a Soroban contract account, not by the platform
3. **Transaction verification** — every payment has a public, immutable on-chain record

The backend talks to Stellar via the `@stellar/stellar-sdk` package using Horizon, Stellar's public REST API:
- **Testnet**: `https://horizon-testnet.stellar.org`
- **Mainnet**: `https://horizon.stellar.org`

The frontend demo uses `stellar-sdk` directly in the browser for the keypair + Friendbot + payment flow. Production flows will use Freighter for user-side signing.

---

### Soroban Smart Contracts

> **Current status:** The contract in `stellance/Contracts/src/lib.rs` is a development scaffold demonstrating the on-chain event pattern. The full escrow contract is in active development.

**Why Soroban for escrow?**

A naive approach would have the Stellance platform hold client funds in a company-controlled Stellar account. This is custodial — the platform can theoretically take the funds. Soroban contracts eliminate this: the escrow logic is code on the blockchain. Neither the client, the freelancer, nor Stellance can move funds except through the contract's defined paths.

**Planned contract interface:**

```rust
/// Lock client funds into escrow for a specific contract.
/// Only callable once per contract_id.
pub fn fund(
    env: Env,
    contract_id: Symbol,   // Stellance DB contract UUID
    client: Address,       // Must authorize this call
    freelancer: Address,   // Recipient on release
    amount: i128,          // In stroops (1 XLM = 10_000_000 stroops)
    token: Address,        // XLM native or any Stellar asset contract
) -> Result<(), EscrowError>

/// Release escrowed funds to the freelancer.
/// Callable by the client, or by the platform admin (dispute resolution).
pub fn release(
    env: Env,
    contract_id: Symbol,
    caller: Address,       // Must be client or admin
) -> Result<(), EscrowError>

/// Refund escrowed funds to the client.
/// Callable by the freelancer, or by the platform admin (dispute resolution).
pub fn refund(
    env: Env,
    contract_id: Symbol,
    caller: Address,       // Must be freelancer or admin
) -> Result<(), EscrowError>

/// Mark an escrow as disputed. Freezes funds pending admin resolution.
/// Callable by either party.
pub fn dispute(
    env: Env,
    contract_id: Symbol,
    caller: Address,       // Must be client or freelancer
) -> Result<(), EscrowError>
```

**Contract storage model:**

Each escrow is stored as a persistent entry keyed by `contract_id`:

```rust
struct EscrowEntry {
    client: Address,
    freelancer: Address,
    amount: i128,
    token: Address,
    status: EscrowStatus,  // Funded | Released | Refunded | Disputed
}
```

**Contract deployment:**

The compiled WASM is deployed to the Stellar testnet. The backend stores the deployed contract address in an environment variable (`ESCROW_CONTRACT_ID`). When the backend needs to fund or release escrow, it builds a Soroban invocation transaction and either signs it server-side (for platform-authorised operations) or returns an unsigned XDR envelope for the client to sign via Freighter.

---

## Data Flow: End-to-End Payment

This traces a milestone payment from client wallet to freelancer wallet:

```
1. CLIENT funds escrow
   Frontend ──POST /api/escrow/fund──► Backend
   Backend builds Soroban invocation tx (fund())
   Backend returns unsigned XDR to frontend
   Frontend sends XDR to Freighter for client signature
   Freighter signs ──► Frontend submits tx to Horizon
   Horizon executes Soroban fund() ──► escrow entry created on-chain
   Frontend ──POST /api/contracts/{id}/confirm-fund──► Backend
   Backend verifies tx hash on Horizon ──► sets Contract.escrowTxHash
   Backend sets Contract.status = ACTIVE

2. FREELANCER submits milestone
   Frontend ──PATCH /api/contracts/{id}/milestones/{mid}/submit──► Backend
   Backend sets Milestone.status = IN_REVIEW

3. CLIENT approves milestone
   Frontend ──PATCH /api/contracts/{id}/milestones/{mid}/approve──► Backend
   Backend builds Soroban invocation tx (release())
   Backend signs with platform admin key (or returns XDR for client)
   Soroban release() ──► token transferred to freelancer on-chain
   Backend records stellar tx hash ──► creates Payment record
   Backend sets Milestone.status = PAID
   If all milestones PAID: Contract.status = COMPLETED
```

---

## Auth Architecture

Stellance uses a dual-token auth pattern:

```
Login/Register
     │
     ▼
Access Token (JWT, 15 min)          Refresh Token (opaque, 30 days)
  • Short-lived                       • Stored as SHA-256+pepper hash in DB
  • Returned in response body         • Sent via httpOnly cookie
  • Used in Authorization header        (path=/api/auth, sameSite=strict)
  • Never stored server-side          • Rotated on every use
                                      • Reuse of revoked token → logoutAll
```

Token rotation: every call to `POST /api/auth/refresh` creates a new refresh token and revokes the old one. If a revoked token is reused (stolen token replay), the service detects the chain break and revokes all sessions for that user by incrementing `User.tokenVersion`.

---

## Key Design Decisions

**Why Stellar over Ethereum/Solana?**
Stellar is purpose-built for payments. Transaction fees are ~$0.0001, settlement is 3–5 seconds, and Soroban provides smart contract capability without the complexity of EVM. For a B2C freelance payments product, Stellar's UX and cost profile are much better fits than general-purpose chains.

**Why Soroban instead of multisig escrow?**
Multisig (requiring N-of-M signers) is simpler to implement but requires the platform to hold a signer key, keeping custody. A Soroban contract makes the rules code, not trust — the platform cannot unilaterally move funds.

**Why Prisma with generated types in `src/generated/`?**
Prisma 7 with `output = "../src/generated/prisma"` keeps generated types co-located with application code, visible in the IDE, and committed to the repo. This makes types available without a separate build step in CI.

**Why httpOnly cookies for refresh tokens?**
Refresh tokens stored in `localStorage` are accessible to JavaScript and vulnerable to XSS. httpOnly cookies are not. The access token (short-lived, 15m) is returned in the response body for use in the `Authorization` header; if compromised, it expires quickly.

---

## What Is Not Yet Built

| Feature | Needed for | Tracked in |
|---------|-----------|-----------|
| Jobs API (CRUD, apply) | Core marketplace | GitHub Issues |
| Contracts API (create, approve, dispute) | Core payment flow | GitHub Issues |
| Milestone API | Milestone payments | GitHub Issues |
| Escrow service (backend) | Stellar tx building | GitHub Issues |
| Full Soroban escrow contract | Trustless custody | GitHub Issues |
| Freighter wallet integration (frontend) | User signing | GitHub Issues |
| Marketplace UI pages | User-facing product | GitHub Issues |
| docker-compose | Local dev ergonomics | GitHub Issues |

See the [CHANGELOG](../CHANGELOG.md) for what has been shipped.
