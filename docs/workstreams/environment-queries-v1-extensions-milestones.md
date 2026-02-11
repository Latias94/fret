---
title: Environment Queries (v1) — Preference Extensions — Milestones
status: draft
date: 2026-02-10
scope: contract + mechanism + runner wiring + diagnostics evidence for new preference keys
---

# Environment Queries (v1) — Preference Extensions — Milestones

ADR anchor:

- `docs/adr/0246-environment-queries-preference-extensions-v1.md`

## M0 — Contract locked

Definition of done:

- ADR 0246 is present and referenced from `docs/adr/README.md`.

## M1 — Runtime mechanism + diagnostics

Definition of done:

- `crates/fret-core::WindowMetricsService` exposes storage for:
  - `text_scale_factor`
  - `prefers_reduced_transparency`
  - `accent_color`
- `crates/fret-ui` commits these fields into the per-window environment snapshot and supports
  dependency tracking + cache key fingerprints.
- Diagnostics bundles export them under `debug.environment` (best-effort).

Evidence:

- `crates/fret-core/src/window.rs` (`WindowMetricsService::{text_scale_factor,prefers_reduced_transparency,accent_color}`)
- `crates/fret-ui/src/elements/cx.rs` (`environment_text_scale_factor`, `environment_prefers_reduced_transparency`, `environment_accent_color`)
- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`ElementEnvironmentSnapshotV1`)

## M2 — Runner sources (best-effort)

Definition of done:

- Web/wasm runner supplies best-effort `text_scale_factor` and `prefers_reduced_transparency`.
- Desktop runners set best-effort values (leave `None` when unavailable).

Evidence:

- `crates/fret-launch/src/runner/web/render_loop.rs` (`read_text_scale_factor`, `prefers_reduced_transparency`)
- `crates/fret-launch/src/runner/desktop/mod.rs` (`read_desktop_text_scale_factor`, `read_desktop_prefers_reduced_transparency`, `read_desktop_accent_color`)

## M3 — Ecosystem adoption gates

Definition of done:

- At least one shadcn-aligned recipe respects reduced transparency via `fret-ui-kit` helper
  defaults and has a regression gate (unit test or `fretboard diag` script).

Evidence:

- `ecosystem/fret-ui-kit/src/declarative/glass.rs` (`glass_panel`)
- `ecosystem/fret-ui-kit/src/recipes/glass.rs` (`glass_effect_chain_for_environment` + unit test)
