# AppUi Manual Date Picker Raw Owner Audit — 2026-04-17

Status: Landed slice + targeted evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_MANUAL_FORM_RAW_OWNER_AUDIT_2026-04-17.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_ADVANCED_HELPER_RAW_OWNER_AUDIT_2026-04-17.md`
- `apps/fret-examples/src/date_picker_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Scope

Decide how the remaining `date_picker_demo` no-`Deref` failures should be classified:

- should this manual `render_root_with_app_ui(...)` proof widen `AppUi`,
- or should it keep an app-lane read phase plus an explicit raw builder/container owner?

This note is intentionally about a manual render-root proof, not about widening the default
authoring façade.

## Method

1. Inspect the post-advanced-helper no-`Deref` failure map and confirm that
   `apps/fret-examples/src/date_picker_demo.rs` is still one of the largest remaining clusters.
2. Keep only the real app-lane reads on `AppUi`:
   - `cx.theme_snapshot()`
   - `LocalState::layout_value(...)`
   - `month.layout(cx).read_ref(...)`
3. Enter the explicit raw owner before the manual layout/build phase:
   - `let cx = cx.elements();`
4. Leave the rest of the manual render-root proof on that raw lane:
   - header `ui::h_row(...)`
   - `cx.flex(...)`
   - direct `DatePicker` / `Calendar` late-landing
   - `cx.text(...)`
   - `cx.container(...)`
5. Re-run:
   - `cargo test -p fret-examples --lib manual_date_picker_demo_uses_app_ui_render_root_bridge`
   - `cargo test -p fret-examples --lib selected_raw_owner_examples_keep_escape_hatches_explicit`
   - `cargo check -p fret-examples --all-targets --message-format short`
6. Re-run the temporary local no-`Deref` compile audit:
   - `cargo check -p fret-examples --all-targets --message-format short`
   - `cargo check -p fret-cookbook --all-targets --message-format short`

The temporary `AppUi` no-`Deref` patch was reverted immediately after collecting the new failure
map.

## Findings

### 1) `date_picker_demo` is another manual raw-owner callsite, not a façade gap

The remaining failures in this file were the same raw authoring family already seen in
`form_demo`:

- direct late-landing on manual controls,
- `cx.flex(...)`,
- `cx.text(...)`,
- `cx.container(...)`.

That is evidence about callsite ownership, not evidence that `AppUi` should absorb the raw
`ElementContext` builder surface.

### 2) The correct split is app-lane reads first, explicit raw build second

The owner-correct implementation keeps the manual proof honest:

- tracked state/theme reads stay on `AppUi`,
- the manual layout/build phase enters `let cx = cx.elements();`,
- the rest of the proof remains explicit raw builder/container authoring.

This matches the contract already established by `form_demo` instead of inventing a second manual
root story.

### 3) The follow-on no-`Deref` audit gets materially smaller again

With the paired `embedded_viewport_demo` follow-on landed and the temporary local patch that
removes both `impl Deref` and `impl DerefMut` from `AppUi`:

- `fret-examples` drops from `67` to `49` previous errors,
- `fret-cookbook` remains on the same failure map,
- `date_picker_demo` no longer appears in the `fret-examples` output,
- and the current top `fret-examples` clusters are now led by:
  - `todo_demo.rs` (`8`)
  - `async_playground_demo.rs` (`6`)
  - `markdown_demo.rs` (`5`)

That is strong evidence that `date_picker_demo` was another raw-owner cleanup slice rather than a
new framework-surface requirement.

## Evidence

- `apps/fret-examples/src/date_picker_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `cargo test -p fret-examples --lib manual_date_picker_demo_uses_app_ui_render_root_bridge`
- `cargo test -p fret-examples --lib selected_raw_owner_examples_keep_escape_hatches_explicit`
- `cargo check -p fret-examples --all-targets --message-format short`
- temporary local no-`Deref` audit:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `cargo check -p fret-cookbook --all-targets --message-format short`
  - `/tmp/fret_examples_noderef_date_embedded_v2.txt`
  - `/tmp/fret_cookbook_noderef_date_embedded_v2.txt`

## Outcome

The repo now has an explicit owner answer for the manual date-picker proof:

1. `date_picker_demo` keeps only its real app-lane reads on `AppUi`
2. manual header/toggle/picker/calendar/container authoring moves through an explicit
   `cx.elements()` lane
3. the next structural work can move to the remaining helper-signature/default-surface clusters

