# SaladCart 0.4.9 — release notes

Release date: 2026-10-29
Source: `main` at build time (authorized Douyin trend reference)
Status: **unsigned macOS prerelease**. Windows packaging intentionally skipped.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.9_aarch64.dmg` | 21.91 MiB | `c4ac157d7aec099952e2d191797a51f00a46639388e899d623b1e6abb8d0d526` |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**.

## New: 抖音热门参考

- Dashboard entry **抖音热门参考** opens a dialog that reads a configured authorized
  hot-list endpoint and renders the top 20 items with rank and heat.
- **No web scraping**: Douyin has no unauthenticated hot-video API, and scraping the web
  page is intentionally not implemented. Use the official Open Platform with the
  required scope, or another authorized endpoint.
- Settings → **抖音热门参考** stores an HTTPS URL and an optional Bearer token in the
  Keychain. Backend rejects non-HTTPS, localhost/private hosts, credentials in URL,
  redirects, and responses over 2 MB.
- Parser accepts arrays or nested `data`/`result`/`list` arrays and the common
  `title`/`word`, `hot_value`/`hot`, `url`/`cover` aliases.
- **生成参考建议** sends the project premise/genre/tone plus the top 10 trends to the
  default real LLM (DeepSeek), asking for hooks, 30-second structure, character turns,
  titles and copying risks. Mock LLM is rejected for this action.
- Details and endpoint contract: [trend-reference.md](trend-reference.md).

## Verification

- `cargo test` — **35 automated tests passed**, 3 ignored/manual. New tests cover nested
  hot-list parsing, heat suffixes and rejection of HTTP/private hosts.
- `pnpm test` — 10/10; `pnpm build` and `pnpm tauri build` passed; DMG built.
- Existing real DeepSeek script/novel, Seedream keyframe/portraits, Seedance video,
  native TTS, timeline assembly and 1080×1920 export remain valid from 0.4.5.
- A live hot-list source was not available in this workspace; the feature requires the
  user's authorized endpoint/token. No mock trend data is injected.

## Remaining setup

- Configure an authorized HTTPS hot-list endpoint in Settings. Without it the dialog
  points to setup rather than fabricating data.
- GenBGM IAM key and Windows/Actions verification remain external.
- macOS artifact is unsigned; no public GitHub Release was created.
