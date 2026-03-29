# imui editor-grade surface closure v1 - TODO

Tracking doc: `docs/workstreams/imui-editor-grade-surface-closure-v1/DESIGN.md`

Milestones: `docs/workstreams/imui-editor-grade-surface-closure-v1/MILESTONES.md`

Gap audit: `docs/workstreams/imui-editor-grade-surface-closure-v1/EDITOR_GRADE_GAP_AUDIT_2026-03-29.md`

Predecessor closeout:

- `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/imui-stack-fearless-refactor-v1/TODO.md`
- `docs/workstreams/imui-stack-fearless-refactor-v1/MILESTONES.md`

This board assumes a fearless refactor posture.
Compatibility shims are explicitly out of scope.

## M0 - Scope freeze and owner split

- [x] Freeze the follow-on scope to editor-grade closure, not another broad `imui` redesign.
- [x] Classify each candidate gap as one of:
      `fret-ui-kit::imui`, `fret-ui-editor::imui`, or docking/workspace-owned.
- [x] Record which current gaps are P0 versus explicit defer.
- [x] Lock the non-goals:
      no style stack, no last-item implicit context, no compatibility aliases, no second widget implementations.
- [x] Record the proof/demo surfaces that must simplify if this workstream is successful.

## M1 - Close editor composite adapter gaps

- [x] Add a thin `imui` adapter for `PropertyGroup`.
- [x] Add a thin `imui` adapter for `PropertyGrid`.
- [x] Add a thin `imui` adapter for `PropertyGridVirtualized`.
- [x] Add a thin `imui` adapter for `InspectorPanel`.
- [x] Decide whether `GradientEditor` belongs in the promoted immediate composite set or remains a
      declarative-only composite for now.
      Decision: keep `GradientEditor` declarative-only for now; it is a richer editor recipe, not a
      missing generic inspector skeleton.
- [x] Add a source-policy test that locks composite adapters to one-hop `into_element` forwarding.
- [x] Migrate first-party proof/demo call sites that currently wrap these composites manually.

## M2 - Close generic editor-shell helper gaps

- [x] Add a first-class immediate tooltip helper on `fret-ui-kit::imui`.
- [ ] Add a generic collapsing-header / tree-node immediate family on `fret-ui-kit::imui`.
- [ ] Write explicit ID guidance for tree/outliner authoring so ImGui/egui ports do not invent
      ad hoc naming tricks.
- [ ] Decide whether any currently requested shell-like helper is actually docking/workspace-owned
      and should be routed out of this workstream.
- [ ] Keep `fret-imui` itself free of these richer policy helpers.

Current M2 progress:

- `fret-ui-kit::imui` now exposes `tooltip_text(...)`, `tooltip_text_with_options(...)`,
  `tooltip(...)`, and `tooltip_with_options(...)`.
- `TooltipOptions` now records placement, estimated size, window margin, hoverable-content policy,
  delay overrides, and `test_id`.
- The helper stays in `fret-ui-kit::imui` and reuses the existing tooltip/overlay substrate instead
  of adding a second tooltip state machine or widening `fret-imui`.
- Proof/demo coverage now includes tooltip usage on the editor proof controls in
  `apps/fret-examples/src/imui_editor_proof_demo.rs`.
- Focused tooltip smoke coverage now lives in
  `ecosystem/fret-ui-kit/tests/imui_tooltip_smoke.rs`.

## M3 - Evaluate drag/drop immediate closure

- [ ] Audit runtime drag contracts against Dear ImGui / egui payload expectations.
- [ ] Decide whether a portable immediate drag/drop helper family can land cleanly.
- [ ] If yes, add a typed thin authoring surface in the correct owner crate.
- [ ] If no, write a defer note that states exactly what boundary is still missing.
- [ ] Prove the decision on a real editor scenario (outliner reorder, asset drop target, or similar).

## M4 - Tests, proof surfaces, and docs

- [ ] Extend `imui_editor_proof_demo` with at least one composite-heavy inspector scenario.
- [ ] Extend proof/demo coverage with at least one tree/outliner scenario.
- [ ] Add focused tests for any new tooltip/tree/drag-drop surfaces.
- [ ] Update `docs/workstreams/README.md` so the immediate-mode map points to this follow-on lane.
- [ ] Update parity/audit notes if the shipped immediate surface meaningfully changes.
- [ ] Capture a closeout summary that says which gaps were closed, which were intentionally deferred,
      and which owner crate each surviving gap belongs to.
