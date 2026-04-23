# ImUi Collection Context Menu v1

Status: closed closeout reference
Last updated: 2026-04-23

Status note (2026-04-23): this document remains the lane-opening rationale. The shipped verdict now
lives in `M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md` and
`CLOSEOUT_AUDIT_2026-04-23.md`. References below to broader collection action breadth should be
read as lane-opening rationale rather than an invitation to reopen either the collection
delete-action folder or generic menu/key-owner/helper questions.

Related:

- `M0_BASELINE_AUDIT_2026-04-23.md`
- `M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md`
- `CLOSEOUT_AUDIT_2026-04-23.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_context_menu_surface.rs`
- `repo-ref/imgui/imgui_demo.cpp`

This lane exists because the closed collection delete-action follow-on already proved the current
asset-browser surface can own deeper collection action policy locally, but it explicitly deferred
context-menu breadth.

The narrow remaining question is now:

> land one app-owned collection context-menu slice on the existing proof surface, keep the generic
> menu/key-owner/helper widening verdicts closed, and avoid turning one demo's quick actions into
> a new public `fret-ui-kit::imui` collection context-menu helper.

## Why this is a new lane

This work should not be forced back into `imui-collection-delete-action-v1`.

That folder is already closed on a bounded delete-selected verdict. Reopening it would blur:

- collection depth already shipped
  - background marquee / box-select, collection keyboard owner, delete-selected via key/button;
- collection depth still open
  - right-click quick actions, selection adoption for item context, and broader select-all /
    rename / menu-owned command posture.

This work also should not reopen the generic menu policy or key-owner lanes.
The question here is narrower and first-party:

- does the current collection-first proof need one app-owned context menu,
- while still leaving generic popup/menu floors, key-owner APIs, runtime widening, and shared
  collection command helpers closed?

## Assumptions-first baseline

### 1) The closed collection delete-action lane already deferred context-menu action breadth.

- Evidence:
  - `docs/workstreams/imui-collection-delete-action-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this folder would duplicate or muddy the delete-action closeout instead of owning the next
    narrow collection slice.

### 2) The current proof surface already has the right ingredients for a narrow app-owned collection context menu:

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
  - `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would invent extra helper surface instead of exercising the already-shipped popup/menu
    floor from one real proof surface.

### 3) The menu/popup helper floor already exists generically, so this lane is not a justification to widen shared helper ownership.

- Evidence:
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`
  - `apps/fret-examples/src/imui_response_signals_demo.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would accidentally reopen the generic menu-policy closeout for a product-owned
    collection action question.

### 4) Dear ImGui keeps the asset-browser context menu at the proof surface and routes delete through the same selection model instead of inventing a separate command contract.

- Evidence:
  - `repo-ref/imgui/imgui_demo.cpp`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - the lane could overfit the local demo rather than following the upstream parity posture.

### 5) The lane should land and close narrowly.

- Evidence:
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - the folder would expand into select-all, rename, or shared helper growth instead of one
    reviewable context-menu slice.

## Goals

1. Land one app-owned collection context-menu slice on the current collection-first proof.
2. Reuse the existing delete helper and popup/menu floors instead of inventing a second action path.
3. Freeze that generic menu/key-owner/helper widening verdicts remain intact.
4. Leave one repro, one gate package, and one evidence set for the slice.

## Non-goals

- Widening `crates/fret-ui`.
- Widening public `fret-ui-kit::imui` with a collection context-menu helper.
- Reopening generic menu/tab policy ownership.
- Reopening the generic key-owner question.
- Solving select-all, rename, or broader command palette integration.
- Solving lasso / freeform drag-rectangle policy.

## Initial target surface

The current collection proof already has the ingredients this slice needs:

- stable item ids,
- visible-order reversal,
- `ImUiMultiSelectState<Arc<str>>`,
- app-owned keyboard owner and delete helper,
- and generic popup/menu seams that already exist elsewhere in first-party proof.

The first landable target is therefore narrow:

1. reuse the current app-owned delete helper inside one shared collection popup scope,
2. support right-click on both assets and collection background,
3. adopt right-clicked unselected assets into selection before opening the popup,
4. keep quick actions explicit in the proof demo instead of inventing a shared collection command
   surface,
5. and leave select-all / rename / broader command breadth for different follow-ons.

## Default owner split

### `apps/fret-examples`

Owns:

- the shared popup scope id and anchor request model,
- the selection-adoption rule for right-clicked assets,
- the quick-action menu content and reuse of the delete helper,
- and the source-policy teaching surface for this lane.

### `fret-ui-kit::imui`

Owns:

- the existing popup/menu helper floor and response signals this proof builds on,
- but not a new public collection context-menu helper in this lane.

### Not owned here

- `docs/workstreams/imui-collection-delete-action-v1/`
  - remains the closed delete-action verdict.
- `docs/workstreams/imui-menu-tab-policy-depth-v1/`
  - remains the closed generic menu floor verdict.
- `docs/workstreams/imui-key-owner-surface-v1/`
  - remains the closed generic key-owner verdict.
- `crates/fret-ui`
  - runtime/mechanism contract stays unchanged.
- select-all / rename / command palette integration
  - remain separate follow-ons if stronger first-party proof appears later.

## First landable target

Do not start by designing a shared collection context-menu helper or broader command surface.

The first correct target is:

- one proof-local popup-anchor model and one shared popup scope inside
  `apps/fret-examples/src/imui_editor_proof_demo.rs`,
- right-click on items/background plus a tiny quick-actions menu,
- source-policy and unit-test gates that keep the slice visible,
- and a closeout that freezes generic menu/key-owner/helper widening as still unjustified.
