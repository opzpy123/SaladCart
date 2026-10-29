# Windows CI verification

SaladCart ships unsigned Windows x64 installers from GitHub Actions. This note records
the evidence used to treat Windows as build-and-selftest verified.

## v0.4.14 run

- Workflow: **Build desktop installers**
- Run: [`40836638033`](https://github.com/opzpy123/SaladCart/actions/runs/40836638033)
  (`v0.4.14`, commit `04d2d96`)
- Windows job: `build (windows-latest, windows-x64, x86_64-pc-windows-msvc)` — success
- Passed steps include bundled-FFmpeg verification, Rust tests, Windows TTS smoke test,
  installer build, and **Windows packaged selftest**.
- Artifact: `saladcart-windows-x64` (52,025,345 bytes zip), containing the NSIS
  `*-setup.exe` and MSI installers.
- macOS arm64 also passed in this run.

## v0.4.12 run

- Workflow: **Build desktop installers**
- Run: [`40829025583`](https://github.com/opzpy123/SaladCart/actions/runs/40829025583)
  (`v0.4.12`, commit `34e0932`)
- Windows job: `build (windows-latest, windows-x64, x86_64-pc-windows-msvc)` — success
- Passed steps:
  - Verify bundled FFmpeg (Windows)
  - Rust tests with bundled FFmpeg (Windows)
  - Windows TTS smoke test
  - Build installers (Windows)
  - **Windows packaged selftest** (`saladcart.exe --selftest`)
- Artifact: `saladcart-windows-x64` (52,000,203 bytes zip), containing the NSIS
  `*-setup.exe` and MSI installers.

The same Windows job also passed on the v0.4.11 run `40825442646`; the macOS arm64 and
Windows jobs passed there as well. The v0.4.11 release job alone failed at
“Create draft GitHub release”, which does not affect the installers.

## Local Windows cross-check

`npm run check:windows` (cargo-xwin against `x86_64-pc-windows-msvc`) passes locally on
the 0.4.12 tree:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Limits

- The installers are **unsigned**; Windows SmartScreen may warn.
- CI ran the packaged selftest on a real `windows-latest` runner, but no interactive
  Windows desktop session (window creation, drag/drop, device playback) was observed.
- Artifacts are not downloadable without GitHub authentication; the CI reporter copies
  installers to public `ci-status-*` branches when the workflow has permissions.
