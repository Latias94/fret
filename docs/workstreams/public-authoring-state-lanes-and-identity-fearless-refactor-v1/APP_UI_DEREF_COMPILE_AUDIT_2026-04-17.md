# AppUi Deref Compile Audit — 2026-04-17

Status: Evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEREF_PRESSURE_CLASSIFICATION_AUDIT_2026-04-15.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_ROOT_ACCESSOR_AUDIT_2026-04-15.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/UICX_DEFAULT_PRELUDE_DEMOTION_AUDIT_2026-04-17.md`
- `ecosystem/fret/src/view.rs`

## Scope

Re-run the “what if `AppUi` lost `Deref` today?” audit after the recent render-lane cleanup
slices:

- default-lane wording freeze,
- explicit `fret::env` lane freeze,
- `UiCx` demotion out of `fret::app::prelude::*`.

This note is intentionally evidence-only. It does not land `Deref` removal.

## Method

Temporary local audit patch:

- disable `impl Deref for AppUi<'_, '_, _>`
- disable `impl DerefMut for AppUi<'_, '_, _>`

Then run:

- `cargo check -p fret-examples --all-targets --message-format short`
- `cargo check -p fret-cookbook --all-targets --message-format short`

The temporary patch was reverted immediately after collecting the failure clusters.

## Findings

### 1) The repo is still not ready for blind `Deref` deletion

The current compile audit still fails hard:

- `fret-examples`: roughly 150 errors
- `fret-cookbook`: multiple example failures on the same underlying clusters

That is enough evidence to reject any “delete `Deref` now and mop up later” plan.

### 2) The remaining failures now fall into a much clearer set of buckets

The dominant clusters are no longer one undifferentiated grep tail.

#### A) Stale explicit `ElementContext` helper signatures

Examples still pass `&mut AppUi` into helpers or APIs that are typed as:

- `&mut ElementContext<'_, App>`
- `&mut ElementContext<'_, _>`

Representative files:

- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/src/date_picker_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/form_demo.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-cookbook/examples/effects_layer_basics.rs`

This is mostly a helper-boundary problem, not a proof that the default lane needs raw `ElementContext`.

#### B) Stale `cx.app` field syntax

The earlier accessor slice (`app()`, `app_mut()`, `window_id()`) landed, but a remaining batch
still spells:

- `cx.app`
- `&*cx.app`
- `&mut *cx.app`

Representative files:

- `apps/fret-examples/src/api_workbench_lite_demo.rs`
- `apps/fret-examples/src/emoji_conformance_demo.rs`
- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-cookbook/examples/theme_switching_basics.rs`
- `apps/fret-cookbook/examples/toast_basics.rs`
- `apps/fret-cookbook/examples/commands_keymap_basics.rs`
- `apps/fret-cookbook/examples/overlay_basics.rs`

This bucket is migration debt in first-party surfaces, not missing framework capability.

#### C) Inherited `ElementContext` method families that still lack an explicit verdict

Without `Deref`, `AppUi` loses access to method families such as:

- `text`
- `container`
- `flex`
- `text_props`
- `theme`
- `layout_query_bounds`
- `layout_query_region_with_id`
- `pointer_region`
- `request_animation_frame`
- `action_is_enabled`
- `watch_model`
- `cached_subtree_with`

This is the real framework-design bucket.

However, these names are not all the same kind of problem:

- some should probably migrate call sites onto existing `ui::*` / patch-builder / grouped helper
  lanes (`text`, `container`, `flex`, parts of `text_props` usage),
- some need an explicit app-facing owner decision (`request_animation_frame`,
  `action_is_enabled`, layout-query helpers, maybe `pointer_region`),
- some may remain advanced/raw-only rather than being promoted (`cached_subtree_with`,
  some `watch_model` cases).

#### D) Type-inference fallout is mostly secondary

Several `E0282` / `E0277` failures appear after the method/signature losses above.

These should not be treated as primary design signals until the three buckets above are reduced.

### 3) The next correct framework step is classification, not blanket forwarding

The audit does **not** justify copying a large swath of `ElementContext` methods onto `AppUi`
verbatim.

That would risk re-importing the raw kernel surface under a new facade name.

The correct next step is narrower:

1. keep migrating first-party stale syntax to already-approved lanes,
2. classify the remaining inherited method families into:
   - migrate-to-existing-app-lane,
   - add small explicit app-facing sugar,
   - keep advanced/raw-only,
3. only then reopen `Deref` removal.

## Evidence

- `ecosystem/fret/src/view.rs`
- `cargo check -p fret-examples --all-targets --message-format short`
- `cargo check -p fret-cookbook --all-targets --message-format short`

## Outcome

The current verdict is firmer than the older audit:

1. `AppUi` `Deref` removal is still premature.
2. The remaining debt is now classified enough to attack in targeted slices.
3. The next framework work should focus on inherited method-family classification and targeted
   first-party migrations, not on a blind compatibility-bridge deletion.
