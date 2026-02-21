# Diag AI-agent debugging v1 (fearless refactor)

Status: Draft / in progress

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
  - snapshots per window: `(snapshot_seq, frame_id, unix_ms, semantics_fingerprint, screenshot_refs, event_ranges, ...)`
  - optional: test-id presence bloom/sets per snapshot (bounded / hashed)

Deliver an “AI packet” command that writes a directory like:

- `script.result.json` (or equivalent failure summary)
- `bundle.meta.json`
- `bundle.index.json`
- one or more slices (e.g. `slice.test_id.*.json`, `slice.viewport.*.json`)
- optional: referenced screenshots (bounded)

This keeps the default agent workflow “small by default” and avoids full bundle reads.

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

