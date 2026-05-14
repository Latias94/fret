# M16 Source Drift Guard - 2026-05-14

Status: source drift guard; no behavior change.

This note records a source-only guard for the docking multi-window lane. The shipped behavior did
not change in this slice; the purpose is to prevent stale parity notes from misclassifying already
promoted docking diag gates as open gaps.

Follow-up (2026-05-15): the same source guard now also parses the Wayland real-host campaign and
bounded Wayland degradation script directly. This keeps local non-Linux maintenance honest: the
campaign must remain host-admitted by `platform.capabilities`, and the script must still prove
"attempt tear-off -> one known OS window -> captured bundle" before any real Wayland acceptance note
can count.

## Assumptions-First Resume

1. Confident: `docking-multiwindow-imgui-parity` remains the active lane for runner/backend-owned
   multi-window hand-feel. Evidence: `WORKSTREAM.json` has `status: active` and still points to the
   bounded P3 campaign, Wayland real-host campaign, and platform-specific acceptance docs. If wrong,
   this guard should move with the new lane owner.
2. Confident: the standalone `docking-multi-window-imgui-alignment-v1.md` note is still useful for
   behavior vocabulary, but no longer owns current execution state. Evidence: the dedicated lane now
   has `WORKSTREAM.json`, M13-M15 status notes, and current campaign gates. If wrong, the standalone
   note would need a full promotion instead of a status note.
3. Likely: tab overflow should not remain listed as an ungated docking gap. Evidence: docking
   arbitration has promoted tab overflow/autoscroll scripts in the full suite, and
   `diag-hardening-smoke-docking` includes the overflow menu select/close smoke gates. If wrong,
   the TODO should name the missing tab-strip behavior precisely instead of using the old broad line.
4. Confident: this slice must not widen `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui`.
   Evidence: the lane scope is runner/backend + `fret-docking`, and the change is source policy plus
   workstream evidence only. If wrong, an ADR-level owner change is required first.

## What Changed

- Added `tools/gate_docking_multiwindow_workstream_source.py`.
- Wired it into `python tools/gate_imui_workstream_source.py` so the canonical IMUI source-policy
  gate also catches docking multi-window source drift.
- Updated `docs/workstreams/standalone/docking-multi-window-imgui-alignment-v1.md` with a current
  status note and corrected tab overflow status.
- Recorded the new gate in `WORKSTREAM.json`, the TODO tracker, and the repo-wide workstream/roadmap
  indexes.
- Follow-up: extended the same gate to source-check
  `tools/diag-campaigns/imui-p3-wayland-real-host.json` and
  `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-wayland-degrade-no-os-tearoff.json`.

## Guarded Invariants

- Root docking arbitration suite equals the platform split:
  - `tools/diag-scripts/suites/docking-arbitration/common/suite.json`
  - `tools/diag-scripts/suites/docking-arbitration/windows/suite.json`
  - `tools/diag-scripts/suites/docking-arbitration/suite.json`
- `tools/diag-scripts/suites/diag-hardening-smoke-docking/suite.json` stays small while still
  covering the representative title-bar, under-moving-window, tearoff-merge, and overflow menu
  smoke gates.
- The standalone behavior-first note no longer teaches tab overflow as an ungated gap.
- Current execution continues through the dedicated lane and Wayland real-host acceptance remains
  open.
- `imui-p3-wayland-real-host` stays a manual, host-admitted campaign whose environment predicate
  requires Linux plus `ui.window_tear_off=false`, `ui.window_hover_detection=none`, and
  `ui.window_z_level=none`.
- The canonical Wayland degradation script still waits for hover detection `none`, performs a long
  tear-off gesture, asserts `known_window_count_is(n=1)`, and captures
  `docking-arbitration-demo-wayland-degrade-no-os-tearoff`.

## Commands Run

```powershell
python tools/gate_docking_multiwindow_workstream_source.py
python tools/gate_imui_workstream_source.py
python -m py_compile tools/gate_docking_multiwindow_workstream_source.py tools/gate_imui_workstream_source.py
python -m json.tool docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

Follow-up command set (2026-05-15):

```powershell
python -m py_compile tools/gate_docking_multiwindow_workstream_source.py tools/gate_imui_workstream_source.py
python tools/gate_docking_multiwindow_workstream_source.py
python tools/gate_imui_workstream_source.py
```

## Results

- `python tools/gate_docking_multiwindow_workstream_source.py` passed.
- `python tools/gate_imui_workstream_source.py` passed, including the nested docking multi-window
  source guard.
- `python -m py_compile` passed for both source gate scripts.
- `python -m json.tool` passed for `WORKSTREAM.json`.
- `python tools/check_workstream_catalog.py` passed with 370 dedicated directories and 47
  standalone markdown files.
- `git diff --check` passed.
- 2026-05-15 follow-up source guard passed locally and now fails if the Wayland campaign stops using
  `platform.capabilities` admission or if the bounded script stops asserting one-window fallback
  evidence.

## Verdict

This is a source-policy and evidence-alignment slice only. It improves the next-maintainer entry
point and guards the current suite split, but it does not close `DW-P1-linux-003`; the Wayland
compositor acceptance runbook remains the next true platform-specific closure path.
