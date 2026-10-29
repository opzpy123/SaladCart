# SaladCart 0.4.6 — release notes

Release date: 2026-10-29
Source: `main` at build time (music routing + credential guard)
Status: **unsigned macOS prerelease**. Windows packaging intentionally skipped.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.6_aarch64.dmg` | 21.89 MiB | `c9e2d67fc34ca2702ed788f0c4c59c8c68cd83ffcd4bf93e8410effc10e11e00` |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**.

## Changes since 0.4.5

- Music generation now preselects an enabled, configured real provider instead of always
  defaulting to Mock, matching the LLM/image/video routing behavior.
- The music provider list only offers enabled providers and labels offline/key state.
- `set_provider_secret` rejects a non-`AccessKey:SecretKey` value for the Volcengine
  pure-music provider, so an Ark API key cannot be stored as an IAM credential.
- Documentation records the real Seedance take and 1080×1920 export.

## Verification carried over from 0.4.5

- Real Seedream 5.0 Flash keyframe (1440×2560) and three character portraits.
- Real Seedance 1.0 Pro Fast image-to-video (1088×1920, 5.04s); attached to shot 1,
  timeline assembled to 1 clip, FFmpeg export 1080×1920 (2,518,406 bytes); frame checked.
- DeepSeek script (16.8s) and two-chapter novel adaptation (16.5s) passed.
- `cargo test` 33 automated + 3 manual/ignored; `pnpm test` 10/10; `pnpm build` and
  `pnpm tauri build` passed.
- 0.4.6 DMG built after the music routing/credential guard changes.

## Remaining setup

- GenBGM v5.0 still needs an IAM `AccessKey:SecretKey` and model activation; the Ark API
  key is not accepted. The macOS screen was locked during the final attempt, which also
  blocks Keychain reads (the app now times out instead of hanging).
- Windows packaging is skipped; Actions artifacts remain unverified.
- macOS artifact is unsigned; no public GitHub Release was created.

See [release-0.4.5.md](release-0.4.5.md) for the real-media release notes.
