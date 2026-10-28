# SaladCart 0.3.0 — release notes

Release date: 2026-10-28
Source: `main` at build time (phases 5–6: long-novel adaptation)
Status: **unsigned prerelease** — macOS arm64 artifact built locally. Windows compile gate
green; Windows runtime verification is still pending.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.3.0_aarch64.dmg` | 21.9 MB | `37b0420fea0040a80adc8cc45f2d551911646a07f51dad1fc7637099b165932c` |
| macOS 11+ arm64 (previous) | `release/SaladCart_0.2.0_aarch64.dmg` | 21.8 MB | `484f642fc7acf5cf718298223db7ed58c4c3594eb198064a0d8dd089a94fb29d` |

This DMG is **not signed or notarized**. On macOS, right-click → **Open** or use
System Settings → Privacy & Security → **Open Anyway** on first launch.

## What is new in 0.3.0

- **Full-novel adaptation**: Script Studio → “小说改编” accepts a pasted novel or a
  `.txt/.md/.text` file, stores it in the project (`novel_source`, schema v8), and runs a
  `novel` job.
- **Chapter-aware pipeline**: splits Chinese/English chapter headings, chunks each chapter
  to 6,000 characters, analyzes up to 120 chunks, merges character identities and locked
  traits across chapters, then writes episodes/scenes/shots through the existing script
  store.
- **Offline mock**: Mock LLM produces one episode per chapter so the full flow works
  without API keys.
- **Workspace**: live plan (characters, chapters, chunks, calls), provider/max-episode/
  style/replace options, job-dock progress, and a source badge
  “改编自《…》 · N 章 · M 字”.
- Long novels with more chapters than `maxEpisodes` are grouped; the hard cap is 60
  episodes. Volumes over 120 chunks must be split (≈720k characters).
- Carries forward 0.2.0: take governance, asset import, project budget caps,
  source-audio mute, Windows hardening, provider contract tests, and the Windows compile
  gate.

## Verification performed

- `pnpm exec tsc --noEmit` — passed.
- `pnpm test` — 6/6 frontend tests passed.
- `cargo test` — 29/29 automated Rust tests passed, plus an ignored macOS system-TTS
  test. New coverage: Chinese/English chapter splitting, chunk limits/plan counts,
  novel source persistence, 3-chapter adaptation, episode grouping, and the existing
  store/media/provider/budget/take suites.
- `pnpm check:windows` — `cargo xwin check --target x86_64-pc-windows-msvc` passed.
- Browser: 3-chapter novel → 3 chapters/3 chunks/3 calls → 3 episodes/12 shots; badge
  and job toast verified.
- Native macOS 0.3.0: 3-chapter novel → 3 episodes/9 shots; source badge, job progress
  and schema-v8 persistence verified.
- Windows runtime (TTS/sidecar/export) is **not verified**; see
  [release-checklist.md](release-checklist.md) §4.

## Requirements and caveats

- One novel source per project; re-adapting replaces the stored source and can replace
  existing episodes (default on).
- Real LLM calls need a configured provider and key. Cost estimates use optional
  `input_usd_per_1k` / `output_usd_per_1k` provider options; otherwise they are 0.
- Adapter quality depends on the selected model; the chunked beat analysis is deliberately
  conservative and keeps the project style bible.
- Video providers currently receive a single start frame, not multi-image references.
- Windows is compile-checked only in this workspace; GitHub Actions visibility/`gh`
  authentication is still unavailable. Publish the draft release after the Windows
  runtime gate.
- Jamendo’s free tier is non-commercial; music licence/attribution fields are stored.
- Not included: comments/shared review, multi-user permissions, filing-dossier export,
  lip sync/voice cloning, multi-track NLE, provider billing reconciliation.

## Build and gates

```bash
pnpm install
pnpm test
cargo test --manifest-path src-tauri/Cargo.toml
pnpm check:windows
pnpm tauri build
```

Phase design and verification records live in `docs/design/`. Release gates:
[docs/release-checklist.md](release-checklist.md).
