# Shadow Renderer Primitive (Fearless Refactor v1) — Milestones

Status: Complete (primitive default path, fallback closure, and evidence lane landed)

Last updated: 2026-04-02

Related:

- Design: `docs/workstreams/shadow-renderer-primitive-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/shadow-renderer-primitive-fearless-refactor-v1/TODO.md`
- ADR: `docs/adr/0318-rounded-rect-shadow-scene-primitive-v1.md`

## M0 - Root-cause freeze

Status note (2026-04-01): complete for the design/audit phase. The repo now records that portable
softness correction is real but insufficient, that GPUI/Zed uses a dedicated shadow primitive, and
that `DropShadowV1` is not the generic box-shadow replacement for `ShadowStyle`.

Exit criteria:

- The renderer gap is classified as structural, not token-only.
- The non-substitutability of `DropShadowV1` is documented.

## M1 - Contract lock

Status note (2026-04-01): in progress. The first ADR draft now exists and the related ADR alignment
notes are refreshed, but the contract is not yet accepted.

Exit criteria:

- An ADR defines the new scene primitive.
- ADR alignment is updated for the affected shadow/shape contracts.

## M2 - Renderer primitive lands

Status note (2026-04-01): complete for the default path. `fret-core` exposes the new shadow op, the
default wgpu renderer now renders it through an analytic rounded-rect shadow branch, and focused
conformance exists in `crates/fret-render-wgpu/tests/shadow_rrect_conformance.rs`.

Exit criteria:

- `fret-core` exposes the new shadow op.
- `fret-render-wgpu` implements a dedicated path for it.
- The primitive has focused conformance coverage.

## M3 - UI migration and fallback cleanup

Status note (2026-04-02): complete for the UI/default-path lane. `ShadowStyle` now lowers to the
new scene primitive, the old UI-layer multi-quad expansion is no longer the default path, and the
historical approximation now survives only as the explicit `paint_shadow_quad_fallback(...)` helper.

Exit criteria:

- `ShadowStyle` lowers to the new scene primitive on the default path.
- The multi-quad approximation is no longer the default implementation in `fret-ui`.
- Any remaining quad replay path is documented as explicit fallback/degradation.

## M4 - Evidence and closure

Status note (2026-04-02): complete. The op-count/perf evidence is landed (`24 ShadowRRect` ops vs
`276` fallback quads on a representative 12-card `shadow_lg` surface), representative screenshot
evidence now exists for `Card`, `Calendar`, `Sonner`, and `todo_demo`, the explicit scene-level
fallback helper is documented, and the first-party consumer audit is closed through snapshot +
shadow-inset consumers plus targeted `Card` / `Calendar` / `Sonner` parity gates.

Exit criteria:

- Representative screenshot evidence exists for elevated surfaces.
- At least one perf/op-count comparison exists for the new primitive.
- Shadow docs point to the primitive lane as the renderer-fidelity closure path.
