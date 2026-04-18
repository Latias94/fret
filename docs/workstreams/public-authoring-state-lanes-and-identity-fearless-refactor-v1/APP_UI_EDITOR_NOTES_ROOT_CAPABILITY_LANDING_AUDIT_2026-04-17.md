# AppUi Editor Notes Root Capability Landing Audit — 2026-04-17

Status: Landed slice + targeted evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_MARKDOWN_ROOT_CAPABILITY_LANDING_AUDIT_2026-04-17.md`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Scope

Decide how the remaining `editor_notes_demo` no-`Deref` failures should be classified:

- should the workbench proof fall back to raw `ElementContext` at the app root,
- should the reusable selection/center/inspector helpers be retargeted to `AppUi`,
- or should ordinary shell rails and frame/root late-landing stay on the existing capability-first
  `AppUi` lane while reusable panels remain generic?

This note is intentionally about the app root and shell-owned late-landing, not about widening the
reusable editor helpers or reopening a raw-owner lane.

## Method

1. Inspect the post-`markdown_demo` no-`Deref` failure map and confirm that
   `apps/fret-examples/src/editor_notes_demo.rs` still fails only on:
   - left rail landing
   - right rail landing
   - `WorkspaceFrame` landing
   - outer page-shell landing
2. Confirm that the reusable panels are already on the correct owner shape:
   - `selection_button(...)`
   - `render_selection_panel(...)`
   - `render_center_panel(...)`
   - `render_inspector_panel(...)`
   - all typed over `Cx: fret::app::ElementContextAccess<'a, App>`
3. Move only the ordinary app-root late-landing tail onto the existing capability lane:
   - import `fret_ui_kit::IntoUiElementInExt as _`
   - use `into_element_in(cx)` for:
     - both shell rails
     - `WorkspaceFrame`
     - the outer page shell
4. Add source-policy gates in `apps/fret-examples/src/lib.rs` that freeze:
   - reusable panels stay generic on `ElementContextAccess<'a, App>`
   - the app root uses capability-first late-landing instead of implicit `Deref`
5. Re-run:
   - `cargo test -p fret-examples --lib editor_notes_demo_keeps_reusable_panels_on_generic_element_context_access`
   - `cargo test -p fret-examples --lib editor_notes_demo_prefers_capability_first_landing_for_workspace_shell_root`
   - `cargo check -p fret-examples --all-targets --message-format short`
6. Re-run a temporary local no-`Deref` spot-check:
   - disable `impl Deref` / `impl DerefMut` for `AppUi` temporarily in `ecosystem/fret/src/view.rs`
   - `cargo check -p fret-examples --all-targets --message-format short`

The temporary `AppUi` no-`Deref` patch was reverted immediately after collecting the new failure
map.

## Findings

### 1) `editor_notes_demo` was ordinary workbench-shell late-landing debt, not a new contract gap

The remaining failures were all at the app root:

- left rail shell
- right rail shell
- `WorkspaceFrame`
- outer page shell

The reusable panel helpers were already correctly classified as generic late-builders over
`ElementContextAccess<'a, App>`, so there was no evidence that they should be widened to `AppUi`
or demoted to raw `UiCx<'_>`.

### 2) The correct owner split keeps reusable panels generic and moves only root landing onto `into_element_in(cx)`

The owner-correct shape is now:

- app-level state, actions, and theme reads stay on `AppUi`
- reusable editor helpers remain generic on `ElementContextAccess<'a, App>`
- ordinary shell rails, `WorkspaceFrame`, and the outer root shell use `into_element_in(cx)`

That matches the intended contract: app-owned root assembly without reopening implicit
`ElementContext` inheritance just to land ordinary builders.

### 3) The no-`Deref` spot-check removes `editor_notes_demo` and shrinks the tail again

With a temporary local patch that removes both `impl Deref` and `impl DerefMut` from `AppUi`:

- `fret-examples` drops from `30` to `26` previous errors,
- `editor_notes_demo` no longer appears in the failure map,
- and the current leading `fret-examples` clusters are now:
  - `hello_world_compare_demo.rs` (`4`)
  - `drop_shadow_demo.rs` (`3`)
  - `ime_smoke_demo.rs` (`3`)
  - `query_async_tokio_demo.rs` (`3`)
  - `query_demo.rs` (`3`)
  - `sonner_demo.rs` (`3`)
  - `components_gallery.rs` (`2`)

That is strong evidence that this was another capability-first app-root cleanup slice rather than a
missing framework surface.

## Evidence

- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `cargo test -p fret-examples --lib editor_notes_demo_keeps_reusable_panels_on_generic_element_context_access`
- `cargo test -p fret-examples --lib editor_notes_demo_prefers_capability_first_landing_for_workspace_shell_root`
- `cargo check -p fret-examples --all-targets --message-format short`
- temporary local no-`Deref` spot-check:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `/tmp/fret_examples_noderef_editor_notes_v6.txt`

## Outcome

The repo now has an explicit owner answer for the editor-notes workbench proof:

1. reusable editor panels stay generic on `ElementContextAccess<'a, App>`
2. app-root shell assembly stays on `AppUi`
3. ordinary rails, `WorkspaceFrame`, and the outer page shell use `into_element_in(cx)`
4. `editor_notes_demo` is no longer part of the current no-`Deref` failure tail
