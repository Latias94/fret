---
title: Diagnostics Fearless Refactor v1 (Migration Plan)
status: draft
date: 2026-02-22
scope: diagnostics, artifacts, schema, migration
---

# Migration plan (fearless, incremental)

This plan is intentionally incremental: land sidecars and lite tooling first, then remove legacy/debt once the new path is the
default for in-tree workflows.

## Strategy options

### Option 1 (preferred): Sidecar-first + schema v2 adoption

Keep `bundle.json` canonical, but make day-to-day tooling prefer **bounded sidecars**:

- `frames.index.json` for per-frame triage and hotspots
- `bundle.index.json` / `bundle.meta.json` / `test_ids.index.json` for fast selectors and inventories

Adopt `bundle.json` schema v2 features where available (semantics table + per-snapshot fingerprints) so bundles remain small while
still supporting selector-driven automation.

This option is compatible with “fearless refactors”: tools can regenerate sidecars from `bundle.json`, and missing/invalid sidecars
degrade into clear, actionable errors.

### Option 2 (later): Chunked bundle format / manifest-first storage

If `bundle.json` remains too large even with schema v2 + sidecars, consider a “manifest + chunks” layout as the primary artifact
(still packable into a zip). This is a larger contract change and should only be attempted after Option 1 has eliminated the
current pain for in-tree workflows.

## Phases and exit criteria

### Phase A: Make lite the default triage path

- `diag doctor` is the first step in every scripted repro loop.
- `diag triage --lite` / `diag hotspots --lite` are the default first-pass tools.
- `diag ai-packet` always includes lite reports and does not fail hard when heavy reports cannot be produced.

Exit criteria:

- maintainers can triage “huge bundles” without opening `bundle.json` in memory.

### Phase B: Consolidate semantics traversal

- Centralize semantics traversal in `crates/fret-diag/src/json_bundle.rs` (`SemanticsResolver` + helpers).
- Eliminate repeated JSON path digging (`debug.semantics.nodes`) across checks/gates.

Exit criteria:

- new gates/checks never re-implement semantics lookup; they use shared helpers.

### Phase C: Tighten module boundaries (delete redundancy)

- Make module boundaries in `ecosystem/fret-bootstrap/src/ui_diagnostics/` and `crates/fret-diag/src/stats/` stick.
- Delete transitional forwarders/wrappers that exist only due to historical file layout.

Exit criteria:

- large “megafile” churn is reduced; extracted modules are the default home for new code.

### Phase D: Deprecate legacy knobs and legacy-only compatibility layers

- Mark unused env knobs as deprecated (warn + docs), then remove after the in-tree migration window.
- Remove legacy-only schema shims once in-tree bundles no longer rely on them.

Exit criteria:

- the tooling surface stays small and consistent; compatibility code is bounded and justified.

