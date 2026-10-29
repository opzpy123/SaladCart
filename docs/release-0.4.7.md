# SaladCart 0.4.7 — release notes

Release date: 2026-10-29
Source: `main` at build time (locked-Keychain visibility)
Status: **unsigned macOS prerelease**. Windows packaging intentionally skipped.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.7_aarch64.dmg` | 21.89 MiB | `961b24acd1278b0315720d1f94ddf62af2ea6adfc90f7ba7642f3b4bef6fae9e` |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**.

## Changes since 0.4.6

- Settings now reports `keychainAvailable`; when macOS is locked or the Keychain is
  unresponsive, the provider page shows a warning and a **重新读取** action instead of
  silently presenting every key as missing.
- Keychain reads have timeouts and only enabled providers are probed, so a locked
  Keychain cannot hold the splash screen.
- Music generation prefers a configured real provider over Mock; the Volcengine
  pure-music field rejects non-`AccessKey:SecretKey` values.
- Version bumped to 0.4.7.

## Verification carried over

- Real Seedream 5.0 Flash keyframe (1440×2560) and three character portraits.
- Real Seedance 1.0 Pro Fast image-to-video (1088×1920, 5.04s), attached to shot 1,
  timeline auto-assembled, 1080×1920 MP4 exported (2,518,406 bytes) and frame-checked.
- DeepSeek script (16.8s) and two-chapter novel adaptation (16.5s) passed.
- `cargo test` 33 automated + 3 ignored/manual; `pnpm test` 10/10; `pnpm build` and
  `pnpm tauri build` passed.
- 0.4.7 starts while macOS is locked (window created without waiting indefinitely on
  Keychain reads).

## Remaining setup

- GenBGM v5.0 requires an IAM `AccessKey:SecretKey` and model activation. Screen lock
  also blocks Keychain reads, so unlock macOS before generating music.
- Windows packaging is skipped; Actions artifacts remain unverified.
- macOS artifact is unsigned; no public GitHub Release was created.
