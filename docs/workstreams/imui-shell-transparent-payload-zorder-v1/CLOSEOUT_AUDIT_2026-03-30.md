# Closeout Audit — 2026-03-30

This audit records the final closeout read for the `imui-shell-transparent-payload-zorder-v1`
lane.

Goal:

- verify whether the transparent moving-window overlap contract is now proven end-to-end,
- separate the real runtime/diagnostics root cause from the later script-signature drift,
- and decide whether this lane should remain active or become historical closeout evidence.

## Audited evidence

Core workstream docs:

- `docs/workstreams/imui-shell-transparent-payload-zorder-v1/DESIGN.md`
- `docs/workstreams/imui-shell-transparent-payload-zorder-v1/TODO.md`
- `docs/workstreams/imui-shell-transparent-payload-zorder-v1/MILESTONES.md`
- `docs/workstreams/imui-shell-transparent-payload-zorder-v1/M1_CONTRACT_FREEZE_2026-03-30.md`
- `docs/workstreams/imui-shell-transparent-payload-zorder-v1/M2_LAUNCHED_PROOF_READ_2026-03-30.md`

Implementation / proof anchors:

- `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer_session.rs`
- `crates/fret-runtime/src/effect.rs`
- `crates/fret-runtime/src/injected_event_scope.rs`
- `crates/fret-launch/src/runner/desktop/runner/effects.rs`
- `crates/fret-launch/src/runner/web/effects.rs`
- `apps/fret-examples/src/docking_arbitration_demo.rs`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-large-transparent-payload-zorder-switch.json`
- `tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-multiwindow-large-transparent-payload-single-move-probe.json`

Validation runs used for closeout:

- `cargo check -p fret-runtime -p fret-launch -p fret-bootstrap -p fret-examples -p fret-ui-gallery`
- `cargo build -p fret-demo --bin docking_arbitration_demo`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-multiwindow-large-transparent-payload-single-move-probe.json --dir target/fret-diag/imui-shell-transparent-payload-zorder-v1 --session-auto --timeout-ms 120000 --env FRET_DIAG_SCRIPT_MIGRATION_TRACE=1 --launch target/debug/docking_arbitration_demo.exe`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-large-transparent-payload-zorder-switch.json --dir target/fret-diag/imui-shell-transparent-payload-zorder-v1 --session-auto --pack --ai-packet --timeout-ms 240000 --env FRET_DIAG_SCRIPT_MIGRATION_TRACE=1 --launch target/debug/docking_arbitration_demo.exe`

Supporting failure-localization read retained during closeout:

- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-large-transparent-payload-zorder-switch.json --dir target/fret-diag/imui-shell-transparent-payload-zorder-v1 --session-auto --pack --ai-packet --timeout-ms 240000 --env FRET_DIAG_SCRIPT_MIGRATION_TRACE=1 --launch target/debug/docking_arbitration_demo.exe`
  before the signature update, producing
  `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774874407316-38304`.

Launched diag proof retained for closeout:

- single-move probe pass:
  `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774873805515-28452`
- large transparent-payload z-order gate pass:
  `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774874752791-64356`
- packed share artifact:
  `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774874752791-64356/share/1774874754797.zip`
- AI packet:
  `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774874752791-64356/1774874754797/ai.packet`

## Findings

### 1. The large-layout transparent-payload lane is no longer blocked by diagnostics starvation

This lane originally remained open because the large overlap proof never reached its final
assertions.

The real blocker was not the transparent payload contract itself.
The blocker was diagnostics getting trapped at `step_index = 19` (`pointer_move`) in the late
tear-off overlap phase.

The closeout runs now prove that the lane has crossed that old blocker:

- `pointer_move` starts in the floating/source window,
- execution migrates to the overlapped main window when that is the window still producing frames,
- the drag continues through transparent moving-window overlap,
- `pointer_up` completes,
- `dock_drag_active_is(false)` completes,
- and the script reaches the final graph assertions.

Conclusion:

- the prior late-phase timeout was an instrumentation / input-delivery problem,
- not a remaining failure to activate transparent moving-window overlap.

### 2. The true runtime root cause was a four-part diagnostics bug, and each part is now closed

The failing late-phase choreography depended on one subtle invariant:

> an implicit `pointer_move` step must stay migratable across windows, but still deliver synthetic
> pointer/drag events back into the window that owns the active pointer session.

