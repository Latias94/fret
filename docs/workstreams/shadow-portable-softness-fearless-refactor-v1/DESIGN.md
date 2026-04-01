# Shadow Portable Softness (Fearless Refactor v1) — Design

Status: In Progress

Last updated: 2026-04-01

Renderer primitive follow-on:

- `docs/workstreams/shadow-renderer-primitive-fearless-refactor-v1/DESIGN.md`

## Context

The previous shadow closure lane (`docs/workstreams/shadow-surface-fearless-refactor-v1/`) finished
the source-alignment work for:

- preset geometry (`shadow-xs/sm/md/lg/xl`),
- shadcn theme seeding,
- shadow footprint gates,
- and the explicit coexistence posture between portable `ShadowStyle` and effect-backed
  `DropShadowV1`.

That closure intentionally did **not** claim CSS-level blur fidelity for the portable painter.

Since then, shadow footprint gates for shared shadcn controls (`button`, `input`, `input-group`,
`textarea`, `select`, `native-select`, `button-group` leaf buttons) have shown that the
`new-york-v4` shadow geometry is numerically aligned, yet the surfaces can still look subjectively
"hard" in Fret.

The main reason is now visible in the portable painter implementation:

- `crates/fret-ui/src/paint.rs` paints component shadows as multiple expanded quads.
- The old alpha ramp used `1 / (1 + i)` per step without normalization.
- As blur/softness increases, the total alpha budget grows with the harmonic sum instead of
  staying bounded by the recipe-owned alpha.

Example:

- `blur=2` → total per-layer alpha budget `≈ 1.833x`
- `blur=3` → total per-layer alpha budget `≈ 2.083x`
- `blur=4` → total per-layer alpha budget `≈ 2.283x`

That means a `shadow-xs` color like `rgba(0, 0, 0, 0.05)` can accumulate into a much denser inner
edge than the recipe author intended, even when the outer footprint matches CSS.

## Problem Statement

The problem is no longer "our shadow tokens drifted from shadcn."

The remaining problem is:

1. Portable `ShadowStyle` preserves footprint, but not a stable opacity budget.
2. Current gates mostly prove footprint/insets, not pixel-profile softness.
3. Users therefore experience a renderer-level "hard shadow" even when recipe numbers are correct.

## Goals

1. Make portable `ShadowStyle` softness bounded and reviewable.
2. Ensure increasing blur redistributes alpha instead of increasing total shadow density.
3. Add unit tests at the mechanism layer so future refactors cannot silently reintroduce the same
   over-darkening.
4. Keep the contract split explicit:
   - `ShadowStyle` remains the portable component baseline.
   - `DropShadowV1` remains an explicit effect-owned blur path.

## Non-goals

1. Implicitly upgrading all `ShadowStyle` usage to `DropShadowV1`.
2. Reopening shadcn token geometry alignment.
3. Claiming full pixel-perfect CSS blur parity in this lane.
4. Changing effect contracts or renderer degradation rules from ADR 0286.

## v1 Decision

Portable shadow layers should use a **normalized alpha budget**:

- keep the current bounded multi-quad approximation,
- keep the current footprint (`spread + blur`) behavior,
- keep inner layers heavier than outer layers,
- but normalize the per-step weights so the total layer opacity remains equal to the
  recipe/theme-owned alpha.

This is a fidelity improvement inside the existing `ShadowStyle` contract, not a new public API.

## Algorithmic Boundary After GPUI/Zed Review

The portable painter is now **budget-correct** for the existing `ShadowStyle` contract, but that
does **not** make it equivalent to a modern GPU shadow implementation.

Reviewing the pinned `repo-ref/zed` renderer shows a materially different architecture:

- `BoxShadow` API values are lowered into a dedicated `Shadow` scene primitive instead of expanded
  CPU-side quads.
- the WGPU backend renders shadows through a dedicated shadow pipeline,
- and the shader computes shadow coverage from rounded-rect distance plus gaussian blur terms.

This means the current Fret portable painter should be treated as:

- a portable fallback with bounded opacity and stable footprint,
- suitable for mechanism-layer `ShadowStyle`,
- but **not** the final renderer-quality answer for elevated surfaces when perceptual softness is a
  product requirement.

In practice, local composition can still amplify the portable fallback's limitations. The current
`todo_demo` was one such example:

- the main card used `shadow_xl`,
- the footer painted a full-bleed tinted bottom band,
- and that combination made the bottom edge read flatter and more rectangular than intended.

This workstream therefore treats two conclusions as simultaneously true:

1. the portable painter needed alpha-budget correction and now has stronger mechanism gates,
2. renderer-level shadow quality still points toward a future dedicated shadow primitive / effect
   path, closer to the GPUI/Zed model.

## Evidence Anchors

- Portable painter: `crates/fret-ui/src/paint.rs`
- Portable shadow contract: `docs/adr/0060-shadows-and-elevation.md`
- Prior closure lane: `docs/workstreams/shadow-surface-fearless-refactor-v1/DESIGN.md`
- Effect-backed blur path: `docs/workstreams/renderer-drop-shadow-effect-v1/renderer-drop-shadow-effect-v1.md`
- Shared control shadow gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
- Demo-local composition evidence: `apps/fret-examples/src/todo_demo.rs`
- GPUI/Zed primitive path: `repo-ref/zed/crates/gpui/src/window.rs`
- GPUI/Zed scene primitive: `repo-ref/zed/crates/gpui/src/scene.rs`
- GPUI/Zed shader shadow path: `repo-ref/zed/crates/gpui_wgpu/src/shaders.wgsl`

## Follow-up After v1

This lane now also has a deterministic mechanism-level softness-profile gate:

- the profile darkens monotonically toward the edge,
- full-overlap darkness stays within the recipe-owned alpha budget under layer compositing,
- and outer bands remain lighter than the full stack.

Renderer-level visual review is now also reproducible via a curated screenshot suite:

- `tools/diag-scripts/suites/ui-gallery-shadow-surface-screenshots/suite.json`
- current representative surfaces:
  - `Card` demo (`shadow-sm`)
  - `Calendar` demo root (`shadow-sm`)
  - `Sonner` demo open toast (generic toast baseline)

The remaining open quality question after that is **automated renderer-level visual parity**:

- footprint parity is already covered,
- alpha-budget sanity is covered,
- mechanism-level softness profile is covered,
- screenshot-backed review evidence now exists for representative elevated surfaces,
- but CSS-like perceptual falloff under actual renderer compositing still needs a dedicated
  readback or screenshot-diff gate if we want to claim stronger visual fidelity later.
