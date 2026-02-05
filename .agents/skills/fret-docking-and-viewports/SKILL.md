---
name: fret-docking-and-viewports
description: Docking, multi-window, and viewport integration in Fret (editor-grade UI). Use when working on DockSpace/tabs/splits/tear-off windows, viewport panels, or input arbitration between docking drags and embedded viewports.
---

# Fret docking and viewports

Docking is a “policy-heavy” editor feature. In Fret it is intentionally split:

- **Core model/ops (stable):** `crates/fret-core` (IDs, docking ops, persistence contracts)
- **UI + interaction arbitration:** `ecosystem/fret-docking` (DockSpace, drag policies, indicators)
- **Runner/windowing:** `crates/fret-launch`, `crates/fret-runner-winit` (multi-window lifecycle, DPI)

## What to optimize for

- Deterministic drag + input arbitration (dock drag vs viewport capture).
- Multi-window correctness (tear-off and re-dock).
- Reproducible diagnostics (scripts + bundles) for regressions.

## Code entry points

- Docking UI (policy-heavy): `ecosystem/fret-docking/src/dock/space.rs`
- Workspace/tab strip helpers: `ecosystem/fret-workspace/src/*`
- Demos / conformance harness:
  - `apps/fret-examples/src/docking_demo.rs`
  - `apps/fret-examples/src/docking_arbitration_demo.rs`

## Debugging workflow (recommended)

- Use scripted repros + bundles:
  - `tools/diag-scripts/docking-demo-drag-indicators.json`
  - `tools/diag-scripts/docking-arbitration-demo-split-viewports.json`
  - `tools/diag-scripts/docking-arbitration-demo-modal-dock-drag-viewport-capture.json`
- When a bug is fixed, keep the script as a gate (prefer invariants over pixels).

See: `fret-diag-workflow`.

## References

- Docking ops + persistence: `docs/adr/0013-docking-ops-and-persistence.md`
- Multi-window display + DPI: `docs/adr/0017-multi-window-display-and-dpi.md`
- Docking arbitration checklist: `docs/docking-arbitration-checklist.md`
- Docking multi-window parity tracker: `docs/workstreams/docking-multiwindow-imgui-parity.md`
- Viewport embedding contracts: `docs/viewport-panels.md`, `docs/adr/0007-viewport-surfaces.md`
