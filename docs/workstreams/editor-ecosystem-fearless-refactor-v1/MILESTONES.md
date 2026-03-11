# Editor ecosystem fearless refactor v1 - milestones

Tracking doc: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/DESIGN.md`

Component-system baseline:
`docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_COMPONENT_SYSTEM.md`

Interaction contract:
`docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_INTERACTION_CONTRACT.md`

TODO board: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/TODO.md`

This file is forward-looking only.
Earlier bring-up steps remain in git history and supporting notes; the milestones below describe the
recommended next execution order.

## Phase A - Boundary freeze and component-system baseline

Status: In progress

Goal:

- make the ownership story boring again,
- reduce document overlap,
- and publish one explicit editor component-system baseline.

Deliverables:

- a directory-level README that explains which documents are primary vs supporting,
- a reset design document that captures the current ownership baseline,
- a forward-looking milestone/TODO structure,
- a dedicated editor component-system note covering component design, default style direction,
  state model, and preset strategy,
- a dedicated interaction contract note covering identity, response semantics, edit sessions, and
  diagnostics conventions.

Exit gates:

- `README.md`, `DESIGN.md`, `MILESTONES.md`, and `TODO.md` point to one coherent plan.
- `EDITOR_COMPONENT_SYSTEM.md` exists and is referenced as the design/style baseline.
- `EDITOR_INTERACTION_CONTRACT.md` exists and is referenced as the interaction/identity baseline.
- `ui-editor-v1.md` remains the detailed widget-surface note instead of being duplicated here.
- Supporting notes (`OWNERSHIP_AUDIT.md`, `TOKEN_INVENTORY.md`, `IMGUI_LIKE_PRESET.md`,
  `PARITY_MATRIX.md`) are treated as references rather than competing trackers.

## Phase B - Editor starter kit closure

Status: Planned

Goal:

- close the minimum credible editor starter set in `fret-ui-editor`,
- make declarative and `imui` authoring paths share one implementation source,
- and lock the highest-risk interaction semantics with proof surfaces and gates.

Deliverables:

- broader `EditorWidgetVisuals` coverage across reusable editor controls,
- `DragValue` closure for real editor workflows,
- richer text-input policy for editor surfaces,
- a promoted starter set definition for controls and composites,
- explicit conventions for `id_source`, response semantics, and `test_id`.

Exit gates:

- starter-set controls do not keep parallel declarative and `imui` implementations,
- `imui_editor_proof_demo` or an equivalent promoted proof surface covers the core editor set,
- at least one focused gate exists for edit-session commit/cancel and state-identity correctness,
- new editor controls follow the component-system baseline instead of ad-hoc style rules.

## Phase C - Shell, adapters, and extraction closure

Status: Planned

Goal:

- close the shell-level baseline in `fret-workspace`,
- align shell and docking visually without ownership collapse,
- and decide the next extraction move for app-local editor protocols.

Deliverables:

- a documented workspace-shell starter set,
- explicit adapter rules for shell/docking/editor preset alignment,
- a decision on future inspector/property protocol extraction,
- a cleanup/migration note for promoting app-local surfaces into ecosystem crates.

Exit gates:

- `fret-workspace` and `fret-docking` no longer have ambiguous tabstrip/chrome ownership,
- shell proof surfaces remain promoted and gated,
- adapter-side seeding remains the default recommendation for skins,
- either a future protocol crate is scheduled or extraction is explicitly deferred with reasons.

## Recommended execution order

1. Finish Phase A document closure and keep it stable for a while.
2. Use Phase B to close the editor starter set and interaction contracts.
3. Use Phase C only after the starter set is coherent enough to justify protocol extraction and
   adapter cleanup.
