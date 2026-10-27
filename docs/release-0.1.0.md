# SaladCart 0.1.0 — release notes

Release date: 2026-10-27
Source: `main` at build time (includes batch voice-over, editor voice track, and the reliability pass)
Status: **unsigned prerelease** — macOS arm64 artifact built locally. Windows build is configured but not yet executed.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.1.0_aarch64.dmg` | 21.8 MB | `47e3da42125c5ddadabc745b80a6a949e10e6e1619507109e5dac890e6f6d8fe` |

This DMG is **not signed or notarized**. On first open, macOS may require right-click → **Open**, or System Settings → Privacy & Security → **Open Anyway**. Production distribution needs an Apple Developer ID and notarization.

The Windows build is defined in `.github/workflows/release.yml` (NSIS/MSI + tests, bundled-FFmpeg verification, draft release on tags). Windows x64 code compiles locally via `cargo xwin check`; no Windows runtime was available, and no GitHub Actions run was observed for this private repository (the workspace has no `gh` login or token, and Actions may be disabled). Push a `v0.1.0` tag only after CI is green for a formal artifact release.

## What is included

- Native Tauri 2 / Rust / React desktop workspace for macOS and Windows.
- Project-isolated SQLite with assets, character bible, scenes/shots, jobs, timeline and exports.
- Script → character → shot pipeline with an automatic context compiler.
- Character trait locks, reference portraits, turnaround sheets and continuity frames.
- Image/video provider adapters: Mock, OpenAI Images, Gemini/Imagen, Replicate, ComfyUI, Stability, fal; Kling, Runway, Luma, MiniMax, Veo, Replicate, fal.
- Persistent generation jobs with progress, cost estimates, retry/cancel and version lineage.
- Per-shot take history (`shot_takes`, schema v6): inspect older takes in the shot inspector, switch the active take (which resets it to review), and optionally assemble only approved shots.
- Local asset import into the project library (native multi-file dialog or browser file input), with FFmpeg duration probing and video thumbnails; cross-project character import is now all-or-nothing.
- One-click native demo project generated through the real store/media pipeline (script, characters, keyframes, four FFmpeg clips, music bed, beat-synced timeline).
- Bundled per-platform FFmpeg sidecar (4.4 on macOS, 4.1 on Windows); video preview, beat analysis and export work without Homebrew/winget. Audio mixing is compatible with the older Windows build. License notice in `THIRD_PARTY_NOTICES.md`.
- Timeline auto-assembly, trim/reorder, source audio with per-clip mute, subtitles, music, OS-TTS dialogue voice-over (single and batch across shots) mixed into export, a V1 voice track in the editor, formats, FFmpeg MP4 export with AI-generation metadata.
- The shot keyframe is the canonical video start frame; single-shot generation follows the per-shot continuity link instead of silently continuing from the previous shot. The enqueue-time context prompt is frozen with the job so queued work pays for the previewed prompt.
- Cross-project style copy and an exportable provenance manifest covering characters, shots, provider/model/seed lineage, jobs, and music licences.
- Jamendo licensed music search; Stability Audio / ElevenLabs music generation; native beat detection and auto-cut.
- OS keychain secrets. No telemetry. Browser demo bridge with a complete sample project for UI testing.

## Verification performed

- `pnpm exec tsc --noEmit` — passed.
- `pnpm test` — 6/6 frontend tests passed.
- `cargo test` — 21/21 automated Rust tests passed, plus an ignored macOS system-TTS integration test run manually and passed (store isolation, job round-trip, mock script generation, context compiler, cross-project style copy, asset import/media metadata, atomic character import, demo pipeline, provenance manifest, timeline assembly, beat snap, beat detection, WAV mock, real FFmpeg MP4 render with voice+music mix, source-audio mute mix, Windows filename sanitization, take history/selection, approved-only assembly, TTS label cleanup).
- `pnpm build` — production frontend bundle passed.
- Chromium manual walkthrough: Dashboard, Script, Characters, Shots, Editor, Music, Settings.
- Native macOS walkthrough: seeded demo loaded with 3 characters, 6 shots, 9 images, 4 videos, a 100 BPM music bed and 4 timeline clips; “全部配音” generated 6 system-TTS assets; the editor showed 4 voice clips on the V1 track and the music provider selector. Take history and switching, approved-only assembly, and the native import dialog were also verified.
- Windows x64 target: `cargo xwin check --target x86_64-pc-windows-msvc` passes, so the Windows `#[cfg]` code and dependency graph compile. Resource compilation used a local shim because `llvm-rc` is not installed on this Mac; a real Windows runtime build/test is still pending.
- **Native export verified end to end**: rendering the 19-second timeline produced an H.264/AAC MP4 at 1080×1920, 30 fps, with 6 generated voices, the music bed and source-audio mixing. A synchronous-command/Tokio-runtime panic that crashed the app on export was found in this walkthrough and fixed by spawning jobs on Tauri's async runtime.
- Packaged app detected the bundled FFmpeg 4.4 (`FFmpeg 就绪`) with no environment variables; sidecar lives at `Contents/MacOS/ffmpeg`.
- The release screenshot is [native-dashboard.png](assets/native-dashboard.png).

## Requirements and caveats

- **FFmpeg is bundled** as a platform sidecar (4.4 on macOS arm64, 4.1 on Windows x64). An external FFmpeg can still be selected from Settings or `SALADCART_FFMPEG`; the bundled copy is used otherwise. The mixer avoids the `normalize` option introduced in 4.4 so voice export also works on the Windows sidecar. FFmpeg is GPL/LGPL software; see [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).
- Real provider calls were not executed without user API keys. Mock providers prove the full local pipeline; adapters preserve provider error details and do not fake success. Video providers currently receive one start-frame image plus prompt context; multi-image reference inputs are not sent.
- Windows-specific code was hardened but **not runtime-verified**: PowerShell TTS writes a UTF-8 BOM and auto-selects a zh-CN SAPI voice, filenames strip Windows-illegal/reserved names, and mixing is 4.1-compatible. CI now tests with the bundled sidecar on each platform and fails if it is missing.
- Jamendo’s free API tier is non-commercial; commercial use requires a Jamendo licence. Music imported from the web stores licence, attribution and source URL. CC-BY/CC-BY-NC/CC-BY-ND obligations still apply to the final video.
- AI music providers have their own commercial terms and may change their APIs. Suno/Udio are intentionally not wrapped through unofficial endpoints.
- Not yet included: comments/shared review, multi-user permissions, filing-dossier export, lip sync/voice cloning, multi-track NLE, budget caps, approval-gated assembly, in-app selection between older takes, and provider actual-cost reconciliation.

## Build from source

```bash
pnpm install
pnpm test
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri build
```

The unsigned DMG is produced at `src-tauri/target/release/bundle/dmg/`.
