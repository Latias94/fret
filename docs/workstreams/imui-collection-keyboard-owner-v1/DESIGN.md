# ImUi Collection Keyboard Owner v1

Status: closed closeout reference
Last updated: 2026-04-22

Status note (2026-04-22): this document remains the lane-opening rationale. The shipped verdict now
lives in `M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md` and
`CLOSEOUT_AUDIT_2026-04-22.md`. References below to broader collection keyboard depth should be
read as lane-opening rationale rather than an invitation to reopen either the generic key-owner
lane or this folder into a broader shortcut backlog.

Related:

- `M0_BASELINE_AUDIT_2026-04-22.md`
- `M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md`
- `CLOSEOUT_AUDIT_2026-04-22.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_keyboard_owner_surface.rs`
- `repo-ref/imgui/imgui_demo.cpp`
- `repo-ref/imgui/imgui.h`

This lane exists because the closed collection box-select follow-on already proved the current
asset-browser surface can own deeper collection policy locally, but it explicitly deferred richer
keyboard-owner behavior.

The narrow remaining question is now:

> land one app-owned collection-scope keyboard owner slice on the existing proof surface, keep the
> generic key-owner no-new-surface verdict intact, and avoid turning one demo's collection policy
> into a new public `fret-ui-kit::imui` helper.

## Why this is a new lane

This work should not be forced back into `imui-collection-box-select-v1`.

That folder is already closed on a bounded marquee / box-select verdict. Reopening it would blur:

- collection depth already shipped
  - background-only marquee / box-select;
- collection depth still open
  - collection-scope keyboard-owner behavior around active tile movement, range extension, and
    clear-selection posture.

This work also should not reopen `imui-key-owner-surface-v1`.
That lane is closed on a generic no-new-surface verdict for broader immediate shortcut ownership.
The question here is narrower and first-party:

- does the current collection-first proof need app-owned keyboard selection depth,
- while still leaving generic key-owner facades and shared helper widening closed?

## Assumptions-first baseline

### 1) The generic key-owner lane stays closed; this lane is collection proof depth, not generic helper growth.

- Evidence:
  - `docs/workstreams/imui-key-owner-surface-v1/M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md`
  - `docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this folder would silently reopen broader shortcut-owner API pressure that already closed on a
    no-new-surface verdict.

### 2) The closed box-select lane already froze the correct follow-on split.

- Evidence:
  - `docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would duplicate or muddy the box-select closeout instead of owning the next narrow
    depth slice.

### 3) The narrowest correct owner is still the existing proof demo.

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would promote collection keyboard policy into shared helper code before repeated
    first-party proof exists.

### 4) `crates/fret-ui` and public `fret-ui-kit::imui` surface stay unchanged here.

- Evidence:
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would drift from proof-surface policy into runtime or public-surface widening.

### 5) Dear ImGui-like collection keyboard depth is still broader than one slice, so this lane should land and close narrowly.

- Evidence:
  - `repo-ref/imgui/imgui_demo.cpp`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - the folder would expand into lasso, delete-action, or generic shortcut-owner backlog instead of
    one reviewable keyboard slice.

## Goals

1. Land one app-owned collection keyboard-owner slice on the current asset-browser proof.
2. Keep the implementation explicit in `apps/fret-examples/src/imui_editor_proof_demo.rs`.
3. Freeze that the generic key-owner no-new-surface verdict remains intact.
4. Leave one repro, one gate package, and one evidence set for the slice.

## Non-goals

- Widening `crates/fret-ui`.
- Widening public `fret-ui-kit::imui` with a collection keyboard-owner helper.
- Reopening the generic `SetNextItemShortcut()` / `SetItemKeyOwner()` question.
- Solving lasso / freeform rectangle policy.
- Solving collection delete/select-all/action semantics.
- Reopening the broader collection/pane proof closeout.

## Initial target surface

The current collection proof already has the ingredients this slice needs:

- stable item ids,
- visible-order reversal,
- `ImUiMultiSelectState<Arc<str>>`,
- app-owned background focus and box-select scope,
- and selected-set drag/drop.

The first landable target is therefore narrow:

1. make the collection scope itself a focusable keyboard owner in the proof demo,
2. keep an app-owned active-tile model,
3. use `Arrow` / `Home` / `End` to move the active tile in visible order,
4. use `Shift+navigation` to extend range from the current anchor,
5. use `Escape` to clear the current selection,
6. and keep primary-modifier shortcut claims and shared helper widening out of scope.

## Default owner split

### `apps/fret-examples`

Owns:

- the focusable collection scope,
- the active-tile model,
- the keyboard selection policy over the asset grid,
- and the source-policy teaching surface for this lane.

### `fret-ui-kit::imui`

Owns:

- the existing multi-select, selectable, drag/drop, and child-region seams this proof builds on,
- but not a new public collection keyboard-owner helper in this lane.

### Not owned here

- `docs/workstreams/imui-key-owner-surface-v1/`
  - remains the closed generic key-owner verdict.
- `crates/fret-ui`
  - runtime/mechanism contract stays unchanged.
- lasso / delete / select-all / action commands
  - remain separate follow-ons if stronger first-party proof appears later.

## First landable target

Do not start by designing a shared helper or a new generic shortcut facade.

The first correct target is:

- a focusable collection scope plus app-owned active-tile state inside
  `apps/fret-examples/src/imui_editor_proof_demo.rs`,
- explicit pure selection/navigation helpers,
- source-policy and unit-test gates that keep the slice visible,
- and a closeout that freezes generic key-owner / shared-helper widening as still unjustified.
