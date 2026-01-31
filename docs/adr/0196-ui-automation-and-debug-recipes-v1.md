---
title: "ADR 0196: UI Automation + Debug Recipes v1"
---

# ADR 0196: UI Automation + Debug Recipes v1

Status: Proposed

Scope: debug/observability, scripted interaction automation, repro packaging, and performance triage.

This ADR formalizes a unified, AI-friendly workflow for debugging and profiling, without violating `fret-ui` layering
rules (see ADR 0066). The runtime provides mechanism hooks and small, versioned data shapes; policy and orchestration
live in `fret-bootstrap` + `fretboard`.

Related:

- ADR 0174: `docs/adr/0174-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
- ADR 0033: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- ADR 0036: `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`
- ADR 0066: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- ADR 1155: `docs/adr/1155-cache-root-tracing-contract-v1.md`

Workstream notes:

- `docs/workstreams/ui-automation-and-debug-recipes-v1.md`

## Context

Fret targets editor-grade UI and multi-viewport workflows where correctness and performance bugs are often:

- cross-cutting (focus, overlays, capture, docking arbitration),
- sensitive to caching/invalidation behavior (stale paint / missing redraw),
- hard to reproduce without an “end-user intent” automation layer,
- hard to diagnose without portable, structured artifacts.

We already have building blocks (bundles, scripts, inspector, perf triage, Tracy, RenderDoc). This ADR locks the missing
contracts needed to unify those into a scalable workflow.

## Goals

1. Provide a unified “recipe” workflow for repro + evidence capture, suitable for AI-assisted debugging.
2. Make interaction automation robust across DPI/window sizes by selecting targets via semantics, not coordinates.
3. Make “missing repaint” bugs diagnosable via structured checks and artifacts.
4. Standardize a minimal performance query surface for automated triage and regression gating.
5. Preserve crate boundaries and keep `fret-ui` policy-free.

## Non-goals

- Screenshot goldens as the primary correctness mechanism.
- A production inspector UI surface in `fret-ui`.
- Perfect IME record/replay determinism in CI.

## Decision

### 1) Introduce a “repro recipe” packaging command (tooling)

Tooling (`apps/fretboard`) SHOULD provide a single entrypoint that:

- runs a script or suite,
- applies standardized post-run checks,
- emits a small machine summary,
- packs a shareable zip containing required artifacts.

This command is an orchestration surface only; it does not add new runtime policy.

### 2) Add a high-level action layer (automation ergonomics)

We introduce intent-level automation actions, either by:

- defining Script v2 steps (preferred for long-term clarity), or
- compiling v2 “recipes” into the existing v1 step set.

Minimum v1 recipe actions:

- `ensure_visible`
- `type_text_into`
- `drag_to`
- `set_slider_value`
- `menu_select`

These actions MUST resolve targets via `SemanticsSnapshot` selectors (ADR 0033) and SHOULD prefer `test_id`.

### 3) Standardize “missing repaint” checks and required hooks

The workflow must be able to flag likely repaint bugs with actionable evidence.

Standard checks (tooling-side):

- semantics bounds changed but `scene_fingerprint` did not (existing).
- semantics content changed but `scene_fingerprint` did not (new; requires `semantics_fingerprint`).

Required new runtime hook:

- Each snapshot SHOULD export `semantics_fingerprint: u64` (best-effort hash) so tooling can correlate semantics changes
  with paint changes without scraping logs.

Optional check (gated, best-effort):

- screenshot-backed region hash changes for a given target bounds.

### 4) Semantics value for range controls (enables slider automation)

To support robust slider automation and improve accessibility alignment:

- range-like controls (e.g. sliders) SHOULD export a numeric value via semantics.
- scripts MAY assert/drive based on that value (e.g. `set_slider_value`).

This must not leak into styling/policy; it is a semantics/a11y-aligned contract.

### 5) Performance query surface (bundles)

Bundles SHOULD expose enough data for automated triage:

- per-frame durations (layout/paint/total),
- layout-engine solve and top measure hotspots,
- invalidation and cache-root reuse counters.

Tooling MAY provide regression gates based on these fields.

## Crate boundary guidance

- `crates/fret-ui`:
  - mechanism hooks and small debug data only (feature-gated),
  - must not grow into a component/policy layer.
- `ecosystem/fret-bootstrap`:
  - diagnostics service, script execution, artifact emission hooks.
- `apps/fretboard`:
  - orchestration, packaging, triage, compare/matrix, gating thresholds.

## Implementation notes (as of 2026-01-31)

This ADR is partially implemented in a way that preserves the intended crate boundaries:

- **Repro orchestration + packaging (`fretboard`)**
  - `fretboard diag repro` runs a script or a suite, applies post-run checks, writes a machine summary, and packs a zip.
  - Suite repros are packed as multi-bundle zips with stable prefixes and script sources included under `_root/scripts/`.
  - Evidence: `apps/fretboard/src/diag.rs` (`diag repro`, `pack_repro_zip_multi`).
- **Missing repaint checks (`fretboard`)**
  - Tooling provides multiple “missing repaint” gates, including a coarse check that fails when `semantics_fingerprint`
    changes but `scene_fingerprint` does not (`--check-semantics-changed-repainted`), and includes a small semantics diff
    summary to aid triage. When `--dump-semantics-changed-repainted-json` is set, it also writes a structured
    `check.semantics_changed_repainted.json` next to `bundle.json` for machine consumption.
  - Evidence: `apps/fretboard/src/diag.rs` (`check_bundle_for_semantics_changed_repainted*`).
- **Semantics fingerprint export (`fret-bootstrap`)**
  - Diagnostics snapshots export `semantics_fingerprint` as a best-effort hash derived from the semantics snapshot.
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`semantics_fingerprint` field and computation).

Known gaps:

- `diag repro` does not yet orchestrate Tracy / RenderDoc capture/export as part of a single artifact pack.
- High-level intent actions (Script v2 or a compiler layer) are not yet implemented.
- Range control semantics value (to enable robust `set_slider_value`) is still an open contract item.
