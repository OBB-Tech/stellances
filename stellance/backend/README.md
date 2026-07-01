# Stellance — Backend API

NestJS REST API for Stellance. Handles authentication, user management, jobs, contracts, milestones, and Stellar payment records.

## Stack

| Layer | Technology |
|-------|-----------|
| Framework | NestJS 11 |
| Database | PostgreSQL via Prisma 7 |
| Auth | JWT (access) + rotating refresh tokens (argon2 + httpOnly cookies) |
| Validation | class-validator / class-transformer |
| API docs | Swagger (`/docs`) |

## Quick Start

### Prerequisites

- Node.js ≥ 20
- PostgreSQL running locally (or use Docker — see root `docker-compose.yml`)

### Setup

```bash
cd stellance/backend
cp .env.example .env
# Edit .env — set DATABASE_URL and JWT_SECRET at minimum

npm install
npx prisma migrate dev
npm run start:dev
```

API is available at `http://localhost:3001/api`.  
Swagger docs at `http://localhost:3001/docs`.

## Environment Variables

See `.env.example` for the full list. Minimum required:

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string |
| `JWT_SECRET` | Secret for signing JWTs |
| `JWT_ACCESS_EXPIRES_IN` | Access token TTL (e.g. `15m`) |
| `JWT_REFRESH_DAYS` | Refresh token TTL in days (default `30`) |
| `REFRESH_TOKEN_PEPPER` | Extra secret mixed into refresh token hashing |
| `FRONTEND_URL` | CORS allowed origin (e.g. `http://localhost:3000`) |

## Auth Endpoints

All routes are prefixed `/api/auth`.

| Method | Path | Description |
|--------|------|-------------|
| POST | `/register` | Create account, returns access token + sets refresh cookie |
| POST | `/login` | Login, returns access token + sets refresh cookie |
| POST | `/refresh` | Rotate refresh token, returns new access token |
| POST | `/logout` | Revoke current refresh token, clear cookies |
| POST | `/logout-all` | Revoke all sessions (increments `tokenVersion`) |

Access tokens are short-lived JWTs. Refresh tokens are hashed with argon2 + pepper before DB storage and rotate on every use. Reuse of a revoked token triggers a full session revoke for that user.

## Scripts

```bash
npm run start:dev    # Watch mode
npm run build        # Compile TypeScript
npm run test         # Unit tests (Jest)
npm run test:cov     # Unit tests with coverage report
npm run test:e2e     # End-to-end tests
npm run lint         # ESLint
npm run format       # Prettier
```

## Project Structure

```
src/
├── app.module.ts          # Root module
├── main.ts                # Bootstrap (helmet, CORS, validation, Swagger)
├── auth/                  # Auth module — JWT, refresh rotation, guards
│   ├── strategies/        # Passport strategies (local, JWT)
│   ├── guards/            # JwtAuthGuard, LocalAuthGuard
│   ├── dto/               # RegisterDto, LoginDto
│   └── decorators/        # @Public() decorator
├── users/                 # Users CRUD
├── prisma/                # PrismaService (database client wrapper)
├── generated/prisma/      # Auto-generated Prisma types (do not edit)
└── test-utils/            # Shared test mocks (PrismaServiceMock)
```

## Database Schema

Key models: `User`, `Job`, `Contract`, `Milestone`, `Payment`, `RefreshToken`.

Each `Contract` stores an `escrowTxHash` linking it to a Stellar transaction. Each `Payment` stores a `stellarTxHash` for on-chain verification. See `prisma/schema.prisma` for the full schema.

## Running Migrations

```bash
npx prisma migrate dev       # Apply pending migrations (development)
npx prisma migrate deploy    # Apply in production (non-interactive)
npx prisma studio            # Visual DB browser
```
