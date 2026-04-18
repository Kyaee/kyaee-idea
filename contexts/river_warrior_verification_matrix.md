# River Warrior — Full Verification Matrix

**Purpose:** Define how each capability is verified, with preconditions, methods, pass/fail criteria, and evidence.  
**PRD:** [`river_warrior_prd_master.md`](river_warrior_prd_master.md)  
**Roadmap:** [`river_warrior_gap_remediation.md`](river_warrior_gap_remediation.md)

**Legend — current repo status column:** `N/A` = not yet implementable until roadmap item lands; `Smoke` = minimal scaffold check only.

---

## 1. Contract (Soroban)

| Req ID | Capability | Owner | Preconditions | Expected behavior | Test method | Pass criteria | Evidence | Roadmap ref | Current |
|--------|------------|-------|---------------|-----------------|-------------|---------------|----------|-------------|---------|
| REQ-C-01 | Soroban build produces WASM | `river_warrior` crate | Rust + `wasm32-unknown-unknown`, soroban-cli | `*.wasm` artifact | `soroban contract build` | Build exits 0; WASM under `target/wasm32-unknown-unknown/release/` | CI log / local log | GAP-1 | N/A |
| REQ-C-02 | Initialize idempotent guard | Contract | Fresh deploy | Second `initialize` panics | Unit test or CLI double-call | Second init fails | Test output | GAP-1 | N/A |
| REQ-C-03 | Storage keys present after init | Contract | After `initialize` | Admin, token, bounty, total readable | Unit test reads storage | Values match inputs | Test assertion | GAP-1 | N/A |
| REQ-C-04 | Admin auth on disburse | Contract | Initialized | Non-admin invoke panics | Test without mock auths | Panic / auth error | Test `#[should_panic]` or auth assert | GAP-1 | N/A |
| REQ-C-05 | Double-claim rejected | Contract | First `disburse_reward` ok | Second in same period panics | Unit test | Expected panic message | Test output | GAP-1 | N/A |
| REQ-C-06 | USDC transfer amount | Contract | Funded contract + minted test token | Collector balance += bounty | Token balance before/after | Exact stroops match | Test assertion | GAP-1 | N/A |
| REQ-C-07 | Total disbursed increments | Contract | After payout | `get_total_disbursed` increases by bounty | Unit test | Equality | Test output | GAP-1 | N/A |
| REQ-C-08 | Event published | Contract | After payout | Event topic + payload parseable | `env.events().all()` in test or RPC | Event present with amount | Test / RPC capture | GAP-1 | N/A |
| REQ-C-09 | Read-only getters | Contract | Initialized | `get_bounty`, `get_total_disbursed` | Unit / CLI `contract invoke -- read` | Match stored | CLI output | GAP-1 | N/A |
| REQ-C-10 | set_bounty admin-only | Contract | Initialized | Non-admin fails; admin updates | Two tests | Auth + value update | Test output | GAP-1 | N/A |
| REQ-C-11 | Release profile safety | `Cargo.toml` | Release build | `overflow-checks`, small wasm | Inspect `Cargo.toml` + build | Profile keys present | File diff + build | GAP-1 | N/A |

**Minimum contract test suite (from technical PDF):** 5 tests — happy path, double claim, total disbursed, unauthorized disburse, set_bounty + payout amount.

---

## 2. Backend (orchestration)

| Req ID | Capability | Owner | Preconditions | Expected behavior | Test method | Pass criteria | Evidence | Roadmap ref | Current |
|--------|------------|-------|---------------|-----------------|-------------|---------------|----------|-------------|---------|
| REQ-B-01 | HTTP (or queue) API for submission | Backend service | Server running | Accepts multipart or JSON+image | Integration test / curl | 200 + job id | HTTP log | GAP-2 | N/A |
| REQ-B-02 | AI gate | Backend | Mock AI | Rejected → no chain call | Integration test with mock | Zero Soroban invocations | Mock assert | GAP-2 | N/A |
| REQ-B-03 | Verified → invoke | Backend | Mock AI verified + testnet | `disburse_reward` submitted | Staging integration | Tx success | Tx hash | GAP-2 | N/A |
| REQ-B-04 | Secret hygiene | Backend | Config | No secret in logs/repo | Grep + code review + secret scan | Clean | Scan report | GAP-2 | N/A |
| REQ-B-05 | Idempotency | Backend | Duplicate request | Single payout | Replay test | One transfer | DB / idempotency key log | GAP-2 | N/A |

---

## 3. Frontend

| Req ID | Capability | Owner | Preconditions | Expected behavior | Test method | Pass criteria | Evidence | Roadmap ref | Current |
|--------|------------|-------|---------------|-----------------|-------------|---------------|----------|-------------|---------|
| REQ-F-01 | Mobile-first UI | Client | Build | Usable viewport | Manual / Playwright | Core actions visible | Screenshot | GAP-3 | N/A |
| REQ-F-02 | Status UX | Client | Mock API | Verified/rejected + tx hash | E2E with mocks | Copy matches API | E2E video/log | GAP-3 | N/A |
| REQ-F-03 | Error UX | Client | Simulated errors | User-readable message | E2E | No crash; message shown | E2E log | GAP-3 | N/A |

---

## 4. End-to-end (chain)

| ID | Capability | Preconditions | Test method | Pass criteria | Evidence | Roadmap ref | Current |
|----|------------|---------------|-------------|----------------|----------|-------------|---------|
| E2E-01 | Testnet deploy + init | Admin + USDC token ids | soroban deploy + invoke | Contract id + init ok | CLI transcript | GAP-4 | N/A |
| E2E-02 | Fund contract | USDC on contract | Transfer/mint to contract | Sufficient balance | Horizon / explorer | GAP-4 | N/A |
| E2E-03 | Collector trustline | Collector account | Add trustline | Can receive USDC | Account query | GAP-4 | N/A |
| E2E-04 | Full demo path | AI stub returns verified | Backend invokes | Balance increased | Before/after balances | GAP-4 | N/A |

---

## 5. Optional / bonus

| ID | Capability | Test method | Pass criteria | Roadmap ref |
|----|------------|-------------|----------------|-------------|
| BONUS-01 | Leaderboard from Horizon | Query payments to contract / events | Top N earners correct | GAP-4 |

---

## 6. Scaffold regression (current repo)

These verify **today’s** codebase only; they do **not** prove River Warrior capability.

| ID | Scope | Command / action | Pass criteria | Evidence | Current |
|----|-------|------------------|---------------|----------|---------|
| SMOKE-R-01 | River Warrior contract | `cd contracts/river_warrior && cargo test` | 5 tests pass | Terminal output | **Runnable** |
| SMOKE-R-02 | Counter lib (legacy) | `cd contracts/counter && cargo test` | All tests pass | Terminal output | **Runnable** |
| SMOKE-R-03 | Backend binary | `cd backend && cargo build` | Compiles | Terminal output | **Runnable** |
| SMOKE-R-04 | Client | `cd client && npm run build && npm run lint` | Exit 0 | Terminal output | **Runnable** |

---

## 7. Traceability summary

- Every **REQ-C-***, **REQ-B-***, **REQ-F-*** in the master PRD maps to at least one row above.
- Rows with **Current = N/A** are blocked until the linked **GAP-** phase in [`river_warrior_gap_remediation.md`](river_warrior_gap_remediation.md) is implemented.
