---
title: Diag Fearless Refactor v1
status: draft
date: 2026-02-24
scope: diagnostics, automation, bundle-schema, refactor
---

# Diag Fearless Refactor v1

This workstream is about making Fret’s diagnostics runtime + tooling easier to evolve **without regressions**:

- reduce “single file with everything” maintenance risk (especially `UiDiagnosticsService`),
- keep bundle artifacts **portable** and **AI-friendly** even when `bundle.json` is large,
- evolve schemas and capabilities with explicit versioning and fast, structured failure modes.

This is an implementation-focused companion to the contract-first workstreams:

- Extensibility + capabilities contract: `docs/workstreams/diag-extensibility-and-capabilities-v1/README.md`
- AI/size tooling (indexes, packets, hotspots): `docs/workstreams/diag-ai-agent-debugging-v1.md`

## Problem statement

Today, diagnostics is powerful but the “fearless refactor” tax is high:

- The in-app runtime implementation historically accreted into a very large file with multiple concerns
  (filesystem triggers, WS transport, script execution, bundle export, pick/inspect UX, evidence/trace capture).
- `bundle.json` can become too large to parse/search/share comfortably, especially for AI loops.
- Schema evolution must remain compatible across:
  - native filesystem transport,
  - web runner via DevTools WS export,
  - offline tooling (pack, slice, index, compare, gates).

## Goals (v1)

1. **Modular runtime**: extract cohesive subsystems out of `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` into
   small modules with clear responsibilities and narrow visibility.
2. **Explicit schema evolution**: keep bundle artifacts versioned and backward compatible.
   - Runtime supports `FRET_DIAG_BUNDLE_SEMANTICS_MODE=all|changed|last|off`.
3. **Small-by-default AI evidence**: keep default debugging workflows bounded:
   - prefer `bundle.schema2.json` + indexes + targeted slices + `diag ai-packet` over shipping the full `bundle.json`.
4. **Refactor safety gates**: after each meaningful refactor, keep at least one fast workspace gate green
   (recommended: `cargo check -p fret-ui-gallery`).

## Non-goals (v1)

- Replacing JSON with a database.
- A new scripting language (we keep JSON schema v1/v2 scripts).
- Breaking changes that require ecosystem apps to rewrite immediately.

## Current state (as of 2026-02-24)

- Runtime exports schema v2 bundles by default (semantics are deduplicated via `tables.semantics`).
- Older bundles may still be schema v1 (inline-only semantics, no tables); tooling remains compatible.
- Tooling can:
  - convert bundles (`fretboard diag bundle-v2`) for measurement/compat,
  - build bounded sidecars (`bundle.meta.json`, `bundle.index.json`, `test_ids.index.json`),
  - export bounded “AI packets” (`fretboard diag ai-packet`),
  - slice bundles without grepping `bundle.json` (`fretboard diag slice`).
- Tooling treats `bundle.schema2.json` as a first-class **bundle artifact** input (alongside `bundle.json`), and can
  “heal” older bundle dirs via `fretboard diag bundle-doctor fix`.
- Runtime `bundle.index.json` includes a bounded `test_id` bloom (`test_id_bloom_hex`) on tail snapshots to make
  `diag query snapshots --test-id ...` fast without loading the full bundle.
- Runtime script dumps include `script.result.json`, and `bundle.index.json` may include additive `script.steps` markers for
  mapping `step_index` to a snapshot selector without re-parsing the full bundle.

## Plan: two-phase evolution (preferred order)

This workstream follows the repo preference: **finish “Plan 1” before “Plan 2”**.

### Plan 1: Schema2-first + bounded evidence (ship now)

- Prefer `bundle.schema2.json` as the default portable artifact for tooling and AI loops.
- Keep `bundle.json` as an optional compatibility artifact (older runs, deep debugging).
- Make schema v2 + semantics-table the default for runtime-produced bundles.
- Prefer `FRET_DIAG_BUNDLE_SEMANTICS_MODE=last|changed` for script runs and AI loops.
- Treat indexes and slices as the default review/debug units:
  - `bundle.index.json` for “jump to the right snapshot quickly”,
  - `test_ids.index.json` for fast `test_id` discovery,
  - bounded `slice.*.json` for per-target semantics ancestry, without shipping the entire semantics table,
  - bounded `diag ai-packet` outputs for AI triage and automated debugging.

### Plan 2: Manifest-first chunked bundles (optional, after Plan 1 is solid)

- Prototype a manifest + chunked file layout for huge bundles (snapshots/logs/semantics split).
- Provide a compatibility materializer to re-create `bundle.json` on demand (and/or to derive `bundle.schema2.json`).
- Keep packing/hashing stable so artifacts remain shareable and integrity-checkable.

See also: `docs/workstreams/diag-ai-agent-debugging-v1.md` (Phase 2 notes).

## Schema migration policy (v1 → v2) (draft)

This section documents an intended compatibility-first path for migrating bundle artifacts from schema v1 to v2, and
eventually removing redundant “v1-only” emission paths. It is intentionally conservative:

- **Tooling must remain backward compatible**: `crates/fret-diag` should continue to *read* v1 bundles for a long time.
- **Runtime defaults can change only after Plan 1 is stable**: sidecars, schema2-first AI packet flow, and selectors
  must work well without requiring ad-hoc parsing of full `bundle.json`.

Proposed phases:

1. **Today** (current behavior):
   - Runtime emits schema v2 bundles (semantics are deduplicated via `tables.semantics`).
   - Tooling continues to read v1/v2 bundles.
2. **Remove v1 emission** (after Plan 1 closure is proven in daily use):
   - Runtime no longer emits schema v1 bundles; only v2 is produced.
   - Tooling keeps reading v1 and can upgrade bundles when needed (`fretboard diag bundle-v2`).

Exit criteria (what “proven” means) before flipping defaults:

- `bundle.index.json` + `bundle.meta.json` + `test_ids.index.json` are present and consumed in the common pack/repro workflow.
- `diag ai-packet` is a viable default for AI triage without opening full `bundle.json`.
- Script step markers and `--test-id` snapshot filtering work reliably on runtime-produced bundles.
