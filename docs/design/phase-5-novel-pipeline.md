# Phase 5 — Long-novel adaptation backend pipeline

## Problem

The existing script generator starts from a short premise. A full novel needs chapter
ingestion, bounded LLM calls, character merge across chapters, and a deterministic mapping
from analysis to episodes/scenes/shots so the result is production-ready and resumable.

## Scope

- Store one novel source per project (title, UTF-8 text, source path, counts, status,
  analysis).
- Split chapters by Chinese/English headings, then chunk each chapter to ≤6,000 chars.
- Adapt every chunk with the LLM (or a deterministic mock) and merge characters by name.
- Group chapters into at most 60 episodes; each analyzed beat becomes a scene with one
  3–10 second shot; prompts inherit the project style bible.
- Expose async job kind `novel` with per-chunk progress, cost estimate, budget guard and
  a result summary.
- Commands: `preview_novel`, `get_novel`, `start_novel_adaptation`, `read_novel_file`.

## Non-goals

- Multiple simultaneous novels per project, automatic translation, or publishing.
- LLM fine-tuning; very large novels over 120 chunks must be split into volumes.
- Real-provider cost reconciliation (the estimate uses optional
  `input_usd_per_1k`/`output_usd_per_1k` provider options).

## Data (schema v8)

```sql
CREATE TABLE novel_source (
  id TEXT PRIMARY KEY,          -- 'current'
  title TEXT NOT NULL,
  text TEXT NOT NULL,
  source_path TEXT,
  char_count INTEGER NOT NULL,
  chapter_count INTEGER NOT NULL,
  chunk_count INTEGER NOT NULL,
  status TEXT NOT NULL,         -- ready | adapted
  analysis_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

The text is also copied to `assets/novels/novel.txt`; `ProjectSnapshot.novel` returns the
summary without the full text.

## Pipeline

1. `preview_novel` validates ≥50 chars and returns characters/chapters/chunks/calls.
2. `start_novel_adaptation` validates provider/budget, saves the source, creates a `novel`
   job.
3. `run_novel` splits chapters, chunks to 6,000 chars, and calls the provider once per
   chunk with the previous summary as continuity context.
4. Each chunk returns `summary`, `characters[]`, `beats[]`; characters are merged
   (union locked traits, max 8).
5. Chapters are grouped into episodes (`maxEpisodes` 0 = one per chapter, capped 60);
   beats become scenes/shots; empty chapters are skipped.
6. The generated script is applied through `apply_generated_script` and the novel status
   becomes `adapted` with chapter summaries in `analysis_json`.

Mock providers use the existing deterministic script template per chapter, so the full
pipeline runs offline.

## Verification record

- `cargo test`: 29 automated Rust tests passed (10 unit incl. chapter/chunk/plan, 2 media,
  3 provider contracts, 14 store incl. novel adaptation and grouping); 1 ignored TTS.
- `cargo check` passes; no schema migration errors on v8.
- Phase artifact and UI walkthrough are delivered in Phase 6.
