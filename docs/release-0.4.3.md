# SaladCart 0.4.3 — release notes

Release date: 2026-10-29
Source: `main` at build time (real provider routing + Volcengine providers)
Status: **unsigned macOS prerelease**. Windows packaging intentionally skipped.

## Artifact

| Platform | File | Size | SHA-256 |
| --- | --- | --- | --- |
| macOS 11+ arm64 | `release/SaladCart_0.4.3_aarch64.dmg` | 21.87 MiB | `c23416a6d31046009e6286b8aff50842e3c6774508a286f7799bae1187d8e1d3` |

The DMG is **not signed or notarized**. On first launch use right-click → **Open**, or
System Settings → Privacy & Security → **Open Anyway**.

## Changes since 0.4.2

- **Real provider routing**: configured real LLMs are preferred over the offline mock;
  the New Project dialog has a 编剧模型 selector, and the Script dialog only offers
  enabled providers. Saving an LLM key promotes it when the default is still mock.
- **Dashboard truthfulness**: the hard-coded 沈知遥 / `MOCK` context card was replaced by
  the current project's first character, next shot, aspect/duration and default model.
- **Volcengine Ark providers**:
  - Image: Seedream 5.0 Flash (`doubao-seedream-5-0-flash-260915`) via
    `POST /api/v3/images/generations`; 2K sizes, reference images, url/b64 responses.
  - Video: Seedance 2.0 Mini (`doubao-seedance-2-0-mini-260615`) via
    `POST /api/v3/contents/generations/tasks` + task polling; text/image inputs.
  - Music: GenBGM v5.0 via the SAMI OpenAPI with Volcengine HMAC-SHA256 signing
    (`sami` / `cn-north-1`), GenBGM + QueryTask, `ResultUrl` download.
- Ark model-not-activated 404s now surface a Chinese activation hint instead of the raw
  provider string. GenBGM’s key field explains that it needs `AccessKey:SecretKey`.
- Updated approximate pricing estimates for Ark Flash/Mini/GenBGM.
- Version bumped to 0.4.3 across package, Cargo, Tauri config, mock bridge and Settings.

## Verification

- `pnpm test` — **10/10** frontend tests passed (provider routing included).
- `cargo test` — **32 automated Rust tests passed**, 2 ignored/manual tests
  (macOS TTS, live DeepSeek credits).
- Live DeepSeek: script generation completed in 16.8s and returned a structured 3-act
  episode (`凌晨四点的失物便利店`); the user project was regenerated with 3 episodes /
  6 scenes / 18 shots and 3 real characters.
- Ark live probe: request reached the Ark API and authenticated; the account returned
  `not activated` for `doubao-seedream-5-0-flash-260915`. No image was produced until the
  model is activated in the Ark console.
- GenBGM live attempt: the saved key is not an IAM `AccessKey:SecretKey`, so the adapter
  returned a clear missing-credential error; contract tests cover signing, submit and
  polling.
- `pnpm build` and `pnpm tauri build` passed; `.app` (46.87 MiB) and DMG (21.87 MiB)
  built. Native 0.4.3 dashboard verified with the dynamic context card and `v0.4.3`.

## Requirements and caveats

- Activate **Seedream 5.0 Flash** and **Seedance 2.0 Mini** in the Volcengine Ark console
  before generating; activation is per model.
- GenBGM is not the Ark API: create an IAM AccessKey/SecretKey and paste it as
  `AccessKey:SecretKey` in Settings → 音乐 → 火山引擎 纯音乐.
- The project owner regenerated the previously mock-filled project; the demo project
  remains separate and can be archived without affecting user projects.
- macOS artifact is unsigned; Windows packaging is skipped by request.
- No public GitHub Release was created (no `gh` authentication in this workspace).

See [release-0.4.2.md](release-0.4.2.md) for the prior icon release.
