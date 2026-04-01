# Shadow Renderer Primitive (Fearless Refactor v1) — TODO

Status: Draft

Last updated: 2026-04-01

Related:

- Design: `docs/workstreams/shadow-renderer-primitive-fearless-refactor-v1/DESIGN.md`
- Milestones: `docs/workstreams/shadow-renderer-primitive-fearless-refactor-v1/MILESTONES.md`

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

- [ ] SRPFR-adr-010 Draft the ADR for a first-class rounded-rect shadow scene primitive.
  - Scope:
    - exact op shape,
    - ordering semantics,
    - clipping/transform semantics,
    - degradation rules for unsupported backends.

- [ ] SRPFR-adr-011 Refresh ADR alignment notes once the contract lands.
  - Expected rows:
    - `0030-shape-rendering-and-sdf-semantics.md`
    - `0060-shadows-and-elevation.md`
    - new ADR row for the primitive

## C. Core scene surface

- [ ] SRPFR-core-020 Add the new shadow scene operation to `crates/fret-core/src/scene/mod.rs`.

- [ ] SRPFR-core-021 Add sanitize/validation/fingerprint/replay support for the new op.

- [ ] SRPFR-core-022 Add focused unit tests proving ordering and serialization/fingerprint stability.

## D. Renderer implementation

- [ ] SRPFR-wgpu-030 Add encoder/plumbing for the new shadow op in `fret-render-wgpu`.

- [ ] SRPFR-wgpu-031 Add a dedicated shader/pipeline path for rounded-rect shadows.

- [ ] SRPFR-wgpu-032 Add deterministic clipping and transform coverage for the new primitive.

- [ ] SRPFR-fallback-033 Decide and implement the non-native fallback path.
  - Requirement: fallback may replay the normalized quad approximation, but it must not keep
    `fret-ui::paint_shadow` as the default representation of box shadow.

## E. UI/runtime migration

- [ ] SRPFR-ui-040 Change `ShadowStyle` lowering in `crates/fret-ui/src/paint.rs` to emit one shadow
  op per logical layer instead of expanding into many quads.

- [ ] SRPFR-ui-041 Keep the existing quad approximation only as an explicit fallback helper after the
  primitive lands.

- [ ] SRPFR-ui-042 Audit first-party consumers for assumptions tied to the old expanded-quad path.

## F. Gates and evidence

- [ ] SRPFR-test-050 Add renderer conformance for blur footprint, spread, offset, corner radii, and
  clip behavior.

- [ ] SRPFR-diag-051 Extend screenshot evidence to representative elevated surfaces:
  - `Card`
  - `Calendar`
  - `Sonner`
  - `todo_demo`

- [ ] SRPFR-perf-052 Add at least one perf/op-count comparison on a shadow-heavy surface.
  - Goal: prove the primitive is reviewably better than quad expansion, or explicitly record the
    tradeoff if quality is improved at equal/slightly higher cost.

## G. Cleanup

- [ ] SRPFR-cleanup-060 Delete or demote the UI-layer multi-quad path as the default implementation
  once the renderer primitive is shipped and gated.

- [ ] SRPFR-cleanup-061 Update shadow workstreams so they point to the primitive lane as the next
  fidelity closure step instead of suggesting more painter tuning.
