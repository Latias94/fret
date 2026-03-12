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
- `anchors.json` (optional): resolved selectors for failure anchors (e.g. failed `step_index` → `window` + `frame_id`/`window_snapshot_seq`)
- `bundle.meta.json`
- `bundle.index.json`
- `test_ids.index.json`
- one or more slices (e.g. `slice.test_id.*.json`, `slice.failed_step.*.json`, `slice.viewport.*.json`)
- optional: referenced screenshots (bounded)

This keeps the default agent workflow “small by default” and avoids full bundle reads.

What ships now (Phase 1 subset):

- `bundle.index.json` schema v1 currently records per-snapshot:
  - `window_snapshot_seq` (when present)
  - `frame_id`, `timestamp_unix_ms`, `is_warmup`
  - `semantics_fingerprint`
  - `semantics_source` = `inline|table|none` (inline semantics vs v2 table-resolved vs missing)
  - `has_semantics` (resolved)
  - optional `test_id_bloom_hex` (tail snapshots only; resolved semantics only): a small Bloom filter hint for test-id membership
- `bundle.index.json` schema v1 also includes an optional per-semantics-key bloom index:
  - `semantics_blooms` keyed by `(window, semantics_fingerprint, semantics_source)` to provide broader test-id membership hints
    without storing a bloom per snapshot.
- When `bundle.index.json` is generated next to a `script.result.json` (run-id artifact layout), it may include an optional `script`
  section (schema v1):
  - `script.steps[]` maps `step_index` to a concrete snapshot selector (`window`, `frame_id`/`window_snapshot_seq`), allowing tools/agents
    to jump directly to the relevant snapshot without scanning the full bundle.
- `diag pack --include-root-artifacts` and `diag pack --include-triage` include sidecars under `_root/`:
  - `bundle.meta.json`
  - `bundle.index.json`
  - `test_ids.index.json`
  - `test_ids.json` (human-facing; may be deprecated later)
  - `frames.index.json` (for `triage --lite` / `hotspots --lite` workflows)
- The sidecars are usable on their own (no bundle artifact) for common “AI packet” loops:
  - `fretboard diag meta <packet_dir|bundle.meta.json> --meta-report`
  - `fretboard diag query test-id <packet_dir|test_ids.index.json> <pattern>`
  - `fretboard diag query snapshots <packet_dir|bundle.index.json> [--test-id <id>]`
  - `fretboard diag query snapshots <packet_dir|bundle.index.json> --step-index <n>`
  - `fretboard diag slice <packet_dir> --test-id <id>` (uses precomputed slice if present)
  - `fretboard diag slice <packet_dir|bundle.json> --step-index <n> --test-id <id>` (selects the snapshot nearest step `n`)
- For large bundles, `diag slice` attempts a bounded parse first when an explicit snapshot selector is provided
  (`--frame-id`/`--snapshot-seq`), so it can avoid building the full in-memory `serde_json::Value` for `bundle.json`.
  - Supports both v1 inline semantics (`debug.semantics.nodes`) and v2 table semantics (`tables.semantics.entries`).
  - When no explicit selector is provided, `diag slice` uses `bundle.index.json` (when present) to pick a reasonable default
    snapshot for the bounded-parse attempt (last non-warmup snapshot with resolved semantics in the first window, or the best fallback).
    - If `test_id_bloom_hex` (snapshot) or `semantics_blooms` (per-semantics key) exists, `diag slice --test-id X` prefers a snapshot
      whose bloom filter may contain `X`.
      This is a hint (false positives are allowed); it is used to reduce the frequency of falling back to full bundle parsing.

Known gaps (still planned):

- `diag query` still has large surface area that does not yet use `bundle.index.json` (only `query snapshots` is index-first today).
- `diag slice` uses `bundle.index.json` for validation and as a default snapshot hint, but it still falls back to parsing `bundle.json`
  when it needs to find a better snapshot that contains the requested test-id.
  - `bundle.index.json` v1 includes:
    - `test_id_bloom_hex` for some tail snapshots, and
    - `semantics_blooms` for a bounded set of recent `(window, semantics_fingerprint, semantics_source)` keys.
  These help reduce full-bundle fallbacks, but are still hints and remain bounded.
