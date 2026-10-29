# SaladCart 0.4.10 — release notes

Release date: 2026-10-29
Source: `main` at build time (Douyin hot-list parser hardening)
Status: **unsigned macOS prerelease**. Windows packaging intentionally skipped.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.10_aarch64.dmg` | 21.91 MiB | `39e0f50818c33a7f30d902db2c93d5d461aab76a65ad34b7b122c6f3ee83d111` |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**.

## Changes since 0.4.9

- The hot-list parser now understands the official Douyin hot-video shape where title,
  play count, share URL and cover live under `aweme_info` (`desc`,
  `statistics.play_count`, `share_info.share_url`, `video.cover.url_list`), in addition
  to flat `title`/`word`/`hot_value` items.
- Added a regression test for the nested `aweme_info` response.
- No scraping, no mock trend fallback, and no change to the authorization/HTTPS
  restrictions from 0.4.9.

## Verification

- `cargo test` — **36 automated tests passed**, 3 ignored/manual; `pnpm test` 10/10.
- `pnpm build` and `pnpm tauri build` passed; DMG built.
- Trend parsing tests cover nested list shapes, heat suffixes and private/insecure URL
  rejection.
- A live authorized hot-list endpoint was not available in this workspace; the feature
  requires the user's own official/authorized source and token. UI/browser verification
  remains blocked while the Mac screen is locked.

## Remaining setup

- Configure an authorized HTTPS hot-list endpoint in Settings → 抖音热门参考.
- GenBGM IAM key, Windows/Actions verification and a public GitHub Release remain
  external.

See [trend-reference.md](trend-reference.md) for the endpoint contract.
