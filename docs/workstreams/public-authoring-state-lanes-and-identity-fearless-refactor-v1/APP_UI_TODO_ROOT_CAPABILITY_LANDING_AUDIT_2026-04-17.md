# AppUi Todo Root Capability Landing Audit — 2026-04-17

Status: Landed slice + targeted evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEFAULT_TEXT_BUILDER_SURFACE_AUDIT_2026-04-17.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_EMBEDDED_VIEWPORT_INTEROP_CAPABILITY_AUDIT_2026-04-17.md`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Scope

Decide how the remaining `todo_demo` no-`Deref` failures should be classified:

- should the default Todo app root enter a broad raw `ElementContext` lane,
- should `AppUi` widen again,
- or should ordinary app-facing root builders keep using the existing capability-first
  late-landing lane?

This note is intentionally about a default app-facing root, not a raw-owner proof.

## Method

1. Inspect the post-`date_picker_demo` / `embedded_viewport_demo` no-`Deref` failure map and
   confirm that `apps/fret-examples/src/todo_demo.rs` is now the largest remaining cluster.
2. Keep render-time state/theme/responsive reads on `AppUi`:
   - `TodoLocals::new(cx)`
   - `locals.bind_actions(cx)`
   - viewport/environment queries
   - tracked `LocalState` reads
3. Move only the ordinary root-level late-landing calls onto the existing capability lane:
   - status text
   - progress block shell
   - `ScrollArea` root landing
   - footer shell landing
4. Keep helper-local `ElementContext` closures unchanged where they already own raw builder work.
5. Re-run:
   - `cargo test -p fret-examples --lib todo_demo_prefers_default_app_surface`
   - `cargo test -p fret-examples --lib todo_demo_prefers_capability_first_landing_for_root_builders`
   - `cargo check -p fret-examples --all-targets --message-format short`
6. Re-run a temporary local no-`Deref` spot-check for `fret-examples`:
   - `cargo check -p fret-examples --all-targets --message-format short`

The temporary `AppUi` no-`Deref` patch was reverted immediately after collecting the new failure
map.

## Findings

### 1) `todo_demo` was default-root late-landing debt, not a raw-owner seam

The remaining failures were on ordinary builder landing at the root render surface:

- status text landing,
- progress shell landing,
- rows/scroll shell landing,
- footer shell landing.

Those are not component/internal identity seams and do not justify either `cx.elements()` or a
new broad `AppUi` forwarding surface.

### 2) The correct fix is to reuse the existing capability-first landing lane

The owner-correct implementation keeps the default Todo story narrow and consistent:

- app-facing reads/actions stay on `AppUi`,
- ordinary root builders use `into_element_in(cx)`,
- helper-local `ElementContext` closures keep their existing raw `into_element(cx)` use where
  they already own the raw builder context.

This matches the existing `IntoUiElementInExt` teaching story instead of inventing a special-case
Todo surface.

### 3) The no-`Deref` spot-check shrinks the tail again

With a temporary local patch that removes both `impl Deref` and `impl DerefMut` from `AppUi`:

- `fret-examples` drops from `49` to `41` previous errors,
- `todo_demo` no longer appears in the failure map,
- and the current leading `fret-examples` clusters are now:
  - `async_playground_demo.rs` (`6`)
  - `markdown_demo.rs` (`5`)
  - `editor_notes_demo.rs` (`4`)
  - `hello_world_compare_demo.rs` (`4`)

That is strong evidence that `todo_demo` was another default-root capability-landing cleanup
slice rather than a missing framework surface.

## Evidence

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `cargo test -p fret-examples --lib todo_demo_prefers_default_app_surface`
- `cargo test -p fret-examples --lib todo_demo_prefers_capability_first_landing_for_root_builders`
- `cargo check -p fret-examples --all-targets --message-format short`
- temporary local no-`Deref` spot-check:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `/tmp/fret_examples_noderef_todo_v3.txt`

## Outcome

The repo now has an explicit answer for the Todo app root:

1. root state/theme/responsive reads stay on `AppUi`
2. ordinary root-level late-landing uses `into_element_in(cx)`
3. helper-local raw builder work remains on its owned `ElementContext` closures
