# SaladCart 0.2.0 — release notes

Release date: 2026-10-27
Source: `main` at build time (phase 1–4 delivery)
Status: **unsigned prerelease** — macOS arm64 artifact built locally. Windows compile gate
green; Windows runtime verification is still pending.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.2.0_aarch64.dmg` | 21.8 MB | `484f642fc7acf5cf718298223db7ed58c4c3594eb198064a0d8dd089a94fb29d` |
| macOS 11+ arm64 (previous) | `release/SaladCart_0.1.0_aarch64.dmg` | 21.8 MB | `8ea0293b17add9128c903a0e3f822c1bbb2c5ca2ddf8f6d6a9753198e4f9bc19` |

This DMG is **not signed or notarized**. On first open, macOS may require right-click →
**Open**, or System Settings → Privacy & Security → **Open Anyway**. Production
distribution needs an Apple Developer ID and notarization.

## What is new in 0.2.0

- **Take governance**: `shot_takes` (schema v6) retains every selected video take; the
  shot inspector can switch takes, which resets the shot to `needs_review`; auto-assembly
  can include only approved shots.
- **Asset import**: import images, video and audio from the native file dialog (or a
  browser file input in the demo), copy into the project, probe duration and generate
  video thumbnails; cross-project character import is atomic.
- **Budget guardrail**: per-project cap (schema v7) sums queued estimates and finished
  actuals before image, video or music jobs are created; Settings shows cap and committed
  spend.
- **Provider contracts**: wiremock tests cover OpenAI Images and Kling submit/poll.
- **Generation fixes**: keyframes are the video start frame, per-shot continuity is
  honored, the enqueue-time context is frozen with the job, and queue/export jobs run on
  Tauri's async runtime instead of panicking on a missing Tokio reactor.
- **Editing**: source audio is mixed with real per-clip mute, voices align with rendered
  clips, music switches clear stale beats, and the music provider is selectable.
- **Windows hardening**: TTS UTF-8 BOM + zh-CN SAPI selection, Windows-safe filenames,
  FFmpeg 4.1-compatible audio mixing, sidecar-based CI tests, and a `pnpm check:windows`
  compile gate.
- **Delivery**: Windows compile workflow, release checklist, draft release on tags, and
  refreshed macOS artifacts.

## Verification performed

- `pnpm exec tsc --noEmit` — passed.
- `pnpm test` — 6/6 frontend tests passed.
- `cargo test` — 24/24 automated Rust tests passed, plus an ignored macOS system-TTS
  integration test run manually and passed. Covers store isolation, job round-trip, mock
  script generation, context compiler, cross-project style copy, asset import/media
  metadata, atomic character import, demo pipeline, provenance, timeline assembly, beat
  snap/detection, WAV mock, real FFmpeg render with voice/music/source audio, Windows
  filename sanitization, take history/selection, approved-only assembly, project budget,
  OpenAI Images/Kling contracts, and TTS label cleanup.
- `pnpm check:windows` — `cargo xwin check --target x86_64-pc-windows-msvc` passed
  (compile-only, local resource-compiler shim because `llvm-rc` is not installed).
- Browser walkthrough: take generation/switching, approved-only assembly, asset import,
  budget save, voice track and music provider.
- Native macOS walkthrough: v0.2.0 app launch, schema v6→v7 migration, take switching
  (version/status/preview), native import dialog, budget section, 6-voice batch, V1 voice
  track, and a 19-second 1080×1920 H.264/AAC export with music and voices.
- Windows runtime (TTS, sidecar, export) is **not verified**; see
  [release-checklist.md](release-checklist.md) §4.

## Requirements and caveats

- FFmpeg is bundled (4.4 macOS arm64, 4.1 Windows x64); an external FFmpeg can be chosen
  in Settings or `SALADCART_FFMPEG`. The mixer avoids 4.4-only filter options.
- Real provider calls were not executed without user API keys. Mock providers prove the
  local pipeline; adapters propagate provider errors rather than faking success. Video
  providers currently receive one start frame plus prompt context, not multi-image
  references.
- Windows code is compile-checked but not runtime-tested in this workspace. GitHub
  Actions runs were not observable for this private repository; no `gh` login/token is
  configured. Publish the draft release only after the Windows gate.
- Jamendo’s free tier is non-commercial; music licence/attribution fields are stored.
  AI music providers have their own terms.
- Not yet included: comments/shared review, multi-user permissions, filing-dossier export,
  lip sync/voice cloning, multi-track NLE, in-app selection of older takes outside the shot
  inspector, provider billing reconciliation.

## Build and gates

```bash
pnpm install
pnpm test
cargo test --manifest-path src-tauri/Cargo.toml
pnpm check:windows
pnpm tauri build
```

The unsigned DMG is produced at `src-tauri/target/release/bundle/dmg/`. Phase design and
verification records live in `docs/design/`.
