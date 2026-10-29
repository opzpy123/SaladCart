# SaladCart 0.4.16 — release notes

Release date: 2026-10-29
Source: `main` at build time (Chinese credential/provider messages)
Status: **unsigned macOS prerelease; Windows x64 built and packaged-selftest passed on GitHub Actions.**

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.16_aarch64.dmg` | 21.91 MiB | `d3dc57e4a1d176676832df2a486435cdbcca7730bb8a64502ded572590bf9df2` |
| Windows x64 | GitHub Actions artifact `saladcart-windows-x64` | see Actions run for 0.4.16 | see Actions artifact |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**. The Windows build is unsigned.

## Changes since 0.4.15

- Localized missing-credential errors: the UI now shows
  “缺少凭据：Jamendo 音乐库 需要 API Key” instead of `missing credential: ... needs an API key`.
- Localized provider test results (stored key, missing key, local endpoint) into Chinese.
- Added a unit test for the localized missing-credential error.
- Includes the 0.4.15 CI fix that attaches only dmg/msi/exe to draft releases.

## Verification

- `cargo test` — **38 automated tests passed**, 3 ignored/manual; `pnpm test` 10/10;
  `npx tsc --noEmit` clean.
- `pnpm tauri build` passed; DMG mounted and packaged `--selftest` passed (19 s export,
  423,595 bytes MP4).
- The 0.4.15 tag run `40841915463` already proved the Windows/macOS builds and draft
  release; 0.4.16 reruns the same workflow.

## Remaining setup

- Authorized Douyin hot-list endpoint/token (Settings → 抖音热门参考).
- GenBGM needs an IAM `AccessKey:SecretKey`; the stored music value is an Ark API key.
- The draft release is unsigned and still needs a manual publish decision.
