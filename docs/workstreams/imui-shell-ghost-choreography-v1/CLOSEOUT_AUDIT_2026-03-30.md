# Closeout Audit — 2026-03-30

This audit records the final closeout read for the `imui-shell-ghost-choreography-v1` lane.

Goal:

- verify what the first shell-aware ghost choreography lane actually closed,
- separate the landed docking-owned payload ghost rule from the explicitly deferred wider shell
  preview questions,
- and decide whether this lane should remain active or become historical closeout evidence.

## Audited evidence

Core workstream docs:

- `docs/workstreams/imui-shell-ghost-choreography-v1/DESIGN.md`
- `docs/workstreams/imui-shell-ghost-choreography-v1/TODO.md`
- `docs/workstreams/imui-shell-ghost-choreography-v1/MILESTONES.md`
- `docs/workstreams/imui-shell-ghost-choreography-v1/M1_CONTRACT_FREEZE_2026-03-30.md`

Implementation / proof anchors:

- `ecosystem/fret-docking/src/dock/paint.rs`
- `ecosystem/fret-docking/src/dock/space.rs`
- `ecosystem/fret-docking/src/dock/tests/drop_hints.rs`
- `crates/fret-runtime/src/interaction_diagnostics.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/docking_diagnostics.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`
- `crates/fret-diag-protocol/src/lib.rs`
- `crates/fret-diag/src/bundle_index.rs`
- `apps/fret-examples/src/docking_arbitration_demo.rs`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-reorder-two-tabs.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-under-moving-window-basic.json`

Validation run used for closeout:

- `cargo test -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_dock_drag_payload_ghost_visible_predicate -- --exact`
- `cargo test -p fret-diag bundle_index::tests::dock_routing_dedups_repeated_frames_and_records_key_fields -- --exact`
- `cargo test -p fret-docking dock::tests::drop_hints::dock_drag_payload_ghost_renders_for_tabs_drag_without_moving_window -- --exact`
- `cargo test -p fret-docking dock::tests::drop_hints::dock_drag_payload_ghost_is_suppressed_when_runner_reports_moving_window -- --exact`
- `cargo test -p fret-docking dock::tests::drop_hints::dock_drag_payload_ghost_only_renders_in_current_window -- --exact`
- `cargo check -p fret-docking -p fret-bootstrap -p fret-runtime -p fret-diag -p fret-diag-protocol`
- `python tools/check_layering.py`

Launched diag proof used for closeout:

- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-reorder-two-tabs.json --dir target/fret-diag/imui-shell-ghost-choreography-v1 --session-auto --pack --ai-packet --timeout-ms 300000 --launch target/debug/docking_arbitration_demo.exe`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-under-moving-window-basic.json --dir target/fret-diag/imui-shell-ghost-choreography-v1 --session-auto --pack --ai-packet --timeout-ms 300000 --launch target/debug/docking_arbitration_demo.exe`
- bounded evidence queries:
  - `cargo run -p fretboard-dev -- diag resolve latest --dir target/fret-diag/imui-shell-ghost-choreography-v1/sessions/1774861155182-40004`
  - `cargo run -p fretboard-dev -- diag resolve latest --dir target/fret-diag/imui-shell-ghost-choreography-v1/sessions/1774861194369-24632`
  - `cargo run -p fretboard-dev -- diag dock-routing target/fret-diag/imui-shell-ghost-choreography-v1/sessions/1774861155182-40004/1774861172700-docking-arbitration-demo-tab-reorder-two-tabs/bundle.schema2.json --json`
  - `cargo run -p fretboard-dev -- diag dock-routing target/fret-diag/imui-shell-ghost-choreography-v1/sessions/1774861194369-24632/1774861201903-diag-run/bundle.schema2.json --json`

Audit artifacts retained under:

- `target/fret-diag/imui-shell-ghost-choreography-v1/sessions/1774861155182-40004/1774861169629/ai.packet`
- `target/fret-diag/imui-shell-ghost-choreography-v1/sessions/1774861155182-40004/share/1774861169629.zip`
- `target/fret-diag/imui-shell-ghost-choreography-v1/sessions/1774861194369-24632/1774861197102/ai.packet`
- `target/fret-diag/imui-shell-ghost-choreography-v1/sessions/1774861194369-24632/share/1774861197102.zip`

## Findings

### 1. The first shell-aware owner split is now closed on the intended layer

This lane opened to answer one narrow question:

> when generic cross-window `current_window` ownership is not sufficient for editor-grade
> docking/tear-out UX, where should shell-aware choreography live?

The current repo now answers that question with a shipped yes.

The first landed shell-aware rule lives in the intended owner:

- `ecosystem/fret-docking`

What did not happen matters more:

- no shell choreography moved into `fret-ui-kit::imui`,
- no generic recipe widening was introduced for docking-specific ownership,
- and runtime crates still publish shell inputs rather than shell policy.

Conclusion:

- the first shell-aware choreography owner question is closed,
- and the correct owner remains docking-aware ecosystem code, not generic `imui` or generic
  recipe defaults.

### 2. The landed rule is explicit, narrow, and reviewable

The shipped first-slice rule is now concrete:

- paint the docking payload ghost only when `drag.current_window == self.window`,
- suppress that in-window ghost once `drag.moving_window.is_some()`,
- and let runner-owned hover/drop truth stay keyed to the existing drag session diagnostics.

This matters because the repo can now answer the previously fuzzy choreography question without
inventing a second owner:

- same-window shell feedback remains local and visible,
- moving-window shell feedback does not duplicate the in-window payload ghost,
- and the transition point is driven by real runner truth rather than ad hoc demo state.

Conclusion:

- the first shell-aware choreography rule is now explicit and auditable.

### 3. The diagnostics contract now proves the same rule the renderer follows

This lane would not be closed if the renderer and diagnostics could drift.

That drift is now prevented because:

- `DockDragDiagnostics` publishes `payload_ghost_visible`,
- `dock_drag_payload_ghost_visible_is` is a first-class scripted predicate,
- bundle indexing preserves the field for bounded triage,
- and the docking renderer and diagnostics both consume the same local visibility rule.

Conclusion:

- the repo no longer needs to infer ghost visibility indirectly from pixels or ad hoc logs,
- and shell choreography evidence is now part of the durable diagnostics contract.

### 4. The lane now has real launched proof for both halves of the contract

The most important closeout evidence is no longer scene-only coverage.

The launched docking arbitration demo now proves both sides of the rule:

1. same-window drag:
   `docking-arbitration-demo-tab-reorder-two-tabs`
   produced a dock-routing bundle entry with:
   - `moving_window: null`
   - `payload_ghost_visible: true`
2. moving-window shell handoff:
   `docking-arbitration-demo-multiwindow-under-moving-window-basic`
   produced dock-routing bundle entries with:
   - `moving_window: 4294967298`
   - `window_under_moving_window: 4294967297`
   - `payload_ghost_visible: false`

That is the proof this lane needed:

- visible while the shell still owns local payload feedback,
- suppressed once the runner reports a real moving-window shell transition,
- and observed through the first-party docking arbitration demo rather than a synthetic harness.

Conclusion:

- this lane cleared the “real proof surface + gate + evidence” bar.

### 5. The remaining backlog is a successor-lane backlog, not unfinished v1 shell choreography

What remains after this lane is real, but it is not “finish the same visibility rule.”

The surviving deferred items are:

- transparent-payload / moving-window z-order choreography beyond payload ghost suppression,
- aggregate previews,
- native or external preview surfaces,
- non-docking shell families that still lack proof,
- descriptor widening only if a future shell proof forces it.

Those are new owner/proof questions.
They are not missing pieces of the first shell-aware payload ghost contract.

Conclusion:

- this folder should now be read as closeout evidence for the first docking-owned shell ghost
  choreography slice,
- and future shell preview questions should land in successor lanes instead of keeping this one
  nominally active.

## Decision from this audit

Treat `imui-shell-ghost-choreography-v1` as:

- closed for the first docking-owned shell ghost choreography slice,
- historical closeout evidence by default,
- complete with explicit defers for wider shell preview questions,
- and succeeded by `docs/workstreams/imui-shell-transparent-payload-zorder-v1/` for the next
  transparent moving-window overlap lane.

## Immediate execution consequence

From this point forward:

1. keep shell payload ghost ownership in `ecosystem/fret-docking`,
2. keep the shipped visibility rule keyed to `current_window` plus `moving_window` suppression,
3. keep `payload_ghost_visible` in the diagnostics contract and bundle index,
4. use the docking arbitration demo scripts as the first proof/gate surface for regressions,
5. move transparent-payload z-order, aggregate preview, external preview, and non-docking shell
   questions into successor lanes rather than widening this one.
