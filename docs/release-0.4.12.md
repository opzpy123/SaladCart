# SaladCart 0.4.12 — release notes

Release date: 2026-10-29
Source: `main` at build time (Chinese error messages for trend setup)
Status: **unsigned macOS prerelease; Windows x64 installer built and packaged-selftest passed on GitHub Actions.**

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.12_aarch64.dmg` | 21.91 MiB | `bd613b68dc33a1504aa5b9646c3b938aacc7332ee04ed02d61c922cd4ea3b495` |
| Windows x64 | GitHub Actions artifact `saladcart-windows-x64` (NSIS setup + MSI) | 52,000,203 bytes (zip) | see Actions artifact |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**. The Windows build is unsigned.
See [windows-ci-verification.md](windows-ci-verification.md) for the CI evidence.

## Changes since 0.4.11

- Localized trend-source validation and fetch errors so the Chinese UI no longer shows
  `invalid input: trend source must use https`-style messages.
- Localized the shared `Invalid` error prefix to `输入无效：`.
- Verified the new messages in the running app: an `http://` source is rejected with
  “输入无效：热榜接口必须使用 HTTPS”.

## Verification

- `cargo test` — **36 automated tests passed**, 3 ignored/manual; `pnpm test` 10/10.
- `pnpm tauri build` passed; DMG mounted and packaged `--selftest` passed (19 s export,
  423,595 bytes MP4).
- UI walkthrough on the real project: dashboard, script, characters, 18-shot board,
  context compiler (¥0.12 estimate, 1 reference), editor, asset provenance, video
  playback, new-project isolation, archive-to-trash, and UI export.
- UI export produced a new 1080×1920 H.264/AAC MP4, 5.04 s, 30 fps; task queue returned
  to idle and `provenance.json` was refreshed.
- Normal (non-env) launch read **4/4 provider keys** from the Keychain after the user
  completed the one-time macOS authorization sheet; DeepSeek showed READY.
- GitHub Actions run `40829025583` (v0.4.12): the Windows job passed bundled-FFmpeg
  verification, Rust tests, TTS smoke test, installer build, and **packaged
  `--selftest`** on `windows-latest`; macOS arm64 also passed. Windows installers are in
  the `saladcart-windows-x64` artifact.

## Remaining setup

- Authorized Douyin hot-list endpoint/token (Settings → 抖音热门参考).
- GenBGM needs an IAM `AccessKey:SecretKey`; the stored music value is an Ark API key.
- A public GitHub Release was not created; CI artifacts and `ci-status-*` branches carry
  the unsigned installers.

See [trend-reference.md](trend-reference.md) for the endpoint contract.
