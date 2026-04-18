# AppUi Advanced Helper Raw Owner Audit — 2026-04-17

Status: Landed slice + evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_MANUAL_FORM_RAW_OWNER_AUDIT_2026-04-17.md`
- `apps/fret-examples/src/postprocess_theme_demo.rs`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Scope

Classify the next advanced/reference no-`Deref` failure clusters:

- `apps/fret-examples/src/postprocess_theme_demo.rs`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`

The question is whether these surfaces want broader `AppUi` forwarding, or whether they should
keep their helper/render shell phase on an explicit raw owner.

## Method

1. Inspect the post-`form_demo` no-`Deref` failure map.
2. Confirm that both files already split naturally into:
   - an `AppUi` phase for tracked state/global reads
   - an advanced helper/render shell phase typed as `ElementContext` / `UiCx`
3. Land the narrow callsite fix:
   - keep app-lane reads on `AppUi`
   - then enter `let cx = cx.elements();`
4. Keep the fallback/teaching surface explicit:
   - `postprocess_theme_demo` uses `into_element_in(cx)` for the early unavailable fallback
   - `imui_interaction_showcase_demo` keeps context-owned theme reads in its raw helper phase
5. Lock the choice with targeted source-policy gates and re-run:
   - `cargo test -p fret-examples --lib selected_raw_owner_examples_keep_escape_hatches_explicit`
   - `cargo test -p fret-examples --lib selected_element_context_examples_prefer_context_theme_reads`
   - `cargo test -p fret-examples --lib renderer_theme_bridge_proofs_keep_explicit_host_theme_reads`
   - `cargo check -p fret-examples --all-targets --message-format short`
6. Re-run the temporary local no-`Deref` audit:
   - `cargo check -p fret-examples --all-targets --message-format short`
   - `cargo check -p fret-cookbook --all-targets --message-format short`

The temporary `AppUi` no-`Deref` patch was reverted immediately after collecting the new failure
map.

## Findings

### 1) These are advanced helper/raw-shell owners, not missing default-lane APIs

Both files already advertise themselves as advanced/reference surfaces.

Their remaining failures were not about grouped state/actions/data access. They were about passing
`&mut AppUi` into helpers that intentionally speak raw `ElementContext` / `UiCx`:

- post-process renderer/theme bridge helpers
- IMUI showcase shell/composite helpers

That is not evidence that `AppUi` should absorb those helper surfaces.

### 2) The correct fix is an explicit callsite boundary

The owner-correct shape for both files is:

- do tracked reads and host/global access on `AppUi`
- then spell `let cx = cx.elements();`
- then call the advanced helper/render shell phase on the raw owner

This keeps the advanced lane explicit and avoids re-importing a broad raw helper surface into the
default app façade.

### 3) The no-`Deref` audit shrinks sharply again

With a temporary local patch that removes both `impl Deref` and `impl DerefMut` from `AppUi`:

- `fret-examples` drops from `88` error lines to `67`
- `fret-cookbook` remains at `19`
- neither `postprocess_theme_demo` nor `imui_interaction_showcase_demo` appears in the failure map

The remaining top `fret-examples` clusters are now led by:

- `date_picker_demo.rs` (`9`)
- `embedded_viewport_demo.rs` (`9`)
- `todo_demo.rs` (`8`)
- `async_playground_demo.rs` (`6`)

That is strong evidence that both advanced clusters were callsite-owner debt rather than new
framework surface pressure.

## Evidence

- `apps/fret-examples/src/postprocess_theme_demo.rs`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `cargo test -p fret-examples --lib selected_raw_owner_examples_keep_escape_hatches_explicit`
- `cargo test -p fret-examples --lib selected_element_context_examples_prefer_context_theme_reads`
- `cargo test -p fret-examples --lib renderer_theme_bridge_proofs_keep_explicit_host_theme_reads`
- `cargo check -p fret-examples --all-targets --message-format short`
- temporary local no-`Deref` audit:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `cargo check -p fret-cookbook --all-targets --message-format short`
  - `/tmp/fret_examples_noderef_postprocess_fixed.txt`
  - `/tmp/fret_cookbook_noderef_postprocess_fixed.txt`

## Outcome

The repo now keeps these advanced helper/render-shell surfaces explicit:

1. `postprocess_theme_demo` keeps app-lane reads on `AppUi`, then enters `cx.elements()` for its
   advanced renderer/theme helper phase
2. `imui_interaction_showcase_demo` keeps app-lane reads on `AppUi`, then enters `cx.elements()`
   for its advanced shell/composite helper phase
3. the next work should move to the remaining mixed helper-signature/default-surface clusters
