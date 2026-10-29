# SaladCart 0.4.8 — release notes

Release date: 2026-10-29
Source: `main` at build time (IAM credential check + Keychain guidance)
Status: **unsigned macOS prerelease**. Windows packaging intentionally skipped.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.8_aarch64.dmg` | 21.89 MiB | `e657b69db6a2f4ee0f716b8d32f0e18f344afd8d8f62f67acc21660008ff58e0` |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**.

## Changes since 0.4.7

- Volcengine pure-music “检测” now checks the stored value is `AccessKey:SecretKey`
  rather than merely checking that some key exists.
- Keychain-lock guidance, real-first music routing and the settings banner are included.
- The empty validation project and the mock demo project were archived to `trash/`,
  leaving only the real project. The demo can be recreated from “载入演示项目”.

## Verification

- Real Seedream 5.0 Flash keyframe (1440×2560) and three character portraits.
- Real Seedance 1.0 Pro Fast image-to-video (1088×1920, 5.04s), attached to shot 1,
  timeline assembled, 1080×1920 MP4 exported (2,518,406 bytes), frame checked.
- DeepSeek script (16.8s) and two-chapter novel adaptation (16.5s) passed.
- Native macOS TTS generated a 1.56s Chinese voice asset for shot 2 and linked it to the shot.
- `cargo test` 33 automated + 3 ignored/manual; `pnpm test` 10/10; `pnpm build` and
  `pnpm tauri build` passed.
- Project isolation verified: one real project remains; archived items are in `trash/`.

## Remaining setup

- GenBGM v5.0 needs an IAM `AccessKey:SecretKey` and model activation. A locked macOS
  screen also blocks Keychain reads; unlock before generating music.
- Windows packaging is skipped; Actions artifacts remain unverified.
- macOS artifact is unsigned; no public GitHub Release was created.
