---
title: Diag Fearless Refactor v1
status: draft
date: 2026-02-21
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
2. **Explicit schema evolution**: keep `bundle.json` versioned and backward compatible.
   - Runtime supports `FRET_DIAG_BUNDLE_SCHEMA_VERSION=1|2`.
   - Runtime supports `FRET_DIAG_BUNDLE_SEMANTICS_MODE=all|changed|last|off`.
3. **Small-by-default AI evidence**: keep default debugging workflows bounded:
   - prefer `bundle.index.json` + targeted slices + `diag ai-packet` over shipping the full bundle.
4. **Refactor safety gates**: after each meaningful refactor, keep at least one fast workspace gate green
   (recommended: `cargo check -p fret-ui-gallery`).

## Non-goals (v1)

- Replacing JSON with a database.
- A new scripting language (we keep JSON schema v1/v2 scripts).
- Breaking changes that require ecosystem apps to rewrite immediately.

## Current state (as of 2026-02-21)

- Runtime can export schema v1 and v2 bundles (configurable via env vars; defaults differ for manual vs script dumps).
- Tooling can:
  - convert bundles (`fretboard diag bundle-v2`) for measurement/compat,
  - build bounded sidecars (`bundle.meta.json`, `bundle.index.json`, `test_ids.index.json`),
  - export bounded “AI packets” (`fretboard diag ai-packet`),
  - slice bundles without grepping `bundle.json` (`fretboard diag slice`).

## Plan: two-phase evolution (preferred order)

This workstream follows the repo preference: **finish “Plan 1” before “Plan 2”**.

### Plan 1: Keep `bundle.json`, add indexes + bounded evidence (ship now)

- Keep `bundle.json` as the canonical portable artifact.
- Make schema v2 + semantics-table the default for script-driven dumps.
- Prefer `FRET_DIAG_BUNDLE_SEMANTICS_MODE=last|changed` for script runs and AI loops.
- Treat indexes and slices as the default review/debug units:
  - `bundle.index.json` for “jump to the right snapshot quickly”,
  - `test_ids.index.json` for fast `test_id` discovery,
  - bounded `slice.*.json` for per-target semantics ancestry, without shipping the entire semantics table.

### Plan 2: Manifest-first chunked bundles (optional, after Plan 1 is solid)

- Prototype a manifest + chunked file layout for huge bundles (snapshots/logs/semantics split).
- Provide a compatibility materializer to re-create `bundle.json` on demand.
- Keep packing/hashing stable so artifacts remain shareable and integrity-checkable.

See also: `docs/workstreams/diag-ai-agent-debugging-v1.md` (Phase 2 notes).

