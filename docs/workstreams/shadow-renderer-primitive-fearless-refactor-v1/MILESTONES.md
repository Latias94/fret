# Shadow Renderer Primitive (Fearless Refactor v1) — Milestones

Status: Draft

Last updated: 2026-04-01

Related:

- Design: `docs/workstreams/shadow-renderer-primitive-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/shadow-renderer-primitive-fearless-refactor-v1/TODO.md`

## M0 - Root-cause freeze

Status note (2026-04-01): complete for the design/audit phase. The repo now records that portable
softness correction is real but insufficient, that GPUI/Zed uses a dedicated shadow primitive, and
that `DropShadowV1` is not the generic box-shadow replacement for `ShadowStyle`.

Exit criteria:

- The renderer gap is classified as structural, not token-only.
- The non-substitutability of `DropShadowV1` is documented.

## M1 - Contract lock

Status note (2026-04-01): open.

Exit criteria:

- An ADR defines the new scene primitive.
- ADR alignment is updated for the affected shadow/shape contracts.

## M2 - Renderer primitive lands

Status note (2026-04-01): open.

Exit criteria:

- `fret-core` exposes the new shadow op.
- `fret-render-wgpu` implements a dedicated path for it.
- The primitive has focused conformance coverage.

## M3 - UI migration and fallback cleanup

Status note (2026-04-01): open.

Exit criteria:

- `ShadowStyle` lowers to the new scene primitive on the default path.
- The multi-quad approximation is no longer the default implementation in `fret-ui`.
- Any remaining quad replay path is documented as explicit fallback/degradation.

## M4 - Evidence and closure

Status note (2026-04-01): open.

Exit criteria:

- Representative screenshot evidence exists for elevated surfaces.
- At least one perf/op-count comparison exists for the new primitive.
- Shadow docs point to the primitive lane as the renderer-fidelity closure path.
