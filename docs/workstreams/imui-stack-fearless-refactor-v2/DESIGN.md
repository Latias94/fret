# imui stack fearless refactor v2 - design

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Dear ImGui: `repo-ref/imgui`
- egui: `repo-ref/egui`
- Zed / GPUI: `repo-ref/zed`, `repo-ref/gpui-component`
- shadcn/ui: `repo-ref/ui`
- Base UI: `repo-ref/base-ui`
- Radix primitives: `repo-ref/primitives`

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

Status: Closed

Last updated: 2026-03-31

Closeout audit:
`docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`

## Purpose

This workstream is the new execution surface for the in-tree `imui` stack after the v1 stack reset
and the narrower vocabulary/editor closure lanes.

It exists to do one more fearless pass across:

- `ecosystem/fret-imui`
- `ecosystem/fret-ui-kit::imui`
- `ecosystem/fret-ui-editor::imui`
- the first-party proof/demo/docs surfaces that teach those APIs

The goal is not to preserve compatibility or to keep older helper shapes alive.
The goal is to make the shipped stack:

- current-source-of-truth in docs,
- complete enough for editor-grade immediate authoring,
- stricter about mechanism vs policy vs adapter ownership,
- and delete-ready when overlap or direct lower-layer leakage is found.

## Current assessment

The architecture direction remains correct:

- `fret-authoring::{UiWriter, Response}` is still the right minimal shared contract.
- `fret-imui` is still an authoring frontend, not a second runtime.
- `fret-ui-kit::imui` already owns the generic richer helper layer.
- `fret-ui-editor::imui` already reads as a thin adapter layer where coverage exists.
- runtime mechanisms remain in `crates/fret-ui`.

The remaining problems are no longer stack-direction questions.
They are closure and source-of-truth questions.

Current observed drift:

- documentation drift:
  - some `imui` workstreams still describe shipped generic helpers as missing even though code
    already has `selectable`, `combo`, `table`, `virtual_list`, `separator_text`, tooltip,
    tree/disclosure, and typed drag/drop.
- adapter coverage drift:
  - `fret-ui-editor` still exports editor nouns that are not reachable through the official
    immediate adapter layer, notably `FieldStatusBadge` and `GradientEditor`,
  - and `PropertyRow` still needs an explicit boundary decision.
- proof-surface drift:
  - before this lane's proof/demo migration, `imui_editor_proof_demo` dropped below
    `fret_ui_editor::imui` for some immediate authoring parity rows.
- reviewability drift:
  - `fret-ui-kit::imui` is split by concern compared with the v1 stack reset, but several files
    still package multiple ownership seams together and should only be further split when it
    clarifies the boundary.

## Why this needs a new lane

The repo already closed the initial stack reset.
It also closed several narrower lanes that were active at the time:

- generic immediate vocabulary closure,
- editor-grade helper closure,
- sortable recipe closure,
- and ghost/drag-preview closure lanes.

What is missing now is one current document set that reflects the shipped code as of 2026-03-31
and can drive the next code-moving batch without teaching stale gaps.

Without that reset:

- contributors will reopen already-shipped helper questions,
- proof/demo code will keep bypassing the intended adapter layer,
- and deletion-ready refactors will hesitate because old notes still imply compatibility or
  missing-surface uncertainty.

## Evidence baseline

This lane starts from `BASELINE_AUDIT_2026-03-31.md`.
That audit was executed against the current local tree and a local `repo-ref/imgui` checkout at
`148bd34a7`.
That local SHA should be read as audit evidence only; `docs/repo-ref.md` remains the repo-wide
pinned-reference note.

## Goals

### G1 - Make this directory the only active source of truth for the `imui` stack

- Repoint docs entrypoints and workstream maps here.
- Downgrade stale active lanes to historical or partially superseded notes.

### G2 - Finish the thin-adapter closure for editor-owned immediate nouns

Promote the missing thin adapters in `fret-ui-editor::imui` where the declarative owner already
exists and the adapter can remain one-hop:

- `FieldStatusBadge`
- `GradientEditor`
- and a deliberate boundary decision for `PropertyRow` (promote it or explicitly keep it
  declarative-only)

### G3 - Stop first-party immediate proof surfaces from bypassing the official adapter layer

When a promoted editor adapter exists:

- first-party immediate demos should use the adapter,
- not direct `.into_element(cx)` calls on the immediate side.

### G4 - Keep `crates/fret-ui` mechanism-only

