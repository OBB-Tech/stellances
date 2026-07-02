# Stellance — API Reference

Base URL: `http://localhost:3001/api` (development)  
Interactive docs (Swagger): `http://localhost:3001/docs`

## Table of Contents

- [Authentication](#authentication)
- [Auth Endpoints](#auth-endpoints)
  - [POST /auth/register](#post-authregister)
  - [POST /auth/login](#post-authlogin)
  - [POST /auth/refresh](#post-authrefresh)
  - [POST /auth/logout](#post-authlogout)
  - [POST /auth/logout-all](#post-authlogout-all)
- [User Endpoints](#user-endpoints)
  - [GET /users/me](#get-usersme)
  - [PATCH /users/me](#patch-usersme-planned)
- [Jobs Endpoints](#jobs-endpoints-planned)
- [Contracts Endpoints](#contracts-endpoints-planned)
- [Milestones Endpoints](#milestones-endpoints-planned)
- [Payments Endpoints](#payments-endpoints-planned)
- [Error Format](#error-format)
- [Status Codes](#status-codes)

---

## Authentication

Stellance uses a dual-token scheme:

| Token | Transport | TTL | Notes |
|-------|-----------|-----|-------|
| Access token (JWT) | `Authorization: Bearer <token>` header | 15 minutes | Returned in response body |
| Refresh token (opaque) | `refresh_token` httpOnly cookie | 30 days (configurable) | Rotated on every use |

**Protecting requests:** include the access token in the `Authorization` header:

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

When the access token expires, call `POST /auth/refresh` — the refresh token is sent automatically via cookie.

---

## Auth Endpoints

### POST /auth/register

Create a new user account. Returns an access token and sets the `refresh_token` cookie.

**Auth required:** No

**Request body:**

```json
{
  "email": "alice@example.com",
  "name": "Alice Smith",
  "password": "minimum8chars",
  "role": "CLIENT"
}
```

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `email` | string | ✅ | Must be a valid email address |
| `name` | string | ✅ | Display name |
| `password` | string | ✅ | Minimum 8 characters |
| `role` | `"CLIENT"` \| `"FREELANCER"` | ✅ | Determines permissions throughout the app |

**Response `201 Created`:**

```json
{
  "message": "Registered successfully",
  "access_token": "eyJhbGci...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "alice@example.com",
    "name": "Alice Smith",
    "role": "CLIENT",
    "stellarPublicKey": null,
    "tokenVersion": 0,
    "createdAt": "2026-07-01T12:00:00.000Z",
    "updatedAt": "2026-07-01T12:00:00.000Z"
  }
}
```

Sets cookie: `refresh_token` (httpOnly, sameSite=strict, path=/api/auth)

**Errors:**

| Status | Code | Meaning |
|--------|------|---------|
| 400 | `BAD_REQUEST` | Validation failed (missing field, invalid email, short password) |
| 409 | `CONFLICT` | Email already registered |

---

### POST /auth/login

Authenticate with email and password. Returns an access token and sets the `refresh_token` cookie.

**Auth required:** No

**Request body:**

```json
{
  "email": "alice@example.com",
  "password": "minimum8chars"
}
```

**Response `200 OK`:**

```json
{
  "message": "Logged in successfully",
  "access_token": "eyJhbGci...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "alice@example.com",
    "name": "Alice Smith",
    "role": "CLIENT",
    "stellarPublicKey": null
  }
}
```

Sets cookie: `refresh_token`

**Errors:**

| Status | Meaning |
|--------|---------|
| 400 | Missing email or password |
| 401 | Wrong credentials |

---

### POST /auth/refresh

Exchange a valid refresh token for a new access token. The old refresh token is revoked and a new one is set via cookie (rotation).

**Auth required:** No (uses `refresh_token` cookie)

**Request body:** none

**Response `200 OK`:**

```json
{
  "access_token": "eyJhbGci..."
}
```

Sets cookie: new `refresh_token` (old one is revoked)

**Errors:**

| Status | Meaning |
|--------|---------|
| 401 | Missing, expired, or invalid refresh token |
| 403 | Refresh token reuse detected — all sessions revoked (possible theft) |

---

### POST /auth/logout

Revoke the current refresh token and clear both cookies.

**Auth required:** No (uses `refresh_token` cookie)

**Request body:** none

**Response `200 OK`:**

```json
{
  "message": "Logged out successfully"
}
```

Clears cookies: `access_token`, `refresh_token`

---

### POST /auth/logout-all

Revoke all active sessions for the authenticated user. Increments the user's `tokenVersion`, which invalidates all existing refresh tokens and access tokens simultaneously.

**Auth required:** Yes (access token)

**Request body:** none

**Response `200 OK`:**

```json
{
  "message": "Logged out everywhere"
}
```

Use this endpoint when a user suspects their credentials have been compromised.

---

## User Endpoints

### GET /users/me

Return the authenticated user's profile.

**Auth required:** Yes

**Response `200 OK`:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "alice@example.com",
  "name": "Alice Smith",
  "role": "CLIENT",
  "stellarPublicKey": "GDQOE23CFSUMSVQK4Y5JHPPYK73VYCNHZHA7ENKCV37P6SUEO6XQBKPP",
  "tokenVersion": 0,
  "createdAt": "2026-07-01T12:00:00.000Z",
  "updatedAt": "2026-07-01T12:00:00.000Z"
}
```

The `password` field is never returned. `stellarPublicKey` is `null` until the user connects a wallet.

**Errors:**

| Status | Meaning |
|--------|---------|
| 401 | Missing or expired access token |

---

### PATCH /users/me (Planned)

> **Status:** not yet implemented. This endpoint is needed to let users save their Stellar wallet address after connecting Freighter.

Update the authenticated user's profile. All fields are optional; only the provided fields are updated.

**Auth required:** Yes

**Request body:**

```json
{
  "name": "Alice Smith",
  "stellarPublicKey": "GDQOE23CFSUMSVQK4Y5JHPPYK73VYCNHZHA7ENKCV37P6SUEO6XQBKPP"
}
```

| Field | Type | Notes |
|-------|------|-------|
| `name` | string | Display name |
| `stellarPublicKey` | string | Must be a valid Stellar public key (starts with `G`, 56 chars). Each key can only be linked to one account. |

**Response `200 OK`:** Updated user object (same shape as `GET /users/me`).

**Errors:**

| Status | Meaning |
|--------|---------|
| 400 | Invalid Stellar public key format |
| 401 | Missing or expired access token |
| 409 | Stellar public key already linked to another account |

---

## Jobs Endpoints (Planned)

> These endpoints are specified here as the intended contract. Implementation is in progress.

### GET /jobs

List open jobs. Supports filtering and pagination.

**Auth required:** No (public)

**Query params:**

| Param | Type | Default | Notes |
|-------|------|---------|-------|
| `status` | `OPEN\|IN_PROGRESS\|COMPLETED\|CANCELLED` | `OPEN` | Filter by status |
| `category` | string | — | Filter by category |
| `page` | number | `1` | Page number |
| `limit` | number | `20` | Results per page (max 100) |

**Response `200 OK`:**

```json
{
  "data": [
    {
      "id": "...",
      "title": "Build a Stellar payment integration",
      "description": "...",
      "budget": "500.0000000",
      "category": "Blockchain",
      "status": "OPEN",
      "clientId": "...",
      "createdAt": "2026-07-01T12:00:00.000Z"
    }
  ],
  "meta": {
    "total": 42,
    "page": 1,
    "limit": 20,
    "totalPages": 3
  }
}
```

---

### POST /jobs

Post a new job.

**Auth required:** Yes — `role = CLIENT`

**Request body:**

```json
{
  "title": "Build a Stellar payment integration",
  "description": "We need a NestJS service that submits XLM payments via Horizon.",
  "budget": "500.00",
  "category": "Blockchain"
}
```

**Response `201 Created`:** The created `Job` object.

---

### GET /jobs/:id

Get a single job by ID.

**Auth required:** No

**Response `200 OK`:** The `Job` object, including the associated `Contract` if one exists.

---

## Contracts Endpoints (Planned)

### POST /contracts

Create a contract (client hires a freelancer for a job). Also returns the unsigned Soroban `fund()` XDR for the client to sign via Freighter.

**Auth required:** Yes — `role = CLIENT`, must own the job

**Request body:**

```json
{
  "jobId": "...",
  "freelancerId": "...",
  "milestones": [
    { "title": "Initial design", "amount": "150.00" },
    { "title": "Implementation", "amount": "300.00" },
    { "title": "Testing and handoff", "amount": "50.00" }
  ]
}
```

**Response `201 Created`:**

```json
{
  "contract": {
    "id": "...",
    "jobId": "...",
    "freelancerId": "...",
    "clientId": "...",
    "status": "ACTIVE",
    "escrowTxHash": null,
    "milestones": [...]
  },
  "fundingXdr": "AAAAAgAAAA..."
}
```

The `fundingXdr` is an unsigned Soroban invocation transaction. The frontend passes this to Freighter, the user signs it, and the frontend submits it to Horizon, then calls `POST /contracts/:id/confirm-fund` with the resulting hash.

---

### POST /contracts/:id/confirm-fund

Confirm that the escrow funding transaction was submitted on-chain.

**Auth required:** Yes — must be the client on this contract

**Request body:**

```json
{
  "txHash": "a1b2c3d4e5f6..."
}
```

**Response `200 OK`:**

```json
{
  "id": "...",
  "status": "ACTIVE",
  "escrowTxHash": "a1b2c3d4e5f6..."
}
```

---

### GET /contracts

List the authenticated user's contracts (as client or freelancer).

**Auth required:** Yes

**Response `200 OK`:** Array of `Contract` objects with nested `milestones` and `payments`.

---

### GET /contracts/:id

Get full contract detail.

**Auth required:** Yes — must be client or freelancer on this contract

**Response `200 OK`:** `Contract` with nested `milestones`, `payments`, and user summaries.

---

### POST /contracts/:id/dispute

Raise a dispute on an active contract. Funds remain in escrow pending admin resolution.

**Auth required:** Yes — must be client or freelancer on this contract

**Request body:**

```json
{
  "reason": "Deliverable does not match the agreed specification."
}
```

**Response `200 OK`:**

```json
{
  "id": "...",
  "status": "DISPUTED"
}
```

---

## Milestones Endpoints (Planned)

### PATCH /contracts/:id/milestones/:mid/submit

Freelancer submits a milestone for review.

**Auth required:** Yes — `role = FREELANCER`, must be the freelancer on this contract

**Response `200 OK`:**

```json
{
  "id": "...",
  "status": "IN_REVIEW"
}
```

---

### PATCH /contracts/:id/milestones/:mid/approve

Client approves a milestone. Triggers a Soroban `release()` call and creates a `Payment` record.

**Auth required:** Yes — `role = CLIENT`, must be the client on this contract

**Response `200 OK`:**

```json
{
  "milestone": {
    "id": "...",
    "status": "PAID"
  },
  "payment": {
    "id": "...",
    "amount": "150.0000000",
    "stellarTxHash": "a1b2c3d4...",
    "createdAt": "2026-07-01T14:30:00.000Z"
  }
}
```

---

## Payments Endpoints (Planned)

### GET /payments

List all payments for the authenticated user's contracts.

**Auth required:** Yes

**Response `200 OK`:** Array of `Payment` objects with `stellarTxHash` for on-chain verification.

---

## Error Format

All errors use NestJS's standard exception format:

```json
{
  "statusCode": 400,
  "message": ["email must be an email", "password must be longer than or equal to 8 characters"],
  "error": "Bad Request"
}
```

Validation errors return `message` as an array of strings. Other errors return a single string:

```json
{
  "statusCode": 401,
  "message": "Invalid credentials",
  "error": "Unauthorized"
}
```

---

## Status Codes

| Code | Meaning |
|------|---------|
| 200 | OK |
| 201 | Created |
| 400 | Bad Request — validation error or malformed body |
| 401 | Unauthorized — missing or invalid access token |
| 403 | Forbidden — authenticated but not authorized for this action |
| 404 | Not Found — resource does not exist |
| 409 | Conflict — e.g. duplicate email, already-funded contract |
| 500 | Internal Server Error |
| 502 | Bad Gateway — Stellar/Horizon call failed |
