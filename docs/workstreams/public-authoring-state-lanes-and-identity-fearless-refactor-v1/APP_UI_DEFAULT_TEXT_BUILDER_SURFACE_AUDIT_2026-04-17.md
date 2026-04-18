# AppUi Default Text Builder Surface Audit — 2026-04-17

Status: Landed slice + evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEREF_COMPILE_AUDIT_2026-04-17.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_RAW_TEXT_AUTHORING_OWNER_AUDIT_2026-04-17.md`
- `apps/fret-examples/src/hello_counter_demo.rs`
- `apps/fret-cookbook/examples/hello_counter.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-cookbook/src/lib.rs`

## Scope

Decide how the default counter teaching surfaces should resolve their remaining no-`Deref` pressure
without teaching raw `ElementContext` ownership:

- `apps/fret-examples/src/hello_counter_demo.rs`
- `apps/fret-cookbook/examples/hello_counter.rs`

This note is intentionally about teaching-surface owner choice, not a new framework API.

## Method

1. Inspect the post-`emoji_conformance_demo` no-`Deref` failure map and confirm that the default
   counter surfaces still fail on two narrow families:
   - raw `TextProps` authoring
   - direct `.into_element(cx)` late-landing on ordinary builders
2. Keep the examples on the default app lane by reusing existing app-facing helpers:
   - replace `cx.text_props(...)` with `ui::text(...)` / `ui::text_block(...)`
   - replace direct ordinary `.into_element(cx)` callsites with `.into_element_in(cx)` where the
     builder already participates in `IntoUiElement`
3. Add source-policy gates:
   - `apps/fret-examples/src/lib.rs`
   - `apps/fret-cookbook/src/lib.rs`
4. Re-run:
   - `cargo test -p fret-examples --lib hello_counter_demo_prefers_root_helper_surface`
   - `cargo test -p fret-examples --lib hello_counter_demo_prefers_app_lane_text_builders_and_capability_first_landing`
   - `cargo test -p fret-cookbook --lib migrated_basics_examples_use_the_new_app_surface`
   - `cargo check -p fret-examples --all-targets --message-format short`
   - `cargo check -p fret-cookbook --all-targets --message-format short`
5. Re-run the temporary local no-`Deref` audit:
   - `cargo check -p fret-examples --all-targets --message-format short`
   - `cargo check -p fret-cookbook --all-targets --message-format short`

The temporary `AppUi` no-`Deref` patch was reverted immediately after collecting the new failure
map.

## Findings

### 1) Default teaching surfaces should prefer existing app-lane text builders

The `hello_counter` examples were still using raw `TextProps` authoring for:

- the large count readout
- the status line
- the step-help copy

That is the wrong teaching surface for the default lane when the repo already ships:

- `ui::text(...)`
- `ui::text_block(...)`
- text builder refinements such as `text_size_px(...)`, `font_bold()`, `text_color(...)`,
  `text_align(...)`, and `nowrap()`

The correct fix is to reuse those builders rather than teaching `cx.elements()` or widening
`AppUi` with raw text APIs.

### 2) Ordinary late-landing should use the capability-first lane, not raw `.into_element(cx)`

`hello_counter_demo` also still used direct `.into_element(cx)` calls for ordinary builders such
as:

- `ui::h_flex(...)`
- `ui::v_flex(...)`
- `shadcn::Input`
- `shadcn::Card`

Those are not raw mechanism seams. They already participate in the generic
`IntoUiElementInExt::into_element_in(cx)` capability lane.

The correct default-lane teaching story is therefore:

- keep render-time state/actions on `AppUi`
- use `.into_element_in(cx)` for ordinary late-landing
- reserve explicit `cx.elements()` only for genuinely raw-owner callsites

### 3) The no-`Deref` audit gets smaller on both first-party teaching surfaces

With a temporary local patch that removes both `impl Deref` and `impl DerefMut` from `AppUi`:

- `fret-examples` drops from `111` error lines to `101`
- `fret-cookbook` drops from `22` to `19`
- neither output reports `hello_counter_demo` or cookbook `hello_counter`

The remaining `fret-examples` top clusters are still led by:

- `form_demo.rs` (`13`)
- `postprocess_theme_demo.rs` (`11`)
- `imui_interaction_showcase_demo.rs` (`10`)

That is strong evidence that the `hello_counter` failures were default-surface teaching debt, not
a missing framework owner.

## Evidence

- `apps/fret-examples/src/hello_counter_demo.rs`
- `apps/fret-cookbook/examples/hello_counter.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-cookbook/src/lib.rs`
- `cargo test -p fret-examples --lib hello_counter_demo_prefers_root_helper_surface`
- `cargo test -p fret-examples --lib hello_counter_demo_prefers_app_lane_text_builders_and_capability_first_landing`
- `cargo test -p fret-cookbook --lib migrated_basics_examples_use_the_new_app_surface`
- `cargo check -p fret-examples --all-targets --message-format short`
- `cargo check -p fret-cookbook --all-targets --message-format short`
- temporary local no-`Deref` audit:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `cargo check -p fret-cookbook --all-targets --message-format short`
  - `/tmp/fret_examples_noderef_hello_counter.txt`
  - `/tmp/fret_cookbook_noderef_hello_counter.txt`

## Outcome

The repo now keeps the default counter teaching surfaces on the intended app-facing lane:

1. default counter examples use `ui::text(...)` / `ui::text_block(...)` instead of raw
   `TextProps`
2. `hello_counter_demo` uses `into_element_in(cx)` for ordinary late-landing instead of direct raw
   `.into_element(cx)`
3. the next structural work can stay focused on `form_demo` and the remaining advanced/helper
   clusters rather than re-debating these first-contact examples
