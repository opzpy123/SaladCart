# Provider Setup

SaladCart stores every API key in the OS keychain (macOS Keychain / Windows Credential Manager). Keys are never written to `settings.json`, project databases, logs, or the renderer. `apiKeySet` only reports presence.

> The built-in **Mock** providers work offline and cost nothing: LLM story generation, placeholder character/keyframe art, FFmpeg-rendered placeholder clips, and a synthesized 100 BPM instrumental score. Use them to validate the full pipeline before adding keys.

## LLM (script)

All LLM adapters use the OpenAI-compatible `POST /chat/completions` shape with JSON object output.

| Provider | Base URL | Example model | Key |
| --- | --- | --- | --- |
| Mock | local | `saladcart-mock-llm` | none |
| OpenAI / compatible | `https://api.openai.com/v1` | `gpt-4.1-mini` | `OPENAI_API_KEY` |
| DeepSeek | `https://api.deepseek.com/v1` | `deepseek-chat` | `DEEPSEEK_API_KEY` |
| Qwen / DashScope | `https://dashscope.aliyuncs.com/compatible-mode/v1` | `qwen-plus` | `DASHSCOPE_API_KEY` |
| Ollama | `http://127.0.0.1:11434/v1` | `qwen2.5:7b` | none |

Qwen keys are region-bound. If a compatible endpoint rejects `response_format`, the adapter retries once without it.

## Image

| Provider | Notes |
| --- | --- |
| Mock | Local generated PNG key art, zero cost. |
| OpenAI Images | Uses `/images/generations` or multipart `/images/edits` when references exist. Sizes are mapped to 1024×1536 / 1536×1024 / 1024×1024. |
| Gemini Image | `generateContent` with `responseModalities: TEXT+IMAGE`, optional `imageConfig.aspectRatio`; supports inline reference images. Imagen models use `:predict`. |
| Replicate | Creates a prediction on `/models/{owner}/{name}/predictions`, polls, downloads the first output. Model-specific inputs vary. |
| ComfyUI | Local `/prompt` + `/history` + `/view`; expects an API-format workflow. Custom workflows can use `{{PROMPT}}`, `{{NEGATIVE}}`, `{{SEED}}`, `{{WIDTH}}`, `{{HEIGHT}}`, `{{REF_IMAGE}}`. |
| Stability | Stable Image API with `Accept: image/*`; text prompt + aspect ratio. Reference images are not passed by this basic adapter. |
| fal.ai | Queue API; text prompt + aspect ratio; polls status/response URLs. |
| 火山方舟 Seedream | `https://ark.cn-beijing.volces.com/api/v3` + `POST /images/generations`; Bearer `ARK_API_KEY`. Default `doubao-seedream-5-0-flash-260915`; references are sent as data URIs; accepts `url` or `b64_json` responses. |

## Video

| Provider | Base | Notes |
| --- | --- | --- |
| Mock | local | FFmpeg placeholder clip, zero cost. |
| Kling | `https://api-singapore.klingai.com` | New API keys use Bearer; AccessKey:SecretKey can be pasted for legacy HS256 JWT. Default paths are legacy `/v1/videos/{text2video,image2video}`; override `submitPathText`, `submitPathImage`, `pollPath`, `imageField` in provider options if your account uses the 2026 task API. |
| Runway | `https://api.dev.runwayml.com` | Requires `X-Runway-Version: 2024-11-06`. Image input is a data URI. |
| Luma | `https://api.lumalabs.ai` | Dream Machine generations endpoint; start frame via `keyframes.frame0` data URI. |
| MiniMax | `https://api.minimax.io` | Add `groupId` to provider options if your account requires it. Create → query → retrieve file. |
| Veo | `https://generativelanguage.googleapis.com/v1beta` | Long-running operation polling. Download uses the Gemini key. Preview model names change; edit the default model in Settings. |
| Replicate | `https://api.replicate.com/v1` | Model-specific input field for the start frame (`start_image`, `first_frame_image`, `image`, detected from the model name). |
| fal.ai | `https://queue.fal.run` | Queue submit → status → response. `Authorization: Key <FAL_KEY>`. |
| 火山方舟 Seedance | `https://ark.cn-beijing.volces.com/api/v3` | `POST /contents/generations/tasks`, poll `GET /contents/generations/tasks/{id}`; Bearer `ARK_API_KEY`. Default `doubao-seedance-1-0-pro-fast-251015`; content uses `text` + `image_url` (start frame/reference), plus `ratio`, `duration`, `seed`. |

