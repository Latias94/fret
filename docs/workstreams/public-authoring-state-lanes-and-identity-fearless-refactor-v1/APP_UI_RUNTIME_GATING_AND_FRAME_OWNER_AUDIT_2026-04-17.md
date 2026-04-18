# AppUi Runtime Gating and Frame Owner Audit — 2026-04-17

Status: Landed slice + follow-up evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEREF_COMPILE_AUDIT_2026-04-17.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`

## Scope

Classify one narrow subset of the remaining `AppUi`-vs-`ElementContext` pressure after the
2026-04-17 compile audit:

- explicit command-gating reads/dispatch (`fret::actions::ElementCommandGatingExt`),
- one-shot frame progression (`request_animation_frame()`),
- and the immediate neighboring raw method-family tail (`pointer_region`,
  `cached_subtree_with`, `watch_model`, `layout_query_*`).

This note does not reopen blanket `AppUi` forwarding.

## Method

1. Inspect current first-party usage pressure:
   - `rg -n "action_is_enabled\\(|request_animation_frame\\(|layout_query_bounds\\(|layout_query_region_with_id\\(|pointer_region\\(|watch_model\\(|cached_subtree_with\\(" apps/fret-examples apps/fret-cookbook`
2. Land the smallest framework slice:
   - add inherent `AppUi::request_animation_frame()`,
   - implement `fret_ui_kit::command::ElementCommandGatingExt` directly for `AppUi`,
   - add source-policy and compile-time gates.
3. Re-run the “what if `AppUi` lost `Deref` today?” audit with a temporary local patch that
   removes both `impl Deref` and `impl DerefMut`, then run:
   - `cargo check -p fret-examples --all-targets --message-format short`
   - `cargo check -p fret-cookbook --all-targets --message-format short`

The temporary patch was reverted immediately after collecting the new failure clusters.

## Findings

### 1) Command gating and frame requests are correct `AppUi` owners

Both surfaces fit the ordinary app-facing lane:

- `request_animation_frame()` is a small runtime control helper for frame-driven progression.
  It does not reopen component/internal identity or raw builder plumbing.
- `fret::actions::ElementCommandGatingExt` is already documented as an explicit import lane in
  `docs/crate-usage-guide.md`; the missing piece was that `AppUi` still reached it only through
  the temporary `Deref` bridge.

The landed slice therefore keeps the explicit teaching posture while removing hidden bridge
dependence.

### 2) The new slice actually removed that pressure from the no-`Deref` failure map

After landing the slice and temporarily disabling `AppUi` `Deref` / `DerefMut`:

- `cargo check -p fret-examples --all-targets --message-format short`
- `cargo check -p fret-cookbook --all-targets --message-format short`

no longer emitted “method not found” failures for either:

- `request_animation_frame`
- `action_is_enabled`

The remaining clusters moved elsewhere:

- stale helper signatures still typed as `&mut ElementContext<'_, _>`,
- stale `cx.app` field syntax,
- broad inherited builder helpers such as `text`, `container`, `flex`, `text_props`, `theme`,
- and a smaller raw-owner tail such as `pointer_region`, `cached_subtree_with`, and
  `watch_model`.

### 3) The neighboring raw-owner tail should stay explicit for now

The current evidence does **not** justify copying the remaining raw methods onto `AppUi`.

The strongest current owner split is:

- keep explicit raw/advanced:
  - `pointer_region`
  - `cached_subtree_with`
  - current `watch_model` call sites
- still needs a narrower owner decision:
  - `layout_query_bounds`
  - `layout_query_region_with_id`
- do not blanket-forward onto `AppUi`:
  - `text`
  - `container`
  - `flex`
  - `text_props`
  - `theme`

This keeps the next step focused on real authoring-surface design rather than recreating
`ElementContext` under a façade name.

## Evidence

- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`
- `cargo nextest run -p fret grouped_authoring_surfaces_replace_flat_app_ui_helpers`
- `cargo nextest run -p fret app_ui_keeps_command_gating_and_animation_frame_surface_without_deref app_lane_exports_explicit_render_authoring_capability_surface`
- temporary local no-`Deref` audit:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `cargo check -p fret-cookbook --all-targets --message-format short`

## Outcome

The repo now has one smaller but important explicit `AppUi` surface slice that no longer depends
on the temporary bridge:

1. `request_animation_frame()` is explicitly owned by `AppUi`.
2. Explicit command-gating imports now work on `AppUi` directly.
3. The next work should focus on the remaining raw-owner and helper-signature buckets rather than
   reopening blanket forwarding or immediate `Deref` deletion.
