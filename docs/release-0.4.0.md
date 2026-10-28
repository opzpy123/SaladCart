# SaladCart 0.4.0 — release notes

Release date: 2026-10-28
Source: `main` at build time (Liquid Glass redesign)
Status: **unsigned macOS prerelease**. Windows packaging intentionally skipped for this release.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.0_aarch64.dmg` | 21.94 MiB | `54bd031ee453ff65aed4e51be81056b0a22e0decfed21979db4329c33e54500a` |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**.

Windows packaging was skipped in this release at the project owner's request. The
existing Windows compile gate (`.github/workflows/windows-check.yml`) and the release
matrix are unchanged; no Windows artifact is published for 0.4.0.

## Changes since 0.3.1

- **Liquid Glass redesign** referencing the macOS 26 / iOS 26 visual language:
  - cool neutral canvas with a fixed blue/violet/green aurora;
  - dark translucent glass sidebar, frosted app bar and translucent panels
    (`backdrop-filter: blur(24px) saturate(160%)`, 22px radii, inset glass rim);
  - 28px modal sheets with a dark blurred backdrop;
  - SF Pro Display / Segoe UI Variable typography with tight display tracking;
  - system blue as the single interaction colour, green reserved for success,
    amber for beat markers;
  - blue timeline/waveform/playhead, green subtitle chips, purple voice chips;
  - glass project switcher, blue active rows, pill buttons, blue focus rings;
  - page-entry spring animation, card hover lift and `prefers-reduced-motion` fallback.
- Sidebar footer contrast raised to `#a8a8af` on the dark rail; version label `#d1d1d6`.
- Version bumped to 0.4.0 across `package.json`, Cargo, Tauri config, mock bridge and
  Settings fallback.
- Design spec: [design/ui-liquid-glass.md](design/ui-liquid-glass.md).
- Refresh of the native dashboard screenshot
  ([native-dashboard.png](assets/native-dashboard.png)) taken from the packaged 0.4.0
  macOS app.

## Verification

- `pnpm exec tsc --noEmit` — passed.
- `pnpm test` — 6/6 frontend tests passed.
- `cargo test` — 29 automated Rust tests passed (10 lib + 2 media + 3 provider contract
  + 14 store), 1 macOS system-TTS test ignored (manual).
- `pnpm build` — production frontend bundle passed (1957 modules, 83.22 kB CSS).
- `pnpm tauri build` — macOS arm64 `.app` (46.87 MiB) and DMG (21.94 MiB) built.
- Native 0.4.0 app launched and verified: dashboard renders the glass theme, demo
  project loaded, sidebar footer and version `v0.4.0` readable; window capture is the
  release screenshot.
- Browser (Chromium) walkthrough in earlier phases covered Dashboard, Script, Characters,
  Shots, Editor, Music, Settings and the novel-adaptation dialog with the same tokens.

## Requirements and caveats

- Live provider calls still require user API keys; all automated verification uses Mock
  and local provider contract tests.
- macOS artifact is unsigned; notarization needs an Apple Developer ID.
- Windows build/runtime verification is unchanged from 0.3.1 and no 0.4.0 Windows
  artifact was produced.
- No public GitHub Release was created (no `gh` authentication in this workspace), and
  GitHub Actions results for the latest commits were not observed locally.
- Not included: comments/shared review, multi-user permissions, filing-dossier export,
  lip sync/voice cloning, multi-track NLE, provider billing reconciliation.

See [release-0.3.1.md](release-0.3.1.md) for the previous release and
[release-checklist.md](release-checklist.md) for the gates.
