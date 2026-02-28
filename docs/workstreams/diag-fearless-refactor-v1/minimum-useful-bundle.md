---
title: Diagnostics Fearless Refactor v1 (Minimum Useful Bundle)
status: draft
date: 2026-02-22
scope: diagnostics, artifacts, schema
---

# Minimum useful bundle contract

This note defines the **minimum useful contract** for the diagnostics **bundle artifact** (`bundle.json` or `bundle.schema2.json`)
and the boundaries between:

- the canonical, shareable artifact (the bundle artifact), and
- optional, regeneratable accelerators (sidecars like `bundle.index.json`).

Goal: make the day-to-day debug loop (including AI/agentic triage) fast **without** requiring a new storage format.

Non-goal: locking the entire bundle artifact schema; only the *minimum useful surfaces* are described here.

## Definitions

- **Bundle**: a directory that contains a bundle artifact (`bundle.json` or `bundle.schema2.json`) and optional adjacent files
  (sidecars, screenshots, manifests).
- **Sidecar**: a bounded JSON file derived from a bundle, intended to make queries cheap without opening the full bundle.
- **Regeneratable**: can be derived from the bundle artifact alone (no hidden state).

## Contract: what must remain in the bundle artifact

### MUST

The bundle artifact must include enough information to:

1. Identify and order snapshots.
2. Correlate snapshots with the app/page under test.
3. Support selector-driven automation when semantics export is enabled.

Practical minimum requirements (schematic, not exhaustive):

- Bundle header/config:
  - `schema_version`
  - `config` (at least: snapshot limits + redaction flags)
- Windows:
  - `windows[]`
  - `windows[].window` (stable window id)
  - `windows[].snapshots[]`
- Snapshot identity and ordering:
  - `snapshots[].frame_id` and/or `snapshots[].timestamp_unix_ms`
  - `snapshots[].tick_id` (preferred when available)
- App context:
  - `snapshots[].app_snapshot` (or an equivalent structured summary that lets tools detect the demo/page under test)

### SHOULD

For stable selector-based triage and scripted repros, bundles should include at least one snapshot with exported semantics:

- schema v1: `snapshots[].debug.semantics`
- schema v2: either inline `debug.semantics` or table-backed semantics via `tables.semantics` + per-snapshot fingerprints

Recommendation:

- For **script dumps**, default to schema v2 + `semantics_mode=last` so the bundle stays small but still contains at least one full semantics snapshot.
- For **manual dumps**, default to schema v1 + `semantics_mode=all` when you’re actively inspecting semantics across many frames.

### MAY

Anything that is expensive, redundant, or primarily query-acceleration may be moved out of the bundle artifact into sidecars, as long as:

- tools can still function (possibly slower) from the bundle artifact alone, and
- sidecars are clearly versioned and additive-only.

Examples:

- test-id catalogs
- per-snapshot bloom filters / fingerprints summaries
- script step → snapshot marker tables (derived from `script.result.json` + snapshot ids/timestamps)

## What should be sidecars (not required for correctness)

Sidecars must be:

- **bounded** (size-controlled),
- **optional** (tools regenerate or degrade gracefully),
- **versioned** (`schema_version`),
- **additive-only** in schema evolution.

Current intended sidecars (native dumps):

- `bundle.index.json`: per-window/per-snapshot selectors + bounded test-id bloom + optional script markers.
- `bundle.meta.json`: counts + uniqueness summaries.
- `test_ids.index.json`: last-resolved semantics test-id catalog + duplicate hints.
- `script.result.json` (script runs only): bounded evidence + outcome summary.

## Compatibility and failure behavior

- Missing sidecars must not cause hangs/timeouts. Tools should:
  - either regenerate them from the bundle artifact, or
  - fail fast with a clear, actionable error (what file is missing and which command regenerates it).
- Sidecars must not become the only source-of-truth; the bundle artifact remains canonical.
