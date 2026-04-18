# AppUi Layout Query Owner Audit — 2026-04-17

Status: Landed slice + evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEREF_COMPILE_AUDIT_2026-04-17.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_RUNTIME_GATING_AND_FRAME_OWNER_AUDIT_2026-04-17.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`
- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Scope

Decide whether the remaining `layout_query_*` pressure should stay implicit raw
`ElementContext` inheritance or become an explicit `AppUi` surface on the default app lane:

- `layout_query_bounds(...)`
- `layout_query_region_with_id(...)`
- `layout_query_region(...)`

This note does not reopen blanket `AppUi` forwarding.

## Method

1. Inspect first-party usage pressure:
   - `rg -n "layout_query_bounds\\(|layout_query_region_with_id\\(|layout_query_region\\(" apps/fret-examples apps/fret-cookbook docs ecosystem/fret -g '!target'`
2. Land the smallest framework slice:
   - add explicit `AppUi` helpers for the three layout-query surfaces,
   - carry grouped action-registration state through `layout_query_region_with_id(...)`,
   - migrate the remaining `markdown_demo` mutable host bridge to `cx.app_mut().models_mut()`,
   - add source-policy + compile-time gates.
3. Re-run the temporary local no-`Deref` audit:
   - `cargo check -p fret-examples --all-targets --message-format short`
   - `cargo check -p fret-cookbook --all-targets --message-format short`

The temporary no-`Deref` patch was reverted immediately after collecting the new failure map.

## Findings

### 1) Layout-query helpers are app-facing geometry authoring, not component/internal state substrate

Current first-party usage pressure is narrow and consistent: the concrete app-facing case is
`apps/fret-examples/src/markdown_demo.rs`, where the code:

- observes committed bounds for anchor and viewport regions, and
- creates named layout-query regions to support ordinary scrolling/anchor behavior.

That is closer to the existing `AppUi::environment_viewport_bounds(...)` owner than to raw
component/internal identity/state primitives such as `state_for(...)`, `slot_state(...)`, or
`local_model(...)`.

### 2) The region-builder closure must preserve grouped action registration

`layout_query_region_with_id(...)` cannot be a trivial forwarder.

Its nested builder still needs the same grouped `AppUi` surface as the surrounding render lane,
including `cx.actions()` and the cached action-handler installation path. The correct
implementation therefore carries:

- `action_root`,
- `action_handlers`,
- `action_handlers_used`

through the nested region builder instead of falling back to implicit `Deref`.

### 3) The no-`Deref` audit no longer reports this cluster

With a temporary local patch that removes both `impl Deref` and `impl DerefMut` from `AppUi`:

- `cargo check -p fret-examples --all-targets --message-format short` now emits `140`
  error lines,
- `cargo check -p fret-cookbook --all-targets --message-format short` now emits `22`
  error lines,
- and neither output reports:
  - `layout_query_bounds`
  - `layout_query_region_with_id`
  - `pointer_region`
  - `watch_model`
  - `cached_subtree_with`
  - `request_animation_frame`
  - `action_is_enabled`

The remaining failure clusters are now more clearly concentrated on:

- stale `cx.app` field syntax,
- helper signatures still typed as `&mut ElementContext<'_, _>`,
- and broad inherited builder/helper families such as `text`, `container`, `flex`,
  `text_props`, and `theme`.

## Evidence

- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`
- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `cargo nextest run -p fret grouped_authoring_surfaces_replace_flat_app_ui_helpers`
- `cargo nextest run -p fret app_ui_keeps_command_gating_and_animation_frame_surface_without_deref app_lane_exports_explicit_render_authoring_capability_surface`
- `cargo nextest run -p fret-examples markdown_demo_keeps_layout_query_authoring_on_app_ui_lane selected_raw_owner_examples_keep_escape_hatches_explicit`
- `cargo check -p fret-examples --all-targets --message-format short`
- temporary local no-`Deref` audit:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `cargo check -p fret-cookbook --all-targets --message-format short`

## Outcome

The repo now has an explicit owner decision for the current `layout_query_*` tail:

1. `AppUi` owns the app-facing layout-query geometry helpers.
2. `markdown_demo` stays on the ordinary app lane for anchor/scroll geometry authoring and no
   longer uses the stale `cx.app.models_mut()` field syntax.
3. The next structural work should keep focusing on helper signatures and broader builder
   families rather than re-debating `layout_query_*`.
