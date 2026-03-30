# M2 Launched Proof Read — 2026-03-30

This note records the first launched proof read for
`docs/workstreams/imui-shell-transparent-payload-zorder-v1/`.

## Scope

Goal:

- run the first transparent-payload overlap scripts against the real docking arbitration demo,
- determine whether current diagnostics are sufficient to explain the observed behavior,
- and decide whether the next step is a code delta or a diagnostics/gate reshape.

## Commands used

Baseline transparent-payload overlap:

- `cargo run -p fretboard -- diag run tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json --dir target/fret-diag/imui-shell-transparent-payload-zorder-v1 --session-auto --pack --ai-packet --timeout-ms 300000 --launch target/debug/docking_arbitration_demo.exe`

Large preset transparent-payload overlap:

- `cargo run -p fretboard -- diag run tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-large-transparent-payload-zorder-switch.json --dir target/fret-diag/imui-shell-transparent-payload-zorder-v1 --session-auto --pack --ai-packet --timeout-ms 300000 --launch target/debug/docking_arbitration_demo.exe`

Failure-localization rerun:

- `cargo run -p fretboard -- diag run tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-large-transparent-payload-zorder-switch.json --dir target/fret-diag/imui-shell-transparent-payload-zorder-v1 --session-auto --pack --ai-packet --timeout-ms 360000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=1 --launch target/debug/docking_arbitration_demo.exe`
- `cargo run -p fretboard -- diag run tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-multiwindow-large-transparent-payload-zorder-switch.debug-late-phase.json --dir target/fret-diag/imui-shell-transparent-payload-zorder-v1 --session-auto --pack --ai-packet --timeout-ms 240000 --launch target/debug/docking_arbitration_demo.exe`

Bounded evidence reads:

- `cargo run -p fretboard -- diag resolve latest --dir target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774861703998-19664`
- `cargo run -p fretboard -- diag dock-routing target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774861703998-19664/1774861713034-docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch/bundle.schema2.json --json`
- `cargo run -p fretboard -- diag resolve latest --dir target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774861703143-49836`
- `cargo run -p fretboard -- diag dock-routing target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774862148351-10948/1774862154236-script-step-0019-pointer_move/bundle.schema2.json --json`
- `cargo run -p fretboard -- diag dock-routing target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774862695071-65560/1774862700901-debug.after-under-moving-window-switch/bundle.schema2.json --json`

## Result summary

### 1. Baseline transparent-payload overlap script passed

Session:

- `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774861703998-19664`

Artifacts:

- `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774861703998-19664/1774861710190/ai.packet`
- `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774861703998-19664/share/1774861710190.zip`
- bundle:
  `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774861703998-19664/1774861713034-docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch/bundle.schema2.json`

Bounded dock-routing evidence shows:

- `moving_window = 4294967298`
- `window_under_moving_window = 4294967297`
- `window_under_moving_window_source = "platform_win32"`
- `current_window = 4294967297`
- `transparent_payload_applied = true`
- `transparent_payload_hit_test_passthrough_applied = true`
- `payload_ghost_visible = false`

Interpretation:

- the base overlap contract already works end-to-end,
- transparent moving-window routing can select the window under the moving window,
- and the in-window payload ghost stays suppressed while that shell transition is active.

### 2. Large preset overlap script timed out instead of finishing

Initial session:

- `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774861703143-49836`

Observed result:

- `script.result.json` remained at `stage = "running"`
- `step_index = 29`
- no final bundle artifact was written

That timeout alone was insufficient to localize the problem, so a second run enabled
`FRET_DIAG_SCRIPT_AUTO_DUMP=1`.

### 3. Auto-dump proves the large case reaches transparent peek-behind before hanging

Auto-dump session:

- `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774862148351-10948`

Last retained bundle:

- `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774862148351-10948/1774862154236-script-step-0019-pointer_move/bundle.schema2.json`

Bounded dock-routing evidence from that auto-dumped bundle shows:

- one snapshot with:
  - `current_window = 4294967297`
  - `moving_window = 4294967298`
  - `window_under_moving_window = 4294967297`
  - `transparent_payload_applied = true`
  - `transparent_payload_hit_test_passthrough_applied = true`
  - `payload_ghost_visible = false`
- neighboring snapshots still show transient source-window ownership / payload visibility during
  the handoff window

Interpretation:

- the large preset does **not** fail because transparent peek-behind never activates,
- it reaches the intended moving-window + under-window routing state,
- and the remaining problem is later in the choreography, likely around z-order stabilization after
  the script raises windows again.

### 4. User-observed flicker is consistent with the remaining failure mode

During the failing large run, the observed behavior was:

- the child window kept flashing,
- and docking did not appear to settle cleanly.

That observation is consistent with the bounded diagnostics read:

- the script got past initial transparent payload enablement,
- but did not converge to a stable end state in the later z-order switch section of the script.

### 5. A late-phase probe localizes the stall to `raise_window(first_seen)`

The debug probe script inserted bundle captures around the late overlap choreography:

- `debug.after-under-moving-window-switch`
- `debug.after-raise-main`
- `debug.after-reraise-floating`

Probe session:

- `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774862695071-65560`

Observed result:

- the probe timed out with `last_seen_step_index = 30`,
- only `debug.after-under-moving-window-switch` was written,
- `debug.after-raise-main` was never produced.

Interpretation:

- the large preset still reaches the healthy state immediately before the explicit
  `raise_window(first_seen)` step,
- the stall begins at or immediately after that raise-main transition,
- and the remaining issue is more specific than “transparent payload routing fails”.

The healthy pre-raise bundle proves the state just before the stall:

- `current_window = 4294967297`
- `moving_window = 4294967298`
- `window_under_moving_window = 4294967297`
- `transparent_payload_applied = true`
- `transparent_payload_hit_test_passthrough_applied = true`
- `payload_ghost_visible = false`

## Verdict on current diagnostics

Current diagnostics are sufficient to answer the first-order question:

- yes, transparent payload overlap can be activated,
- yes, under-moving-window routing can target the overlapped main window,
- and yes, payload ghost suppression still holds during that state.

Current diagnostics are **not yet sufficient** to explain the final large-preset timeout with full
confidence, because:

- the timeout happened after the last auto-dumped bundle,
- there is still no post-raise-main bundle proving which late wait condition failed repeatedly,
- and the current artifact set does not provide a stable per-step timeline across the later
  raise-window / wait-until sequence.

## Decision from this proof read

Treat Phase C as:

- partially successful proof,
- with one passing base overlap gate,
- one reproducible large-preset instability,
- and enough evidence to avoid guessing where the failure begins.

The next action should target:

1. z-order / focus stabilization during `raise_window(first_seen)` in the large overlap path,
2. or improved diagnostics around that exact raise-main transition if a code delta cannot be
   justified immediately.
