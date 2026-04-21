# M5 Wayland Compositor Acceptance Runbook - 2026-04-21

Status: active capture runbook

Related:

- `WORKSTREAM.json`
- `M4_WAYLAND_DEGRADATION_POLICY_2026-04-21.md`
- `docking-multiwindow-imgui-parity-todo.md`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-wayland-degrade-no-os-tearoff.json`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- `docs/adr/0083-multi-window-degradation-policy.md`
- `crates/fret-launch/src/runner/desktop/runner/effects.rs`
- `ecosystem/fret-docking/src/runtime.rs`

## Purpose

`DW-P1-linux-003` now has a frozen source policy (`M4`), but the lane still needs one practical
answer:

> on a real Linux Wayland compositor host, what exact commands and bounded evidence should a
> maintainer use to verify that tear-off degrades to in-window floating instead of creating a
> second OS window?

This note freezes that runbook.

## Target host

Run this only on a Linux native Wayland session.

Minimum host assumptions:

- `XDG_SESSION_TYPE=wayland`
- `WAYLAND_DISPLAY` is non-empty
- the app is running on the native Wayland backend rather than a forced X11/XWayland fallback
- the host can launch `docking_arbitration_demo` with diagnostics enabled through `fretboard-dev`

If the script never reaches `platform_ui_window_hover_detection_is(quality=none)`, do not mark the
run accepted. Record whether:

- the session was not actually Wayland,
- the app was forced onto X11/XWayland,
- or the runner reported the wrong capability posture.

## Canonical command set

### 1) Run the bounded Wayland degradation script on a real host

```bash
FRET_DOCK_TEAROFF_LOG=1 cargo run -p fretboard-dev -- diag run \
  tools/diag-scripts/docking/arbitration/docking-arbitration-demo-wayland-degrade-no-os-tearoff.json \
  --dir target/fret-diag/docking-multiwindow-imgui-parity/wayland-real-host \
  --session-auto \
  --timeout-ms 180000 \
  --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release
```

Why this script:

- it waits until `ui.window_hover_detection` is actually `none`,
- it attempts the same tear-off gesture a maintainer would perform manually,
- it asserts `known_window_count_is(n=1)` after the gesture,
- and it captures one bounded bundle for later review.

### 2) Resolve the captured bundle from the latest session

```bash
cargo run -p fretboard-dev -- diag resolve latest \
  --dir target/fret-diag/docking-multiwindow-imgui-parity/wayland-real-host
```

The expected bundle label is:

- `docking-arbitration-demo-wayland-degrade-no-os-tearoff`

### 3) Inspect bounded evidence from that bundle

Window inventory:

```bash
cargo run -p fretboard-dev -- diag windows <bundle_dir> --json
```

Dock graph summary:

```bash
cargo run -p fretboard-dev -- diag dock-graph <bundle_dir> --json
```

What to confirm:

- `diag windows` still reports one known OS window,
- `diag dock-graph` shows the panel remained owned by the main window as an in-window floating
  container rather than a new `DockFloating` OS window,
- and the script result is `stage=passed`.

### 4) Optionally inspect the tear-off log for forbidden create effects

When `FRET_DOCK_TEAROFF_LOG=1` is enabled, the runner writes:

- `target/fret-dock-tearoff.log`

Check that the attempted Wayland tear-off did not emit a DockFloating create effect:

```bash
rg -n "\\[effect-window-create\\].*DockFloating" target/fret-dock-tearoff.log
```

Expected result:

- no matches for the acceptance run

## Acceptance checklist

The real-host Wayland acceptance run is good enough for this lane when all of the following hold:

1. The script reaches `platform_ui_window_hover_detection_is(quality=none)` on the real host.
2. The script completes with `known_window_count_is(n=1)` and `script.result.json` reports
   `stage=passed`.
3. `diag windows <bundle_dir> --json` still reports one OS window.
4. `diag dock-graph <bundle_dir> --json` shows the floated panel inside the main window rather than
   a second `DockFloating` OS window.
5. `target/fret-dock-tearoff.log` does not contain `[effect-window-create]` lines for
   `DockFloating` during the attempted tear-off.

## Recording rule

When a run satisfies the checklist above, record it in a new dated evidence note under this lane.

Minimum contents for that note:

- host summary:
  - distro / compositor,
  - whether the session was native Wayland,
  - GPU / backend notes only if relevant,
- canonical command used,
- session directory,
- resolved bundle directory,
- `diag windows --json` summary line,
- `diag dock-graph --json` summary line,
- whether the tear-off log grep returned zero matches,
- and whether any compositor-specific follow-up still looks necessary.

## Failure recording rule

If the run fails, record at least:

- whether the script stalled before `platform_ui_window_hover_detection_is(quality=none)`,
- whether `known_window_count_is(n=1)` failed,
- whether `diag dock-graph` still showed the panel in the main window,
- whether the log grep found any `[effect-window-create]` `DockFloating` lines,
- and whether the failure looks like:
  - host/session mismatch,
  - runner capability-posture drift,
  - or docking fallback drift.

## Decision

From this point forward:

1. this runbook is the default real-host acceptance path for `DW-P1-linux-003`,
2. `docking-arbitration-demo-wayland-degrade-no-os-tearoff.json` is the canonical scripted proof
   surface for this slice,
3. `diag windows`, `diag dock-graph`, and the optional tear-off log grep are the bounded review
   surfaces for this acceptance,
4. and future Wayland acceptance notes should reference this runbook instead of inventing a new
   command sequence.
