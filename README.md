# River Warrior (Soroban)

Environmental bounty flow: collectors submit a **Stellar address** and a **photo**; the **backend** runs **vision AI** (optional); on **VERIFIED**, the admin may invoke the **Soroban** contract to pay **USDC** from contract escrow.

## Repository layout

| Path | Role |
|------|------|
| [`contracts/river_warrior`](contracts/river_warrior) | Soroban contract: `initialize`, `disburse_reward`, `set_bounty`, getters |
| [`backend`](backend) | HTTP API: `POST /api/submit`, optional OpenAI vision, optional `stellar contract invoke` |
| [`client`](client) | Mobile-first Vite + React UI (proxies `/api` to the backend in dev) |
| [`contexts`](contexts) | PRD, verification matrix, gap roadmap, PDF templates |

## Prerequisites

- Rust stable + `rustup target add wasm32-unknown-unknown`
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools) (`stellar`) for building WASM and optional live invokes
- Node.js 20+ for the client

## Soroban contract

```bash
cd contracts/river_warrior
cargo test
stellar contract build
# WASM typically under target/wasm32-unknown-unknown/release/river_warrior.wasm
```

If your CLI still uses the `soroban` command, use `soroban contract build` / `soroban contract deploy` instead of `stellar`.

## Backend

```bash
cd backend
cargo run
# Listens on 0.0.0.0:8787 (override with PORT=...)
```

### Environment variables

| Variable | Purpose |
|----------|---------|
| `PORT` | HTTP port (default `8787`) |
| `OPENAI_API_KEY` | If set, GPT-4o-mini vision path; if unset, AI defaults to **VERIFIED** (dev only) |
| `OPENAI_VISION_MODEL` | Optional override (default `gpt-4o-mini`) |
| `MOCK_AI_STATUS` | `REJECTED` or `VERIFIED` to force AI outcome without OpenAI |
| `USE_STELLAR_CLI` | `1` / `true` to run `stellar contract invoke` after verification |
| `RIVER_WARRIOR_CONTRACT_ID` | Deployed contract id (contract strkey) |
| `STELLAR_ADMIN_SECRET` | **Secret** key that signs `disburse_reward` (never commit) |
| `STELLAR_RPC_URL` | Default `https://soroban-testnet.stellar.org` |
| `STELLAR_NETWORK_PASSPHRASE` | Default testnet passphrase |
| `STELLAR_CLI_BIN` | Default `stellar` |

**Security:** do not log `STELLAR_ADMIN_SECRET`. Use a dedicated low-balance admin on testnet.

### Idempotency

Send header `Idempotency-Key` or form field `idempotency_key`. The backend caches the first outcome per key so retries do not double-invoke.

## Client

```bash
cd client
npm install
npm run dev
```

Vite proxies `/api` and `/health` to `http://127.0.0.1:8787`. Start the backend first.

## Sample testnet invoke (after deploy + init)

Initialize (once):

```bash
stellar contract invoke --id "$RIVER_WARRIOR_CONTRACT_ID" --source "$ADMIN_SECRET" \
  --network testnet -- initialize \
  --admin "$ADMIN_ADDRESS" --token "$USDC_CONTRACT_ID" --bounty_amount 10000000
```

Disburse:

```bash
stellar contract invoke --id "$RIVER_WARRIOR_CONTRACT_ID" --source "$ADMIN_SECRET" \
  --network testnet -- disburse_reward --collector "$COLLECTOR_ADDRESS"
```

## Legacy scaffold

[`contracts/counter`](contracts/counter) remains a minimal Rust library example and is not part of the River Warrior deployment path.

## Documentation

- [Master PRD](contexts/river_warrior_prd_master.md)
- [Verification matrix](contexts/river_warrior_verification_matrix.md)
- [Gap remediation / phases](contexts/river_warrior_gap_remediation.md)
