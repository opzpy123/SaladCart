# SaladCart 0.4.5 — release notes

Release date: 2026-10-29
Source: `main` at build time (real Ark image/video + resilient startup)
Status: **unsigned macOS prerelease**. Windows packaging intentionally skipped.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.5_aarch64.dmg` | 21.89 MiB | `c3ff82a41259c9d2f1b0caf2f234f902082668ff7176908fc6cfd11cbdb09297` |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**.

## Changes since 0.4.3

- **Seedance image inputs**: first frame is submitted with `role=first_frame` (and
  references as `reference_image`); this fixes Ark's
  `role must be specified for image contents` 400.
- Default video model switched to the activated **Seedance 1.0 Pro Fast**
  (`doubao-seedance-1-0-pro-fast-251015`); image default remains **Seedream 5.0 Flash**.
- Ark cost estimates updated for Flash/Mini/Pro Fast pricing.
- **Keychain startup hardening**: keychain reads now time out (300 ms for presence,
  2 s for secrets) and only enabled providers are probed. A locked/unresponsive
  Keychain no longer leaves the app stuck on the splash screen.
- Provider cards keep their full model list when saving, render it as a select, and
  offer a **控制台** shortcut for Ark/IAM setup.
- Novel character aliases are merged (`穿雨衣的老人` / `雨衣老人` → `雨衣老人`) and cast
  names are mapped to canonical characters.
- Version bumped to 0.4.5 across package, Cargo, Tauri config, mock bridge and Settings.

## Verification

- **Real image**: Seedream 5.0 Flash generated a 1440×2560 JPEG for shot 1; character
  portraits for 嘉豪 / 林小满 / 王叔 also succeeded.
- **Real video**: Seedance 1.0 Pro Fast image-to-video task `cgt-20261029113407-2mrt7`
  succeeded in 36 seconds with a downloadable MP4 URL (direct provider probe using the
  exact app adapter).
- **Real script / novel**: DeepSeek script generation (16.8s) and a two-chapter novel
  adaptation (16.5s, 2 canonical characters) passed.
- `cargo test` — **33 automated tests passed**, 3 ignored/manual (DeepSeek live ×2,
  macOS TTS).
- `pnpm test` — **10/10** frontend tests; `pnpm build` and `pnpm tauri build` passed.
- Packaged `--selftest` — 4 clips, 19,000 ms MP4, 423,595 bytes.
- The 0.4.5 `.app` creates its window and reaches the dashboard; a fresh build starts
  cleanly with the Keychain timeout in place.
- `release/` is gitignored; the DMG is **not** in the tag.

## Remaining external setup

- **GenBGM music** still needs an IAM `AccessKey:SecretKey` pasted into Settings →
  音乐 → 火山引擎 纯音乐, and GenBGM v5.0 activation. The Ark API key is not accepted.
- Windows packaging remains skipped; Actions artifacts have not been verified.
- macOS artifact is unsigned; no public GitHub Release was created.

See [release-0.4.3.md](release-0.4.3.md) for the prior provider-routing release.
