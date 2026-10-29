# SaladCart 0.4.13 — release notes

Release date: 2026-10-29
Source: `main` at build time (novel adaptation field compatibility)
Status: **unsigned macOS prerelease; Windows x64 built and packaged-selftest passed on GitHub Actions.**

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.13_aarch64.dmg` | 21.91 MiB | `86598cd3e4b545e31273d3c97de91845df825e1f1b3bffb07453c1df7218970c` |
| Windows x64 | GitHub Actions artifact `saladcart-windows-x64` | see Actions run for 0.4.13 | see Actions artifact |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**. The Windows build is unsigned.

## Changes since 0.4.12

- Fixed a data-loss bug in novel adaptation: the model returns `lockedTraits`,
  `timeOfDay` and `durationSec`, but the Rust structs expected `locked_traits`,
  `time_of_day` and `duration_sec`. Serde silently dropped them, so every adapted
  character had **no locked traits** and every shot fell back to the default 3 seconds.
  Added camelCase aliases while keeping snake_case support.
- Added a regression test that deserializes the real camelCase JSON shape and asserts
  traits, time of day and duration survive.
- Real DeepSeek adaptation was exercised through the UI: a 411-character, 2-chapter
  story produced 1 episode / 13 scenes / 13 shots / 3 characters. That run exposed the
  bug; the fix is covered by the new test and ships in this build.

## Verification

- `cargo test` — **37 automated tests passed**, 3 ignored/manual; `pnpm test` 10/10.
- `pnpm tauri build` passed; DMG mounted and packaged `--selftest` passed (19 s export,
  423,595 bytes MP4).
- macOS arm64 and Windows x64 CI jobs will rerun this tree; the previous 0.4.12 Windows
  job passed bundled-FFmpeg checks, Rust tests, SAPI TTS smoke, installer build and
  packaged `--selftest` (see [windows-ci-verification.md](windows-ci-verification.md)).
- UI re-run of the adapted project after this fix requires the next app launch to pass
  the one-time macOS Keychain authorization for the new ad-hoc signature.

## Remaining setup

- Authorized Douyin hot-list endpoint/token (Settings → 抖音热门参考).
- GenBGM needs an IAM `AccessKey:SecretKey`; the stored music value is an Ark API key.
- A public GitHub Release was not created; CI artifacts and `ci-status-*` branches carry
  the unsigned installers.

See [trend-reference.md](trend-reference.md) for the endpoint contract.
