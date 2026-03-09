# Editor Ecosystem Fearless Refactor v1 - Milestones

Tracking doc: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/DESIGN.md`
TODO board: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/TODO.md`

## M0 - Lock the orchestration layer and the boundary ADR

Goal:

- Publish a single directory workstream that coordinates editor controls, workspace shell,
  `imui` convergence, and theme/token ownership.
- Lock the token/skinning boundary before scaling editor/workspace surface area further.

Exit gates:

- `DESIGN.md`, `TODO.md`, and `MILESTONES.md` exist under one workstream directory.
- ADR 0316 is present and linked from the ADR index.
- `docs/workstreams/README.md` lists the new workstream.
- A parity matrix exists for imgui/egui outcome tracking.

Status: Implemented in this worktree (2026-03-09)

Progress:

- Initial preset draft exists:
  - `docs/workstreams/editor-ecosystem-fearless-refactor-v1/IMGUI_LIKE_PRESET.md`

## M1 - Ownership map and extraction rubric

Goal:

- Make crate ownership explicit across:
  - `fret-imui`
  - `fret-ui-editor`
  - `fret-workspace`
  - `fret-docking`
  - `apps/fret-editor`

Exit gates:

- A compact ownership audit exists for current editor-facing surfaces.
- `apps/fret-editor` modules are classified as:
  app-specific, ecosystem candidate, or incubating.
- No new editor/workspace policy work is added to `crates/fret-ui`.

Status: In progress (2026-03-09)

Progress:

- Initial ownership audit landed:
  - `docs/workstreams/editor-ecosystem-fearless-refactor-v1/OWNERSHIP_AUDIT.md`
- The first extraction shortlist is now explicit:
  - property/inspector protocol is the strongest extraction candidate,
  - viewport tool code should converge with `fret-viewport-tooling` / `fret-gizmo` before any new crate move,
  - project/asset services remain app-owned.

## M2 - Authoring convergence for editor widgets

Goal:

- Make `fret-ui-editor` the single implementation source for reusable editor widgets, regardless of
  authoring syntax.

Exit gates:

- `fret-ui-editor::imui` is a real thin adapter layer over declarative widgets.
- The starter set does not maintain parallel declarative and `imui` implementations.
- Response, `id_source`, and `test_id` conventions are documented and exercised in proof demos.

Status: In progress (2026-03-09)

Progress:

- A starter `fret-ui-editor::imui` adapter surface now exists for:
  `TextField`, `Checkbox`, `DragValue`, `Slider`, and `EnumSelect`.
- `imui_editor_proof_demo` now includes a shared-model side-by-side parity section plus shared-state
  readouts for scripted verification.
- The current gates are:
  - `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
  - `tools/diag-scripts/ui-editor/imui/imui-editor-proof-authoring-parity-shared-models.json`
- The promoted native diagnostics run now passes end-to-end under
  `FRET_IMUI_EDITOR_PRESET=imgui_like_dense` against `target/debug/imui_editor_proof_demo.exe`.
  The current enum-select proof path is anchored by per-item `test_id`s, and the `imui` item click
  uses `click_stable` with `stable_frames: 1` because the popup item can become non-hit-testable
  after an extra stabilization frame in this proof surface.

## M3 - Reusable editor starter set plus workspace shell baseline

Goal:

- Close the minimum credible editor ecosystem surface:
  - controls/composites in `fret-ui-editor`,
  - shell chrome in `fret-workspace`,
  - docking interactions in `fret-docking`.

Exit gates:

- `fret-ui-editor` has a documented v1 starter set with clear ownership.
- `fret-workspace` has a documented shell starter set with no docking-policy duplication.
- At least one proof surface demonstrates the combined editor + workspace stack.

Status: Planned

## M4 - Theme/token namespaces and skin adapters

Goal:

- Make editor/workspace theming stable enough that multiple design systems can skin the same crates
  without re-opening crate boundaries.

Exit gates:

- `editor.*` and `workspace.*` ownership is documented and consistent with ADR 0316.
- A shadcn-aligned seeding/alias path is identified.
- Material/custom-app adapter rules are documented as one-way skinning.
- Missing-token behavior for the new namespaces follows ADR 0270.

Status: In progress

Progress:

- Token inventory and the first namespace plan now live in:
  - `docs/workstreams/editor-ecosystem-fearless-refactor-v1/TOKEN_INVENTORY.md`
- Current findings are now explicit:
  - `fret-ui-editor` already owns explicit `editor.*` keys and an opt-in preset patch path,
  - `fret-workspace` has partial `workspace.*` readers and now has adapter-side shell seeding for
    the first shadcn proof path,
  - `fret-docking` already owns `component.docking.*` drag/drop chrome and that seeding path is
    already proven in `fret-ui-shadcn`.
- The current naming drift is now recorded:
  canonical `workspace.tabstrip.*`, with current code still reading legacy
  `workspace.tab_strip.*`.
- The current adapter recommendation is now explicit:
  keep stable seeding in adapter crates such as `fret-ui-shadcn`, keep owner-local proof presets
  optional, and avoid creating a dedicated editor/workspace adapter crate before namespace cleanup.
- `fret-workspace` now has a small internal token resolver surface in
  `ecosystem/fret-workspace/src/theme_tokens.rs`.
- Shell frame/top bar/status bar now read
  `workspace.frame.*`, `workspace.top_bar.*`, and `workspace.status_bar.*` with generic fallback
  preserved, and shell tabstrip chrome now resolves canonical `workspace.tabstrip.*` keys before
  legacy `workspace.tab_strip.*` compatibility spellings.
- `fret-ui-shadcn` now seeds shell-level `workspace.frame.*`, `workspace.top_bar.*`,
  `workspace.status_bar.*`, and `workspace.tabstrip.*` families in its new-york presets.
- `ui_gallery` is now the first end-to-end workspace shell proof surface for this path via
  `tools/diag-scripts/ui-gallery/workspace-shell/ui-gallery-workspace-shell-chrome-shadcn-screenshot.json`.
- The current shell seeding closure is intentionally small:
  it does not seed `workspace.tab.*` yet.
- Focused unit coverage now locks the ADR 0270 fallback story for these workspace namespaces in
  `ecosystem/fret-workspace/src/theme_tokens.rs`.

## M5 - Proofs, gates, and cleanup

Goal:

- Finish the refactor with durable evidence and less duplication than before.

Exit gates:

- Proof demos exist for the highest-risk flows:
  editor widgets, workspace shell composition, token fallback/skinning.
- Focused test/script gates cover the most failure-prone behaviors.
- Duplicated app-local/editor-local implementations are deleted or explicitly quarantined.
- The docs point to one boring recommended path for new editor ecosystem work.

Status: In progress

Progress:

- `imui_editor_proof_demo` remains the promoted proof for authoring convergence.
- `ui_gallery` is now the promoted workspace-shell proof surface for shadcn shell seeding:
  it exercises `WorkspaceFrame`, `WorkspaceTopBar`, `WorkspaceStatusBar`, and
  `WorkspaceTabStrip` together under an explicit preset switch.
- The current proof/gate set now includes:
  - `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
  - `tools/diag-scripts/ui-editor/imui/imui-editor-proof-authoring-parity-shared-models.json`
  - `tools/diag-scripts/ui-gallery/workspace-shell/ui-gallery-workspace-shell-chrome-shadcn-screenshot.json`
