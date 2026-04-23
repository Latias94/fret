# ImUi Collection Inline Rename v1

Status: closed closeout reference
Last updated: 2026-04-23

Status note (2026-04-23): this document remains the lane-opening rationale. The shipped verdict now
lives in `M1_APP_OWNED_INLINE_RENAME_SLICE_2026-04-23.md` and
`CLOSEOUT_AUDIT_2026-04-23.md`.

Related:

- `M0_BASELINE_AUDIT_2026-04-23.md`
- `M1_APP_OWNED_INLINE_RENAME_SLICE_2026-04-23.md`
- `CLOSEOUT_AUDIT_2026-04-23.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-collection-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_rename_surface.rs`
- `repo-ref/imgui/imgui.h`

This lane exists because the closed collection rename lane already landed modal/dialog breadth and
left inline product depth open.

The narrow remaining question is now:

> can the current collection-first proof ship one app-owned inline rename slice that feels more
> editor-grade, while still keeping generic `imui` helper growth and runtime widening out of
> scope?

## Why this is a new lane

This work should not be forced back into `imui-collection-rename-v1`.

That folder is already closed on a bounded rename verdict. Reopening it would blur:

- collection depth already shipped
  - background marquee / box-select, collection keyboard owner, delete-selected, context-menu,
    zoom/layout, select-all, modal rename;
- collection depth still open
  - inline rename posture, any second proof surface strong enough to justify shared helper growth,
    and any broader collection command/helper package.

This work also should not widen `crates/fret-ui` or generic `fret-ui-kit::imui` collection policy.

## Assumptions-first baseline

### 1) The closed collection rename lane already landed modal/dialog breadth and left inline product depth open.

- Evidence:
  - `docs/workstreams/imui-collection-rename-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this folder would duplicate or muddy the modal rename closeout instead of owning the next
    narrow product-depth slice.

### 2) The current proof surface already has the right ingredients for a narrow app-owned inline rename slice:

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would invent a second proof or a shared helper instead of exercising the existing
    active-tile, context-menu, and selection seams on one real surface.

### 3) The repo already has an editor-owned inline text-entry control we can embed locally without widening `fret-ui-kit::imui`.

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `ecosystem/fret-ui-editor/src/controls/text_field.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would accidentally reopen generic input/focus helper questions instead of reusing a
    shipped editor-owned control locally.

### 4) Dear ImGui-class collection/product depth now points at inline rename posture more than another popup contract.

- Evidence:
  - `repo-ref/imgui/imgui.h`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - the lane could overfit the local demo to a stale modal posture instead of improving the
    collection proof where editor hand-feel still matters.

### 5) The lane should land and close narrowly.

- Evidence:
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - the folder would expand into generic inline-edit helper growth or a second proof surface
    before one reviewable inline rename slice ships.

## Goals

1. Land one app-owned inline rename slice on the current collection-first proof.
2. Reuse the existing F2 shortcut plus the current context-menu entry.
3. Keep the inline editor inside the current active tile instead of reopening popup ownership.
4. Leave one repro, one gate package, and one evidence set for the slice.

## Non-goals

- Widening `crates/fret-ui`.
- Widening public `fret-ui-kit::imui` with a generic inline-edit or collection rename helper.
- Reopening the closed modal rename lane.
- Adding a second proof surface.
- Turning collection product depth into a generic command package.

## Initial target surface

The closed collection rename lane already landed modal/dialog depth.
The first landable target is therefore still narrow:

1. route F2 plus the existing context-menu entry through one app-owned inline rename session,
2. render the editor inside the existing active asset tile,
3. keep stable ids, visible order, and collection-scope selection ownership unchanged,
4. restore focus back to the existing collection proof after commit/cancel,
5. and leave second-proof-surface / broader helper pressure for different follow-ons.

## Default owner split

### `apps/fret-examples`

Owns:

- inline rename session targeting,
- inline focus handoff and focus restore,
- proof-local commit/cancel policy,
- and the source-policy teaching surface for this lane.

### `fret-ui-editor`

Owns:

- the existing `TextField` control this proof embeds locally,
- but not a new shared collection inline-edit helper in this lane.

### Not owned here

- `docs/workstreams/imui-collection-rename-v1/`
  - remains the closed modal rename closeout.
- `crates/fret-ui`
  - runtime/mechanism contract stays unchanged.
- second proof surface / broader collection command package
  - remain separate follow-ons if stronger first-party proof appears later.

## First landable target

Do not reopen the closed modal lane by widening `fret-ui-kit::imui` with a generic inline-edit helper.

The first correct target is:

- one proof-local inline rename editor mounted inside the active collection tile,
- one proof-local focus handoff so the editor can auto-focus without new `imui` surface area,
- source-policy, surface, and unit-test gates that keep the slice visible,
- and a closeout that freezes generic key-owner/helper widening as still unjustified.
