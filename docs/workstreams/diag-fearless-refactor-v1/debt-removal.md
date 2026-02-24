---
title: Diagnostics Fearless Refactor v1 (Debt Removal)
status: draft
date: 2026-02-24
scope: diagnostics, env-knobs, schema, refactor, debt
---

# Debt removal (env knobs + compatibility layers)

This document tracks “remove-the-baggage” work for the diagnostics stack after the sidecar-first
migration (Option 1) has landed.

Goals:

- converge on a small set of **canonical** env vars (avoid alias sprawl),
- delete legacy schema/compat layers once in-tree consumers have migrated,
- keep docs aligned with the implementation (one source of truth).

Non-goals:

- changing the core UI mechanism contracts (`crates/fret-ui`, `crates/fret-core`),
- removing sidecar validation / self-healing (`diag doctor --fix-sidecars` stays).

## Env knob policy

- Canonical knobs are documented in `docs/ui-diagnostics-and-scripted-tests.md`.
- Tooling should **set** canonical knobs (do not introduce new aliases).
- Legacy aliases should be:
  1. documented as deprecated,
  2. removed from tooling first,
  3. removed from runtime parsing once no in-tree consumers rely on them.

## Bundle artifact terminology + compatibility keys

Policy:

- Treat the diagnostics **bundle artifact** as the canonical input/output across tooling:
  - `bundle.json` (raw / legacy / can be huge)
  - `bundle.schema2.json` (preferred portable artifact for sidecar-first and AI loops)
- Tooling and docs should say **bundle artifact** when they mean “either file”.
- When tooling must talk about the legacy file specifically, say **raw `bundle.json`**.

Compatibility strategy (fearless refactor prerequisite):

- Output JSON produced by tooling may keep **legacy keys** for a while (e.g. `bundle_json`) as aliases,
  but new code should prefer writing/reading the canonical key:
  - canonical: `bundle_artifact`
  - legacy alias: `bundle_json`

Exit criteria to remove legacy keys:

1. All in-tree scripts and consumers are updated to read `bundle_artifact` (or accept both keys).
2. A deprecation period is documented (internal) and warnings are emitted when legacy keys are consumed.
3. Legacy keys are removed from tooling outputs, then removed from any internal parsers.

## Canonical vs legacy (initial inventory)

### Screenshots

- Canonical:
  - `FRET_DIAG_GPU_SCREENSHOTS=1`: enables script-driven GPU readback screenshots (`capture_screenshot`).
  - `FRET_DIAG_BUNDLE_SCREENSHOT=1`: writes a `frame.bmp` alongside bundle dumps.
- Removed legacy aliases (no longer supported as of 2026-02-23):
  - `FRET_DIAG_SCREENSHOTS=1` (use `FRET_DIAG_GPU_SCREENSHOTS=1`)
  - `FRET_DIAG_SCREENSHOT=1` (use `FRET_DIAG_BUNDLE_SCREENSHOT=1`)

Evidence anchors:

- `ecosystem/fret-bootstrap/src/ui_diagnostics/config.rs` (runtime env parsing)
- `crates/fret-launch/src/runner/desktop/runner/diag_screenshots.rs` (desktop runner GPU screenshots)
- `crates/fret-launch/src/runner/desktop/runner/diag_bundle_screenshots.rs` (bundle `frame.bmp`)
- `crates/fret-diag/src/lib.rs` / `crates/fret-diag/src/diag_suite.rs` / `crates/fret-diag/src/compare.rs`
  (tooling launch env assembly)

### Frame clock (fixed delta)

- Canonical:
  - `FRET_DIAG_FIXED_FRAME_DELTA_MS=<n>`: forces a fixed frame tick for deterministic scripts.

Evidence anchors:

- `crates/fret-core/src/window.rs` (`WindowFrameClockService::fixed_delta_from_env`)
- `crates/fret-diag/src/lib.rs` (`--fixed-frame-delta-ms` wiring)

### Bundle schema / dump policy

- Canonical:
  - `FRET_DIAG_BUNDLE_SEMANTICS_MODE=all|changed|last|off`
  - `FRET_DIAG_BUNDLE_JSON_FORMAT=pretty` (optional; for human review)

Evidence anchors:

- `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_dump.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_dump_policy.rs`

## Compatibility layers to delete (tracked)

This list stays intentionally short and actionable. When a compatibility layer is removed, add the
commit hash + evidence anchor(s) here.

- See also: `docs/workstreams/diag-fearless-refactor-v1/redundancy-removal-checklist.md` (risk-tiered removal plan).

- Legacy schema-v1-only traversal helpers in tooling once all in-tree dumps default to schema v2.
- Remaining `bundle.json`-only assumptions in CLI tooling once the bundle-artifact sweep is complete
  (error messages, help text, and path resolution should accept `<bundle_dir|bundle.json|bundle.schema2.json>`).
- Transitional output JSON aliases (`bundle_json` keys) once all in-tree consumers have migrated to `bundle_artifact`.

Completed:

- Removed legacy screenshot env aliases from runtime parsing (`3793c44ec`).
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics/config.rs`,
    `crates/fret-launch/src/runner/desktop/runner/diag_screenshots.rs`,
    `crates/fret-launch/src/runner/desktop/runner/diag_bundle_screenshots.rs`
- Removed legacy fixed frame delta env alias (`FRET_DIAG_FRAME_DELTA_MS`) (`f93507648`).
  - Evidence: `crates/fret-core/src/window.rs`
- Removed schema-v1 bundle emission from the runtime (`bundle.schema_version` is now always v2) (`e3b5b6d5d`).
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_dump.rs`
