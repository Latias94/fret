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

## Phase B - Foundation closure before component growth

Status: In progress

Goal:

- fix the current editor baseline before scaling the promoted component surface,
- make screenshots and proof surfaces genuinely reviewable,
- and converge layout/state/tokens across the existing starter set.

Deliverables:

- clearer default editor visual hierarchy and token ownership,
- editor-owned baseline replay that survives host theme resets and environment-driven theme sync,
- broader `EditorWidgetVisuals` coverage across reusable editor controls, including shared
  typed-edit and invalid semantics for field-like surfaces,
- shared property-grid / inspector layout metrics for label, value, reset, status, group, and panel
  lanes,
- one editor-owned trailing affordance baseline so reset/clear/remove/icon actions stop drifting
  between narrow hit targets and row-height targets,
- stronger typed-edit, focus, active, and invalid state clarity,
- screenshot proof coverage for the neutral default baseline,
- focused authoring-affordance screenshots that pin clear-button alignment and percent readout
  composition on the full proof surface,
- a review-only proof composition that hides unrelated parity/docking surfaces during baseline
  screenshot capture,
- a decision and follow-up plan for `imgui_like_dense` screenshot parity,
- proof-surface cleanup so overview / typing / error states are visible without manual scene setup,
- numeric typing diagnostics that cover both double-click focus handoff and the real first-edit
  input path (`KeyDown` to arm replacement, then `TextInput` / IME commit to insert text),
- text-like policy defaults that distinguish general text fields from search boxes without
  reintroducing widget-local key hooks everywhere,
- a buffered text-field baseline across single-line and multiline surfaces so editor text entry
  stops mutating external models mid-edit and instead proves draft/commit/cancel semantics
  directly on the proof surface,
- a first editor-grade extension seam on top of that baseline: password-mode rendering, explicit
  commit/cancel outcome hooks, assistive semantics hooks for future completion/history surfaces,
- one minimal promoted completion/history proof on top of that seam, keeping focus on the owning
  input while exposing a controlled listbox relationship plus `active_descendant` state,
- focused diag coverage for buffered blur commit, multiline explicit commit, and Escape cancel on
  the promoted proof surface,
- and a boring close-out path for screenshot automation after typed-mode interactions and reruns.

Exit gates:

- the default editor baseline is visually legible enough to review without "squinting through gray",
- proof/demo startup and host theme sync no longer silently erase the intended editor preset,
- overview / typing / invalid screenshots are all meaningful and reproducible,
- authoring-parity screenshots make clear-button alignment and percent slider composition reviewable
  without manual proof setup,
- the screenshot proof can switch into a review-only composition without manual window/layout setup,
- starter-set controls share one layout/state grammar instead of per-control heuristics,
- repeated screenshot runs reset proof-local filter/search state instead of depending on a fresh
  launch,
- buffered text-session proof coverage demonstrates blur commit, multiline explicit commit, and
  cancel/revert without relying on manual inspection,
- a promoted text-assist/history proof plus diag gate demonstrate input-owned assist semantics
  (`expanded`, controlled listbox relation, `active_descendant`) and Enter-accept behavior without
  moving primary focus into the popup,
- repeated-control identity coverage exists on a promoted loop-built surface rather than only in
  local reasoning or code comments,
- and this workstream can point to clear proof/gate evidence for baseline correctness.

## Phase C - Editor starter kit closure

Status: Planned

Goal:

- close the minimum credible editor starter set in `fret-ui-editor`,
- make declarative and `imui` authoring paths share one implementation source,
- and lock the highest-risk interaction semantics with proof surfaces and gates.

Deliverables:

- `DragValue` closure for real editor workflows,
- richer text-input policy for editor surfaces beyond the shared buffered baseline
  (lifting the promoted proof into reusable `fret-ui-kit` text-assist glue, deciding the formal
  popup/list abstraction, specialized blur ownership where needed, and deeper editor integrations
  above the new password/outcome/assistive extension seam),
- a promoted starter set definition for controls and composites,
- explicit conventions for `id_source`, response semantics, and `test_id`,
- and a "no new promoted components without gates" landing rule.

Exit gates:

- starter-set controls do not keep parallel declarative and `imui` implementations,
- `imui_editor_proof_demo` or an equivalent promoted proof surface covers the core editor set,
- focused gates exist for edit-session commit/cancel, state-identity correctness, and screenshot
  baseline review,
- new editor controls follow the component-system baseline instead of ad-hoc style rules.

## Phase D - Shell, adapters, and extraction closure

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
2. Use Phase B to fix the current editor baseline and proof/gate quality.
3. Use Phase C to resume starter-set closure only after the baseline is coherent.
4. Use Phase D only after the starter set is coherent enough to justify protocol extraction and
   adapter cleanup.
