# Diag AI-agent debugging v1 (fearless refactor)

Status: Draft / in progress

Current state (as of 2026-02-21):

- Tooling can generate `bundle.index.json` (schema v1) via `fretboard diag index <bundle_dir|bundle.json>`.
- Tooling can export a bounded “AI packet” directory via `fretboard diag ai-packet ...`.
- Index and packet writers live in:
  - `crates/fret-diag/src/bundle_index.rs`
  - `crates/fret-diag/src/commands/index.rs`
  - `crates/fret-diag/src/commands/ai_packet.rs`
- Slice bounded-parse fast-path lives in:
  - `crates/fret-diag/src/commands/slice_streaming.rs`
  - `crates/fret-diag/src/commands/slice_payload.rs`

## Motivation

`bundle.json` is a great “single file artifact”, but it can grow large enough that it becomes:

- slow to open/parse/search (humans + tools),
- expensive to ship over DevTools WS or store in CI artifacts,
- hard for AI agents to consume within bounded context budgets.

We want **bounded, indexable, sliceable evidence** with clear “what matters for this failure” defaults.

This workstream is intentionally “fearless refactor” friendly: we prefer additive schema and shims over breaking changes.

## Goals

1. **AI-friendly minimal packets**: produce a small, self-contained subset that is usually enough to debug a failure.
2. **Fast slicing**: make “find the right frame/snapshot” cheap without parsing the whole bundle repeatedly.
3. **Explicit indexing**: add stable indexes for frames/snapshots/events so tools can jump directly.
4. **Progressive disclosure**: keep a shareable manifest + optional heavier payloads (semantics, screenshots, logs).
5. **Compatibility**: keep v1/v2 bundles readable; new layouts should degrade to “materialize bundle.json” when needed.

## Non-goals (initially)

- Replace JSON with a DB.
- Full GPU capture/replay.
- A new scripting language (we keep JSON v1/v2 scripts).

## Proposed direction

### Phase 1 (preferred first): indexing + minimal packets

Add/standardize a small set of structured summaries that tools (and agents) can rely on:

- `bundle.meta.json`: bounded summary (counts, sizes, time range, windows, clipping flags).
- `bundle.index.json`: fast jump tables:
  - windows list + stable window ids
  - snapshots per window: `(window_snapshot_seq, frame_id, timestamp_unix_ms, semantics_fingerprint, semantics_source, has_semantics, ...)`
  - optional: test-id presence bloom/sets per snapshot (bounded / hashed)

Deliver an “AI packet” command that writes a directory like:

- `script.result.json` (or equivalent failure summary)
- `bundle.meta.json`
- `bundle.index.json`
- `test_ids.index.json`
- one or more slices (e.g. `slice.test_id.*.json`, `slice.viewport.*.json`)
- optional: referenced screenshots (bounded)

This keeps the default agent workflow “small by default” and avoids full bundle reads.

What ships now (Phase 1 subset):

- `bundle.index.json` schema v1 currently records per-snapshot:
  - `window_snapshot_seq` (when present)
  - `frame_id`, `timestamp_unix_ms`, `is_warmup`
  - `semantics_fingerprint`
  - `semantics_source` = `inline|table|none` (inline semantics vs v2 table-resolved vs missing)
  - `has_semantics` (resolved)
  - optional `test_id_bloom_hex` (tail snapshots only; inline semantics only): a small Bloom filter hint for test-id membership
- `diag pack --include-root-artifacts` and `diag pack --include-triage` include sidecars under `_root/`:
  - `bundle.meta.json`
  - `bundle.index.json`
  - `test_ids.index.json`
  - `test_ids.json` (human-facing; may be deprecated later)
- The sidecars are usable on their own (no `bundle.json`) for common “AI packet” loops:
  - `fretboard diag meta <packet_dir|bundle.meta.json> --meta-report`
  - `fretboard diag query test-id <packet_dir|test_ids.index.json> <pattern>`
  - `fretboard diag query snapshots <packet_dir|bundle.index.json> [--test-id <id>]`
  - `fretboard diag slice <packet_dir> --test-id <id>` (uses precomputed slice if present)
- For large bundles, `diag slice` attempts a bounded parse first when an explicit snapshot selector is provided
  (`--frame-id`/`--snapshot-seq`), so it can avoid building the full in-memory `serde_json::Value` for `bundle.json`.
- When no explicit selector is provided, `diag slice` uses `bundle.index.json` (when present) to pick a reasonable default
  snapshot for the bounded-parse attempt (last non-warmup snapshot with resolved semantics in the first window, or the best fallback).
  - If `test_id_bloom_hex` exists, `diag slice --test-id X` prefers a snapshot whose bloom filter may contain `X`.
    This is a hint (false positives are allowed); it is used to reduce the frequency of falling back to full bundle parsing.

Known gaps (still planned):

- `diag query` does not yet prefer `bundle.index.json` for fast-path selection (it still parses `bundle.json` when computing new outputs).
- `diag slice` uses `bundle.index.json` for validation and as a default snapshot hint, but it still falls back to parsing `bundle.json`
  when it needs to find a better snapshot that contains the requested test-id (since per-snapshot test-id presence is not indexed yet).
- “Test-id presence per snapshot” is not yet indexed; finding “first snapshot that contains X” still requires semantics reads.

### Phase 2: on-disk layout (manifest + chunked payloads)

Keep `bundle.json` as a materialized compatibility surface, but treat it as a derived artifact:

- store snapshots as `*.ndjson` (optionally `zstd`)
- store large tables (semantics, logs) as chunked files
- store a typed manifest with file list + hashes + sizes

Tools prefer the manifest + indexes and only materialize/parse `bundle.json` when needed.

## Related workstreams / dependencies

- Semantics dedup: `docs/workstreams/diag-bundle-schema-v2.md`
- Overall diag simplification: `docs/workstreams/diag-simplification-v1.md`
- DevTools GUI + agent hooks: `docs/workstreams/diag-devtools-gui-v1-ai-mcp.md`

## Tracking

See:

- `docs/workstreams/diag-ai-agent-debugging-v1-todo.md`
- `docs/workstreams/diag-ai-agent-debugging-v1-milestones.md`