- “Test-id presence per snapshot” is still not guaranteed to be indexed for every snapshot in very large bundles; finding “first snapshot
  that contains X” can still require streaming semantics reads and may occasionally fall back to parsing the full bundle JSON.

### Phase 2: on-disk layout (manifest + chunked payloads)

Keep `bundle.json` as a materialized compatibility surface, but treat it as a derived artifact:

- store snapshots as `*.ndjson` (optionally `zstd`)
- store large tables (semantics, logs) as chunked files
- store a typed manifest with file list + hashes + sizes

Tools prefer the manifest + indexes and only materialize/parse `bundle.json` when needed.

## Schema evolution and deprecations

The default direction is additive + compat-first:

- `bundle.json`:
  - v1: inline per-snapshot semantics (`debug.semantics.nodes`).
  - v2: semantics table (`tables.semantics.entries`) + snapshots reference by `semantics_fingerprint`.
  - Tracking: `docs/workstreams/diag-bundle-schema-v2/diag-bundle-schema-v2.md`.
- `bundle.index.json` (sidecar):
  - schema v1: per-snapshot jump table + bounded membership hints (`test_id_bloom_hex`, `semantics_blooms`).
  - Future: schema v2 can add event markers / step ids without breaking v1 readers (ship as additive fields first).
- Deprecation candidates (not removed yet):
  - `test_ids.json` is human-facing and may be replaced by `test_ids.index.json` + query tooling once parity is proven.
  - “Open the full `bundle.json` and grep” workflows should be replaced by `diag meta/query/slice` (bounded, cacheable, shareable).

## Related workstreams / dependencies

- Semantics dedup: `docs/workstreams/diag-bundle-schema-v2/diag-bundle-schema-v2.md`
- Overall diag simplification: `docs/workstreams/diag-simplification-v1/diag-simplification-v1.md`
- DevTools GUI + agent hooks: `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md`

## Tracking

See:

- `docs/workstreams/diag-ai-agent-debugging-v1/diag-ai-agent-debugging-v1-todo.md`
- `docs/workstreams/diag-ai-agent-debugging-v1/diag-ai-agent-debugging-v1-milestones.md`

## Inventory tooling (M0)

To measure where bytes go in representative bundles, use:

- `fretboard diag hotspots <bundle_dir|bundle.json> --hotspots-top 30 --max-depth 7 --min-bytes 4096`

This produces an approximate, whitespace-free JSON size estimate per aggregated path (arrays use `[]` wildcards), which is good
enough to identify the biggest subtrees (snapshots, semantics tables, logs, etc.) and to drive budget decisions.

To generate a local schema-v2 baseline from an existing v1 bundle (for measurement and comparison), use:

- `fretboard diag bundle-v2 <bundle_dir|bundle.json> --mode last --out <bundle.schema2.last.json>`
- `fretboard diag bundle-v2 <bundle_dir|bundle.json> --mode changed --out <bundle.schema2.changed.json>`

### Hot spot inventory (local samples, 2026-02-21)

These runs were measured on local `schema_version=1` bundles under `.fret/diag/`:

- `fretboard diag hotspots .fret/diag/1770260419048-ui-gallery-avatar/bundle.json --hotspots-top 40 --max-depth 7 --min-bytes 4096`
- `fretboard diag hotspots .fret/diag/1770260415986-script-step-0027-click/bundle.json --hotspots-top 40 --max-depth 7 --min-bytes 4096`

Top contributors (approx minified bytes; sums are aggregated across all snapshots):

- `ui-gallery-avatar` (~87.3 MiB file; ~38.0 MiB estimated minified):
  - `$.windows[].snapshots[].debug.semantics.nodes`: ~26.6 MiB
  - `$.windows[].snapshots[].debug.command_gating_trace`: ~6.1 MiB
  - `$.windows[].snapshots[].debug.removed_subtrees`: ~1.9 MiB
