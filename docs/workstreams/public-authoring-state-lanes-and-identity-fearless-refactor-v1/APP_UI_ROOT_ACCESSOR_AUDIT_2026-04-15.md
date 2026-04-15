# AppUi Root Accessor Audit — 2026-04-15

Status: follow-on audit for the active public authoring lane

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/ADVANCED_ENTRY_CAPABILITY_AUDIT_2026-04-15.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/API_WORKBENCH_FRAMEWORK_PRIORITY_AUDIT_2026-04-15.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `ecosystem/fret/src/view.rs`
- `apps/fret-examples/src/lib.rs`

## Assumptions First

### A1) This work still belongs in the active public authoring lane

Confidence: Confident

Evidence:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/API_WORKBENCH_FRAMEWORK_PRIORITY_AUDIT_2026-04-15.md`

If wrong:

- this cleanup should have been split into a narrower follow-on lane instead of extending the
  current execution surface.

### A2) The advanced-entry adapter batch changed the remaining `AppUi` pressure from raw entry
adapter debt into smaller root-accessor debt

Confidence: Confident

Evidence:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/ADVANCED_ENTRY_CAPABILITY_AUDIT_2026-04-15.md`
- `apps/fret-examples/src/lib.rs`

If wrong:

- another broad `cx.elements()` sweep would still be the right next move, and this audit would be
  prematurely narrowing the problem.

### A3) `AppUi::{app, app_mut, window_id}` already provides the right explicit lane for `AppUi`
roots

Confidence: Confident

Evidence:

- `ecosystem/fret/src/view.rs`
- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/src/embedded_viewport_demo.rs`
- `apps/fret-examples/src/hello_world_compare_demo.rs`

If wrong:

- the repo would need another public capability trait or a wider prelude story just to remove
  `Deref` syntax from `AppUi` roots.

### A4) Remaining helper-local `cx.elements()` seams are mostly intentional late-landing or
advanced/raw ownership, not ordinary root sugar debt

Confidence: Likely

Evidence:

- `apps/fret-examples/src/assets_demo.rs`
- `apps/fret-examples/src/image_heavy_memory_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/components_gallery.rs`

If wrong:

- this audit would be understating the amount of remaining default-lane work before any future
  `Deref` removal attempt.

## Question

After the advanced-entry adapter batch landed, what is the next correct structural cleanup before
any future `AppUi` `Deref` deletion attempt?

## Verdict

Keep the lane active.

Do **not** delete `AppUi` `Deref` yet.

The next correct follow-on was narrower:

- stop selected `AppUi` roots from using `cx.app` / `cx.window` through the compatibility bridge
  when they only need the explicit app-facing host/window lane,
- use `AppUi::{app, app_mut, window_id}` instead of reopening the broader `ElementContext`
  surface or inventing another trait,
- and lock the cleanup with a source-policy gate that proves the old bridge syntax does not drift
  back into this batch.

This is the right slice because it removes real compatibility debt without confusing two different
things:

- ordinary `AppUi` root access to host/window state,
- versus intentional helper-local or advanced/raw `ElementContext` ownership.

## Findings

### 1) Several `AppUi` roots were still using `Deref` only for host/window reads

Before this slice, selected roots in `apps/fret-examples/src` still spelled compatibility-bridge
syntax such as:

- `cx.app`
- `cx.window`
- `&*cx.app`

even though those callsites were not actually asking for raw `ElementContext` facilities.

The landed cleanup covers:

- `embedded_viewport_demo`
- `async_playground_demo`
- `markdown_demo`
- `postprocess_theme_demo`
- `genui_demo`
- `hello_world_compare_demo`

Conclusion:

- this was real `Deref` syntax debt,
- but it was narrower than "remove `Deref` everywhere now".

### 2) The correct fix was to prefer explicit `AppUi` accessors, not another public surface

`AppUi` already exposes:

- `app()`
- `app_mut()`
- `window_id()`

That means the root cleanup did not need:

- another trait,
- a wider `fret::app::prelude::*`,
- or a new helper taxonomy.

Conclusion:

- the right move was to use the existing explicit root lane and keep the public surface smaller.

### 3) This cleanup had to stay scoped to `AppUi` roots, not helper-local raw seams

Some remaining `cx.elements()` usage is still intentional because the helper or proof surface owns
an advanced/raw boundary itself.

Representative examples:

- `assets_demo`
- `image_heavy_memory_demo`
- `workspace_shell_demo`
- `imui_editor_proof_demo`
- `components_gallery`
- low-level interop roots such as `external_texture_imports_demo` and
  `external_video_imports_*_demo`

Conclusion:

- those seams should not be "cleaned up" just to make a `Deref` diff look bigger,
- and they remain separate from the ordinary root-accessor story.

### 4) The remaining `Deref` pressure is now better classified, but it is still real

After this slice, the remaining direct `cx.app` / `cx.window` usage in first-party examples is not
one uniform debt class.

It now spans multiple intentional categories, including:

- renderer/effect or theme-bridge proofs,
- docking manager ownership,
- low-level interop surfaces,
- IMUI/editor proof machinery,
- helper-local late-landing seams that intentionally own raw context.

Conclusion:

- the repo is in a better place to attempt a future `Deref` audit,
- but the evidence still does **not** support deleting `Deref` in this turn.

## Landed Slice

This follow-on lands two things together:

1. Selected `AppUi` roots now use explicit app/window accessors instead of `cx.app` /
   `cx.window` bridge syntax.
2. `apps/fret-examples/src/lib.rs` now contains
   `selected_app_ui_roots_prefer_explicit_render_context_accessors_over_deref`, a source-policy
   gate that locks this batch.

Evidence anchors:

- `apps/fret-examples/src/embedded_viewport_demo.rs`
- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-examples/src/postprocess_theme_demo.rs`
- `apps/fret-examples/src/genui_demo.rs`
- `apps/fret-examples/src/hello_world_compare_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Repro, Gate, Evidence

Repro target:

- `cargo run -p fretboard -- dev native --bin embedded_viewport_demo`
- `cargo run -p fretboard -- dev native --bin async_playground_demo`

Primary gates:

- `cargo nextest run -p fret-examples selected_app_ui_roots_prefer_explicit_render_context_accessors_over_deref`
- `cargo nextest run -p fret-examples advanced_helper_contexts_prefer_uicx_aliases`
- `cargo check -p fret-examples --all-targets`

What these gates prove:

- the selected roots no longer fall through `Deref` just to reach app/window access,
- the existing advanced helper alias policy still holds,
- and the examples crate still compiles after the cleanup.
