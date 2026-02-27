---
title: Diag v2 Hardening + Switches Refactor v1 - Deprecations
status: draft
date: 2026-02-26
scope: diagnostics, config, env-vars, deprecation
---

# Deprecations (switches / env vars)

This note tracks *deprecated aliases* and the staged removal plan for diagnostics “switches”.

## Canonical env var set (recommended)

Prefer a config file (`FRET_DIAG_CONFIG_PATH`) plus a small set of explicit overrides:

- `FRET_DIAG`
- `FRET_DIAG_CONFIG_PATH`
- `FRET_DIAG_GPU_SCREENSHOTS`
- `FRET_DIAG_REDACT_TEXT`
- `FRET_DIAG_FIXED_FRAME_DELTA_MS`

Other `FRET_DIAG_*` env vars remain supported as compatibility/escape hatches, but should not be
required for the common `fretboard diag ... --launch` flows.

## Deprecated env var aliases

### `FRET_FRAME_CLOCK_FIXED_DELTA_MS` → `FRET_DIAG_FIXED_FRAME_DELTA_MS`

Status:

- **Deprecated** (prefer `FRET_DIAG_FIXED_FRAME_DELTA_MS` for diagnostics).
- Still supported as a generic frame clock knob.

Rationale:

- Diagnostics should have a namespaced, discoverable override that tooling can document and
  validate via `diag config doctor`.

Removal plan (staged):

- Stage A (now): `diag config doctor` emits an info note when `FRET_FRAME_CLOCK_FIXED_DELTA_MS` is set.
- Stage B (P2): escalate to a warning in tooling output (and optionally surface in `triage.json`).
- Stage C (P3): remove/stop reading the alias in `fret-core` once in-repo scripts/CI no longer use it.

Evidence anchors:

- Alias parsing: `crates/fret-core/src/window.rs` (`WindowFrameClockService::fixed_delta_from_env`).
- Tooling warning: `crates/fret-diag/src/commands/config.rs` (`diag.config.fixed_frame_delta_generic_alias`).

## Non-goal: deprecating all overrides immediately

Many env vars are intentionally retained as manual escape hatches (especially for ad-hoc debugging),
even when their values can also be expressed via `FRET_DIAG_CONFIG_PATH`. Deprecation is reserved
for *aliases* (multiple names for the same knob) and for legacy switches that are superseded by
capability gating or config-file fields.
