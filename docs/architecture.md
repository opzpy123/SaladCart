# SaladCart Architecture

Status: accepted for v0.1, 2026-10-27. This document is the implementation contract for the initial release.

## 1. Product thesis

AI comic-drama creators lose most of their time moving between a chat window, image generator, video generator, asset folders, and an NLE. The outputs are inconsistent because every tool receives a different, partially remembered description of the same character and scene. SaladCart's core bet: **the project graph is the prompt compiler**. A creator maintains one project (story bible, characters, scenes, shots, assets) and every generation is automatically assembled from that graph, previewed before spending money, and versioned afterward.

The app is local-first. Projects and media live on disk. Cloud providers receive only the selected, previewed context.

## 2. Scope decision: editing is included

A one-stop workstation that stops at raw clips still forces creators into another tool for assembly, subtitles, and delivery. v0.1 therefore includes a **rough-cut editor**, not a full NLE:

- Auto-assemble all shot videos in script order.
- Reorder, trim, mute, replace clips; add subtitles and one music bed.
- Search licensed music (Jamendo) or generate with Stability Audio / ElevenLabs; analyze tempo/beat grid natively and snap cuts to beats.
- Preview in-app with an HTML video player.
- Export H.264/AAC MP4 with burned-in subtitles at vertical 9:16, horizontal 16:9, square 1:1, or 4:5, using FFmpeg.

Deliberately out of scope for v0.1: multi-track compositing, keyframes, transitions beyond cuts, color grading, advanced audio mixing, and collaborative editing. Those are follow-up work once the generation loop is proven.

## 3. Stack decision

| Layer | Choice | Why |
| --- | --- | --- |
| Desktop shell | **Tauri 2** | Small native binaries, system WebView (WKWebView/WebView2), Rust core for CPU/IO-heavy work, first-class macOS + Windows packaging. Native `NSWindow`/Win32 window under the hood. |
| UI | **React 19 + TypeScript + Vite** | Fast iteration for a dense workstation UI; no Electron runtime cost. |
| State | **Zustand** | Small, explicit stores; avoids heavy framework coupling. |
| Core | **Rust + Tokio** | Persistence, provider HTTP, job queue, ffprobe/FFmpeg orchestration, thumbnails, secure key storage. |
| Storage | **SQLite (rusqlite, bundled) per project + filesystem assets** | Project isolation, transactional edits, portable project folders, no server. WAL mode. |
| Secrets | **OS keychain** via `keyring` (macOS Keychain / Windows Credential Manager) | API keys never stored in the project DB or plaintext settings. |
| Media | **FFmpeg/ffprobe** (system or managed binary) | Reliable decode/encode, fast preview/export, cross-platform. |
| Tests | **cargo test**, **Vitest + Testing Library**, browser demo bridge | Backend logic tested against temp dirs and mocked HTTP; UI verified in Chromium via the demo bridge and in Tauri manually. |

Rejected: Electron (large runtime, weaker native story), Flutter (media timeline and desktop text editing would require more custom work; weaker web ecosystem for this UI), pure web app (no local project isolation, secret handling, or FFmpeg).

### Tauri tradeoffs and mitigations

- WebView differences: macOS WKWebView vs Windows WebView2. Keep to broadly supported CSS/JS; CI builds both; avoid OS-specific APIs in the renderer.
- Heavy media work stays out of the WebView. Rust drives FFmpeg and streams progress events.
- Tauri signing/notarization: CI prepares unsigned dev artifacts and documents signing hooks; local `pnpm build:tauri` produces an installable bundle.

## 4. System shape

```
┌──────────────────────────────────────────────────────────────────┐
│ React renderer (WKWebView / WebView2)                            │
│  Dashboard · Script · Characters · Shots · Editor · Settings      │
│  Zustand store · context preview · job stream                    │
└───────────────▲───────────────────────────┬──────────────────────┘
                │ events                     │ invoke (typed IPC)
┌───────────────┴───────────────────────────▼──────────────────────┐
│ Rust application core                                            │
│  Commands ── Services: script, context compiler, job runner,     │
│              media, assets, settings/secrets                     │
│  Provider traits: LLM · Image · Video ── adapters                │
│  Storage: project registry + one SQLite DB per project           │
└───────┬───────────────────┬───────────────────┬──────────────────┘
        │                   │                   │
  app data dir        OS keychain          remote APIs
  projects/<id>/      API keys             LLM/image/video
    project.db
    assets/…
    exports/…
```

### Project storage