The old diagnostics path violated that invariant in four ways:

1. `preferred_window_for_active_script(...)` pinned implicit `pointer_move` steps to the pointer
   session window even when another window was the only one producing frames.
2. `handle_pointer_move_step(...)` interpreted `window: null` as `current_window`, which created a
   `1v1 -> 2v1` handoff loop immediately after migration.
3. synthetic cross-window events had no diagnostics-only runner delivery path, so the script had
   no safe way to keep driving the source `UiTree` after migration.
4. `pointer_move steps = 1` still required an extra completion frame before `next_step += 1`,
   which made single-frame probes hang if that empty follow-up frame never arrived.

The landed fix closes all four:

- implicit `pointer_move` steps stay migratable,
- implicit `pointer_move` no longer resolves to `current_window` for handoff purposes,
- diagnostics can emit `Effect::DiagInjectEvent { window, event }`,
- native/web runners deliver those events inside an injected-event scope,
- and the final move segment now completes the step in the same frame.

Conclusion:

- the repo now has the right diagnostics-only bridge for multi-window pointer-session playback,
- and that bridge stays out of generic component/policy layers.

### 3. The launched proof now demonstrates the intended transparent overlap contract end-to-end

The pass session
`target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774874752791-64356`
proves the intended shell-visible state, not just a partial precondition.

The closing run reaches and verifies:

- `dock_drag_active_is(true)` after late floating-window activation,
- `transparent_payload_applied = true`,
- `transparent_payload_hit_test_passthrough_applied = true`,
- `window_under_moving_window = first_seen`,
- `dock_drag_current_window_is(first_seen)` during overlap,
- successful late `raise_window(first_seen)` / `raise_window(last_seen)` choreography,
- `pointer_up`,
- `dock_drag_active_is(false)`,
- and final dock graph assertions.

Conclusion:

- this lane now satisfies the “launched first-party proof, not scene-only inference” bar.

### 4. The final failure was a script contract drift, not a docking-state failure

Once the runtime/diagnostics bug was fixed, the large gate reached `step_index = 36` and failed on
the first dock-graph assertion.

The retained failure session
`target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774874407316-38304`
proved that the actual graph structure was correct, but the asserted panel ids were stale:

- expected in old scripts:
  `demo.viewport.extra.0`
- actual current signature:
  `demo.viewport.extra#0`

That drift matches the current demo contract:

- `extra_viewport_panel_key(ix)` uses
  `PanelKey::with_instance("demo.viewport.extra", ix.to_string())`

So the correct fix was to update script expectations to the current canonical signature format,
not to mutate docking state or runner behavior.

Conclusion:

- the remaining “failure” after the runtime fix was a stale gate,
- and the lane needed a script corpus alignment, not another shell contract change.

### 5. The lane is now closed; remaining work is maintenance, not an unresolved transparent-payload contract

After this closeout, the remaining work items are maintenance-shaped:

- keep future docking-arbitration scripts aligned with `PanelKey::with_instance(...)`
  canonicalization,
- reuse `Effect::DiagInjectEvent` only for diagnostics/script plumbing,
- and continue treating transparent moving-window posture as runner truth plus docking-owned
  preview policy.

Those are not new open questions for this lane.
They are normal follow-through on a closed contract.

Conclusion:

- `imui-shell-transparent-payload-zorder-v1` should now be read as closed closeout evidence,
- not as an active unresolved shell overlap lane.

## Decision from this audit

Treat `imui-shell-transparent-payload-zorder-v1` as:

- closed for the transparent moving-window overlap / z-order proof slice,
- historical closeout evidence by default,
- complete with a diagnostics-only multi-window pointer-session bridge,
- and complete with script corpus alignment to current panel-instance signatures.

## Immediate execution consequence

From this point forward:

1. keep transparent moving-window truth in runner/runtime diagnostics plus docking-owned preview
   policy,
2. keep diagnostics-only cross-window synthetic delivery out of generic `imui` and recipe policy
   layers,
3. treat implicit pointer-session `pointer_move` as migratable execution plus session-owned
   delivery,
4. keep docking-arbitration graph signatures aligned with `PanelKey::with_instance(...)`
   canonical `kind#instance` formatting,
5. open a new successor lane only if a new shell-visible behavior question appears beyond this
   now-proven overlap contract.
