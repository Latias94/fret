# AppUi Raw Text Authoring Owner Audit — 2026-04-17

Status: Landed slice + evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEREF_COMPILE_AUDIT_2026-04-17.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_OVERLAY_ROOT_CAPABILITY_SURFACE_AUDIT_2026-04-17.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `apps/fret-examples/src/emoji_conformance_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Scope

Decide whether the remaining `emoji_conformance_demo` pressure around raw text/builder authoring
should widen the default `AppUi` surface or stay as an explicit raw-owner callsite split.

This note is intentionally narrower than a new framework API slice. It only covers the current
conformance proof where the remaining no-`Deref` failures were concentrated around:

- `text_props(...)`
- direct `into_element(...)` late-landing on shadcn card/scroll wrappers
- `Separator::new().into_element(...)`

## Method

1. Inspect the temporary no-`Deref` failure tail for `apps/fret-examples` and confirm that
   `emoji_conformance_demo` is now the dominant remaining product-proof cluster after the overlay
   root slice.
2. Keep app-lane work on `AppUi`:
   - `cx.theme_snapshot()`
   - `cx.app().global::<FontCatalogCache>()`
   - `emoji_font_override.layout_value(cx)`
3. Enter the raw lane explicitly at the current proof boundary:
   - `let cx = cx.elements();`
4. Leave the raw text/builder authoring on that explicit owner:
   - `cx.text_props(...)`
   - direct `Card*` / `ScrollArea` / `Separator` `into_element(cx)` landing
5. Lock the choice with targeted source-policy tests and re-run:
   - `cargo test -p fret-examples --lib selected_raw_owner_examples_keep_escape_hatches_explicit`
   - `cargo test -p fret-examples --lib selected_app_ui_roots_prefer_explicit_render_context_accessors_over_deref`
   - `cargo test -p fret-examples --lib manual_ui_tree_examples_keep_root_wrappers_on_local_typed_helpers`
   - `cargo check -p fret-examples --all-targets --message-format short`
6. Re-run the temporary local no-`Deref` audit:
   - `cargo check -p fret-examples --all-targets --message-format short`
   - `cargo check -p fret-cookbook --all-targets --message-format short`

The temporary `AppUi` no-`Deref` patch was reverted immediately after collecting the new failure
map.

## Findings

### 1) This pressure is raw leaf authoring, not proof that `AppUi` should grow `text_props(...)`

The remaining `emoji_conformance_demo` failures were all in one narrow bucket:

- raw text leaf construction through `cx.text_props(...)`
- direct raw late-landing on shadcn card/scroll wrappers

That is not the same ownership class as the already-approved app-lane helpers such as:

- `request_animation_frame()`
- command gating
- `layout_query_*`
- direct overlay-root `*_in(...)`

Promoting `text_props(...)`, `text(...)`, `container(...)`, or `flex(...)` onto `AppUi` from this
evidence would be an overreaction and would re-expand the raw `ElementContext` surface under a new
name.

### 2) The correct fix is an explicit lane split inside the proof

`emoji_conformance_demo` naturally divides into two ownership phases:

- app-lane reads and state/global access on `AppUi`
- raw text/builder authoring on `ElementContext`

The owner-correct implementation is therefore:

- keep `LocalState` reads, theme snapshot, and font-catalog access on `AppUi`
- then enter `let cx = cx.elements();`
- and only after that perform `text_props(...)` and direct `into_element(cx)` landing

This keeps the escape hatch explicit without teaching the wrong default surface.

### 3) The no-`Deref` audit gets materially smaller again

With a temporary local patch that removes both `impl Deref` and `impl DerefMut` from `AppUi`:

- `fret-examples` drops from `126` error lines to `111`
- `fret-cookbook` remains at `22`
- `emoji_conformance_demo` disappears from the failure tail entirely

The remaining top `fret-examples` clusters are now led by:

- `form_demo.rs` (`13`)
- `postprocess_theme_demo.rs` (`11`)
- `hello_counter_demo.rs` (`10`)
- `imui_interaction_showcase_demo.rs` (`10`)

That is strong evidence that the `emoji_conformance_demo` cluster was a callsite-owner problem,
not a missing framework surface.

## Evidence

- `apps/fret-examples/src/emoji_conformance_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `cargo test -p fret-examples --lib selected_raw_owner_examples_keep_escape_hatches_explicit`
- `cargo test -p fret-examples --lib selected_app_ui_roots_prefer_explicit_render_context_accessors_over_deref`
- `cargo test -p fret-examples --lib manual_ui_tree_examples_keep_root_wrappers_on_local_typed_helpers`
- `cargo check -p fret-examples --all-targets --message-format short`
- temporary local no-`Deref` audit:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `cargo check -p fret-cookbook --all-targets --message-format short`
  - `/tmp/fret_examples_noderef_text_owner.txt`
  - `/tmp/fret_cookbook_noderef_text_owner.txt`

## Outcome

The repo now has an explicit owner answer for the current raw text/builder conformance tail:

1. `emoji_conformance_demo` keeps app-lane reads on `AppUi`.
2. Raw text/builder authoring moves through an explicit `cx.elements()` escape hatch instead of
   widening `AppUi`.
3. The next structural work should focus on the remaining helper-signature and ordinary builder
   clusters rather than re-debating this proof.