```
~/Library/Application Support/com.saladcart.studio   (macOS)
%APPDATA%\com.saladcart.studio                       (Windows)
  settings.json          # non-secret preferences and provider profiles
  projects/
    <uuid>/
      project.db         # WAL SQLite; all project state
      project.json       # portable display metadata + schema version
      assets/images|videos|audio|novels/
      thumbs/
      exports/
  trash/                 # archived projects, never hard-deleted by UI
```

A project is self-contained and movable. Global settings contain no API keys. There is no cross-project DB; "reference from another project" is implemented as an explicit **copy/import** (character, style preset), not a live foreign key, preserving isolation.

## 5. Domain model

All IDs are UUIDs. Times are ISO-8601 UTC strings; durations are integer milliseconds. JSON crosses IPC in camelCase.

- **Project** — name, premise, logline, genre, tone, language, aspect ratio, target episode length, style bible (visual style, palette, lighting, camera language, negative prompt).
- **Character** — name, role, description, appearance, personality, voice, `lockedTraits[]`, negative prompt, reference asset, sheet asset, seed, status. The locked traits + reference image are injected into every shot that features the character.
- **Episode → Scene → Shot** — structured screenplay. Scene carries slugline/location/time/synopsis/script text. Shot carries description, prompt, negative prompt, camera, lens, duration, dialogue, sound, character IDs, continuity link, image/video assets, provider/model/seed, status, and version.
- **Asset** — image/video/audio/subtitle file with dimensions, duration, provenance (provider/model/prompt/seed), parent asset, version, status.
- **GenerationJob** — kind (script/image/video/export), target, provider/model, status, progress, stage, cost estimate/actual, error, result.
- **Timeline** — clips referencing shot + asset, in/out points, order, subtitle entries, format and fps.
- **Settings** — provider profiles, defaults, concurrency, FFmpeg path, storage location. Secrets are keychain refs only.

### Project graph integration rules

- Script generation creates characters and scenes/shots in one transaction, mapping character names to IDs.
- Character generation updates the character's reference asset; all shots using that character immediately see the new reference in their context preview.
- Shot video generation reads: project style bible + character locked traits/references + scene text + shot description/camera/dialogue + continuity (previous shot summary and optional last-frame asset) + target format.
- Assets are written under the project folder and registered transactionally. A failed file write never leaves a dangling asset row.
- Timeline auto-assembly selects each shot's current video in episode/scene/shot order; callers can request approved-only assembly. Older takes are retained in the `shot_takes` table and selectable from the shot inspector; selecting one resets the shot to `needs_review`.
- Image/video and music enqueue check the project budget cap against committed estimates/actuals before creating a job.
- Novel adaptation stores one `novel_source` row plus `assets/novels/novel.txt`, splits chapters (Chinese/English headings), chunks at 6,000 characters, merges characters by name and writes episodes/scenes/shots through the same script transaction. The novel job is capped at 120 chunks and 60 episodes.

## 6. Context compiler (the differentiator)

`services/context.rs` is a pure, testable function:

```
compile(project, characters, scene, shot, previousShot, previousLastFrame, settings) -> CompiledContext
```

It returns `systemPrompt`, `userPrompt`, `negativePrompt`, ordered character reference assets, style fields, warnings (missing reference, no locked traits, duration unsupported by provider, continuity missing), token estimate, and cost estimate. The UI shows this exact payload before generation ("what will be sent"). Both mock and real providers consume the same compiled context, so behavior is testable without network.

Provider adapters only map an already-compiled request into vendor fields. They do not silently add creative context.

## 7. Provider layer

Traits:

```rust
trait LlmProvider { async fn complete(&self, req: LlmRequest) -> Result<LlmResponse>; }
trait ImageProvider { async fn generate(&self, req: ImageRequest) -> Result<ProviderOutput>; }
trait VideoProvider { async fn submit(&self, req: VideoRequest) -> Result<ProviderTask>; async fn poll(&self, task) -> Result<ProviderTask>; }
```

- **LLM**: OpenAI-compatible chat completions (`response_format: json_object`) covers OpenAI, DeepSeek, Qwen/DashScope-compatible, Gemini's OpenAI compatibility, and Ollama. Mock LLM is deterministic and offline.
- **Image**: OpenAI Images, Gemini image generation, Replicate, ComfyUI local. Mock produces local placeholder art.
- **Video**: Kling, Luma, Runway, Replicate, MiniMax, Google Veo, fal.ai; Mock produces a local animated placeholder via FFmpeg.
- **Voice**: operating-system speech synthesis (macOS `say`, Windows SAPI) turns shot dialogue into a project audio asset; no cloud key required.
- **Music**: Jamendo licensed search; Stability Audio and ElevenLabs generation; Mock synthesizes a beat-gridded WAV. Beat analysis and auto-cut are pure Rust DSP over FFmpeg-decoded PCM, with licence/attribution stored on imported assets.
- All remote calls use timeouts, bounded retries for transient HTTP failures, structured errors, and redacted logs.
- API keys resolve from the OS keychain first, then environment variables for development. Keys are never returned to the renderer (`apiKeySet: true/false` only).
- Provider capability metadata declares supported aspect ratios, durations, and reference-image modes; the UI filters invalid options.

