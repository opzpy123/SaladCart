# SaladCart 0.4.2 — release notes

Release date: 2026-10-28
Source: `main` at build time (monochrome icon, Qoder-inspired)
Status: **unsigned macOS prerelease**. Windows packaging intentionally skipped.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.2_aarch64.dmg` | 21.85 MiB | `cfef2ceecb05a8e419a16a8cf1a7e95acc87cf0ddb938c85f3a6a5ae5516827b` |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**.

## Changes since 0.4.1

- Icon simplified to match Qoder's design language: **white squircle + single-colour
  line-art shopping cart** and three solid salad dots. Removed the emerald background,
  red/green produce and all glass colour effects.
- Optically centred: the dark-pixel mass centroid is `(511.6, 511.2)` on the 1024 canvas,
  rather than only centring the geometric bounding box.
- Added the vector source `public/brand/icon.svg`; `public/brand/icon.png` is the
  1024×1024 transparent master.
- Regenerated every Tauri icon (`icon.icns`, `icon.ico`, 32/64/128/256 PNGs, Windows
  Store logos) with `pnpm tauri icon`.
- In-app brand radii aligned to the icon's 22% corner ratio (sidebar 9px, splash 13px,
  settings 11px).
- Version bumped to 0.4.2 across package, Cargo, Tauri config, mock bridge and Settings.

## Verification

- `pnpm test` — 6/6 frontend tests passed.
- `cargo test` — 29 automated Rust tests passed, 1 macOS TTS test ignored (manual).
- `pnpm build` — production frontend bundle passed.
- `pnpm tauri build` — macOS arm64 `.app` (46.82 MiB) and DMG (21.85 MiB) built.
- Packaged `.app` embeds the regenerated icon (`icon.icns` SHA-1
  `4a364ce174b39b7938cc62a4d5b06d61a004183c`, identical to `src-tauri/icons/icon.icns`).
- Icon checked at 32 / 40 / 64 / 128 px; the cart handle, basket and wheels remain
  distinguishable.
- Native 0.4.2 app launched; the white icon renders correctly in the dark sidebar and the
  refreshed screenshot is [native-dashboard.png](assets/native-dashboard.png).

## Requirements and caveats

- Live provider calls still require user API keys; automated verification uses Mock and
  provider contract tests.
- macOS artifact is unsigned; notarization needs an Apple Developer ID.
- Windows packaging was skipped at the project owner's request; the compile gate remains
  unchanged.
- No public GitHub Release was created (no `gh` authentication in this workspace).

See [release-0.4.1.md](release-0.4.1.md) for the previous glass icon iteration.
