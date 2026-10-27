# Phase 2 — Asset import and reuse hardening

## Problem

`import_asset` / `import_asset_path` exist but no screen calls them, so users cannot
bring their own images, takes or audio into a project. Cross-project character import
also ignores failed asset copies, leaving a character with a broken reference portrait.

## Scope

- Asset Library import entry: desktop uses the native file dialog; the browser demo uses
  a file input and the mock bridge.
- Import copies files into the project `assets/{images,videos,audio}` folder, registers
  the asset, probes video/audio duration and generates a video thumbnail with FFmpeg when
  available.
- Atomic cross-project character import: copy reference/sheet assets first, insert the
  character once, and remove copied assets if anything fails.

## Non-goals

- Drag-and-drop, folder import, transcoding, EXIF scrubbing, or cloud URL import.
- De-duplicating identical files.
- Importing a whole project.

## API and data

No schema change.

- `import_asset_path(projectId, input: {sourcePath, kind?, name?}) -> Asset`
  copies the file and returns the registered asset.
- New store helper `set_asset_media(assetId, durationMs, thumbRelPath)` updates probe
  results after import.
- `import_character(sourceProjectId, characterId)` becomes all-or-nothing.

## UX

- Asset Library header: "导入素材". Desktop opens the native multi-file picker;
  browser opens a file input. Accepted: png/jpg/jpeg/webp, mp4/mov/webm, mp3/wav/m4a/aac/flac.
- After import, refresh the project snapshot and show the imported count.
- Cross-project import failure shows an error and does not create a partial character.

## Verification

- Store test: import into target, assert copied file/id; failed copy leaves the character
  count and asset count unchanged.
- `cargo test`, `pnpm exec tsc --noEmit`, `pnpm test`.
- Browser and native walkthrough: import one image and one video, see them in the library;
  import a character between projects and verify its reference portrait is copied.

## Verification record

- `cargo test`: 21 automated Rust tests passed (7 unit + 2 media + 12 store),
  1 ignored macOS TTS; includes asset copy/media metadata and atomic character import.
- `pnpm exec tsc --noEmit`, `pnpm test` (6/6) and production build passed.
- Browser demo: imported `upload-demo.png` and `voice-demo.wav` through the file input;
  both appeared at the top of the asset library.
- Native macOS app: the "导入素材" button opens the native multi-file dialog with the
  expected media filter. File selection inside the system dialog was not automated
  (blocked by the safe-automation policy); the copy/probe path is covered by tests.
- Phase artifact: `release/SaladCart_0.1.0_aarch64.dmg`
  (SHA-256 `47e3da42125c5ddadabc745b80a6a949e10e6e1619507109e5dac890e6f6d8fe`).