A generation job is persisted before the network call. Status transitions (`queued → running → succeeded/failed/canceled`) are streamed as `job:update` events and survive app restarts (running jobs are marked interrupted on boot and can be retried).

## 8. Editor and media pipeline

- `ffprobe` extracts dimensions/duration; thumbnails are generated into `thumbs/`.
- Auto-assembly builds clips from shot videos; trim/order/subtitles/voice links are stored in SQLite.
- Voice clips are anchored to shot starts and mixed with the music bed at export; AI-generation metadata and a provenance manifest are written alongside the MP4.
- Export builds an FFmpeg filter graph: scale/pad to target format, concat video, burn subtitles, mix music, encode H.264/AAC with faststart. Progress parses FFmpeg's `-progress pipe:1`. The muxer is capped to the visual timeline duration so infinite silent/music inputs cannot hang an export.
- Beat analysis decodes mono 22.05 kHz PCM, computes an onset envelope, autocorrelates tempo in the 60–200 BPM range, derives the phase, and persists a millisecond beat grid. Auto-cut snaps a cut to the nearest beat (±180 ms) with a two-beat minimum.
- If FFmpeg is missing, Settings shows an actionable setup state; generation flows still work; export is disabled with a clear message.
- FFmpeg is provisioned as a per-platform Tauri sidecar at build time (`pnpm prepare:ffmpeg`), with discovery order: explicit setting → env → bundled sidecar → PATH. `ffmpeg -i` parsing provides metadata when ffprobe is absent. Packaging CI verifies the bundled binary exposes `subtitles` and `libx264`.

## 9. Pain points addressed in v0.1

(Prioritized from research; see `docs/pain-points.md`.)

1. **Character drift across shots** → locked traits + reference sheet + automatic reference injection + continuity frames.
2. **Fragmented tools and copy-paste** → single project graph; automatic shot breakdown; assets tied to shots.
3. **Prompt rewriting per provider** → context compiler + provider adapters; preview before spend.
4. **Uncontrolled cost and failed jobs** → per-job cost estimates, queue, retries, version history, provider fallback.
5. **Storyboard-to-video tedium** → batch shot generation + auto-assembly timeline.
6. **Delivery friction** → subtitle editor, aspect presets, one-click MP4 export.
7. **Project chaos** → isolated project folders, trash-based removal, portable bundles, character/style import.
8. **Local/private workflows** → Ollama + ComfyUI adapters, local-first storage, OS keychain.

## 10. Phased delivery and gates

| Phase | Deliverable | Gate |
| --- | --- | --- |
| P0 | Spec, scaffold, toolchain | app boots; typecheck passes |
| P1 | Project store, settings, secrets, asset library | Rust tests on temp projects; isolation test |
| P2 | Context compiler, mock + remote provider adapters, script pipeline, job queue | unit tests for compiler/adapters; mock end-to-end script job |
| P3 | UI shell, dashboard, Script Studio, Character Studio | UI smoke via demo bridge; create/edit/generate |
| P4 | Video Studio, batch jobs, continuity, costs | mock video end-to-end; context preview matches job |
| P5 | Timeline, subtitles, FFmpeg export | export a real MP4 locally; verify with ffprobe |
| P6 | Packaging, CI matrix, docs, final audit | macOS bundle built; Windows workflow present; tests green |

Each phase ends with automated checks and a self-review. Multi-agent work is split by file ownership to avoid concurrent edits.

## 11. Security and privacy

- Cloud credentials in OS keychain; redacted provider errors; no keys in logs or project files.
- Renderer never receives filesystem write primitives; file operations are narrow Rust commands restricted to project/app directories.
- Asset protocol scoped to app data.
- Imported files are copied into the project; source paths are not executed.
- No analytics or telemetry in v0.1.
- Destructive project removal moves the folder into `trash/`; permanent deletion is manual.

## 12. Non-goals for v0.1

- Real-time collaborative editing, mobile apps, Linux packaging.
- Model training/fine-tuning.
- Built-in lip sync or voice cloning (provider integrations can be added once the shot pipeline is stable).
- Full NLE features (multi-track, transitions, grading).
- Automatic publishing to social platforms.
