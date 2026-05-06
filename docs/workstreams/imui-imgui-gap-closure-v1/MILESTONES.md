# ImUi Dear ImGui Gap Closure v1 - Milestones

Status: Active
Last updated: 2026-05-06

## M0 - Current Source Baseline

Exit criteria:

- The lane exists with the minimum doc set.
- The current source audit names current Fret capabilities and Dear ImGui comparison anchors.
- The old standalone parity audit has a status note explaining how to read it.
- Repo trackers point to this lane for current gap-closure sequencing.
- P0 doc/source gates are run.
  Result: done on 2026-05-06.

## M1 - Fearless Cleanup Candidate Selection

Exit criteria:

- One cleanup/delete candidate is chosen from current source evidence.
- The candidate has a named owner crate/doc surface.
- The candidate has a focused gate and a rollback-free delete/refactor plan.
- Any closed historical lane remains closed unless a narrower follow-on is created.
  Current first candidate selected: teaching-surface cleanup for `imui_shadcn_adapter_demo`.
  The first code slice routes `TableSortDirection` through `fret::imui::kit` and extends the
  teaching-source gate so the stale direct `fret_ui_kit::imui::TableSortDirection` import cannot
  return to that default-path example.
  The second code slice routes `workspace_shell_demo` pane-proof option types through
  `fret::imui::kit` and extends both IMUI source gates so direct `fret_ui_kit::imui` imports cannot
  return to that default pane-first proof.
  The third code slice routes `imui_editor_proof_demo` and its `collection.rs` module through the
  app-facing `fret::imui` facade for IMUI option/state types while keeping recipe-layer imports
  explicit. `fret-ui-editor::imui` was audited and remains a thin declarative-control adapter.

## M2 - First Cleanup/Refactor Slice

Exit criteria:

- The first P1 slice lands.
- Public teaching surfaces still prefer the app-facing `fret::imui` path.
- `fret-imui` stays policy-light.
- `fret-ui-editor::imui` stays a thin adapter.
- Focused gates pass.
  Result: done for the P1 import cleanup pass on 2026-05-06. Remaining P1 item is duplicate helper
  alias deletion, if source evidence finds a real alias worth removing.
  P1 closeout result: no additional alias delete is justified. The debug draw owner cleanup was
  split into `imui-debug-draw-owner-split-v1` and that follow-on is now closed.

## M3 - User-Usable Golden Path

Exit criteria:

- A single runnable proof teaches a realistic editor panel path.
- It combines immediate authoring, editor controls, actions/commands, popup/menu behavior, and
  diagnostic hooks.
- Cookbook/docs point at that proof without promoting historical smoke demos as the default path.
  Current proof surface: `apps/fret-examples/src/imui_editor_proof_demo.rs` with the demo-local
  `collection.rs` module. The proof has been gated with focused collection source tests plus
  `cargo check -p fret-demo --bin imui_editor_proof_demo`; cookbook/docs promotion now points from
  the focused IMUI cookbook lessons to the heavier `fret-demo` proof without turning it into a
  boring-ladder cookbook example.

## M4 - Follow-On Split

Exit criteria:

- Remaining Dear ImGui-class gaps are split into narrow lanes with owner, repro, gate, and evidence.
- This lane remains the source-audit and priority map, not a dumping ground for all implementation.
  Current public-surface audit result: keep the existing owner split. `fret-imui` stays
  policy-light, apps teach the opt-in `fret::imui` lane, and policy-heavy widgets remain under
  `fret::imui::kit` / `editor` / `docking`. New public helpers need two proof surfaces plus a
  focused gate before they become default authoring vocabulary.
  Current component-surface audit result: do not open a broad widget-backlog lane. The current
  `fret-ui-kit::imui` surface already covers the editor-proof path across controls, text,
  disclosure, menus/popups/tooltips, tabs, tables, drag/drop, child regions, virtual lists, and
  debug draw. List-box, plotting, image item, style-editor, advanced-table, and child-flag work
  should be narrow proof-led follow-ons.
  Current design-surface audit result: keep imgui-class density as an opt-in editor token/preset
  outcome. `EditorThemePresetV1::ImguiLikeDense` is sufficient for the active proof; do not copy
  Dear ImGui's mutable style stack or make a generic style editor without visual/tooling proof.
  Current porting-sugar audit result: keep `SameLine` / item-width / label-ID sugar candidate-only
  until at least two proof surfaces pay the same authoring tax. Prefer typed Fret helpers
  (`horizontal_with_options`, `PropertyGrid::row_with`, explicit `id_source` / `test_id`) over
  copying Dear ImGui's mutable cursor, item-width stack, or label suffix parser.
  Current child-region audit result: keep `child-region depth` as a candidate-only item until a
  behavior target such as `ResizeY`, auto-resize, clipping-return, or nav-flattening has a concrete
  proof and gate.
  Current collection-helper audit result: keep collection behavior app-owned until a second IMUI
  proof repeats the same request/box-select/selection-repair shape. `fret-node` remains domain
  evidence, not an API-freezing proof surface.
  Current execution-priority review result: treat the P3 catalog notes as readiness maps, not an
  implementation queue. Product/golden workflow coherence, runner/backend multi-window hand-feel,
  and diagnostics/DevTools discoverability remain higher-value Dear ImGui-grade closure work than
  blind widget/API mirroring.
