# AppUi Deref Removal Proof Audit — 2026-04-18

Status: Landed slice + shipped no-`Deref` boundary

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_FINAL_NO_DEREF_TAIL_OWNER_AUDIT_2026-04-17.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_ADVANCED_RAW_BUILDER_OWNER_AUDIT_2026-04-17.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `ecosystem/fret/src/view.rs`
- `apps/fret-cookbook/examples/toast_basics.rs`
- `apps/fret-cookbook/examples/effects_layer_basics.rs`
- `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs`

## Scope

Promote the temporary local no-`Deref` compile audit into the shipped framework contract:

1. close the last first-party consumer tails that still depended on implicit `AppUi ->
   ElementContext` coercion,
2. delete the disabled `impl Deref` / `impl DerefMut` blocks from `AppUi`,
3. replace the old “temporary compatibility bridge” wording with an explicit boundary contract,
4. and prove the result across framework, cookbook, examples, and first-party UI Gallery surfaces.

## Method

1. Keep the repo in no-`Deref` mode and close the remaining cookbook failures:
   - `apps/fret-cookbook/examples/toast_basics.rs`
   - `apps/fret-cookbook/examples/effects_layer_basics.rs`
2. Run package compile proofs:
   - `cargo check -p fret --all-targets --message-format short`
   - `cargo check -p fret-examples --all-targets --message-format short`
   - `cargo check -p fret-cookbook --all-targets --message-format short`
3. Audit the next first-party consumer surface that still failed under no-`Deref`:
   - `cargo check -p fret-ui-gallery --all-targets --message-format short`
4. Fix the UI Gallery default-lane late-landing seam in
   `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs` and lock it with
   source-policy tests.
5. Re-run framework/test proofs:
   - `cargo test -p fret --lib`
   - `cargo test -p fret-cookbook --lib`
   - targeted `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app ...`
6. Delete the disabled `AppUi` `Deref` / `DerefMut` impls from `ecosystem/fret/src/view.rs`.
7. Re-run the package compile proofs after the hard delete.

## Findings

### 1) The last real blocker was a first-party teaching-surface late-landing seam

`fret-cookbook` and `fret-examples` were already clean under no-`Deref`, but
`fret-ui-gallery` still failed in `action_first_view.rs` because the snippet ended its `AppUi`
render path with `.into_element(cx)` instead of the capability-first default-lane landing
`.into_element_in(cx)`.

That is exactly the kind of drift this lane is supposed to catch:

- the framework surface was already sufficient,
- the repo's own teaching surface was still teaching the old implicit bridge.

### 2) The cookbook tails were owner splits, not missing `AppUi` surface

The remaining cookbook failures confirmed the same owner rule as the earlier example audits:

- `toast_basics` needs explicit mutable host access through `cx.app_mut()` plus explicit raw
  landing for `Toaster::into_element(cx.elements())`,
- `effects_layer_basics` is a manual/effects-level builder proof and should keep tracked reads on
  `AppUi` before entering `let cx = cx.elements();` for the raw builder phase.

No additional `AppUi` forwarding was justified.

### 3) Removing `Deref` is now a contract clarification, not a behavior gamble

Once the cookbook and UI Gallery surfaces were corrected, deleting the disabled `Deref` impls
caused no new compile failures across the main first-party consumer set:

- `fret`
- `fret-examples`
- `fret-cookbook`
- `fret-ui-gallery`

That is the meaningful threshold for this lane: the repo no longer needs a hidden compatibility
bridge to keep its own default app-authoring story alive.

### 4) The framework wording can now state the boundary directly

`ecosystem/fret/src/view.rs` now documents the real rule:

- `AppUi` does not implement `Deref<Target = ElementContext<...>>`,
- ordinary app-facing render authoring stays on the grouped `AppUi` lane,
- raw builder/state/leaf escape hatches must be spelled explicitly with `cx.elements()`.

The associated source-policy test now checks for the absence of `std::ops::Deref{,Mut}` rather
than preserving a historical compatibility comment.

## Evidence

- `ecosystem/fret/src/view.rs`
- `apps/fret-cookbook/examples/toast_basics.rs`
- `apps/fret-cookbook/examples/effects_layer_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
- `cargo check -p fret --all-targets --message-format short`
- `cargo check -p fret-examples --all-targets --message-format short`
- `cargo check -p fret-cookbook --all-targets --message-format short`
- `cargo check -p fret-ui-gallery --all-targets --message-format short`
- `cargo test -p fret --lib`
- `cargo test -p fret-cookbook --lib`
- targeted UI Gallery tests:
  - `gallery_sources_do_not_depend_on_the_legacy_fret_prelude`
  - `action_first_view_snippet_prefers_action_alias_for_activation_only_widgets`
  - `command_snippets_prefer_ui_cx_on_the_default_app_surface`

## Outcome

The repo has now crossed the boundary that the earlier no-`Deref` audits were only approximating:

1. `AppUi` no longer ships any `Deref` / `DerefMut` bridge to `ElementContext`.
2. The first-party cookbook/examples/gallery surfaces that represent the default authoring story
   compile in that shipped no-`Deref` state.
3. The framework contract is clearer: inherent `AppUi` helpers are deliberate app-lane affordances,
   and raw `ElementContext` ownership is always explicit at `cx.elements()`.
4. The remaining follow-on work is no longer “remove the compatibility bridge”; it is to finish
   the broader M5 cleanup around `UiCx` and final closeout evidence.
