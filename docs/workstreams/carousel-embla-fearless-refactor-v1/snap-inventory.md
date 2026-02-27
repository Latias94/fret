# Snap-like Behaviors Inventory (P4 / CAR-410)

This note inventories "snap-like" behavior patterns in-tree and classifies them so we can share
small headless helpers without conflating semantics.

Non-goal: unify all snapping into one "universal snap model".

## Taxonomy

### A) Scroll / track snaps (Embla-like)

- Snaps are derived from slide geometry and optional containment rules.
- Used for prev/next buttons and drag-release targeting.

### B) Drag settle snap points (sheet-like)

- Snap points are supplied explicitly (often as fractions of a container).
- Drag release selects a target (sometimes with inertia / fling projection).
- May include dismiss thresholds in addition to snapping.

### C) Quantized value snapping (step/ticks)

- Values are clamped and snapped to a step grid.
- Typically updates state (values) rather than translating a track.

### D) Pixel snapping (rendering quantization)

- Rounds rendering geometry to device pixel boundaries.
- Not an interaction snap (out of scope for this workstream).

## In-tree inventory (evidence anchors)

### Carousel — scroll/track snaps (A)

- Headless snap model: `ecosystem/fret-ui-headless/src/carousel.rs` (`snap_model_1d`)
- Contract doc: `docs/workstreams/carousel-embla-fearless-refactor-v1/snap-model-contract.md`
- Recipe integration: `ecosystem/fret-ui-shadcn/src/carousel.rs`
- Recipe-level options (policy-only): `ecosystem/fret-ui-shadcn/src/carousel.rs` (`CarouselOptions`)

Notes:

- Embla-aligned vocabulary: `align`, `containScroll`, `slidesToScroll`, `pixelTolerance`.
- Looping is intentionally not implemented in v1 (containment disabled when `loop_enabled=true`).

### Drawer — drag settle snap points (B)

- Snap points API: `ecosystem/fret-ui-shadcn/src/drawer.rs` (`Drawer::snap_points`)
- Release behavior: `ecosystem/fret-ui-shadcn/src/drawer.rs` (release handler computes `targets`,
  picks nearest, and settles via spring animation)
- Regression: `ecosystem/fret-ui-shadcn/src/drawer.rs` (`drawer_snap_points_settle_to_nearest_point_on_release`)

Notes:

- Drawer snapping operates in **offset space** (target offsets), derived from snap fractions and
  the computed drawer height.
- It additionally has a close/dismiss threshold which is not part of a generic "snap points" helper.
- Nearest target selection uses the shared headless helper:
  - `ecosystem/fret-ui-headless/src/snap_points.rs` (`closest_value_px`)

### Slider — quantized value snapping (C)

- Headless slider math: `ecosystem/fret-ui-headless/src/slider.rs`
  - `snap_value`, `closest_value_index`, `update_multi_thumb_values`, etc.
- shadcn recipe uses the Radix-aligned headless slider primitives:
  - `ecosystem/fret-ui-shadcn/src/slider.rs` (`fret_ui_kit::primitives::slider`)

Notes:

- This is already a shared headless surface and is a good reference for how we structure small,
  deterministic helpers + tests.

### Material3 Top App Bar — threshold snapping (B-ish)

- Settle policy: `ecosystem/fret-ui-material3/src/top_app_bar.rs`
  - `TopAppBarSettlePolicy` (`snap_threshold`) + idle settle behavior

Notes:

- This is a two-state snap based on a fraction threshold, not a list-of-points snap model.

### Pixel snapping (D) — out of scope

- Render quantization flags: `crates/fret-ui/src/element.rs` (`snap_to_device_pixels`)

## Candidate shared helpers (CAR-420)

If duplication persists (e.g. multiple components pick a "nearest target" from an explicit point
list), factor the smallest pure helper(s) into `fret-ui-headless` and re-export via
`fret-ui-kit::headless`.

Status:

- Implemented minimal `Px` closest-point helpers in `ecosystem/fret-ui-headless/src/snap_points.rs`
  and re-exported via `ecosystem/fret-ui-kit/src/headless/mod.rs`.

Candidate functions (examples, not commitments):

- `closest_index_px(points: &[Px], target: Px) -> usize`
- `closest_value_px(points: &[Px], target: Px) -> Option<Px>`
- `next_prev_index(points_len: usize, current: usize, dir: i32) -> usize` (wrap/no-wrap as policy)

Constraints:

- Must remain coordinate-space agnostic (callers map their own spaces).
- Must not depend on `fret-dnd` or UI runtime types.
- Must come with unit tests that lock behavior.
