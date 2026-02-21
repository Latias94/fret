# diag-ai-agent-debugging-v1 TODO

Last updated: 2026-02-21

## M0: Docs + inventory

- [ ] Inventory current bundle hot spots (largest fields by bytes) for v1/v2.
- [ ] Define size budgets for “AI packet” outputs (default + max).
- [ ] Enumerate the minimum fields needed for common triage loops (focus, input, selection, viewport, overlays).

## M1: Index + minimal packet (Phase 1)

- [x] Define `bundle.index.json` v1 schema (typed, bounded).
- [x] Add a tooling writer + `fretboard diag index` for `bundle.index.json`.
- [x] Ensure `diag pack --include-root-artifacts|--include-triage` includes `bundle.index.json`.
- [x] Make `diag slice` validate `--frame-id|--snapshot-seq` against `bundle.index.json` (when present).
- [x] Make `diag slice` attempt a bounded parse for explicit snapshot selection (avoid full-bundle `serde_json::Value` build).
- [x] Modularize `diag slice` fast-path implementation (extract payload + streaming parser modules).
- [x] Allow `diag meta/index/query/slice` to operate on sidecar-only packet dirs (no `bundle.json`) when possible.
- [ ] Prefer index when present (readers + fast-paths in `diag slice/query`).
  - [x] `diag slice`: use index to pick a default snapshot for bounded parse (when no selector is provided).
  - [x] `bundle.index.json`: add optional per-snapshot test-id bloom hints (tail snapshots; inline semantics only).
  - [x] `diag query snapshots`: use `bundle.index.json` to suggest snapshot selectors (optionally filtered by test-id bloom).
  - [x] `diag query test-id`: read `_root/test_ids.index.json` when given extracted packs.
  - [ ] Add per-snapshot test-id presence indexes (bloom/hashed sets) to avoid full semantics scans.
- [x] Add `fretboard diag ai-packet ...` that exports:
  - `bundle.meta.json`
  - `bundle.index.json`
  - stable slice outputs for a given `--test-id` or script failure anchor
- [ ] Add “jump to snapshot” affordances in `diag slice` using index (avoid scanning full semantics when possible).
  - [x] When the default snapshot does not contain the requested test-id, try a small set of index-derived candidates via streaming slice before falling back to full bundle parsing.

## M2: Slice hardening

- [ ] Ensure `diag slice` works consistently on v1/v2 bundles (inline semantics vs table-resolved).
- [x] Add streaming bounded-parse support for v1 inline + v2 table semantics (still falls back to full parse for "find a better snapshot").
- [ ] Add targeted slice regression tests (golden-ish fixtures for indexes + slice output shape).
- [ ] Add a stable “reason_code → slice recipe” mapping for common failures.

## M3: Chunked on-disk layout (Phase 2)

- [ ] Prototype a manifest-first bundle layout (snapshots/logs/semantics as chunked files).
- [ ] Add a compatibility materializer to emit `bundle.json` from the manifest (opt-in).
- [ ] Add packing + hashing conventions to keep artifacts integrity-checkable.