Do not widen runtime contracts to compensate for facade or proof-surface cleanup.
Any pressure that is really policy or authoring sugar stays in `ecosystem/*`.

### G5 - Delete overlap instead of preserving compatibility

This lane explicitly allows flag-day in-tree migration.
If a helper path becomes redundant or a lower-layer call path is no longer justified, delete it.

### G6 - Leave behind reviewable gates and evidence

The next code-moving pass must be locked by:

- focused tests for new adapters,
- updated proof surfaces,
- and doc/audit anchors that state what is current vs historical.

## Non-goals

- Preserving source compatibility for old `imui` helper shapes.
- Reopening already-shipped tooltip/tree/typed-drag-drop questions as if they were still missing.
- Recreating Dear ImGui's full API grammar or ID tricks.
- Pulling docking/workspace shell policy into generic `imui`.
- Growing a second editor widget implementation path outside `fret-ui-editor`.
- Expanding `fret-imui` beyond the minimal frontend role unless the shared contract is genuinely
  wrong.

## Non-negotiable boundaries

| Layer | Owns | Must not own |
| --- | --- | --- |
| `ecosystem/fret-authoring` | minimal shared authoring contract (`UiWriter`, `Response`) | rich immediate helper semantics, editor policy |
| `ecosystem/fret-imui` | minimal immediate frontend, keyed identity, output collection, minimal layout helpers | editor widgets, generic policy-heavy helpers, proof-only convenience drift |
| `ecosystem/fret-ui-kit::imui` | generic immediate vocabulary that is not editor-specific and not shell-specific | editor composites, docking/workspace policy, duplicate declarative owners |
| `ecosystem/fret-ui-editor::imui` | thin adapters over editor controls/composites already owned by `fret-ui-editor` | adapter-local widget logic, duplicate state machines, generic immediate helper ownership |
| `apps/fret-examples` / `apps/fret-demo` | proof/demo surfaces and evidence | hidden second owner for the official `imui` surface |
| `crates/fret-ui` | runtime mechanisms, layout/focus/IME/overlay contracts | authoring sugar, editor-style policy, compatibility helpers for ecosystem cleanup |

## Decision snapshot

### 1) `UiWriter` and `Response` stay where they are

Unless new evidence proves the shared contract wrong, `ecosystem/fret-authoring` remains
unchanged.
This lane is not a pretext to widen the base contract for convenience.

### 2) `fret-imui` stays minimal

`fret-imui` should not absorb more policy-heavy helpers just because first-party demos want fewer
imports.
If a helper is generic and richer, it belongs in `fret-ui-kit::imui`.
If it is editor-specific, it belongs in `fret-ui-editor::imui`.

### 3) Editor nouns must come from the editor adapter layer or remain intentionally declarative-only

If `fret-ui-editor` already owns the declarative control/composite and the immediate adapter can
remain a one-hop forwarder, the official immediate surface should expose it there.
If a surface should remain declarative-only, this lane must say so explicitly and remove
ambiguity.

Current decision (2026-03-31):

- `PropertyRow` stays declarative-only.
- Rationale:
  - it is the foundational row primitive already consumed by `PropertyGrid`,
    `PropertyGridVirtualized`, and `GradientEditor`,
  - most proof-surface `PropertyRow` usage lives inside nested declarative row closures where a
    `UiWriter` adapter would not remove the direct `.into_element(cx)` path,
  - promoting a `property_row(...)` adapter would widen the public immediate surface without
    materially reducing the real proof/demo boundary drift.

### 4) First-party immediate examples are part of the contract story

The proof surface is not just demo fluff.
If the immediate side of `imui_editor_proof_demo` still bypasses the official adapter layer, that
is architectural drift, not a stylistic preference.

### 5) Flag-day migration is correct here

In-tree call sites should move atomically.
This lane does not preserve legacy helper names or keep duplicate authoring paths alive.

## Current target closure set

### A. Source-of-truth doc cleanup

- make this v2 directory the current `imui` execution surface,
- update `docs/README.md`, `docs/roadmap.md`, `docs/todo-tracker.md`, and
  `docs/workstreams/README.md`,
- add historical status notes to stale `imui` workstreams that would otherwise teach outdated gaps.

### B. Editor adapter completion

- add `field_status_badge(...)` in `fret-ui-editor::imui`,
- add `gradient_editor(...)` in `fret-ui-editor::imui`,
- decide `property_row(...)` explicitly:
  - current decision: keep it declarative-only,
  - and treat proof/demo direct `PropertyRow::new().into_element(...)` cleanup as a separate
    proof-surface migration task rather than as evidence that a new public adapter is required.

