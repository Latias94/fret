# AppUi Continuous Frames Runtime Owner Audit — 2026-04-17

Status: Landed slice + targeted evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_RUNTIME_GATING_AND_FRAME_OWNER_AUDIT_2026-04-17.md`
- `apps/fret-examples/src/hello_world_compare_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`

## Scope

Decide whether the remaining `hello_world_compare_demo` no-`Deref` pressure should:

- keep using the raw `set_continuous_frames(...)` helper on `ElementContext`,
- widen `AppUi` with a narrow runtime-control helper,
- or reopen a raw text / raw late-landing lane at the app root.

This note is intentionally about one narrow runtime-control owner question plus the matching proof
surface cleanup in `hello_world_compare_demo`.

## Method

1. Inspect the post-`editor_notes_demo` no-`Deref` failure map and confirm that
   `apps/fret-examples/src/hello_world_compare_demo.rs` still fails on:
   - `set_continuous_frames(cx, ...)`
   - root `layout_probe` late-landing
   - root `swatch_row` late-landing
   - root `text_props(...)` title authoring
   - root shell late-landing through `hello_world_compare_root(...)`
2. Classify the surface by owner:
   - `set_continuous_frames(...)` is the same runtime-control family as
     `request_animation_frame()`
   - closure-local swatches stay on `AppRenderCx<'_>`
   - title/layout/root landing are ordinary app-facing builders, not raw-owner proof
3. Land the smallest framework + proof slice:
   - add `AppUi::set_continuous_frames(enabled)`
   - keep the free helper unchanged for raw/advanced surfaces
   - move `hello_world_compare_demo` onto:
     - `cx.set_continuous_frames(...)`
     - `ui::text(...)` instead of raw `cx.text_props(...)`
     - `into_element_in(cx)` for `layout_probe`, `swatch_row`, and the root shell
4. Re-run:
   - `cargo test -p fret-examples --lib closure_local_app_facing_helpers_can_use_app_render_cx_alias`
   - `cargo test -p fret --test render_authoring_capability_surface app_lane_exports_explicit_render_authoring_capability_surface`
   - `cargo test -p fret app_ui_keeps_command_gating_and_animation_frame_surface_without_deref`
   - `cargo check -p fret-examples --all-targets --message-format short`
5. Re-run a temporary local no-`Deref` spot-check:
   - disable `impl Deref` / `impl DerefMut` for `AppUi` temporarily in `ecosystem/fret/src/view.rs`
   - `cargo check -p fret-examples --all-targets --message-format short`

The temporary `AppUi` no-`Deref` patch was reverted immediately after collecting the new failure
map.

## Findings

### 1) Continuous frame leases are the same owner class as other app-facing runtime controls

`set_continuous_frames(...)` is not raw text, raw builder plumbing, or component-internal state
substrate. It is a runtime-control helper for frame delivery while a mode remains active.

That puts it in the same owner family as the already-landed app-facing runtime helpers:

- `request_animation_frame()`
- explicit command gating on `AppUi`

Keeping this callsite on raw `ElementContext` would only preserve accidental bridge dependence.

### 2) `hello_world_compare_demo` splits cleanly into closure-local carrier, ordinary builders, and one runtime helper

The owner-correct shape is now:

- runtime control on `AppUi`:
  - `cx.set_continuous_frames(...)`
  - `cx.request_animation_frame()`
- closure-local helper carrier stays explicit:
  - `let swatch = |_cx: &mut AppRenderCx<'_>, ...|`
- ordinary app-root builders stay capability-first:
  - `ui::text(...)` title builder
  - `layout_probe`
  - `swatch_row`
  - `hello_world_compare_root(...)` root shell via `into_element_in(cx)`

This keeps the app-facing lane narrower and more coherent than either widening raw text helpers or
reopening `cx.elements()` at the root.

### 3) The no-`Deref` spot-check removes `hello_world_compare_demo` and shrinks the tail again

With a temporary local patch that removes both `impl Deref` and `impl DerefMut` from `AppUi`:

- `fret-examples` drops from `26` to `22` previous errors,
- `hello_world_compare_demo` no longer appears in the failure map,
- and the current leading `fret-examples` clusters are now:
  - `drop_shadow_demo.rs` (`3`)
  - `ime_smoke_demo.rs` (`3`)
  - `query_async_tokio_demo.rs` (`3`)
  - `query_demo.rs` (`3`)
  - `sonner_demo.rs` (`3`)
  - `components_gallery.rs` (`2`)

That is strong evidence that this slice closed one real app-facing runtime gap and one ordinary
late-landing/text-builder tail instead of papering over the problem.

## Evidence

- `apps/fret-examples/src/hello_world_compare_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`
- `cargo test -p fret-examples --lib closure_local_app_facing_helpers_can_use_app_render_cx_alias`
- `cargo test -p fret --test render_authoring_capability_surface app_lane_exports_explicit_render_authoring_capability_surface`
- `cargo test -p fret app_ui_keeps_command_gating_and_animation_frame_surface_without_deref`
- `cargo check -p fret-examples --all-targets --message-format short`
- temporary local no-`Deref` spot-check:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `/tmp/fret_examples_noderef_hello_world_compare_v7.txt`

## Outcome

The repo now has an explicit owner answer for continuous frame leases on the app-facing lane:

1. `AppUi` owns `set_continuous_frames(enabled)` as a narrow runtime-control helper.
2. `hello_world_compare_demo` keeps closure-local helpers on `AppRenderCx<'_>`.
3. Title/root/layout-probe/swatch-row builder landing stays on `into_element_in(cx)` plus
   `ui::text(...)`.
4. `hello_world_compare_demo` is no longer part of the current no-`Deref` failure tail.
