# Phase 1 — Take history, selection, and review-gated assembly

## Problem

Generated videos are already preserved as assets and linked to jobs, but the UI only
shows the currently selected take. Users cannot compare or re-select older takes, and
auto-assembly ignores shot review status. This breaks the "抽卡 → 选片 → 装配" loop.

## Scope

- List video takes for a shot from successful `video` jobs, plus the currently selected
  asset when it has no job (seeded/demo/imported media).
- Select a take; selection writes `shots.video_asset_id` and resets the shot to
  `needs_review` so approval applies to the exact take being shipped.
- Optional review-gated assembly: only include `approved` shots when requested.
- Show take thumbnails, provider/model, duration, seed and cost in the shot inspector.

## Non-goals

- Image/keyframe take history.
- Side-by-side video comparison or diff playback.
- Multi-user review, comments, or immutable approval records.
- Deleting takes (asset lifecycle is unchanged).

## Data and API

Schema v6 adds a `shot_takes` join table:

```sql
CREATE TABLE shot_takes (
  shot_id TEXT NOT NULL REFERENCES shots(id) ON DELETE CASCADE,
  asset_id TEXT NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
  created_at TEXT NOT NULL,
  PRIMARY KEY (shot_id, asset_id)
);
```

- `attach_shot_asset(..., kind="video")` records a row, so every take ever selected is
  retained even when it is no longer the current video.
- Migration backfills the current `shots.video_asset_id` of existing projects.
- Takes are listed newest-first and joined to the latest job that produced the asset for
  provider/cost lineage. Assets without a job (seeded/imported) still appear.

Commands:

- `list_shot_takes(projectId, shotId) -> ShotTake[]`
- `select_shot_take(projectId, shotId, assetId) -> Shot`
- `auto_assemble_timeline(projectId, approvedOnly?) -> Timeline`

Rules:

- A take must be an existing `video` asset.
- Selection sets `status='needs_review'`; `approved` is never inherited across takes.
- `approvedOnly=false` keeps rough-cut behavior (latest selected video per shot).
- Take listing is capped at 50.

## UX

- Script Studio inspector: "历史版本" grid under the preview. It appears whenever a
  video take exists; the current take is marked and clicking another card selects it.
- Editor: "仅已通过" checkbox next to "重新装配".

## Verification

- Store test: list takes (job take + seeded current), select take resets status, and
  approved-only assembly excludes a non-approved shot.
- `cargo test`, `pnpm exec tsc --noEmit`, `pnpm test`.
- Browser demo and native app walkthrough; capture the phase artifact in release notes.

## Verification record

- `cargo test`: 19 automated Rust tests passed (7 unit + 2 media + 10 store),
  1 ignored macOS TTS test; includes take listing, take switching and approved-only
  assembly.
- `pnpm exec tsc --noEmit` and `pnpm test`: 6/6 passed; production build passed.
- Browser demo: generated a second take, switched to the older one (version v3,
  `needs_review`, toast), and approved-only assembly reduced the timeline to the
  single approved clip.
- Native macOS app: schema migrated to v6, both takes appeared, switching updated
  the preview, shot version and review status.
- Phase artifact: `release/SaladCart_0.1.0_aarch64.dmg`
  (SHA-256 `9136ed5790695750082d6775a10df6c2f6e2e79432b7ff2112f9833e35e8d34d`).