Audit status (2026-03-31):

- top-level editor control/composite nouns are closed in `fret-ui-editor::imui`,
- remaining non-adapted exports are subordinate declarative pieces and support types rather than
  missing top-level immediate nouns,
- no additional editor adapter promotions are justified in this lane beyond
  `FieldStatusBadge` and `GradientEditor`.

### C. Proof/demo cleanup

- update `imui_editor_proof_demo` to use promoted immediate editor adapters on the immediate side,
- keep the declarative side-by-side column as declarative proof only,
- remove ad hoc immediate-side direct `.into_element(cx)` calls where the adapter is official.

Current status (2026-03-31):

- the authoring parity immediate column now routes promoted editor nouns through
  `fret_ui_editor::imui`,
- the declarative comparison column remains explicit on purpose,
- and the main `InspectorPanel` subtree still uses declarative editor controls directly because it
  is a declarative proof surface, not an immediate adapter authoring column.

### D. Narrow owner-driven cleanup inside `fret-ui-kit::imui`

- re-audit the current generic helper surface against the code, not stale docs,
- only split the remaining large files when it sharpens ownership and reviewability,
- delete redundant helpers or lower-layer escape hatches only when the canonical surface is
  already proven.

Generic audit status (2026-03-31):

- the generic helper audit confirms that `fret-ui-kit::imui` already ships the vocabulary that
  older notes historically framed as missing, including `selectable`, `combo`, `combo_model`,
  `table`, `virtual_list`, `separator_text`, `collapsing_header`, `tree_node`, tooltip helpers,
  typed drag/drop seams, and floating surface helpers,
- so M3 is primarily a docs/source-of-truth cleanup step before any code deletion decision,
- not a justification to reopen broad generic helper growth,
- and the first concrete redundancy found in this lane is built-in sample wrappers under
  `fret_ui_kit::imui::adapters`, which should not remain in the public module once the seam
  contract itself is stable,
- while the remaining large root file (`ecosystem/fret-ui-kit/src/imui.rs`) is still the
  coordination surface for options, responses, `ImUiFacade`, and `UiWriterImUiFacadeExt`; the
  behavior-heavy code is already split into dedicated submodules, so another root-file split would
  mostly increase navigation churn rather than sharpen ownership,
- and the correct authoring-shape rule is now explicit:
  editor-owned declarative forwarders stay on `&mut impl fret_authoring::UiWriter<H>`,
  generic immediate adapter seams compile against `UiWriterImUiFacadeExt<H>`,
  and neither surface should depend on concrete `fret_imui::ImUi`.

## Dear ImGui comparison stance

Dear ImGui remains an outcome reference for:

- dense editor hand-feel,
- one obvious helper per concept,
- stable immediate authoring vocabulary,
- and popup/window/hover semantics.

This lane does not aim for source-level API parity.
It aims for:

- correct boundaries,
- complete enough editor-grade coverage,
- and a codebase that can delete overlap instead of curating compatibility.

## Regression and proof requirements

Minimum closure for the code-moving phase that follows this doc pass:

- keep `cargo nextest run -p fret-imui --lib` green,
- keep
  `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy`
  green,
- keep
  `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke`
  green,
- add at least one focused gate for each newly promoted editor adapter,
- keep one first-party proof surface runnable and visibly using the intended layer.

## Relationship to earlier `imui` lanes

This lane supersedes the earlier active `imui` execution story for current work.
Earlier directories remain useful as closeout evidence:

- `imui-stack-fearless-refactor-v1/` = baseline reset and ownership cleanup
- `imui-authoring-vocabulary-closure-v1/` = historical generic helper closure notes
- `imui-editor-grade-surface-closure-v1/` = closeout evidence for the previously closed helper set
- `imui-sortable-recipe-v1/` and the ghost lanes = adjacent closeout evidence, not this lane's
  default backlog

## Primary evidence anchors

- `ecosystem/fret-authoring/src/lib.rs`
- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `docs/workstreams/imui-stack-fearless-refactor-v2/BASELINE_AUDIT_2026-03-31.md`

## Definition of done

This workstream is done when:

- docs entrypoints point here,
- stale `imui` lanes stop teaching outdated gap statements,
- the missing editor adapter coverage is closed or explicitly rejected with rationale,
- first-party immediate proof surfaces stop bypassing the official adapter layer,
- and the next code refactor can delete overlap confidently without reopening ownership questions.
