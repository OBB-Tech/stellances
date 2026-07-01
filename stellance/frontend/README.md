# Stellance — Frontend

Next.js 16 frontend for Stellance, the Stellar-powered freelance payment marketplace.

## Stack

| Layer | Technology |
|-------|-----------|
| Framework | Next.js 16 (App Router) |
| Styling | Tailwind CSS 4 |
| Stellar | stellar-sdk 10 (testnet demo) |
| Language | TypeScript |

## Pages

| Route | Description |
|-------|-------------|
| `/` | Marketing landing page |
| `/demo` | Interactive Stellar testnet demo — create a keypair, fund via Friendbot, send XLM |

## Quick Start

```bash
cd stellance/frontend
npm install

# Create environment file
cp /dev/null .env.local
# Add these two lines:
# NEXT_PUBLIC_API_URL=http://localhost:3001/api
# NEXT_PUBLIC_STELLAR_NETWORK=testnet

npm run dev
```

Open `http://localhost:3000`.

The backend must be running at `http://localhost:3001` for API features.  
See [docs/local-development.md](../../docs/local-development.md) for the full setup guide.

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `NEXT_PUBLIC_API_URL` | Backend API base URL | `http://localhost:3001/api` |
| `NEXT_PUBLIC_STELLAR_NETWORK` | `testnet` or `mainnet` | `testnet` |

## Scripts

```bash
npm run dev      # Development server with hot reload
npm run build    # Production build
npm run start    # Start production server
npm run lint     # ESLint
```

## Project Structure

```
app/
├── layout.tsx          # Root layout — fonts, global styles
├── page.tsx            # Marketing landing page
├── globals.css         # Tailwind base styles
└── demo/
    └── page.tsx        # Stellar testnet demo
public/
├── logo.png
└── free.jpeg
```

## Stellar Demo

The `/demo` page demonstrates the core Stellar integration without requiring a wallet extension:

1. Generates a fresh Stellar keypair client-side using `stellar-sdk`
2. Funds the account via [Friendbot](https://laboratory.stellar.org/#account-creator?network=test) (testnet faucet)
3. Constructs, signs, and submits a 1 XLM payment transaction to the Stellar testnet
4. Returns a transaction hash verifiable on [stellar.expert](https://stellar.expert/explorer/testnet)

This is for demonstration only — no real funds are involved.

## What's Coming

- Freighter wallet connection
- Job marketplace UI (browse, filter, post)
- Client dashboard (post jobs, fund escrow, approve milestones)
- Freelancer dashboard (active contracts, submit milestones)
- On-chain payment status tracking
