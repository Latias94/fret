# ADR 0103: Layout-Driven Anchored Overlays (Render-Transform Placement)

Status: Accepted  
Date: 2026-01-06

## Context

Fret currently positions many overlays (dropdown menus, selects, tooltips, popovers) by:

1. Choosing a desired panel size up-front (often based on the trigger width or an arbitrary max),
2. Running the placement solver (`fret_ui::overlay_placement`) to compute a `placed: Rect`,
3. Rendering an absolute-positioned wrapper/panel using that `placed` rect.

This works well for fixed-size overlays but falls short when we want Radix/shadcn-style outcomes:

- **Intrinsic sizing** (width/height determined by content, subject to `min-*`/`max-*`),
- **One-frame correctness** (no “first frame wrong, second frame corrected” jitter),
- **Correct hit-testing and outside-press semantics** under placement transforms,
- **Arrow layout** that depends on the final panel rect.

The key missing substrate is a mechanism to place an overlay *after* the child subtree has been
measured by layout, without introducing a separate offscreen measure pass.

GPUI solves this class of problems with an “anchored” element that:

- Measures its children first,
- Computes a final position that avoids overflowing the window,
- Applies a runtime offset for paint and input.

Fret already supports the critical substrate primitive for this approach:

- `Widget::render_transform(bounds) -> Option<Transform2D>` applies an affine transform to both
  painting and input mapping (hit-testing and pointer event coordinates), while keeping layout
  bounds authoritative.

Zed/GPUI code anchors (non-normative):

- `repo-ref/zed/crates/gpui/src/elements/anchored.rs` (`Anchored`, `anchored()`)
- Usage in menu/popover surfaces:
  - `repo-ref/zed/crates/ui/src/components/popover_menu.rs`
  - `repo-ref/zed/crates/ui/src/components/right_click_menu.rs`

## Decision

Introduce a retained/declarative mechanism-level primitive: **`Anchored`**.

`Anchored`:

- Is a pass-through layout wrapper (like `Opacity`), but **computes a placement transform during
  layout** based on its child subtree’s intrinsic size and a placement policy.
- Applies the placement using `Widget::render_transform`, so the subtree is moved for paint and
  input without affecting layout.
- Optionally writes the computed `AnchoredPanelLayout` to a model so sibling elements (e.g. arrow
  visuals) can align themselves using the same-frame placement result.

This is a mechanism-level decision: component crates (ui-kit / shadcn) should not implement their
own “probe render then re-place” loops. They should rely on `Anchored` for fit-content overlays.

## Non-Goals

- Replacing the existing `fret-ui-kit` overlay controller.
- Introducing a full “two-pass layout” engine in `fret-ui`.
- Implementing a CSS-like `fit-content` length in the core layout model.

## Design

### Inputs

`Anchored` takes:

- `layout`: the wrapper’s layout style (default should be full-window `Fill`).
- `outer_margin`: insets applied to the outer window bounds prior to placement (equivalent to a
  window margin).
- `anchor`: the anchor rect in the same coordinate space as `outer` (window-local logical px).
- Optional: `anchor_element`: a declarative element ID to resolve during layout. When set, the
  layout pass prefers the element's **current-frame** layout bounds as the anchor rect (falling
  back to `anchor` when unavailable). This avoids cross-frame geometry delay from
  `bounds_for_element(...)` / `last_bounds_for_element(...)` queries and better matches GPUI's
  layout-driven placement model. Note that this resolves **layout bounds** only (ADR 0082): if you
  need post-`render_transform` anchoring, provide `anchor` from `visual_bounds_for_element(...)`.
- `side`, `align`, `side_offset`: the primary placement policy.
- `options: AnchoredPanelOptions`: direction, cross-axis offset, and optional arrow options.
- `layout_out: Option<Model<AnchoredPanelLayout>>`: optional output model updated during layout.

### Algorithm

During `layout`:

1. Measure/layout children at the wrapper origin using the available size budget.
2. Compute the subtree intrinsic `desired: Size` as the max non-absolute child size.
3. Compute `outer = inset_rect(cx.bounds, outer_margin)`.
4. Run `anchored_panel_layout_sized_ex(outer, anchor, desired, side_offset, side, align, options)`.
5. Derive `transform = translation(layout.rect.origin - cx.bounds.origin)` and store it on the
   host widget so `render_transform` returns it.
6. If `layout_out` is provided, update the model with the computed `AnchoredPanelLayout`.

### Invariants

- Placement **must not** affect layout.
- Placement **must** affect hit-testing and pointer coordinate mapping (via `render_transform`).
- Placement results must be available in the same frame for paint (model output update happens in
  layout).
- `Anchored` should remain a low-level primitive; styling and chrome remain in component crates.

## Consequences

### Benefits

- Enables true “content-sized” overlays that still avoid window edges.
- Avoids first-frame placement jitter without introducing an offscreen measurement pass.
- Uses existing runtime semantics for transforms and outside-press observation (ADR 0082).

### Costs / Risks

- Subtrees under `render_transform` may opt out of paint caching (acceptable for overlays).
- Authors must avoid relying on “layout position” for overlay placement; instead, use `Anchored`.

## Adoption Plan

1. Land `Anchored` in `crates/fret-ui` + tests.
2. Add ui-kit helpers that mirror Radix/shadcn patterns (popper wrapper + panel under `Anchored`).
3. Migrate high-impact overlays in `ecosystem/fret-ui-shadcn` (`DropdownMenu`, `Select`, `Tooltip`)
   to use intrinsic sizing under `Anchored` (while keeping existing APIs stable where possible).
