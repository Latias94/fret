# Components Gallery Owner Split Audit — 2026-04-16

Status: landed follow-on audit
Last updated: 2026-04-16

Related:

- `TODO.md`
- `MILESTONES.md`
- `apps/fret-examples/src/components_gallery.rs`
- `apps/fret-examples/src/lib.rs`
- `APP_DRIVER_RAW_MODEL_OWNER_AUDIT_2026-04-15.md`

## Why this note exists

`components_gallery` is not one unresolved raw-model bucket anymore.

After the app/driver raw-owner freeze landed, this file remained as the next mixed-owner follow-on
because it combines:

- retained render owner state inside the table-torture subtree,
- app/theme sync owner logic,
- and driver/event owner aggregation for routing keyboard and overlay events.

This audit records which part should move and which part should stay explicit.

## Assumptions

1. The table-torture subtree is still render-time retained authoring, even though it intentionally
   uses lower-level retained helpers.
   Evidence: `render_gallery(...)` builds the table inside `render_root_with_app_ui(...)`,
   `cached_subtree_with(...)`, and a `SemanticsProps` render closure.
   Confidence: Confident.
   Consequence if wrong: moving the revision read onto a tracked builder would hide an intentional
   driver/runtime surface.

2. The theme preset sync path is app-owned orchestration, not render-time app-lane authoring.
   Evidence: `sync_gallery_shadcn_theme(...)` owns `&mut App`, mutates installed shadcn theme
   globals, and compares against `state.applied_theme_preset`.
   Confidence: Confident.
   Consequence if wrong: we would push app/theme orchestration onto render-only helpers.

3. Overlay-open aggregation and tree-key handling are driver/event owners.
   Evidence: both run from `handle_event(...)` / `handle_tree_key_event(...)`, outside render
   closures, and directly decide event dispatch or selection mutation before the UI tree handles
   the event.
   Confidence: Confident.
   Consequence if wrong: we would blur render ownership with event-loop orchestration.

4. No new framework API is required for this slice.
   Evidence: `TrackedStateExt` already exposes `table_state.layout(cx).revision()` for the
   retained render case, while raw `app.models()` remains available for app/driver owners.
   Confidence: Confident.
   Consequence if wrong: this slice would widen API surface instead of tightening owner language.

## Owner resolution

### 1. Retained render owner

Owner: tracked builder on the render surface.

Decision:

- the table-torture subtree should read table-state revision with
  `table_state.layout(cx).revision().unwrap_or(0)`.

Why:

- the read happens inside render-time retained composition,
- `TrackedStateExt` already owns revision observation for explicit `Model<T>` handles,
- and raw `cx.app.models().revision(...)` is unnecessary drift here.

### 2. App/theme sync owner

Owner: helper-owned raw app store access.

Decision:

- keep theme preset reads on raw `app.models()` through
  `ComponentsGalleryWindowState::selected_theme_preset(app)`.

Why:

- this path owns installed-theme synchronization, not render-time composition,
- and it does not have a render context to justify tracked render helpers.

### 3. Driver/event owner

Owner: helper-owned raw app store access.

Decision:

- keep overlay-open aggregation on raw `app.models()` through
  `ComponentsGalleryWindowState::overlays_open(app)`,
- keep `handle_tree_key_event(...)` raw `get_cloned(...)` reads as app/event owner code.

Why:

- these paths decide dispatch/routing in the driver event loop,
- and they intentionally operate before or outside render invalidation ownership.

## Landed result

This audit lands:

- `components_gallery.rs` now uses `table_state.layout(cx).revision()` for the retained render
  owner,
- theme sync and overlay aggregation now route through explicit demo-local owner helpers
  (`selected_theme_preset(app)`, `overlays_open(app)`),
- source-policy tests lock the split and forbid the old retained render raw revision read from
  drifting back.

## Decision from this audit

Treat `components_gallery` as a three-owner proof surface:

- retained render owner,
- app/theme sync owner,
- driver/event owner.

Do not collapse those into one grep-based “raw models debt” bucket.
