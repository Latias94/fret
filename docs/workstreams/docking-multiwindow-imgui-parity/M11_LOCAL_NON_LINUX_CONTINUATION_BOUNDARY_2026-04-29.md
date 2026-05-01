# M11 Local Non-Linux Continuation Boundary - 2026-04-29

Status: local continuation boundary; Wayland real-host acceptance remains open

Related:

- `WORKSTREAM.json`
- `docking-multiwindow-imgui-parity-todo.md`
- `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`
- `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
- `tools/diag-campaigns/imui-p3-mixed-dpi-real-host.json`
- `tools/diag-campaigns/imui-p3-windows-placement-real-host.json`
- `tools/diag-campaigns/imui-p3-wayland-real-host.json`

## Purpose

This lane is still the active P3 owner for editor-grade docking multi-window hand feel. The local
continuation question on 2026-04-29 is narrower: after the Windows placement, mixed-DPI, and v1
window-style slices were accepted, the only explicit unfinished item is the Linux Wayland compositor
acceptance run.

That acceptance cannot be completed from a non-Linux host. This note freezes the local continuation
boundary so future work does not reopen generic IMUI helpers, `crates/fret-ui`, or the maintenance
IMUI umbrella just because the remaining proof needs a different host.

## Assumptions-first resume

1. Confident: this lane remains the active execution owner for P3 multi-window hand-feel closure.
   Evidence: `WORKSTREAM.json` has `status: active`; `docs/workstreams/README.md` lists this as the
   active P3 docking parity execution lane. If wrong, future work could be split into a duplicate
   lane and weaken the first-open state.
2. Confident: the remaining explicit TODO is real-host Wayland acceptance, not another generic IMUI
   helper gap. Evidence: `docking-multiwindow-imgui-parity-todo.md` keeps only "Manual Wayland
   compositor acceptance remains open" under `DW-P1-linux-003`; the source policy, fallback, Windows
   placement, mixed-DPI, and style slices are already recorded as done. If wrong, a missing local TODO
   should be added as a narrow follow-on with its own proof surface.
3. Confident: a non-Linux host cannot accept `DW-P1-linux-003`. Evidence:
   `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md` requires a native Linux Wayland session
   and a `platform.capabilities` admission payload reporting the Wayland-safe posture. If wrong, the
   campaign admission contract would be too strict and should be fixed in the diagnostics lane first.
4. Likely: the best local continuation is validation and documentation of the current boundary.
   Evidence: the campaign manifests validate locally, and the source-policy tests still pass. If
   wrong, the next implementation slice should start from a failing gate or a new real-host bundle,
   not from broad API growth.
5. Confident: generic `fret-ui` widening is not justified by the current evidence. Evidence:
   `WORKSTREAM.json` scopes the lane to runner/backend-owned closure, and the maintenance IMUI
   umbrella says implementation-heavy work should move to narrow follow-ons. If wrong, the contract
   needs ADR-level evidence before changing mechanism-layer surface area.

## Local validation evidence

These commands passed on 2026-04-29 from the local non-Linux continuation pass:

```text
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-wayland-real-host.json --json
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-windows-placement-real-host.json --json
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-mixed-dpi-real-host.json --json
python tools/gate_imui_workstream_source.py
```

Observed result:

- all four campaign manifests validated,
- the Python IMUI workstream source-policy gate passed,
- no Linux/Wayland real-host run was attempted.

## Decision

For non-Linux continuation:

1. Keep this lane active, but treat the remaining implementation acceptance as host-blocked until a
   real Linux Wayland compositor is available.
2. Keep `imui-p3-wayland-real-host` as the canonical admission wrapper and
   `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md` as the acceptance recipe.
3. Treat local work as limited to:
   - source-policy gates,
   - campaign manifest validation,
   - diagnostics drift repair if a validation gate fails,
   - or a new narrow follow-on only if fresh evidence shows a separate non-Linux regression.
4. Do not reopen `imui-editor-grade-product-closure-v1` or widen generic IMUI helpers for this
   remaining platform acceptance item.

The next real closure event for `DW-P1-linux-003` should be a dated Wayland evidence note created
from the M5 runbook after a qualifying Linux host run.