Ark image/video models must be activated per model in the Volcengine Ark console before first use; a 404 `not activated` response is surfaced as a prompt to activate the model. GenBGM is a separate SAMI OpenAPI and needs an IAM `AccessKey:SecretKey`, not the Ark API key.

Remote video APIs change often. Every adapter surfaces provider error bodies and stores the raw provider task on the job. If a vendor changes an endpoint, use the provider overrides where available or update the adapter. No fake success is returned when a provider is unreachable.

## Music

### Search

Jamendo only returns tracks with `audiodownload_allowed=true`. Each track stores its Creative Commons URL, attribution, and source URL on import. Jamendo's free API tier is for non-commercial use; commercial products need a Jamendo licence.

### AI generation

| Provider | Endpoint | Notes |
| --- | --- | --- |
| Mock | local | Synthesizes a 100 BPM beat bed; ideal for testing auto-cut. |
| Stability Audio | `/v2beta/audio/stable-audio/text-to-audio` | Async submit + poll. Instrumental. Commercial terms depend on the Stability licence/revenue threshold. |
| ElevenLabs Music | `/v1/music` | `xi-api-key`, `music_length_ms`, `force_instrumental`; returns MP3. Commercial use depends on plan. |
| 火山引擎 纯音乐 | `https://open.volcengineapi.com` | GenBGM `Version=2024-08-12`, body `Text` + `Duration` + `Version=v5.0`; QueryTask polls `ResultUrl`. Paste `AccessKey:SecretKey` (not the Ark API key) into the key field; signed with Volcengine HMAC-SHA256 (`sami` / `cn-north-1`). |

Suno/Udio are intentionally not integrated through unofficial wrappers.

### Beat analysis & auto-cut

1. Select/import a track and apply it to the timeline.
2. Click **分析节拍**. FFmpeg decodes mono 22.05 kHz PCM; Rust computes an onset envelope, tempo via autocorrelation, and the beat phase. The beat grid is stored in the project DB.
3. Click **自动卡点**. Cut points snap to the nearest beat within 180 ms, with at least a two-beat minimum shot.
4. Export mixes the track (looped) with the visual timeline.

This is a lightweight DSP tracker, not a downbeat-aware ML model. It is fast, cross-platform, and offline. Beat This can be added later as an optional higher-accuracy pass.

**Licensing:** CC-BY requires credit; CC-BY-NC forbids monetised exports; CC-BY-ND may forbid beat-cut adaptations. Keep the licence/provenance metadata that SaladCart stores, and check AI provider terms before publishing.

## FFmpeg

Packaged builds include a platform-specific FFmpeg sidecar next to the application executable (4.4 on macOS, 4.1 on Windows x64). Development builds prepare it with `pnpm prepare:ffmpeg`. The exporter avoids 4.4-only filter options so voice, music and source audio mix on the Windows 4.1 build as well.

Discovery order:

1. Settings → FFmpeg path.
2. `SALADCART_FFMPEG` environment variable.
3. Bundled sidecar next to the executable (or in `Contents/Resources`).
4. `ffmpeg` in `PATH` and common install locations (`/opt/homebrew/bin`, `/usr/local/bin`, `C:\ffmpeg\bin`).

ffprobe is optional; when absent, SaladCart parses `ffmpeg -i` output for duration and video dimensions. The app gracefully disables video mock generation and export only if no FFmpeg is found; script and image generation still work. See [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) for FFmpeg licensing/source.

## Troubleshooting

- **401/403**: key invalid, region-bound, or missing account access. Use Settings → “检测” (only checks presence) then run a real generation.
- **Kling errors**: verify region/base URL and API generation. Override paths in provider options.
- **MiniMax**: set `groupId` under provider options.
- **ComfyUI**: export the workflow in API format, not the UI graph; bind it to 127.0.0.1 and configure the checkpoint.
- **Empty/blocked image**: Gemini may refuse content or the model may not support image output; try another model/account.
- **Export stuck**: SaladCart now caps FFmpeg to the timeline duration; if a provider asset is corrupt, the job fails with FFmpeg's stderr tail.
