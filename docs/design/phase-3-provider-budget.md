# Phase 3 — Provider contracts and project budget guardrails

## Problem

Provider adapters were only exercised through mocks; a request-shape regression could
ship unnoticed. There is also no spend ceiling: a batch of video jobs can be queued
against a project without any project-level constraint.

## Scope

- Wiremock contract tests for one image adapter (OpenAI Images) and one video adapter
  (Kling text-to-video submit + poll), asserting method, path, auth, response parsing and
  output extraction.
- Per-project budget cap (`budget_cap`, 0 = unlimited) stored in the project row
  (schema v7):
  - committed cost = estimates for queued/running jobs + actual cost for finished jobs;
  - generation and music enqueue reject when committed + new estimate exceeds the cap;
  - Settings shows cap, committed spend and a save action.
- Surface the actual cost already recorded on jobs (mock = 0, real = estimate until
  reconciliation).

## Non-goals

- Provider billing reconciliation or automatic price updates.
- Per-shot budgets or multi-currency support.
- Visual regression against live provider APIs (no keys in this workspace).

## API and data

- `project.budget_cap REAL NOT NULL DEFAULT 0`; `Project.budget_cap` exposed as
  `budgetCap`; `UpdateProjectInput.budgetCap` can change it.
- Store helpers:
  - `committed_cost() -> f64`
  - `ensure_budget(additional) -> AppResult<()>`
- Enforced in `enqueue_generation` and `enqueue_music` after the cost estimate.

## UX

- Settings → "项目预算": numeric cap, current committed spend, save button; 0 means
  unlimited. A rejected enqueue shows the cap/spend/estimate in the error toast.

## Verification

- `cargo test` provider contract tests (wiremock) and budget tests.
- `pnpm exec tsc --noEmit`, `pnpm test`.
- Browser: set a low cap, attempt generation, see rejection; raise cap, generation queues.
- Native build artifact + checksum.

## Verification record

- `cargo test`: 24 automated Rust tests passed (7 unit + 2 media + 2 provider contract
  + 13 store), 1 ignored macOS TTS; includes OpenAI Images and Kling submit/poll
  contracts plus project budget commit/reject cases.
- `pnpm exec tsc --noEmit`, `pnpm test` (6/6) and production build passed.
- Browser: Settings → 项目预算 saved ¥12.50 and showed the success toast; committed cost
  displays from queue/actual job data.
- Native macOS: Settings shows the 项目预算 section with cap/committed fields on schema
  v7; the native save action was not automated (safe-automation policy).
- Phase artifact: `release/SaladCart_0.1.0_aarch64.dmg`
  (SHA-256 `8ea0293b17add9128c903a0e3f822c1bbb2c5ca2ddf8f6d6a9753198e4f9bc19`).
