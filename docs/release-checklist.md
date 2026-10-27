# SaladCart release checklist

Use this checklist for every versioned release. A phase is only complete when its
artifact and verification record are committed.

## 1. Pre-flight

- [ ] Working tree clean; version bumped in `package.json`, `src-tauri/Cargo.toml`
      and `src-tauri/tauri.conf.json`.
- [ ] `README.md` release section and `docs/release-<version>.md` point at the new
      artifact and SHA.
- [ ] `docs/design/phase-*.md` verification records are current.

## 2. Automated gates

- [ ] `pnpm exec tsc --noEmit`
- [ ] `pnpm test`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] `pnpm build`
- [ ] `pnpm check:windows` (compile-only Windows gate; real Windows runner required
      for runtime validation)
- [ ] No provider keys or `.env` files in `git status`.

## 3. Native macOS walkthrough

- [ ] Launch the packaged `.app`, open the demo project, confirm images/videos load.
- [ ] Generate a second take, switch to an older take, confirm `needs_review`.
- [ ] Assemble with and without “仅已通过”.
- [ ] Generate voices, confirm the V1 track, export a 1080×1920 MP4 and probe the
      H.264/AAC streams.
- [ ] Confirm the bundled FFmpeg path (`Contents/MacOS/ffmpeg`) is used.

## 4. Windows runtime gate (requires Windows)

- [ ] Build/install the NSIS or MSI artifact on Windows 10/11 x64.
- [ ] Launch with WebView2, create/open a project, and confirm images load.
- [ ] Generate system TTS with Chinese dialogue (zh-CN SAPI voice required).
- [ ] Export a timeline with source audio, music and voice; verify the 4.1 sidecar
      accepts the mix.
- [ ] Confirm no console windows flash while FFmpeg/PowerShell run.
- [ ] Record screenshots/logs in the release notes.

## 5. Artifact and publish

- [ ] Copy the DMG to `release/`, compute `shasum -a 256`, update
      `release/SHA256SUMS`.
- [ ] Tag `v<version>` and push the tag; the release workflow builds
      macOS arm64/x64 + Windows x64 and creates a **draft** GitHub release.
- [ ] Review the draft, attach checksums, note unsigned status.
- [ ] Publish only after the Windows runtime gate is green.
- [ ] If a gate fails, delete the draft release, fix, and bump the patch version;
      never replace a published artifact silently.

## 6. Rollback

- Keep the previous `release/SaladCart_<version>_aarch64.dmg` and its SHA.
- If a release is bad: unpublish the GitHub release, keep the tag for audit, publish
  a new patch version with the fix; do not rewrite the failed tag.
