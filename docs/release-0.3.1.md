# SaladCart 0.3.1 — release notes

Release date: 2026-10-28
Source: `main` at build time (post-0.3.0 hardening + Windows portable artifact)
Status: **unsigned prerelease**. Windows x64 portable binary built; Windows GUI runtime
still needs a real Windows desktop session.

## Artifacts

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.3.1_aarch64.dmg` | 21.9 MB | `c2ac96ffe4d490ba0f9133d580790dd6bfd55314b3daa558ccc4b98e24ecf157` |
| Windows 10/11 x64 (portable) | `release/SaladCart_0.3.1_windows-x64-portable.zip` | 26 MB | `f25eb41aab4dd7ebec175f6d7c1b498e72c91c47d129aa79a7f9f929088f810f` |

The macOS DMG is **not signed or notarized**. Right-click → **Open** or use Privacy &
Security → **Open Anyway** on first launch.

The Windows zip contains `saladcart.exe` + `ffmpeg.exe`; unpack and run. It needs the
WebView2 Runtime (Windows 11 includes it). It was cross-compiled on macOS with
cargo-xwin and uses a stub Windows resource (no custom icon/version metadata), so the
CI NSIS/MSI installer remains the primary distribution path. It is unsigned.

## Changes since 0.3.0

- Dashboard pipeline now exposes **小说改编** alongside Script Studio.
- LLM novel adaptation rejects analyses without recurring characters instead of
  producing empty cast shots.
- Added a wiremock contract test for the real LLM novel path (chunk → summary →
  characters → 2 episodes / 2 scenes each / 2 calls).
- Windows delivery gate: CI now runs a Windows SAPI TTS smoke test, builds installers
  with the local pnpm/Tauri CLI instead of tauri-action, and publishes `build.log` to
  the status branch when a platform fails.
- Version bumps to 0.3.1.

## Verification

- `cargo test`: **29 automated tests passed** + 1 ignored macOS TTS test run manually
  with the bundled FFmpeg.
- `pnpm exec tsc --noEmit`, `pnpm test` (6/6), `pnpm build`.
- Native macOS 0.3.0 walkthrough: 3-chapter novel → 3 episodes / 9 shots; source
  badge, take switching, budget, import dialog, voice track and 19s 1080×1920 export
  verified in prior phases. 0.3.1 DMG contains the dashboard entry and validation fix.
- Windows: `cargo xwin check` passes; release `saladcart.exe` (PE32+ GUI x64) and the
  real FFmpeg 4.1 sidecar were cross-built and zipped. No Windows GUI/TTS runtime
  session was available locally.
- CI: earlier Windows matrix runs completed the bundled-FFmpeg Rust tests successfully;
  the new direct-build + TTS/logging workflow is committed, but GitHub Actions had not
  yet produced a new status branch for the latest commits at release time.

## Requirements and caveats

- Live provider calls require user keys; all current verification uses Mock plus
  local provider contract tests.
- Windows UI/WebView2 launch, SAPI voices, installer signing and SmartScreen behaviour
  remain unverified locally.
- Novel adaptation: one source per project, 120 chunks (~720k chars), 60 episodes max.
- Not included: comments/shared review, multi-user permissions, filing-dossier export,
  lip sync/voice cloning, multi-track NLE, provider billing reconciliation.

See [release-0.3.0.md](release-0.3.0.md) for the full 0.3.0 feature notes and
[release-checklist.md](release-checklist.md) for gates.
