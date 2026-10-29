# SaladCart 0.4.11 — release notes

Release date: 2026-10-29
Source: `main` at build time (Keychain status fix)
Status: **unsigned macOS prerelease**. Windows packaging intentionally skipped.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.11_aarch64.dmg` | 21.91 MiB | `dec480a87bfa008fd812ecc13f2095f7f22f2a1431edc2bd3bb484c574b4d054` |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**.

## Changes since 0.4.10

- Fixed a false “未配置 / Mock 就绪” state when valid API keys already existed in the
  Keychain. `has_secret` reused the 300 ms probe timeout; the first real item read can
  take longer while macOS evaluates the app's code-signature ACL. It now uses the 2 s
  read timeout, while the separate probe still fails fast when the Keychain is locked.
- Environment variables (`DEEPSEEK_API_KEY`, `ARK_API_KEY`, …) now take precedence over
  Keychain entries for provider detection and secret resolution. This is an explicit
  override and avoids reading the Keychain for that provider.
- Removed a redundant settings reload in `set_trend_source`.

## Verification

- `cargo test` — **36 automated tests passed**, 3 ignored/manual; `pnpm test` 10/10.
- `pnpm build` and `pnpm tauri build` passed.
- Packaged `--selftest` passed: 19 s export, 423,595 bytes MP4.
- Live UI verification of the Keychain status fix is still blocked: the Mac screen is
  locked and macOS shows a Keychain authorization dialog that security policy requires
  the user to complete manually. After clicking **始终允许**, Settings → **重新检测**
  refreshes the key state without restarting.
- A new ad-hoc build also changes the code signature, so macOS may ask for Keychain
  access once after each update. “始终允许” makes it persist for that build.

## Remaining setup

- Authorized Douyin hot-list endpoint/token (Settings → 抖音热门参考).
- GenBGM needs an IAM `AccessKey:SecretKey`; the stored value is currently an Ark API
  key, so the UI correctly reports the wrong credential shape.
- Windows/Actions verification and a public GitHub Release remain external.

See [trend-reference.md](trend-reference.md) for the endpoint contract.
