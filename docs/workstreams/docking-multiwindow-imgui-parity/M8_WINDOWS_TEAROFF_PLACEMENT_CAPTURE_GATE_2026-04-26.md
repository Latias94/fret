# M8 Windows Tear-Off Placement Capture Gate - 2026-04-26

Status: landed diagnostic evidence surface; accepted by M9 real-host fix evidence

Related:

- `WORKSTREAM.json`
- `docking-multiwindow-imgui-parity-todo.md`
- `M2_WINDOWS_MIXED_DPI_CAPTURE_PLAN_2026-04-13.md`
- `M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`
- `tools/diag-campaigns/imui-p3-windows-placement-real-host.json`
- `tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-windows-tearoff-placement-capture.debug.json`
- `crates/fret-launch/src/runner/desktop/runner/window.rs`
- `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`
- `crates/fret-diag/src/commands/dock_routing.rs`
- `M9_WINDOWS_TEAROFF_CURSOR_CONTINUITY_FIX_2026-04-26.md`

## Purpose

`DW-P1-win-002` is about one concrete hand-feel invariant:

> when a dock tab tears out into a native Windows OS window, the first placement should keep the
> cursor over the grabbed tab and should not be displaced by the titlebar/non-client offset.

The existing math tests already cover `outer_pos_for_cursor_grab`,
`estimated_outer_pos_for_cursor_grab`, decoration-offset scaling, and client-origin conversion.
What was missing was a bounded real-host readout that can distinguish:

1. creation-time placement estimate drift,
2. follow-mode correction drift,
3. Win32 client-origin/decorations evidence,
4. and generic hover/drop routing state.

## Assumptions-First Resume

- Confident: this lane remains active and runner/backend-owned.
  Evidence: `WORKSTREAM.json` status is `active`, and the baseline note says not to widen
  `crates/fret-ui` for this P3 class. If wrong, this capture would belong in a narrower follow-on.
- Confident: the placement algorithm should not be changed before a richer proof surface exists.
  Evidence: `window.rs` already has pure tests for grab offset, decoration offset, target scale, and
  client-origin round trips. If wrong, the next slice should be a targeted `fret-launch` fix.
- Likely: `dock-routing` was the right bounded evidence owner, but its moving-window geometry was
  incomplete.
  Evidence: previous `dock-routing` output exposed `current_window` geometry and only
  `moving_window` id/scale. If wrong, a separate placement sidecar may be justified.
- Likely: Windows real-host admission should be platform-capabilities based, not monitor-topology
  based.
  Evidence: the placement gate can prove decorations/client-origin on a single-monitor Windows host;
  mixed-DPI crossing still belongs to the M2/M7 monitor-topology runbook.

## Shipped Evidence Surface

The diagnostics path now records moving/follow-window geometry alongside the existing routing target
geometry:

- `moving_window_outer_pos_physical_px`
- `moving_window_decoration_offset_physical_px`
- `moving_window_client_origin_screen_physical_px`
- `moving_window_client_origin_source_platform`
- `moving_window_scale_factor_x1000_from_runner`
- `moving_window_local_pos_from_screen_logical_px`
- `moving_window_cursor_grab_delta_logical_px`
- `moving_window_cursor_grab_error_abs_max_logical_px`

`fretboard-dev diag dock-routing` prints these as:

- `move_outer`
- `move_deco`
- `move_origin`
- `move_origin_src=platform`
- `move_local`
- `move_grab_delta`
- `move_grab_error`
- `sf_move_run`
- `sf_move`

The important field for `DW-P1-win-002` is `move_grab_delta`: it is the logical-pixel delta between
the cursor's moving-window local position and the recorded `cursor_grab_offset`. A healthy settled
Windows run should keep that delta near zero. `moving_window_cursor_grab_error_abs_max_logical_px`
is the machine-gate-friendly absolute max component of the same delta; a large titlebar/DPI error
will show up directly.

## Canonical Commands

Validate the campaign manifest:

```bash
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-windows-placement-real-host.json --json
```

Run the Windows placement capture on a real Windows host:

```bash
cargo run -p fretboard-dev -- diag campaign run imui-p3-windows-placement-real-host \
  --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release
```

Direct script form for local debugging:

```bash
cargo run -p fretboard-dev -- diag run \
  tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-windows-tearoff-placement-capture.debug.json \
  --dir target/fret-diag/docking-multiwindow-imgui-parity/windows-placement-real-host \
  --session-auto \
  --timeout-ms 180000 \
  --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release
```

Inspect the after-tearoff bundles:

```bash
cargo run -p fretboard-dev -- diag dock-routing <bundle_dir>
```

The expected labels are:

- `windows-tearoff-placement-after-tearoff-initial`
- `windows-tearoff-placement-after-tearoff-settled`
- `docking-arbitration-demo-windows-tearoff-placement-capture`

## Acceptance Rule

This gate is accepted for `DW-P1-win-002` when a Windows run shows:

1. the script completes and returns to one canonical dock graph (`floatings=[]`),
2. both after-tearoff bundles contain `moving_window` geometry,
3. `move_origin_src=platform` appears on Windows when HWND client-origin evidence is available,
4. the settled bundle keeps `move_grab_delta` near zero,
5. and mixed-DPI hosts do not regress the M2/M7 `sf_cur` / `sf_move` evidence.

If `move_grab_delta` is large, treat the next slice as a `fret-launch` placement fix, not a docking
policy change.

## Decision

M8 established the repeatable proof surface. M9 used that surface to identify and fix the remaining
cursor-continuity drift in the diagnostics/runner transport and records the accepted real-host
bundle with `move_grab_error=0.0`. Treat `DW-P1-win-002` as closed by M9 while keeping this note as
the reusable Windows placement capture runbook.
