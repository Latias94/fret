# Editor ecosystem fearless refactor v1 - TODO

Tracking doc: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/DESIGN.md`

Milestones: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/MILESTONES.md`

Component-system baseline:
`docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_COMPONENT_SYSTEM.md`

Interaction contract:
`docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_INTERACTION_CONTRACT.md`

## Status legend

- `[ ]` Not started
- `[~]` In progress
- `[x]` Done
- `[?]` Needs ownership decision

## Phase A - Boundary freeze and document reset

- [x] `EER-DOC-100` Add a directory-level README that distinguishes primary documents from
      supporting notes.
- [x] `EER-DOC-101` Rewrite the design doc around the current ownership baseline and execution
      intent instead of keeping a large orchestration narrative.
- [x] `EER-DOC-102` Reset milestones/TODO so they are forward-looking rather than a mixed archive of
      already-landed bring-up tasks.
- [x] `EER-DOC-103` Add an editor component-system note covering design language, density, state
      model, and preset strategy.
- [ ] `EER-DOC-104` Decide whether `PARITY_MATRIX.md` should stay as a short orchestration snapshot
      or be folded into `docs/workstreams/ui-editor-egui-imgui-gap-v1.md`.
- [x] `EER-DOC-105` Publish a short standalone conventions note for `id_source`, `test_id`,
      response semantics, and loop-built widget state.

## Phase B - Editor starter kit closure

- [~] `EER-IMUI-110` Keep expanding `fret-ui-editor::imui` only as a thin facade over declarative
      controls; do not allow a second implementation tree to form.
- [ ] `EER-EDITOR-111` Finish `EditorWidgetVisuals` convergence for all reusable editor controls.
- [ ] `EER-EDITOR-112` Close `DragValue` for real editor workflows:
      prefix/suffix, clamp policy, step, decimals policy, unit helpers, and consistent commit/cancel.
- [ ] `EER-EDITOR-113` Close editor-grade text input policy:
      password mode, completion/history hook placeholders, selection defaults, and clear Enter/Escape
      behavior.
- [ ] `EER-EDITOR-114` Freeze the v1 reusable starter set:
      `TextField`, `Checkbox`, `DragValue`, `Slider`, `EnumSelect`, `ColorEdit`, `VecNEdit`,
      `TransformEdit`, `PropertyGrid`, and `InspectorPanel`.
- [ ] `EER-EDITOR-115` Ensure every promoted reusable control first lands as a declarative
      implementation and only then gains optional `imui` sugar.
- [ ] `EER-EDITOR-116` Promote one proof surface that exercises the starter set under shared state
      and stable diagnostics anchors.

## Phase C - Shell, adapters, and extraction

- [ ] `EER-SHELL-120` Freeze the reusable `fret-workspace` shell starter set:
      frame, top bar, status bar, pane chrome, shell tabstrip, command scope, focus coordination.
- [ ] `EER-SHELL-121` Keep shell tabstrip and docking tab/drop chrome aligned through adapter
      seeding/aliasing rather than crate coupling.
- [ ] `EER-THEME-122` Audit the editor proof preset so it stops mutating shared component/palette
      keys whenever editor-owned families are sufficient.
- [ ] `EER-THEME-123` Decide whether `workspace.tab.*` also needs adapter-side seeding or should
      remain fallback-first for v1.
- [ ] `EER-EXTRACT-124` Decide whether to introduce a future inspector/property protocol crate or
      explicitly defer extraction until a second consumer exists.
- [ ] `EER-EXTRACT-125` Rebase reusable viewport tool logic onto `fret-viewport-tooling` /
      `fret-gizmo` before any new extraction is attempted.

## Gates and migration evidence

- [~] `EER-GATE-130` Add focused edit-session commit/cancel coverage for numeric editing.
      Numeric-input validation/commit coverage now exists via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-validation.json`;
      Escape/cancel-specific proof is still missing.
- [ ] `EER-GATE-131` Add state-identity regression coverage for loop-built or repeated controls.
- [~] `EER-GATE-132` Keep `imui_editor_proof_demo` and the promoted workspace-shell proof surfaces
      as the primary evidence anchors for this workstream.
- [~] `EER-GATE-133` Decide whether the neutral default editor baseline and `imgui_like_dense`
      should both receive screenshot/diag coverage.
      The neutral default baseline now has a screenshot proof via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-editor-components-screenshots-default.json`;
      a matching `imgui_like_dense` screenshot surface is still pending.
- [ ] `EER-MIGRATE-134` Write a short migration note for promoting app-local editor surfaces into
      ecosystem crates.
- [ ] `EER-CLEANUP-135` Delete or quarantine any duplicated editor widget implementations left after
      convergence.

## Open questions

- [x] `EER-Q-140` Should design-system seeding start inside adapter crates such as `fret-ui-shadcn`
      rather than in editor/workspace owner crates?
      Decision: yes for v1; keep owner-local proof presets optional and avoid reverse dependencies.
- [ ] `EER-Q-141` Which exact `apps/fret-editor` protocols are reusable enough to justify a future
      dedicated protocol crate?
- [ ] `EER-Q-142` Should the default reusable editor baseline stay strictly neutral, or should it
      intentionally bias a bit more toward the current `imgui_like_dense` hand-feel?
