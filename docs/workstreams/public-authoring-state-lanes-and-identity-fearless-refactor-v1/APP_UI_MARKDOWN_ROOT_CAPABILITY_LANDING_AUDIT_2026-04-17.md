# AppUi Markdown Root Capability Landing Audit — 2026-04-17

Status: Landed slice + targeted evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_LAYOUT_QUERY_OWNER_AUDIT_2026-04-17.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_ASYNC_PLAYGROUND_HELPER_CAPABILITY_AUDIT_2026-04-17.md`
- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Scope

Decide how the remaining `markdown_demo` no-`Deref` failures should be classified:

- should the markdown app root fall back to raw `ElementContext`,
- should nested `layout_query_region_with_id(...)` closures reopen a raw late-landing lane,
- or should ordinary root/layout-query shell builders stay on the existing capability-first
  `AppUi` lane?

This note is intentionally about ordinary late-landing at an app-facing root, not the explicit raw
`spinner_box` helper inside the markdown image hook.

## Method

1. Inspect the post-`async_playground_demo` no-`Deref` failure map and confirm that
   `apps/fret-examples/src/markdown_demo.rs` still fails on:
   - root `toggles` landing,
   - nested `layout_query_region_with_id(...)` shell landing,
   - root `content` landing,
   - outer page-shell landing.
2. Keep the existing app-lane behavior unchanged:
   - transient invalidation and action registration on `AppUi`
   - `LocalState::{layout_value, layout_read_ref}`
   - `AppUi::{layout_query_bounds, layout_query_region_with_id}`
   - explicit raw `spinner_box` helper typed as `UiCx<'_>` inside the image hook
3. Move only the ordinary late-landing tail onto the existing capability lane:
   - import `fret_ui_kit::IntoUiElementInExt as _`
   - use `into_element_in(cx)` for:
     - `toggles`
     - the inner markdown container in the nested layout-query closure
     - the `ScrollArea` shell in that same closure
     - `content`
     - the outer page shell
4. Re-run:
   - `cargo test -p fret-examples --lib markdown_demo_keeps_layout_query_authoring_on_app_ui_lane`
   - `cargo test -p fret-examples --lib markdown_demo_prefers_capability_first_landing_for_root_and_layout_query_shells`
   - `cargo check -p fret-examples --all-targets --message-format short`
5. Re-run a temporary local no-`Deref` spot-check:
   - `cargo check -p fret-examples --all-targets --message-format short`

The temporary `AppUi` no-`Deref` patch was reverted immediately after collecting the new failure
map.

## Findings

### 1) `markdown_demo` was root/layout-query late-landing debt, not a raw-owner surface

The remaining failures all lived at ordinary app-owned builder landing points:

- `toggles`
- nested layout-query shell/container landing
- `content`
- outer page shell

The explicit raw helper in this file was already clearly marked:

- `let spinner_box = |cx: &mut UiCx<'_>| ...`

So the correct interpretation was not “convert the whole file to `cx.elements()`”; it was “stop
letting ordinary `AppUi` builders lean on implicit `Deref`”.

### 2) The correct fix is capability-first landing on both the root and nested layout-query shell

The owner-correct shape is now:

- root state/effect/layout-query work stays on `AppUi`
- ordinary late-landing uses `into_element_in(cx)`
- nested `layout_query_region_with_id(...)` closures keep using the same app-facing lane
- the image-hook raw helper remains explicit and unchanged

That matches the framework’s intended split: app-lane geometry/query ownership without reopening
raw `ElementContext` just to land ordinary builders.

### 3) The no-`Deref` spot-check removes `markdown_demo` and shrinks the tail again

With a temporary local patch that removes both `impl Deref` and `impl DerefMut` from `AppUi`:

- `fret-examples` drops from `35` to `30` previous errors,
- `markdown_demo` no longer appears in the failure map,
- and the current leading `fret-examples` clusters are now:
  - `editor_notes_demo.rs` (`4`)
  - `hello_world_compare_demo.rs` (`4`)
  - `drop_shadow_demo.rs` (`3`)
  - `ime_smoke_demo.rs` (`3`)
  - `query_async_tokio_demo.rs` (`3`)
  - `query_demo.rs` (`3`)
  - `sonner_demo.rs` (`3`)

That is strong evidence that this was another capability-first late-landing cleanup slice rather
than a missing framework surface.

## Evidence

- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `cargo test -p fret-examples --lib markdown_demo_keeps_layout_query_authoring_on_app_ui_lane`
- `cargo test -p fret-examples --lib markdown_demo_prefers_capability_first_landing_for_root_and_layout_query_shells`
- `cargo check -p fret-examples --all-targets --message-format short`
- temporary local no-`Deref` spot-check:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `/tmp/fret_examples_noderef_markdown_v5.txt`

## Outcome

The repo now has an explicit owner answer for the markdown app root:

1. root state/effect/layout-query work stays on `AppUi`
2. nested `layout_query_region_with_id(...)` closures stay on the same app-facing lane
3. ordinary root and nested shell late-landing use `into_element_in(cx)`
4. the image-hook raw helper remains explicit at `UiCx<'_>`
