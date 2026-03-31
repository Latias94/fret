# imui authoring vocabulary closure v1 - milestones

This workstream is intentionally staged from smallest high-value authoring noun to broader dense
list/data helpers.

## M0 - Contract freeze and gap map

Acceptance:

- `DESIGN.md`, `TODO.md`, `MILESTONES.md`, and `GAP_AUDIT_2026-03-31.md` exist.
- The workstream explicitly records:
  - what is already closed,
  - what remains P0/P1/P2,
  - which layer owns each remaining gap,
  - and which items are explicitly out of scope for generic `imui`.

Deliverables:

- `docs/workstreams/imui-authoring-vocabulary-closure-v1/DESIGN.md`
- `docs/workstreams/imui-authoring-vocabulary-closure-v1/TODO.md`
- `docs/workstreams/imui-authoring-vocabulary-closure-v1/MILESTONES.md`
- `docs/workstreams/imui-authoring-vocabulary-closure-v1/GAP_AUDIT_2026-03-31.md`

## M1 - `selectable` lands as the canonical selection-row helper

Acceptance:

- `fret-ui-kit::imui` exposes one canonical generic row-selection helper.
- The helper works in plain lists and popup bodies.
- Stable ids and activation/selection response semantics are gated.
- The helper does not smuggle menu-, shell-, or docking-specific policy into generic `imui`.

Suggested deliverables:

- implementation in `ecosystem/fret-ui-kit/src/imui/*`
- focused smoke/unit tests
- one proof/demo usage

## M2 - generic `begin_combo` / `combo` lands

Acceptance:

- the immediate combo surface is canonical and teachable,
- `selectable(...)` can be used naturally inside it,
- overlap with the old `select_model` naming is removed instead of multiplied,
- and focus/dismiss outcomes are gated.

Suggested deliverables:

- implementation in `ecosystem/fret-ui-kit/src/imui/*`
- tests for preview/open/close/selection
- proof/demo replacing ad hoc popup-radio plumbing

## M3 - immediate table/columns wrapper lands

Acceptance:

- the repo has a table-oriented immediate vocabulary distinct from plain layout grid helpers,
- header/body authoring is ergonomic enough for dense data panes,
- and one alignment/scroll proof is gated.

Suggested deliverables:

- implementation in `ecosystem/fret-ui-kit/src/imui/*`
- focused tests and proof/demo

## M4 - generic list clipper / virtualized rows land

Acceptance:

- a reusable visible-range helper exists for large immediate lists,
- it composes with `selectable` rows and table bodies,
- and a deterministic visible-range gate exists.

Suggested deliverables:

- implementation in `ecosystem/fret-ui-kit/src/imui/*`
- visible-range tests
- large-list proof/demo

## M5 - cleanup and small hand-feel helpers

Acceptance:

- remaining small helpers such as `separator_text` are closed or explicitly deferred,
- duplicate older helpers were deleted or collapsed,
- and the lane leaves one canonical immediate vocabulary story instead of multiple overlapping
  helper families.
