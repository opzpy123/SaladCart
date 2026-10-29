# SaladCart 0.4.12 — release notes

Release date: 2026-10-29
Source: `main` at build time (Chinese error messages for trend setup)
Status: **unsigned macOS prerelease**. Windows packaging intentionally skipped.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.12_aarch64.dmg` | 21.91 MiB | `bd613b68dc33a1504aa5b9646c3b938aacc7332ee04ed02d61c922cd4ea3b495` |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**.

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
- Normal (non-env) launch read **3 of 4** provider keys from the Keychain. DeepSeek still
  showed unconfigured because macOS displayed its one-time Keychain authorization sheet;
  security policy requires the user to complete that sheet manually. This is the only
  remaining UI verification step for the Keychain fix.

## Remaining setup

- Unlock the Mac, approve the “SaladCart 想要使用…com.saladcart.studio.providers” sheet
  once, then click Settings → **重新检测**. The app should show 4 configured keys.
- Authorized Douyin hot-list endpoint/token (Settings → 抖音热门参考).
- GenBGM needs an IAM `AccessKey:SecretKey`; the stored music value is an Ark API key.
- Windows/Actions verification and a public GitHub Release remain external.

See [trend-reference.md](trend-reference.md) for the endpoint contract.
