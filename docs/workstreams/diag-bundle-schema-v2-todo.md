# diag-bundle-schema-v2 TODO

Last updated: 2026-02-21

## M0: Docs + scaffolding

- [x] Define v2 JSON layout and env flags.
- [x] Add bundle viewer support for v2 semantics table.
- [x] Add fret-diag tooling support for v2 semantics table (meta/query/slice/stats/compare).

## M1: Runtime export

- [x] Export `schema_version=2` bundles for script-driven dumps by default.
- [x] Emit `tables.semantics` with dedup keyed by `(window, semantics_fingerprint)`.
- [x] Keep inline semantics only per `FRET_DIAG_BUNDLE_SEMANTICS_MODE` (default `last` for scripts).

## M2: AI-friendly slices (post v2)

- [ ] Add `diag slice --snapshot-seq` (already supported) and ensure it works on v2 bundles.
- [ ] Add `diag slice --include-semantics` / `--no-semantics` knobs (optional).
- [ ] Provide a “minimal AI packet” export (bundle meta + slice + script.result.json).

## M3: Hardening

- [ ] Add regression tests for v2 parsing across tools.
- [ ] Add metrics to `bundle.meta.json` for semantics density + table size.
- [x] Document upgrade guidance and gotchas in `docs/ui-diagnostics-and-scripted-tests.md` (baseline notes + workstream link).
