# Shadow Renderer Primitive (Fearless Refactor v1) — TODO

Status: Complete (primitive default path, explicit fallback, and first-party consumer audit landed)

Last updated: 2026-04-02

Related:

- Design: `docs/workstreams/shadow-renderer-primitive-fearless-refactor-v1/DESIGN.md`
- Milestones: `docs/workstreams/shadow-renderer-primitive-fearless-refactor-v1/MILESTONES.md`
- ADR: `docs/adr/0318-rounded-rect-shadow-scene-primitive-v1.md`

Tracking legend:

- `[ ]` open
- `[~]` in progress
- `[x]` done

## A. Root-cause freeze

- [x] SRPFR-audit-001 Confirm that portable-shadow softness fixes do not eliminate the structural
  renderer gap.
  - Result: the alpha-budget fix makes the fallback saner, but it does not turn quad expansion into
    a first-class renderer shadow primitive.

- [x] SRPFR-audit-002 Confirm that GPUI/Zed uses a dedicated scene primitive and shader path for box
  shadows.
  - Evidence anchors:
    - `repo-ref/zed/crates/gpui/src/window.rs`
    - `repo-ref/zed/crates/gpui/src/scene.rs`
    - `repo-ref/zed/crates/gpui_wgpu/src/shaders.wgsl`

- [x] SRPFR-audit-003 Confirm that `DropShadowV1` is not the correct generic replacement for
  `ShadowStyle`.
  - Result: `DropShadowV1` remains an explicit effect-owned content shadow under ADR 0286, not a
    generic box-shadow primitive for container chrome.

## B. Contract design

- [x] SRPFR-adr-010 Draft the ADR for a first-class rounded-rect shadow scene primitive.
  - Scope:
    - exact op shape,
    - ordering semantics,
    - clipping/transform semantics,
    - degradation rules for unsupported backends.
  - Landed as `docs/adr/0318-rounded-rect-shadow-scene-primitive-v1.md`.

- [x] SRPFR-adr-011 Refresh ADR alignment notes once the contract lands.
  - Expected rows:
    - `0030-shape-rendering-and-sdf-semantics.md`
    - `0060-shadows-and-elevation.md`
    - new ADR row for the primitive
  - Result: the alignment matrix now distinguishes the new core primitive skeleton from the still-
    missing renderer/default-lowering work.

## C. Core scene surface

- [x] SRPFR-core-020 Add the new shadow scene operation to `crates/fret-core/src/scene/mod.rs`.

- [x] SRPFR-core-021 Add sanitize/validation/fingerprint/replay support for the new op.

- [x] SRPFR-core-022 Add focused unit tests proving ordering and serialization/fingerprint stability.
  - Current evidence:
    - `crates/fret-core/src/scene/{mod.rs,validate.rs,fingerprint.rs}`
    - `crates/fret-core/tests/scene_state_stack_conformance.rs`

## D. Renderer implementation

