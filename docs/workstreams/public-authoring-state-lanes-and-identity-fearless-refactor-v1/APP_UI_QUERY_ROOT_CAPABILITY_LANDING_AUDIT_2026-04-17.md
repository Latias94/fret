# AppUi Query Root Capability Landing Audit — 2026-04-17

Status: Landed slice + targeted evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_CONTINUOUS_FRAMES_RUNTIME_OWNER_AUDIT_2026-04-17.md`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Scope

Decide how the remaining `query_demo` and `query_async_tokio_demo` no-`Deref` failures should be
classified:

- should these first-contact query proofs reopen a raw `ElementContext` lane at the app root,
- should they require another framework surface,
- or should ordinary query-status/detail builders stay on the existing capability-first
  late-landing lane?

This note is intentionally about app-root late-landing only. It does not change grouped query
ownership, theme reads, or the existing page-shell contract.

## Method

1. Inspect the post-`hello_world_compare_demo` no-`Deref` failure map and confirm that:
   - `query_demo` fails only on `status_row`, `buttons`, and `detail_body` late-landing
   - `query_async_tokio_demo` fails on the same three ordinary builders
2. Keep the existing owner split unchanged:
   - grouped state/actions/effects/data on `AppUi`
   - `query_handle.read_layout(cx)` on the app lane
   - `query_page(theme, card)` as the existing default page-shell contract
3. Move only the ordinary root detail-builders onto the existing capability lane:
   - import `fret_ui_kit::IntoUiElementInExt as _`
   - use `into_element_in(cx)` for `status_row`, `buttons`, and `detail_body` in both demos
4. Add a source-policy gate in `apps/fret-examples/src/lib.rs` that freezes those markers for both
   demos
5. Re-run:
   - `cargo test -p fret-examples --lib query_demos_prefer_capability_first_landing_for_root_detail_builders`
   - `cargo check -p fret-examples --all-targets --message-format short`
6. Re-run a temporary local no-`Deref` spot-check:
   - disable `impl Deref` / `impl DerefMut` for `AppUi` temporarily in `ecosystem/fret/src/view.rs`
   - `cargo check -p fret-examples --all-targets --message-format short`

The temporary `AppUi` no-`Deref` patch was reverted immediately after collecting the new failure
map.

## Findings

### 1) This pair was ordinary root late-landing debt, not a new surface gap

Both files failed only at ordinary app-root builder landing points:

- `status_row`
- `buttons`
- `detail_body`

There were no new raw-owner signals such as:

- `text_props(...)`
- `cached_subtree_with(...)`
- `pointer_region(...)`
- root-level `cx.elements()` escape hatches

So the correct interpretation was not “grow `AppUi` again”, but “stop letting ordinary query
detail builders lean on implicit `Deref`”.

### 2) The correct fix is capability-first landing on the existing first-contact query lane

The owner-correct shape is now:

- grouped state/actions/effects/data stay on `AppUi`
- the existing `query_page(theme, card)` contract stays unchanged
- ordinary query detail-builders use `into_element_in(cx)` in both demos

That keeps the first-contact query teaching surface aligned with the rest of the app-facing lane.

### 3) The no-`Deref` spot-check removes both query demos and shrinks the tail again

With a temporary local patch that removes both `impl Deref` and `impl DerefMut` from `AppUi`:

- `fret-examples` drops from `22` to `16` previous errors,
- `query_demo` no longer appears in the failure map,
- `query_async_tokio_demo` no longer appears in the failure map,
- and the current leading `fret-examples` clusters are now:
  - `drop_shadow_demo.rs` (`3`)
  - `ime_smoke_demo.rs` (`3`)
  - `sonner_demo.rs` (`3`)
  - `components_gallery.rs` (`2`)

That is strong evidence that this was a pure capability-first landing cleanup slice.

## Evidence

- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `cargo test -p fret-examples --lib query_demos_prefer_capability_first_landing_for_root_detail_builders`
- `cargo check -p fret-examples --all-targets --message-format short`
- temporary local no-`Deref` spot-check:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `/tmp/fret_examples_noderef_query_pair_v8.txt`

## Outcome

The repo now has an explicit owner answer for the first-contact query proof pair:

1. query state/effects/data stay on `AppUi`
2. `query_page(theme, card)` stays as the existing default page-shell contract
3. ordinary root detail-builders use `into_element_in(cx)` in both demos
4. both query demos are no longer part of the current no-`Deref` failure tail
