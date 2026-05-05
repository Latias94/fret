# ImUi Debug Draw Ellipse Primitives v1

Status: Closed narrow product follow-on
Last updated: 2026-05-05

This lane adds Dear ImGui `AddEllipse`- and `AddEllipseFilled`-style helpers to the canvas-backed
IMUI debug-draw surface. Both helpers accept a center point, x/y radii, rotation in radians, and an
explicit segment count. `segments == 0` uses Fret's stable debug-draw default instead of creating a
new auto-quality contract.

## Ownership

- `fret-ui-kit::imui` owns the debug-draw command-list API.
- `crates/fret-core` owns the vector path command vocabulary.
- `crates/fret-ui` owns Canvas path painting.
- `fret-imui` remains unchanged.

## Must-Be-True Outcomes

- Callers can emit stroked rotated ellipses.
- Callers can emit filled rotated ellipses.
- Fewer than three segments, non-positive radii, and non-finite rotation do not emit paint.
- `segments == 0` uses a stable default segment count.
- Ellipse helpers reuse the existing Canvas path stroke/fill path.

## Non-Goals

- No adaptive circle-quality policy in this lane.
- No retained path builder API.
- No draw-list channel splitting.
- No per-command hit-testing.
- No renderer tessellation contract.
