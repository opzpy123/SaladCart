# SaladCart 0.4.15 — release notes

Release date: 2026-10-29
Source: `main` at build time (draft-release asset fix)
Status: **unsigned macOS prerelease; Windows x64 built and packaged-selftest passed on GitHub Actions.**

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.15_aarch64.dmg` | 21.91 MiB | `86fc0891efc3754dbc282f6a733ee358b0f4be69f7c13d26f89efe1d5422e3bb` |
| Windows x64 | GitHub Actions artifact `saladcart-windows-x64` | see Actions run for 0.4.15 | see Actions artifact |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**. The Windows build is unsigned.

## Changes since 0.4.14

- No app behavior changes. The tag release job now attaches only installer files
  (`*.dmg`, `*.msi`, `*.exe`) instead of every file under `artifacts/**/*`. The previous
  glob included the macOS `.app` directory and build logs, which caused
  `softprops/action-gh-release` to fail with “Not Found” while updating release assets.
- This patch exists to verify that a tag build can create the **draft** GitHub release
  with installers attached.

## Verification

- `cargo test` — **37 automated tests passed**, 3 ignored/manual; `pnpm test` 10/10.
- `pnpm tauri build` passed; DMG mounted and packaged `--selftest` passed (19 s export,
  423,595 bytes MP4).
- The 0.4.15 tag run [`40841915463`](https://github.com/opzpy123/SaladCart/actions/runs/40841915463)
  completed all three platform builds and the `release` job successfully; the draft
  GitHub release was created with the dmg/msi/exe attachments. 0.4.14 already proved the
  Windows and macOS build jobs, bundled-FFmpeg checks, TTS smoke and packaged selftest.

## Remaining setup

- Authorized Douyin hot-list endpoint/token (Settings → 抖音热门参考).
- GenBGM needs an IAM `AccessKey:SecretKey`; the stored music value is an Ark API key.
- The draft release is unsigned and still needs a manual publish decision.
