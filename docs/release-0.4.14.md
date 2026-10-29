# SaladCart 0.4.14 — release notes

Release date: 2026-10-29
Source: `main` at build time (configurable export folder + subtitle layout)
Status: **unsigned macOS prerelease; Windows x64 built and packaged-selftest passed on GitHub Actions.**

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.14_aarch64.dmg` | 21.91 MiB | `83bedc733ede2cf6d966c6a89145c108caba8791af8f25f84ac3f5f4458de74d` |
| Windows x64 | GitHub Actions artifact `saladcart-windows-x64` | see Actions run for 0.4.14 | see Actions artifact |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**. The Windows build is unsigned.

## Changes since 0.4.13

- Added **Settings → 视频导出**: choose a fixed export folder with the native folder
  picker, type a path (including `~/Movies`), or reset to the default. **The default is
  the user's Desktop.** The folder is created when saved and used by the next render.
- Fixed subtitle burn-in sizing. libass lays out SRT at a 288-line play resolution, but
  the old code treated that as the video height, producing ~6.7× oversized text that was
  clipped horizontally and pushed into the middle of the frame. Font size and bottom
  margin are now computed in 288-based units (4.2% / 7.5% of frame height).
- The packaged `--selftest` still writes into its temporary project, not the user's
  Desktop.
- Includes the 0.4.13 novel-adaptation fix: camelCase `lockedTraits`, `timeOfDay` and
  `durationSec` now survive deserialization.

## Verification

- `cargo test` — **37 automated tests passed**, 3 ignored/manual; `pnpm test` 10/10.
- `pnpm tauri build` passed; DMG mounted and packaged `--selftest` passed (19 s export,
  423,595 bytes MP4, temp project path).
- UI verification: set the export folder to `/tmp/saladcart-export-test`, exported a
  1080×1920 MP4 with burned subtitles, confirmed the file landed in that folder, and
  extracted a frame: “字幕烧录测试” is centered at the bottom safe area with normal size.
  The subtitle and test export were then removed and the export folder reset to Desktop.
- Default-path verification: after resetting the export folder, rendered
  `saladcart-default-test.mp4`; the file appeared on `~/Desktop` (1080×1920, 5.04 s,
  H.264/AAC) and was removed after probing.
- Post-fix real DeepSeek rerun: a 411-character / 2-chapter story produced 1 episode,
  12 shots and 3 characters. `locked_traits` were populated (8 / 8 / 4 entries), shot
  durations varied from 5–9 seconds, and scene times were preserved (`深夜`). The test
  project was archived; the real project context card now shows the generated trait
  locks.
- macOS/Windows CI for this tree: run `40836638033` passed the macOS arm64 and Windows
  x64 jobs; Windows bundled-FFmpeg checks, Rust tests, SAPI TTS smoke, installer build
  and packaged `--selftest` all succeeded. Artifact `saladcart-windows-x64` is
  52,025,345 bytes (see [windows-ci-verification.md](windows-ci-verification.md)).

## Remaining setup

- Authorized Douyin hot-list endpoint/token (Settings → 抖音热门参考).
- GenBGM needs an IAM `AccessKey:SecretKey`; the stored music value is an Ark API key.
- A public GitHub Release was not created; CI artifacts and `ci-status-*` branches carry
  the unsigned installers.

See [trend-reference.md](trend-reference.md) for the endpoint contract.
