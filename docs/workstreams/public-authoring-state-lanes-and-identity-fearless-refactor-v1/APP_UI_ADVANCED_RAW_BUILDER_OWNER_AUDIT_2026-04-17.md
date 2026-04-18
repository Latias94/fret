# AppUi Advanced Raw Builder Owner Audit — 2026-04-17

Status: Landed slice + targeted evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_RAW_TEXT_AUTHORING_OWNER_AUDIT_2026-04-17.md`
- `apps/fret-examples/src/drop_shadow_demo.rs`
- `apps/fret-examples/src/ime_smoke_demo.rs`
- `apps/fret-examples/src/sonner_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Scope

Decide whether the remaining `drop_shadow_demo`, `ime_smoke_demo`, and `sonner_demo`
no-`Deref` pressure should:

- widen `AppUi` with raw `container(...)`, `flex(...)`, or `text(...)` helpers,
- or stay as explicit raw-owner callsite splits after app-lane state/theme reads.

This note is intentionally about advanced/manual builder-heavy proof surfaces, not first-contact
teaching surfaces.

## Method

1. Inspect the post-query no-`Deref` failure map and confirm that:
   - `drop_shadow_demo` fails on `cx.container(...)`
   - `ime_smoke_demo` fails on `cx.container(...)`, `cx.flex(...)`, and `cx.text(...)`
   - `sonner_demo` fails on `cx.flex(...)`
2. Keep app-lane reads on `AppUi`:
   - local state reads
   - grouped model reads
   - theme snapshots
3. Enter the raw lane explicitly at the advanced builder boundary:
   - `let cx = cx.elements();`
4. Leave the raw builder families on that explicit owner:
   - `container(...)`
   - `flex(...)`
   - `text(...)`
   - direct `into_element(cx)` late-landing where the surface already speaks raw `ElementContext`
5. Lock the choice through the existing raw-owner source-policy test in
   `apps/fret-examples/src/lib.rs`
6. Re-run:
   - `cargo test -p fret-examples --lib selected_raw_owner_examples_keep_escape_hatches_explicit`
   - `cargo check -p fret-examples --all-targets --message-format short`
7. Re-run a temporary local no-`Deref` spot-check:
   - disable `impl Deref` / `impl DerefMut` for `AppUi` temporarily in `ecosystem/fret/src/view.rs`
   - `cargo check -p fret-examples --all-targets --message-format short`

The temporary `AppUi` no-`Deref` patch was reverted immediately after collecting the new failure
map.

## Findings

### 1) This pressure is explicit advanced raw-builder authoring, not framework surface debt

All three files were already clearly in builder-heavy advanced/manual territory:

- `drop_shadow_demo`: raw `ContainerProps`, effect-layer authoring, renderer semantics validation
- `ime_smoke_demo`: manual `render_root_with_app_ui(...)` shell over a `UiTree`
- `sonner_demo`: manual `render_root_with_app_ui(...)` shell plus overlay/toast orchestration

Promoting `container(...)`, `flex(...)`, or `text(...)` onto `AppUi` from this evidence would
recreate broad `ElementContext` inheritance under a façade name.

### 2) The correct fix is an explicit app-lane-read then raw-builder split

The owner-correct pattern is now:

- keep state/theme/grouped reads on `AppUi`
- then enter `let cx = cx.elements();`
- and only after that perform advanced/manual builder authoring

This keeps the advanced escape hatch explicit without polluting the default app-facing lane.

### 3) The no-`Deref` spot-check removes the whole triplet and shrinks the tail to 7

With a temporary local patch that removes both `impl Deref` and `impl DerefMut` from `AppUi`:

- `fret-examples` drops from `16` to `7` previous errors,
- `drop_shadow_demo` no longer appears in the failure map,
- `ime_smoke_demo` no longer appears in the failure map,
- `sonner_demo` no longer appears in the failure map,
- and the current remaining `fret-examples` failures are now only:
  - `components_gallery.rs` (`2`)
  - `custom_effect_v1_demo.rs` (`1`)
  - `custom_effect_v2_demo.rs` (`1`)
  - `custom_effect_v3_demo.rs` (`1`)
  - `genui_demo.rs` (`1`)
  - `liquid_glass_demo.rs` (`1`)

That is strong evidence that this slice is purely owner-callsite cleanup.

## Evidence

- `apps/fret-examples/src/drop_shadow_demo.rs`
- `apps/fret-examples/src/ime_smoke_demo.rs`
- `apps/fret-examples/src/sonner_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `cargo test -p fret-examples --lib selected_raw_owner_examples_keep_escape_hatches_explicit`
- `cargo check -p fret-examples --all-targets --message-format short`
- temporary local no-`Deref` spot-check:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `/tmp/fret_examples_noderef_raw_builder_triplet_v9.txt`

## Outcome

The repo now has an explicit owner answer for this advanced raw-builder triplet:

1. app-lane state/theme reads stay on `AppUi`
2. advanced `container(...)` / `flex(...)` / `text(...)` authoring moves through explicit
   `cx.elements()`
3. `drop_shadow_demo`, `ime_smoke_demo`, and `sonner_demo` are no longer part of the current
   no-`Deref` failure tail
