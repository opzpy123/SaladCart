# SaladCart 0.4.1 — release notes

Release date: 2026-10-28
Source: `main` at build time (Liquid Glass app icon)
Status: **unsigned macOS prerelease**. Windows packaging intentionally skipped.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.1_aarch64.dmg` | 26.79 MiB | `79515731ff6e79c6115b4f39b2c333caa5ab666e9b05a4e246e2b92137d29331` |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**.

## Changes since 0.4.0

- New **Liquid Glass app icon**: a clear frosted-glass shopping cart carrying three
  translucent salad spheres (lime, jade, tomato red) on a deep emerald-teal glass
  squircle. Replaces the previous flat vector cart.
- 1024×1024 master at `public/brand/icon.png` with transparent corners; all Tauri
  desktop icons (`icon.icns`, `icon.ico`, 32/64/128/256 PNGs and Windows Store logos)
  regenerated with `pnpm tauri icon`.
- The same master is used by the sidebar logo, splash screen and Settings page, so the
  in-app brand mark matches the Dock/installer icon.
- Sidebar footer contrast and version label from 0.4.0 are retained; version reads
  `v0.4.1`.
- Design spec updated with the icon construction rules:
  [design/ui-liquid-glass.md](design/ui-liquid-glass.md#app-图标041).

## Verification

- `pnpm test` — 6/6 frontend tests passed.
- `cargo test` — 29 automated Rust tests passed, 1 macOS TTS test ignored (manual).
- `pnpm build` — production frontend bundle passed.
- `pnpm tauri build` — macOS arm64 `.app` (49.87 MiB) and DMG (26.79 MiB) built.
- Packaged `.app` embeds the regenerated `icon.icns` (SHA-1
  `a947e4fce58e56be4208c6adbd92e54f1769dc7c`, identical to `src-tauri/icons/icon.icns`).
- Icon checked at 32 / 40 / 64 / 128 px; cart, handle, wheels and the red accent remain
  distinguishable.
- Native 0.4.1 app launched and photographed; the sidebar shows the new mark and the
  refreshed screenshot is [native-dashboard.png](assets/native-dashboard.png).

## Requirements and caveats

- Live provider calls still require user API keys; automated verification uses Mock and
  provider contract tests.
- macOS artifact is unsigned; notarization needs an Apple Developer ID.
- Windows packaging was skipped for 0.4.0 and 0.4.1 at the project owner's request; the
  compile gate remains unchanged.
- No public GitHub Release was created (no `gh` authentication in this workspace).

See [release-0.4.0.md](release-0.4.0.md) for the Liquid Glass redesign notes.
