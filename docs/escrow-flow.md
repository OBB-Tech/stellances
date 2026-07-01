# Stellance — Escrow, Milestone & Dispute Flows

This document specifies how money moves through Stellance: from a client's wallet into escrow, through milestone approvals, and through dispute resolution. It covers the state machines for each entity, the on-chain steps at each transition, and the backend API calls that drive them.

## Table of Contents

1. [Entities and Their States](#entities-and-their-states)
2. [Flow 1 — Hiring and Funding Escrow](#flow-1--hiring-and-funding-escrow)
3. [Flow 2 — Milestone Payment (Happy Path)](#flow-2--milestone-payment-happy-path)
4. [Flow 3 — Dispute Resolution](#flow-3--dispute-resolution)
5. [Flow 4 — Contract Cancellation](#flow-4--contract-cancellation)
6. [On-Chain Mechanics](#on-chain-mechanics)
7. [Error Cases and Guards](#error-cases-and-guards)

---

## Entities and Their States

### Contract state machine

```
                   ┌─────────┐
                   │         │
    create ────►  │ ACTIVE  │ ◄──────────────────────────────────────┐
                   │         │                                        │
                   └────┬────┘                                        │
                        │                                             │
          ┌─────────────┼──────────────┐                             │
          │             │              │                             │
    all milestones   either party   client or
    PAID             raises dispute  freelancer
          │             │              │
          ▼             ▼              ▼
     ┌─────────┐   ┌──────────┐   ┌────────────┐
     │COMPLETED│   │ DISPUTED │   │ CANCELLED  │
     └─────────┘   └────┬─────┘   └────────────┘
                        │
               admin resolves
                        │
           ┌────────────┴────────────┐
           │                         │
    release to                  refund to
    freelancer                   client
           │                         │
           ▼                         ▼
      ┌─────────┐               ┌─────────┐
      │COMPLETED│               │CANCELLED│
      └─────────┘               └─────────┘
```

**Valid transitions:**

| From | To | Trigger | Who |
|------|----|---------|-----|
| ACTIVE | COMPLETED | All milestones reach PAID | System (automatic) |
| ACTIVE | DISPUTED | Dispute raised | Client or Freelancer |
| ACTIVE | CANCELLED | Mutual agreement before escrow funded | Client |
| DISPUTED | COMPLETED | Admin releases funds | Admin |
| DISPUTED | CANCELLED | Admin refunds client | Admin |

---

### Milestone state machine

```
              ┌─────────┐
  create ───► │ PENDING │
              └────┬────┘
                   │
         freelancer submits work
                   │
                   ▼
             ┌───────────┐
             │ IN_REVIEW │
             └─────┬─────┘
                   │
          ┌────────┴────────┐
          │                 │
   client approves    either party
          │           raises dispute
          ▼                 │
      ┌──────────┐          │ (Contract → DISPUTED,
      │ APPROVED │          │  milestone stays IN_REVIEW)
      └────┬─────┘          │
           │
    on-chain release
    tx confirmed
           │
           ▼
        ┌──────┐
        │ PAID │
        └──────┘
```

**Valid transitions:**

| From | To | Trigger | Who |
|------|----|---------|-----|
| PENDING | IN_REVIEW | Work submitted | Freelancer |
| IN_REVIEW | APPROVED | Work approved | Client |
| APPROVED | PAID | Stellar release tx confirmed | System |
| IN_REVIEW | IN_REVIEW | (stays) — dispute raised on contract | — |

---

## Flow 1 — Hiring and Funding Escrow

**Precondition:** A job exists with `status = OPEN`. Both users have Stellar public keys saved to their profiles.

```
Client browser                 Backend                      Stellar network
     │                            │                               │
     │  1. POST /api/contracts    │                               │
     │  { jobId, freelancerId,    │                               │
     │    milestones: [...] }     │                               │
     │──────────────────────────►│                               │
     │                            │ Validates client owns job     │
     │                            │ Creates Contract (ACTIVE)     │
     │                            │ Creates Milestone records     │
     │                            │ (all PENDING)                 │
     │  { contractId, xdr }      │                               │
     │◄──────────────────────────│                               │
     │                            │                               │
     │  2. User signs XDR         │                               │
     │  via Freighter wallet      │                               │
     │  (fund() invocation)       │                               │
     │                            │                               │
     │  3. Frontend submits       │                               │
     │  signed tx to Horizon      │──────────── tx ─────────────►│
     │                            │                               │ Soroban
     │                            │                               │ fund()
     │                            │                               │ executes
     │  4. POST /api/contracts    │                               │
     │  /{id}/confirm-fund        │                               │
     │  { txHash }                │                               │
     │──────────────────────────►│                               │
     │                            │ Verifies txHash on Horizon    │
     │                            │ Confirms escrow entry exists  │
     │                            │ Sets Contract.escrowTxHash    │
     │  { status: "ACTIVE" }     │                               │
     │◄──────────────────────────│                               │
```

**What gets stored:**
- `Contract` row with `status = ACTIVE`, `escrowTxHash = <stellar tx hash>`
- `Milestone` rows (one per milestone), all `status = PENDING`
- Soroban on-chain: `EscrowEntry { client, freelancer, amount, token, status: Funded }`

**What the client actually signed:**
A Soroban `invokeContractFunction` operation calling `fund(contract_id, client_address, freelancer_address, total_amount, token_address)`. The contract holds the funds — not the platform.

---

## Flow 2 — Milestone Payment (Happy Path)

**Precondition:** Contract is `ACTIVE`. At least one milestone is `PENDING`.

```
Freelancer browser             Backend                      Stellar network
     │                            │                               │
     │  1. PATCH /api/contracts   │                               │
     │  /{id}/milestones/{mid}    │                               │
     │  /submit                   │                               │
     │──────────────────────────►│                               │
     │                            │ Validates caller = freelancer │
     │                            │ Sets Milestone → IN_REVIEW    │
     │  { status: "IN_REVIEW" }  │                               │
     │◄──────────────────────────│                               │

Client browser                  Backend                      Stellar network
     │                            │                               │
     │  2. PATCH /api/contracts   │                               │
     │  /{id}/milestones/{mid}    │                               │
     │  /approve                  │                               │
     │──────────────────────────►│                               │
     │                            │ Validates caller = client     │
     │                            │ Sets Milestone → APPROVED     │
     │                            │ Builds release() invocation   │
     │                            │ Signs with platform admin key │
     │                            │ OR returns XDR for client sig │
     │                            │────── release tx ────────────►│
     │                            │                               │ Soroban
     │                            │                               │ release()
     │                            │                               │ transfers
     │                            │                               │ tokens to
     │                            │                               │ freelancer
     │                            │ Horizon confirms tx           │
     │                            │ Creates Payment record        │
     │                            │ Sets Milestone → PAID         │
     │                            │ Checks: all milestones PAID?  │
     │                            │   Yes → Contract → COMPLETED  │
     │  { status: "PAID",        │                               │
     │    stellarTxHash: "..." } │                               │
     │◄──────────────────────────│                               │
```

**What gets stored:**
- `Milestone.status = PAID`
- `Payment` row with `stellarTxHash`, `amount`, `contractId`, `milestoneId`
- If this was the last milestone: `Contract.status = COMPLETED`

**Milestone amounts are independent.** A contract can have 3 milestones; each one releases its own escrow amount when approved. The Soroban contract tracks partial releases by milestone index.

---

## Flow 3 — Dispute Resolution

**Precondition:** Contract is `ACTIVE`. At least one milestone is `IN_REVIEW` or `PENDING`.

```
Either party                   Backend                      Stellar network
     │                            │                               │
     │  1. POST /api/contracts    │                               │
     │  /{id}/dispute             │                               │
     │  { reason: "..." }         │                               │
     │──────────────────────────►│                               │
     │                            │ Validates caller = client     │
     │                            │ or freelancer on contract     │
     │                            │ Sets Contract → DISPUTED      │
     │                            │ Funds remain in Soroban       │
     │                            │ (no on-chain action yet)      │
     │  { status: "DISPUTED" }   │                               │
     │◄──────────────────────────│                               │

Admin (platform)               Backend                      Stellar network
     │                            │                               │
     │  2. PATCH /api/admin       │                               │
     │  /contracts/{id}/resolve   │                               │
     │  { decision: "release"     │                               │
     │    | "refund"              │                               │
     │    | "split",              │                               │
     │    freelancerPct: 60 }     │                               │
     │──────────────────────────►│                               │
     │                            │ Validates caller = ADMIN role │
     │                            │                               │
     │                   ┌────────┴─────────┐                    │
     │             "release"          "refund"                    │
     │                   │                  │                     │
     │             Builds release()   Builds refund()            │
     │             invocation tx      invocation tx              │
     │                   └────────┬─────────┘                    │
     │                            │──────── admin tx ────────────►│
     │                            │                               │ Soroban
     │                            │                               │ executes
     │                            │                               │ transfer
     │                            │ Records Payment(s)            │
     │                            │ Sets Contract → COMPLETED     │
     │                            │   (release) or CANCELLED      │
     │                            │   (refund)                    │
     │  { resolved: true }       │                               │
     │◄──────────────────────────│                               │
```

**Dispute state guarantees:**
- Funds cannot leave escrow while `Contract.status = DISPUTED` except through an admin `resolve` call
- The Soroban contract checks the stored status before executing any transfer
- Neither the client nor the freelancer can call `release()` or `refund()` once a dispute is open — only the admin address can

**Split resolution:**
When the admin decides on a split (e.g., 60% to freelancer, 40% refund), the contract executes two transfers in the same transaction to ensure atomicity. Either both transfers succeed or neither does.

---

## Flow 4 — Contract Cancellation

Two cancellation paths:

### Before escrow is funded (no on-chain state)

```
Client ──POST /api/contracts/{id}/cancel──► Backend
Backend: Contract.escrowTxHash is null
         Sets Contract → CANCELLED
         No Stellar tx needed
```

### After escrow is funded (on-chain refund required)

```
Client ──POST /api/contracts/{id}/cancel──► Backend
Backend: Contract.escrowTxHash is set
         Validates mutual agreement OR admin override
         Builds refund() invocation
         Signs and submits to Horizon
         On confirmation: Contract → CANCELLED, Payment (refund) recorded
```

Mutual agreement cancellation requires both parties to call the cancel endpoint (or the freelancer to acknowledge), which is enforced at the backend before any on-chain action.

---

## On-Chain Mechanics

### Token support

The escrow contract is token-agnostic. It accepts any Stellar asset that implements the SEP-41 token interface (XLM native or issued assets). In the initial release, only XLM will be supported to keep the UX simple.

### Escrow key

The platform deploys a single Soroban contract instance. Each freelance contract maps to an entry in that contract's persistent storage, keyed by the UUID of the off-chain `Contract` record. This means:

- One deployed WASM contract handles all escrows
- Storage isolation is per `contract_id` key
- The platform admin address is set at deploy time and cannot be changed without redeployment

### Transaction fees

All Soroban invocation fees are paid by the account that submits the transaction:
- `fund()` — client pays (they're submitting the funding tx)
- `release()` / `refund()` — platform pays (submitted by backend)
- `dispute()` — calling party pays

Stellar fees are typically 0.00001 XLM per operation, negligible compared to payment amounts.

---

## Error Cases and Guards

| Scenario | Guard | Response |
|----------|-------|----------|
| Client tries to fund an already-funded contract | `Contract.escrowTxHash != null` checked in backend | `409 Conflict` |
| Freelancer tries to approve their own milestone | `caller.id !== contract.freelancerId` guard | `403 Forbidden` |
| Client tries to release while disputed | Contract status check | `409 Conflict` |
| Soroban `release()` called by non-authorized address | Contract authorization check on-chain | Transaction fails, Horizon returns error |
| `fund()` called twice for same `contract_id` | Soroban storage check: entry already exists | Transaction fails with `EscrowAlreadyFunded` |
| Milestone approved but Stellar tx fails | Backend rolls back Milestone status | `502 Bad Gateway`, retryable |
| Refresh token reuse detected | `tokenVersion` mismatch triggers `logoutAll` | `403 Forbidden`, all sessions revoked |