- `script-step-0027-click` (~38.5 MiB file; ~17.0 MiB estimated minified):
  - `$.windows[].snapshots[].debug.semantics.nodes`: ~11.8 MiB
  - `$.windows[].snapshots[].debug.command_gating_trace`: ~2.1 MiB
  - `$.windows[].snapshots[].debug.removed_subtrees`: ~1.7 MiB

Implication for AI packets:

- Raw per-snapshot `debug.semantics.nodes` dominates bundle size, so default agent workflows should avoid shipping full semantics for
  many snapshots. Prefer index-first selection + small targeted slices, and rely on bounded membership hints for test-id search.

### Hot spot inventory (schema v2, converted samples, 2026-02-21)

Converted the same v1 bundles with `fretboard diag bundle-v2` and re-ran hotspots.

Key observation: v2 moves the semantics payload to `tables.semantics.entries[].semantics.nodes` (dedup by `(window, semantics_fingerprint)`),
so “table semantics” becomes the dominant hot spot when inline semantics are stripped.

- `ui-gallery-avatar`:
  - v1 estimated minified: ~38.0 MiB (inline semantics repeated across snapshots)
  - v2 `--mode last` estimated minified: ~20.9 MiB
    - `$.tables.semantics.entries[].semantics.nodes`: ~9.3 MiB
    - `$.windows[].snapshots[].debug`: ~11.2 MiB (mostly non-semantics debug)
- `script-step-0027-click`:
  - v1 estimated minified: ~17.0 MiB
  - v2 `--mode last` estimated minified: ~13.5 MiB
    - `$.tables.semantics.entries[].semantics.nodes`: ~8.1 MiB
    - `$.windows[].snapshots[].debug`: ~5.2 MiB

Implication for schema v2:

- v2 helps reduce repeated per-snapshot semantics, but the semantics payload still exists (now as a table).
- AI packets should continue to avoid shipping raw semantics tables by default; ship `bundle.index.json` + targeted `slice.*.json` instead.

### AI packet budgets (enforced by tooling)

The intent is to keep the common case “small by default”, while allowing opt-in escalation.
Tooling enforces these budgets during `fretboard diag ai-packet` by clipping or dropping optional files when necessary, and writes
an `ai.packet.json` report into the output directory (for auditability).
When `script.result.json.stage=failed`, the report may also include a `failed_step_slices` section that summarizes which anchored
`slice.failed_step.*.json` files were written (or why they were skipped).

- Default AI packet total: <= 2 MiB
- Max AI packet total (before clipping/escalation): <= 20 MiB
- Per-file (guidelines):
  - `bundle.meta.json`: <= 128 KiB
  - `bundle.index.json`: <= 4 MiB (depends on snapshot count; should remain bounded)
  - `test_ids.index.json`: <= 2 MiB (bounded by `--max-test-ids`)
  - each `slice.*.json`: <= 2 MiB (bounded by `--max-matches`, `--max-ancestors`, and string clipping)

Measured sizes (local samples; 2026-02-21):

- `script-step-0027-click` packet (test-id `ui-gallery-command-palette`): ~46 KiB total
- `ui-gallery-avatar` packet (test-id `ui-gallery-nav-search`): ~83 KiB total

Next: align all budget overruns to a stable `reason_code` taxonomy and make the clipping behavior more explicit for agents (e.g. a
single “packet manifest” that lists which files were clipped/dropped and why).

Current behavior (as of 2026-02-21):

- `diag ai-packet` enforces:
  - per-file caps for `bundle.meta.json`, `bundle.index.json`, `test_ids.index.json`, and each `slice.*.json`
  - total hard cap (dropping `triage.json`/`manifest.json` first if present)
- When clipping occurs, the clipped file includes an additive `clipped` object (schema v1) describing the applied bounds.
