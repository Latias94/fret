# AppUi Async Playground Helper Capability Audit — 2026-04-17

Status: Landed slice + targeted evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_TODO_ROOT_CAPABILITY_LANDING_AUDIT_2026-04-17.md`
- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Scope

Decide how the remaining `async_playground_demo` no-`Deref` failures should be classified:

- should this root widen `AppUi` again,
- should the whole surface collapse into a raw `ElementContext` owner,
- or should the remaining named helpers move onto the existing app-facing capability lane while
  keeping only the true raw leaf explicit?

This note is intentionally about helper-signature debt on an ordinary app-facing surface, not a
new raw-owner exception.

## Method

1. Inspect the post-`todo_demo` no-`Deref` failure map and confirm that
   `apps/fret-examples/src/async_playground_demo.rs` still fails on:
   - `tracked_query_inputs(cx, ...)` at the `AppUi` root,
   - root helper entry (`header_bar(...)`, `body(...)`),
   - and root late-landing.
2. Keep root reads/effects on `AppUi`:
   - `LocalState::{layout_value,...}`
   - `cx.data().invalidate_query(...)`
   - `cx.data().cancel_query(...)`
   - `cx.data().invalidate_query_namespace(...)`
   - `cx.theme_snapshot()`
3. Migrate named helpers that only need grouped state/query/late-landing onto the existing
   default helper lane:
   - use `fret::app::AppRenderContext<'a>` for helper signatures,
   - use `IntoUiElementInExt::into_element_in(cx)` for ordinary late-landing,
   - use `handle.read_layout(cx)`, `layout_read_ref(cx)`, and `layout_value(cx)` on the same
     capability lane.
4. Keep the one real raw leaf explicit instead of widening `AppUi`:
   - `catalog_item(...)` still spells `cx.elements().pressable(...)`.
5. Re-run:
   - `cargo test -p fret-examples --lib async_playground_demo_prefers_app_render_context_helpers_and_root_capability_landing`
   - `cargo test -p fret-examples --lib advanced_helper_contexts_prefer_uicx_aliases`
   - `cargo test -p fret-examples --lib selected_app_ui_roots_prefer_explicit_render_context_accessors_over_deref`
   - `cargo check -p fret-examples --all-targets --message-format short`
6. Re-run the temporary local no-`Deref` compile audit:
   - `cargo check -p fret-examples --all-targets --message-format short`

The temporary `AppUi` no-`Deref` patch was reverted immediately after collecting the new failure
map.

## Findings

### 1) `async_playground_demo` was helper-signature debt, not a raw-owner proof

The remaining failures were not evidence that the whole demo wanted raw `ElementContext`.

They clustered on:

- named helper signatures that still required `UiCx<'_>`,
- grouped query/layout reads that still assumed the raw carrier,
- and ordinary root late-landing.

That is app-facing helper-surface debt, not a signal to widen `AppUi`.

### 2) The correct fix is `AppRenderContext<'a>` plus capability-first landing

The owner-correct implementation keeps the root honest:

- root reads/effects stay on `AppUi`,
- named helpers move to `fret::app::AppRenderContext<'a>`,
- ordinary late-landing uses `into_element_in(cx)`,
- grouped query/layout helpers stay on `cx.data()` / tracked-read helpers,
- and the one real raw leaf stays explicit at `cx.elements().pressable(...)`.

This preserves the default app-facing helper story instead of teaching `UiCx<'_>` as the normal
named-helper carrier on a first-party app root.

### 3) The no-`Deref` spot-check shrinks again and removes `async_playground_demo`

With a temporary local patch that removes both `impl Deref` and `impl DerefMut` from `AppUi`:

- `fret-examples` drops from `41` to `35` previous errors,
- `async_playground_demo` no longer appears in the failure map,
- and the current leading `fret-examples` clusters are now:
  - `markdown_demo.rs` (`5`)
  - `editor_notes_demo.rs` (`4`)
  - `hello_world_compare_demo.rs` (`4`)
  - `drop_shadow_demo.rs` (`3`)
  - `ime_smoke_demo.rs` (`3`)
  - `query_async_tokio_demo.rs` (`3`)
  - `query_demo.rs` (`3`)
  - `sonner_demo.rs` (`3`)

That is strong evidence that `async_playground_demo` belonged on the app-facing helper-capability
lane, not on a reopened raw-owner lane.

## Evidence

- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `cargo test -p fret-examples --lib async_playground_demo_prefers_app_render_context_helpers_and_root_capability_landing`
- `cargo test -p fret-examples --lib advanced_helper_contexts_prefer_uicx_aliases`
- `cargo test -p fret-examples --lib selected_app_ui_roots_prefer_explicit_render_context_accessors_over_deref`
- `cargo check -p fret-examples --all-targets --message-format short`
- temporary local no-`Deref` spot-check:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `/tmp/fret_examples_noderef_async_v4.txt`

## Outcome

The repo now has an explicit answer for the async playground helper tail:

1. root state/effect/theme work stays on `AppUi`
2. named app-facing helpers prefer `fret::app::AppRenderContext<'a>`
3. ordinary root/helper late-landing uses `into_element_in(cx)`
4. the remaining raw leaf stays explicit at `cx.elements().pressable(...)`
