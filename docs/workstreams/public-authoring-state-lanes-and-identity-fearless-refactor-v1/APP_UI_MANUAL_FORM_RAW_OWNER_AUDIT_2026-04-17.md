# AppUi Manual Form Raw Owner Audit — 2026-04-17

Status: Landed slice + evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEREF_COMPILE_AUDIT_2026-04-17.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEFAULT_TEXT_BUILDER_SURFACE_AUDIT_2026-04-17.md`
- `apps/fret-examples/src/form_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Scope

Decide how the remaining `form_demo` no-`Deref` failures should be classified:

- should this manual `render_root_with_app_ui(...)` surface widen `AppUi`,
- or should it keep a narrower app-lane read phase plus an explicit raw builder/container owner?

This note is intentionally about a manual root proof, not about the default first-contact lane.

## Method

1. Inspect the post-`hello_counter` no-`Deref` failure map and confirm that
   `apps/fret-examples/src/form_demo.rs` is now the leading remaining cluster.
2. Keep the `AppUi` phase only for the reads that actually belong there:
   - `cx.theme_snapshot()`
   - `form_state.layout(cx)...`
   - `status.layout_value(cx)`
3. Enter the raw owner explicitly for the rest of the manual render-root proof:
   - `let cx = cx.elements();`
4. Leave the raw builder/container authoring on that explicit lane:
   - `ui::h_row(...)`
   - `cx.text(...)`
   - `shadcn::FormField` / `Input` / `Select` / `DatePicker` late-landing
   - `cx.container(...)`
   - `cx.flex(...)`
5. Lock the choice with targeted gates:
   - `cargo test -p fret-examples --lib manual_form_demo_uses_app_ui_render_root_bridge`
   - `cargo test -p fret-examples --lib selected_raw_owner_examples_keep_escape_hatches_explicit`
   - `cargo check -p fret-examples --all-targets --message-format short`
6. Re-run the temporary local no-`Deref` audit:
   - `cargo check -p fret-examples --all-targets --message-format short`
   - `cargo check -p fret-cookbook --all-targets --message-format short`

The temporary `AppUi` no-`Deref` patch was reverted immediately after collecting the new failure
map.

## Findings

### 1) This is another explicit raw-owner callsite, not a missing default-lane API

`form_demo` is a manual `UiTree` / `render_root_with_app_ui(...)` proof. Its remaining failures
were not about grouped state/actions or default app-lane teaching surface gaps.

The unresolved calls were all on raw authoring families:

- `cx.text(...)`
- direct late-landing on form controls
- `cx.container(...)`
- `cx.flex(...)`

That is the same ownership class as other explicit raw escape hatches, not evidence that `AppUi`
should absorb the raw container/flex/text surface.

### 2) The correct split is app-lane reads first, then explicit raw layout/building

The owner-correct implementation keeps the render-root proof honest:

- tracked state/theme reads stay on `AppUi`
- the manual layout/build phase enters `let cx = cx.elements();`
- the rest of the surface remains explicit raw builder/container authoring

That preserves the contract line between app-facing reads and raw `ElementContext` layout/building
without inventing new façade sugar.

### 3) The no-`Deref` audit gets materially smaller again

With a temporary local patch that removes both `impl Deref` and `impl DerefMut` from `AppUi`:

- `fret-examples` drops from `101` error lines to `88`
- `fret-cookbook` remains at `19`
- `form_demo` disappears from the failure tail entirely

The remaining `fret-examples` top clusters are now led by:

- `postprocess_theme_demo.rs` (`11`)
- `imui_interaction_showcase_demo.rs` (`10`)
- `date_picker_demo.rs` (`9`)
- `embedded_viewport_demo.rs` (`9`)

That is strong evidence that `form_demo` was another callsite-owner cleanup slice rather than a
framework-surface gap.

## Evidence

- `apps/fret-examples/src/form_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `cargo test -p fret-examples --lib manual_form_demo_uses_app_ui_render_root_bridge`
- `cargo test -p fret-examples --lib selected_raw_owner_examples_keep_escape_hatches_explicit`
- `cargo check -p fret-examples --all-targets --message-format short`
- temporary local no-`Deref` audit:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `cargo check -p fret-cookbook --all-targets --message-format short`
  - `/tmp/fret_examples_noderef_form_demo.txt`
  - `/tmp/fret_cookbook_noderef_form_demo.txt`

## Outcome

The repo now has an explicit owner answer for the manual form root:

1. `form_demo` keeps only its real app-lane reads on `AppUi`
2. raw builder/container/flex authoring moves through an explicit `cx.elements()` lane
3. the next structural work can focus on the remaining helper-signature / advanced proof clusters
   rather than revisiting this manual root
