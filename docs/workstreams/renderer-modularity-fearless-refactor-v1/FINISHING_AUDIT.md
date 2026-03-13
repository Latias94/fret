# Renderer Modularity (Fearless Refactor v1) — Finishing Audit

Status: Closed for v1

Last updated: 2026-03-13

## Scope

This audit closes the remaining "should we keep splitting?" questions for the renderer modularity
workstream:

- `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs`
- `crates/fret-render-wgpu/src/renderer/services.rs`
- `crates/fret-render-wgpu/src/renderer/mod.rs`

## Findings

### 1. `recorders/effects.rs` is no longer an ownership hotspot

Current shape:

- the file is now a small recorder-family owner with six homogeneous fullscreen recorder entrypoints
- each entrypoint delegates shared bind-group / pipeline / masked-vs-unmasked pass wiring to
  `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects_shared.rs`
- the file no longer owns target allocation, executor access policy, custom-effect family logic, or
  clip-mask/composite family entrypoints

Evidence:

- `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects_shared.rs`
- `docs/workstreams/renderer-modularity-fearless-refactor-v1/TODO.md`

Assessment:

- further splitting `AlphaThreshold`, `ColorAdjust`, `ColorMatrix`, `Dither`, `Noise`, and
  `DropShadow` into one-file-per-family would mostly duplicate one shared helper pattern
- v1 ownership clarity is already achieved because the remaining file no longer mixes helper
  infrastructure with unrelated effect families

Decision:

- do not split `recorders/effects.rs` further in v1
- revisit only if a new utility family introduces family-local state, target allocation rules, or
  nontrivial fallback logic that stops fitting the current shared-helper pattern

### 2. `services.rs` is already at the intended boring shape

Current shape:

- `crates/fret-render-wgpu/src/renderer/services.rs` now contains only `TextService` and
  `PathService`
- custom-effect service ownership moved to
  `crates/fret-render-wgpu/src/renderer/services_custom_effects.rs`
- SVG/material service ownership moved to
  `crates/fret-render-wgpu/src/renderer/services_assets.rs`

Evidence:

- `crates/fret-render-wgpu/src/renderer/services.rs`
- `crates/fret-render-wgpu/src/renderer/services_custom_effects.rs`
- `crates/fret-render-wgpu/src/renderer/services_assets.rs`

Assessment:

- the remaining file is small, single-purpose, and no longer coordinates cross-owner registry or
  pipeline mutation
- further splitting would reduce readability because `TextService` and `PathService` are already
  one-owner forwarding surfaces

Decision:

- keep `services.rs` as the v1 home for `TextService` and `PathService`

### 3. Renderer owner extraction goals are satisfied for v1

Current shape:

- `Renderer` is now a compact state shell whose mutable subdomains route through dedicated owners
- the domain owners called out by this workstream are all extracted:
  - text
  - SVG
  - materials/custom effects
  - intermediate budgeting/pools
  - diagnostics

Evidence:

- `crates/fret-render-wgpu/src/renderer/mod.rs`
- `crates/fret-render-wgpu/src/renderer/diagnostics.rs`
- `crates/fret-render-wgpu/src/renderer/intermediate_pool.rs`
- `crates/fret-render-wgpu/src/renderer/material_effects.rs`
- `crates/fret-render-wgpu/src/renderer/path.rs`
- `crates/fret-render-wgpu/src/renderer/svg/mod.rs`

Assessment:

- the remaining `Renderer` fields are now mostly owner handles rather than loose cross-domain
  bookkeeping
- the workstream's renderer-owner goals are complete enough to close `RMFR-renderer-041`,
  `RMFR-renderer-042`, and `RMFR-renderer-043` for v1

## v1 Closure Decision

For `renderer-modularity-fearless-refactor-v1`, the remaining work is no longer "split more
files". The remaining work is:

- docs/closure bookkeeping
- gates staying green
- future drift detection if new families re-grow the owner seams

This means:

- `RMFR-renderer-041` can close as done
- `RMFR-renderer-042` can close as done
- `RMFR-renderer-043` can close as done
- `recorders/effects.rs` stays as-is for v1
- `services.rs` stays as-is for v1
