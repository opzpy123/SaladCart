# SaladCart 0.1.0 — release notes

Release date: 2026-10-27
Source commit: `69b903e` (`feat: ship SaladCart v0.1 AI comic-drama workstation`)
Status: **unsigned prerelease** — macOS arm64 artifact built locally. Windows build is configured but not yet executed.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.1.0_aarch64.dmg` | 21.6 MB | `162d802de47dc6520aeae97c09b471c0de98105ad7dc46144c28ced8b3a8edf8` |

This DMG is **not signed or notarized**. On first open, macOS may require right-click → **Open**, or System Settings → Privacy & Security → **Open Anyway**. Production distribution needs an Apple Developer ID and notarization.

The Windows build is defined in `.github/workflows/release.yml` (NSIS/MSI + tests, FFmpeg sidecar verification). The workflow now also runs on pushes to `main`; this release was pushed there to trigger macOS arm64/x64 + Windows verification. A Windows runtime has not been available locally, so Windows remains CI-verified only (check the repository Actions tab; no `gh`/token is configured in this workspace). Push a `v0.1.0` tag after CI is green for a formal artifact release.

## What is included

- Native Tauri 2 / Rust / React desktop workspace for macOS and Windows.
- Project-isolated SQLite with assets, character bible, scenes/shots, jobs, timeline and exports.
- Script → character → shot pipeline with an automatic context compiler.
- Character trait locks, reference portraits, turnaround sheets and continuity frames.
- Image/video provider adapters: Mock, OpenAI Images, Gemini/Imagen, Replicate, ComfyUI, Stability, fal; Kling, Runway, Luma, MiniMax, Veo, Replicate, fal.
- Persistent generation jobs with progress, cost estimates, retry/cancel and version lineage.
- One-click native demo project generated through the real store/media pipeline (script, characters, keyframes, four FFmpeg clips, music bed, beat-synced timeline).
- Bundled per-platform FFmpeg 4.4 sidecar; video preview, beat analysis and export work without Homebrew/winget. License notice in `THIRD_PARTY_NOTICES.md`.
- Timeline auto-assembly, trim/reorder, subtitles, music, formats, FFmpeg MP4 export.
- Jamendo licensed music search; Stability Audio / ElevenLabs music generation; native beat detection and auto-cut.
- OS keychain secrets. No telemetry. Browser demo bridge with a complete sample project for UI testing.

## Verification performed

- `pnpm exec tsc --noEmit` — passed.
- `pnpm test` — 6/6 frontend tests passed.
- `cargo test` — 11/11 Rust tests passed (store isolation, job round-trip, mock script generation, context compiler, full demo pipeline, timeline assembly, beat snap, beat detection, WAV mock, real FFmpeg MP4 render).
- `pnpm build` — production frontend bundle passed.
- Chromium manual walkthrough: Dashboard, Script, Characters, Shots, Editor, Music, Settings.
- macOS `.app` debug and release bundles launched; a seeded native demo loaded with 3 characters, 6 shots, 9 images, 4 videos, a 100 BPM music bed and 4 timeline clips. Native image display was fixed to use the Tauri asset protocol for macOS absolute paths.
- Packaged app detected the bundled FFmpeg 4.4 (`FFmpeg 就绪`) with no environment variables; sidecar lives at `Contents/MacOS/ffmpeg`.
- The release screenshot is [native-dashboard.png](assets/native-dashboard.png). Full native UI click automation was partially blocked by macOS Computer Use being locked; the same data pipeline is covered by the demo service test and browser bridge.

## Requirements and caveats

- **FFmpeg 4.4 is bundled** as a platform sidecar. An external FFmpeg can still be selected from Settings or `SALADCART_FFMPEG`; the bundled copy is used otherwise. FFmpeg is GPL/LGPL software; see [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).
- Real provider calls were not executed without user API keys. Mock providers prove the full local pipeline; adapters preserve provider error details and do not fake success.
- Jamendo’s free API tier is non-commercial; commercial use requires a Jamendo licence. Music imported from the web stores licence, attribution and source URL. CC-BY/CC-BY-NC/CC-BY-ND obligations still apply to the final video.
- AI music providers have their own commercial terms and may change their APIs. Suno/Udio are intentionally not wrapped through unofficial endpoints.
- Not yet included: comments/shared review, multi-user permissions, filing-dossier export, AI label metadata in the MP4, lip sync/voice cloning, multi-track NLE, budget caps, and provider actual-cost reconciliation.

## Build from source

```bash
pnpm install
pnpm test
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri build
```

The unsigned DMG is produced at `src-tauri/target/release/bundle/dmg/`.