- [x] SRPFR-wgpu-030 Add encoder/plumbing for the new shadow op in `fret-render-wgpu`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/shadow.rs`
    - `crates/fret-render-wgpu/src/renderer/tests.rs`

- [x] SRPFR-wgpu-031 Add a dedicated shader/pipeline path for rounded-rect shadows.
  - Result: the default wgpu renderer now evaluates rounded-rect shadow coverage analytically in
    the scene-draw quad shader branch (`quad_part_{a,b}.wgsl`) keyed by shadow-mode pipeline
    specialization.

- [x] SRPFR-wgpu-032 Add deterministic clipping and transform coverage for the new primitive.
  - Result: the primitive now rides the existing scene draw clip/mask/transform/opacity stacks,
    with focused GPU conformance in `crates/fret-render-wgpu/tests/shadow_rrect_conformance.rs`.

- [x] SRPFR-fallback-033 Decide and implement the non-native fallback path.
  - Result: the non-default degradation lane now lives at the scene layer via
    `shadow_rrect_fallback_quads(...)` and
    `SceneRecording::push_shadow_rrect_quad_fallback(...)`, while `fret-ui` retains only the
    explicit `paint_shadow_quad_fallback(...)` compatibility helper.
  - Evidence:
    - `crates/fret-core/src/scene/shadow.rs`
    - `crates/fret-ui/src/paint.rs`

## E. UI/runtime migration

- [x] SRPFR-ui-040 Change `ShadowStyle` lowering in `crates/fret-ui/src/paint.rs` to emit one shadow
  op per logical layer instead of expanding into many quads.
  - Evidence:
    - `crates/fret-ui/src/paint.rs`
    - `crates/fret-ui/src/declarative/tests/layout/container.rs`

- [x] SRPFR-ui-041 Keep the existing quad approximation only as an explicit fallback helper after the
  primitive lands.
  - Result: `crates/fret-ui/src/paint.rs` now keeps the historical multi-quad path only as
    `paint_shadow_quad_fallback(...)`; the default `paint_shadow(...)` path remains one
    `SceneOp::ShadowRRect` per logical layer.

- [x] SRPFR-ui-042 Audit first-party consumers for assumptions tied to the old expanded-quad path.
  - Result: first-party snapshot serialization and shadow-inset extraction now accept
    `SceneOp::ShadowRRect`, and targeted parity gates confirm the `Card`, `Calendar`, and `Sonner`
    consumers no longer depend on the historical expanded-quad shape.
  - Evidence:
    - `ecosystem/fret-ui-shadcn/tests/snapshots.rs`
    - `ecosystem/fret-ui-shadcn/tests/support/shadow_insets.rs`
    - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_calendar.rs`
    - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome/sonner.rs`

## F. Gates and evidence

- [x] SRPFR-test-050 Add renderer conformance for blur footprint, spread, offset, corner radii, and
  clip behavior.
  - Evidence:
    - shadow encode contract: `crates/fret-render-wgpu/src/renderer/tests.rs`
    - blur footprint + content-hole + rounded-corner shape + positive spread + clip-stack behavior:
      `crates/fret-render-wgpu/tests/shadow_rrect_conformance.rs`

- [x] SRPFR-diag-051 Extend screenshot evidence to representative elevated surfaces:
  - `Card`
  - `Calendar`
  - `Sonner`
  - `todo_demo`
  - Evidence:
    - `target/fret-diag/shadow-card-20260402/screenshots/1775089240729-ui-gallery-card-demo/window-4294967297-tick-36-frame-37.png`
    - `target/fret-diag/shadow-calendar-20260402/screenshots/1775089413759-ui-gallery-calendar-demo-shadow/window-4294967297-tick-36-frame-37.png`
    - `target/fret-diag/shadow-sonner-20260402/screenshots/1775089661544-ui-gallery-sonner-docs.01-demo/window-4294967297-tick-12-frame-13.png`
    - `target/fret-diag/shadow-todo-20260402/screenshots/1775089623373-todo-demo-shadow/window-4294967297-tick-14-frame-14.png`

- [x] SRPFR-perf-052 Add at least one perf/op-count comparison on a shadow-heavy surface.
  - Result: the focused `fret-ui` gate now proves that a representative 12-card `shadow_lg`
    surface lowers to `24` `SceneOp::ShadowRRect` entries on the primitive path versus `276`
    `SceneOp::Quad` entries on the historical fallback path.
  - Evidence:
    - `crates/fret-ui/src/paint.rs` (`paint_shadow_quad_fallback`)
    - `crates/fret-ui/src/paint.rs` (`paint_shadow_shadow_heavy_surface_reduces_scene_ops_vs_quad_fallback`)

## G. Cleanup

- [x] SRPFR-cleanup-060 Delete or demote the UI-layer multi-quad path as the default implementation
  once the renderer primitive is shipped and gated.
  - Result: `crates/fret-ui/src/paint.rs` no longer expands default `ShadowStyle` layers into many
    quads; default lowering now emits `SceneOp::ShadowRRect`.

- [x] SRPFR-cleanup-061 Update shadow workstreams so they point to the primitive lane as the next
  fidelity closure step instead of suggesting more painter tuning.
  - Result: the design workstream now reflects the landed default primitive path and records the
    historical multi-quad painter only as an explicit fallback lane.
