# Phase 6 — Novel adaptation workspace

## Scope

- Script Studio entry "小说改编" opens a wide dialog.
- Paste text or import `.txt/.md/.text`: desktop uses the native file dialog and
  `read_novel_file`; the browser demo uses a file input and `File.text()`.
- Live plan (chars/chapters/chunks/estimated calls) via `preview_novel`, debounced.
- Options: LLM provider, max episodes (0 = one per chapter, capped 60), style notes,
  replace existing script (default on).
- On submit: save novel + enqueue `novel` job, show job dock, refresh snapshot.
- Novel source badge in Script Studio: title, chapter count, characters.
- Browser mock simulates chapter splitting and produces one episode per chapter.

## UX rules

- At least 50 characters; plan must resolve before submit enables.
- Missing title falls back to the first line or chapter title.
- The dialog states the 6,000-character chunking and 120-chunk limit.
- Job progress uses the existing job dock with a "小说改编" label; success toast reports
  episodes and shots.

## Verification record

- Browser: 3-chapter novel → plan 3 chapters / 3 chunks / 3 calls → job completes →
  badge "改编自《雨夜钥匙》" and 3 episodes / 12 shots.
- Native macOS v0.3.0: 3-chapter novel → 3 episodes / 9 shots; job result "已完成";
  schema v8 source persisted and displayed in the header.
- `cargo test`: 29 automated tests; `pnpm test`: 6/6; TypeScript and production build pass.
- Artifact: `release/SaladCart_0.3.0_aarch64.dmg`
  (SHA-256 `37b0420fea0040a80adc8cc45f2d551911646a07f51dad1fc7637099b165932c`).
