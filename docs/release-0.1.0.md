# SaladCart 0.1.0 — release notes

Release date: 2026-10-27
Source commit: `69b903e` (`feat: ship SaladCart v0.1 AI comic-drama workstation`)
Status: **unsigned prerelease** — macOS arm64 artifact built locally. Windows build is configured but not yet executed.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.1.0_aarch64.dmg` | 6.6 MB | `866de0c335610ef5f6ded14ad8ebaf4413e9dcb1f82b7e3fb9fc13c6c7fb8443` |

This DMG is **not signed or notarized**. On first open, macOS may require right-click → **Open**, or System Settings → Privacy & Security → **Open Anyway**. Production distribution needs an Apple Developer ID and notarization.

The Windows build is defined in `.github/workflows/release.yml` (NSIS/MSI + tests). It has not been run because no tag was pushed and no Windows runner was available in this session. Push a `v0.1.0` tag after reviewing the workflow to run macOS arm64/x64 and Windows builds.

## What is included

- Native Tauri 2 / Rust / React desktop workspace for macOS and Windows.
- Project-isolated SQLite with assets, character bible, scenes/shots, jobs, timeline and exports.
- Script → character → shot pipeline with an automatic context compiler.
- Character trait locks, reference portraits, turnaround sheets and continuity frames.
- Image/video provider adapters: Mock, OpenAI Images, Gemini/Imagen, Replicate, ComfyUI, Stability, fal; Kling, Runway, Luma, MiniMax, Veo, Replicate, fal.
- Persistent generation jobs with progress, cost estimates, retry/cancel and version lineage.
- Timeline auto-assembly, trim/reorder, subtitles, music, formats, FFmpeg MP4 export.
- Jamendo licensed music search; Stability Audio / ElevenLabs music generation; native beat detection and auto-cut.
- OS keychain secrets. No telemetry. Browser demo bridge with a complete sample project for UI testing.

## Verification performed

- `pnpm exec tsc --noEmit` — passed.
- `pnpm test` — 6/6 frontend tests passed.
- `cargo test` — 10/10 Rust tests passed (store isolation, job round-trip, mock script generation, context compiler, timeline assembly, beat snap, beat detection, WAV mock, real FFmpeg MP4 render).
- `pnpm build` — production frontend bundle passed.
- Chromium manual walkthrough: Dashboard, Script, Characters, Shots, Editor, Music, Settings.
- macOS `.app` debug and release bundles launched; a real project was created and its database migration was verified. Full native UI generation automation was blocked by macOS Computer Use being locked; the same path is covered by service-level tests and the browser bridge.

## Requirements and caveats

- **FFmpeg** is required for mock video generation, video thumbnails, beat analysis and export; it is not bundled in this artifact. Install `brew install ffmpeg` (macOS) or `winget install Gyan.FFmpeg` (Windows), or set the path in Settings.
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
