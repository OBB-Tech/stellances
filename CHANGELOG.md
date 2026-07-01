# Changelog

All notable changes to Stellance are documented here.  
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [Unreleased]

### Planned
- Soroban escrow contract: `fund`, `release`, `refund`, `dispute` functions
- Jobs API endpoints (list, create, apply)
- Contracts API endpoints (create, approve, dispute)
- Freighter wallet connection in frontend
- Frontend marketplace pages (jobs list, job detail, dashboard)
- docker-compose for local development

---

## [0.2.0] — 2026-06-17

### Added
- Marketing landing page with responsive layout and Stellar branding
- Soroban contract workspace scaffold (`stellance/Contracts/`)
- GitHub issue templates for frontend and backend contributor applications
- CI workflow for backend tests and frontend build

### Changed
- Updated CI to Node.js 20 for Next.js and Prisma compatibility
- Added Stellance logo to README

---

## [0.1.0] — 2026-03

### Added
- NestJS backend bootstrap with app module configuration
- Prisma 7 schema: `User`, `Job`, `Contract`, `Milestone`, `Payment`, `RefreshToken`
- Initial database migration
- JWT auth with rotating refresh tokens (argon2 password hashing, httpOnly cookies)
- Auth endpoints: register, login, refresh, logout, logout-all
- Token reuse detection (triggers full session revoke via `tokenVersion`)
- Helmet, CORS, and global validation pipe in `main.ts`
- Swagger API docs at `/docs`
- Next.js 16 frontend scaffold with Tailwind CSS
- Stellar testnet demo page: keypair generation, Friendbot funding, XLM payment
- `CONTRIBUTING.md` with architecture diagrams, data models, and user flows

---

[Unreleased]: https://github.com/alone-in/stellances/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/alone-in/stellances/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/alone-in/stellances/releases/tag/v0.1.0
