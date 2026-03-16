# Action-First Authoring + View Runtime (Fearless Refactor v1) — Post-v1 Execution Checklist

Status: draft, current execution checklist
Last updated: 2026-03-16

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_SURFACE_SHORTLIST.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/SHARED_SURFACE_EVIDENCE_MATRIX_2026-03-16.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/DEFAULT_PATH_PRODUCTIZATION.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/DEFAULT_PATH_PRODUCTIZATION_AUDIT_2026-03-10.md`
- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`

---

## Purpose

This note turns the current post-v1 conclusions into an operational checklist.

It exists to answer:

> What should the repo do next, in what order, and what should maintainers explicitly refuse to do
> until stronger evidence exists?

This is not a new API design note.
It is an execution rule for keeping post-v1 work narrow, reversible, and correctly owned.

---

## Hard gates

These rules apply before opening any new action-first authoring change:

1. do not use Todo-only pain as sufficient justification for widening shared public surface,
2. do not use advanced/manual-assembly surfaces as default app-lane API justification,
3. do not answer conversion-surface pain by adding unrelated action/local-state sugar,
4. do not let `AppActivateExt` become a growth surface again,
5. do not reopen macros, broad builder-family expansion, or `DataTable` helper expansion on the
   default-path lane.

The evidence rule for gate 1 lives in
`SHARED_SURFACE_EVIDENCE_MATRIX_2026-03-16.md`.

---

## Execution order

### 1. Keep authoring-surface closeout and action-first ownership separate

- Treat `fret-ui-shadcn` discovery-lane closure and `fret` root-lane budgeting as owned first by
  `authoring-surface-and-ecosystem`.
- Do not reopen action-first API expansion while those lane-curation items are still the unstable
  factor behind first-contact confusion.
- Read this workstream as owning default-path density and bridge shrink, not crate-discovery
  strategy.

### 2. Keep conversion-surface cleanup on the dedicated `into-element` track

- Route `.into_element(cx)` vocabulary collapse through
  `docs/workstreams/into-element-surface-fearless-refactor-v1/`.
- Do not mix conversion cleanup with generic action/local-state helper expansion.
- The outcome to chase is one clearer conversion story, not another helper family.

### 3. Run the next default-path productization batch as one unit

- Use the canonical trio as the main implementation/evidence set:
  - `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
  - `apps/fret-examples/src/todo_demo.rs`
  - `apps/fretboard/src/scaffold/templates.rs`
- Move code, generated-template expectations, and first-contact wording together.
- Prefer docs cleanup, source-policy cleanup, and adoption of existing helpers before proposing new
  shared public API.
- Treat keyed/list/default child-collection cleanup as productization unless non-Todo evidence says
  otherwise.

### 4. Keep bridge shrink as a standing policy gate

- `AppActivateExt` must keep shrinking or stay flat; it must not grow.
- Prefer widget-native `.action(...)`, `.action_payload(...)`, or widget-owned `.on_activate(...)`
  when those surfaces already exist.
- Treat any proposal to add a new first-party bridge impl as blocked unless it proves there is no
  stable widget-owned alternative.

### 5. Only reopen shared public-surface discussion on the current watch-list items

The current watch list is intentionally small:

- tracked-value read density,
- coordinated `locals::<A>(...)` write-closure ceremony.

For either item, reopen only if all of the following are true:

1. current docs/adoption/productization work did not remove the pressure,
2. the same pressure appears on the canonical trio and at least one additional non-Todo
   default-facing surface,
3. the proposal keeps action identity, invalidation ownership, and transaction/key context
   explicit,
4. the result improves the default product surface rather than an advanced/manual-assembly lane.

### 6. Keep the following explicitly deferred

- generic keyed-row/shared payload sugar beyond the canonical-trio productization lane,
- macro expansion,
- broad builder-family expansion,
- `DataTable`-specific helper growth inside this workstream,
- local-state architecture redesign disguised as helper work,
- advanced/runtime-owned manual-assembly cleanup as default app-lane work.

---

## Landable batch checklist

For any post-v1 batch on this workstream, confirm all of the following before calling it complete:

1. the change clearly belongs to default-path productization, bridge shrink, or the explicit
   watch-list reopen path,
2. the canonical trio remains aligned on one intended writing style after the change,
3. docs/templates/examples teach the same decision, not three different interpretations,
4. no new bridge growth or accidental public-surface sprawl was introduced,
5. related evidence notes (`POST_V1_ENDGAME_SUMMARY`, shortlist, evidence matrix, or TODO) still
   read consistently.

---

## Verification floor

When a batch changes the default authoring story, keep at least the following verification floor in
view:

- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --no-fail-fast`
- `cargo test -p fretboard scaffold::templates::tests::todo_template_uses_default_authoring_dialect -- --exact`
- `cargo test -p fretboard scaffold::templates::tests::simple_todo_template_has_low_adapter_noise_and_no_query_selector -- --exact`
- `cargo test -p fretboard scaffold::templates::tests::template_readmes_capture_authoring_guidance -- --exact`

Add narrower gates when a batch changes a specific component family or conversion seam.

---

## Practical summary

If a maintainer needs one blunt reading:

1. keep `into-element` moving on its dedicated conversion track,
2. keep this workstream focused on default-path productization,
3. keep bridge residue shrinking,
4. only reopen shared public API on multi-surface evidence,
5. keep everything else deferred until that evidence exists.
