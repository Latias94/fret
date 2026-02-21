# diag-ai-agent-debugging-v1 TODO

Last updated: 2026-02-21

## M0: Docs + inventory

- [ ] Inventory current bundle hot spots (largest fields by bytes) for v1/v2.
- [ ] Define size budgets for “AI packet” outputs (default + max).
- [ ] Enumerate the minimum fields needed for common triage loops (focus, input, selection, viewport, overlays).

## M1: Index + minimal packet (Phase 1)

- [x] Define `bundle.index.json` v1 schema (typed, bounded).
- [x] Add a tooling writer + `fretboard diag index` for `bundle.index.json`.
- [ ] Prefer index when present (readers + fast-paths in `diag slice/query`).
- [x] Add `fretboard diag ai-packet ...` that exports:
  - `bundle.meta.json`
  - `bundle.index.json`
  - stable slice outputs for a given `--test-id` or script failure anchor
- [ ] Add “jump to snapshot” affordances in `diag slice` using index (avoid scanning full semantics when possible).

## M2: Slice hardening

- [ ] Ensure `diag slice` works consistently on v1/v2 bundles (inline semantics vs table-resolved).
- [ ] Add targeted slice regression tests (golden-ish fixtures for indexes + slice output shape).
- [ ] Add a stable “reason_code → slice recipe” mapping for common failures.

## M3: Chunked on-disk layout (Phase 2)

- [ ] Prototype a manifest-first bundle layout (snapshots/logs/semantics as chunked files).
- [ ] Add a compatibility materializer to emit `bundle.json` from the manifest (opt-in).
- [ ] Add packing + hashing conventions to keep artifacts integrity-checkable.
