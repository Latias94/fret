# Material 3 TopAppBar Scroll Diagnostics Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The surface/data-display packet left TopAppBar scroll behavior as a proportional follow-on: only
run a gallery diagnostic if behavior drift is proven or needs explicit evidence. A promoted
Material3 UI Gallery script already existed, but the component matrix still listed scroll
diagnostics as a residual risk.

## Target State

- Treat TopAppBar scroll behavior as `material_recipe` unless another design system proves reusable
  policy pressure.
- Use the existing Material3 UI Gallery script as the gallery gate for pinned, enter-always,
  enter-always-settle, exit-until-collapsed, and exit-until-collapsed-settle scenes.
- Record the run evidence and remove the stale "scroll diagnostics remain" matrix residual.
- Do not change recipe code when the promoted diagnostic passes.

## Layer Mapping

- `material_recipe`: owns TopAppBar variants, action wiring, toolbar semantics, and scroll behavior
  state.
- `gallery` / `diagnostics`: owns the promoted scroll screenshot/bundle script and run evidence.
- `material_foundation`: only token resolution is involved; no new foundation abstraction is needed.
- `kit_policy` / `mechanism`: no shared policy or `crates/*` contract change is justified by this
  packet.

## Non-Goals

- Do not add nested-scroll consumption or fling-velocity contracts.
- Do not add a cross-design-system scroll policy abstraction without a second consumer.
- Do not update screenshots/goldens unless the diagnostic proves drift.

## Upstream References

- Compose Material3 `AppBar.kt`: scroll behavior taxonomy and collapsed/hidden behavior intent.
- Fret TopAppBar recipe: `ecosystem/fret-ui-material3/src/top_app_bar.rs`.
- Gallery gate: `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-top-app-bar-scroll-screenshots.json`.
