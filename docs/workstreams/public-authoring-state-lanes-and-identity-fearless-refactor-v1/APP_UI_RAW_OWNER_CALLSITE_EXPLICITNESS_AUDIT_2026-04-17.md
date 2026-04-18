# AppUi Raw Owner Callsite Explicitness Audit — 2026-04-17

Status: Landed slice + evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEREF_COMPILE_AUDIT_2026-04-17.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_RUNTIME_GATING_AND_FRAME_OWNER_AUDIT_2026-04-17.md`

## Scope

Apply owner-correct follow-up migrations for method families that the current lane no longer needs
to decide at the framework surface:

- explicit raw retained/pointer-region callsites,
- and one remaining app-facing shared-model read that already has a grouped selector owner.

This note is intentionally narrower than a new framework API slice.

## Method

1. Migrate clear raw-owner callsites:
   - `apps/fret-examples/src/components_gallery.rs`
   - `apps/fret-cookbook/examples/drag_basics.rs`
   - `apps/fret-cookbook/examples/gizmo_basics.rs`
2. Migrate one shared-model app-facing read to the existing grouped selector lane:
   - `apps/fret-examples/src/editor_notes_device_shell_demo.rs`
3. Add source-policy gates:
   - `apps/fret-examples/src/lib.rs`
   - `apps/fret-cookbook/src/lib.rs`
4. Re-run the temporary no-`Deref` audit:
   - `cargo check -p fret-examples --all-targets --message-format short`
   - `cargo check -p fret-cookbook --all-targets --message-format short`

The temporary no-`Deref` patch was reverted immediately after collecting the new failure clusters.

## Findings

### 1) Raw retained/pointer-region owners should move at the callsite, not by widening `AppUi`

This slice confirms that the repo already has enough explicit vocabulary for these owners:

- `components_gallery` can enter its retained subtree owner with `let cx = cx.elements();`
  instead of depending on implicit `AppUi` inheritance for `cached_subtree_with(...)`,
  retained-only `text(...)`, `container(...)`, and related raw retained helpers on that branch.
- `drag_basics` and `gizmo_basics` can spell `cx.elements().pointer_region(...)` directly,
  keeping pointer-region hooks on the explicit raw interaction lane.

These changes shrink `AppUi` pressure without creating new façade APIs.

### 2) Shared-model app-facing reads should prefer the grouped selector lane when it already exists

`editor_notes_device_shell_demo` was still using `watch_model(...).paint().cloned_or_default()`
for the same three-model read that `editor_notes_demo` had already moved onto
`cx.data().selector_model_paint(...)`.

Migrating that device-shell proof onto the same grouped lane is more correct than forcing it
through a raw `cx.elements().watch_model(...)` escape hatch.

### 3) The second no-`Deref` audit got materially smaller

Compared with the earlier 2026-04-17 compile audit:

- `fret-examples` dropped from roughly `149` errors to `140`,
- `fret-cookbook` no longer reports the earlier `drag_basics` `pointer_region(...)` failure,
- `fret-examples` no longer reports the earlier `editor_notes_device_shell_demo`
  `watch_model(...)` failures,
- and `components_gallery` no longer reports the retained-branch `cached_subtree_with(...)`
  failure cluster from the previous audit snapshot.

The remaining failure map is now even more concentrated on:

- stale `cx.app` field syntax,
- helper signatures that still require `&mut ElementContext<'_, _>`,
- unresolved app-lane verdicts such as `layout_query_*`,
- and broad inherited render helpers such as `text`, `container`, `flex`, `text_props`, `theme`.

## Evidence

- `apps/fret-examples/src/components_gallery.rs`
- `apps/fret-examples/src/editor_notes_device_shell_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-cookbook/examples/drag_basics.rs`
- `apps/fret-cookbook/examples/gizmo_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `cargo nextest run -p fret-examples selected_raw_owner_examples_keep_escape_hatches_explicit`
- `cargo nextest run -p fret-cookbook advanced_interaction_examples_keep_pointer_region_on_explicit_elements_escape_hatch`
- temporary local no-`Deref` audit:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `cargo check -p fret-cookbook --all-targets --message-format short`

## Outcome

The repo now has a cleaner split for several remaining `AppUi`-era tail callsites:

1. Raw retained/pointer-region owners are more explicit at the callsite.
2. One lingering app-facing shared-model read now uses the existing grouped selector lane.
3. The next structural work can focus on helper signatures, `cx.app` syntax debt, and unresolved
   app-facing method-family verdicts rather than revisiting these already-classified owners.
