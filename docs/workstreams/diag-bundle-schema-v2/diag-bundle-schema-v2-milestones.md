# diag-bundle-schema-v2 Milestones

## M0 (design + consumers)

Done when:

- `tools/fret-bundle-viewer` can open v2 bundles and render semantics.
- `fretboard diag meta/query/slice/stats/compare` work on v1 and v2 bundles.

## M1 (runtime export)

Done when:

- Script-driven dumps default to `schema_version=2` bundles.
- Bundles include `tables.semantics` with dedup entries.
- Default script bundle size is materially reduced on representative UI gallery scripts.

## M2 (AI packets)

Done when:

- There is a single command that exports a small, shareable “AI packet” for a failing script run:
  - `script.result.json` (bounded evidence)
  - a `slice.<test_id>.json` (or similar)
  - `bundle.meta.json`
  - optional screenshots

## M3 (hardening + gates)

Done when:

- `bundle.meta.json` includes semantics table metrics (inline vs table, entries, unique keys).
- `fretboard diag meta --meta-report` surfaces the key size/semantics indicators in a human-readable way.
- CI gate(s) exist to ensure v2 bundles remain parseable and bounded.
- Docs are updated and stable.
