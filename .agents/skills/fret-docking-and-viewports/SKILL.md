---
name: fret-docking-and-viewports
description: Docking, multi-window, and viewport integration in Fret (editor-grade UI). Use when working on DockSpace/tabs/splits/tear-off windows, viewport panels, or input arbitration between docking drags and embedded viewports.
---

# Fret docking and viewports

## When to use

Use this skill when:

- Working on DockSpace/tabs/splits/tear-off windows.
- Debugging drag arbitration (dock drag vs viewport capture vs overlays).
- Ensuring multi-window correctness (tear-off and re-dock) and DPI/metrics issues.

Docking is a “policy-heavy” editor feature. In Fret it is intentionally split:

- **Core model/ops (stable):** `crates/fret-core` (IDs, docking ops, persistence contracts)
- **UI + interaction arbitration:** `ecosystem/fret-docking` (DockSpace, drag policies, indicators)
- **Runner/windowing:** `crates/fret-launch`, `crates/fret-runner-winit` (multi-window lifecycle, DPI)

## Inputs to collect (ask the user)

Ask these before touching docking code (most bugs are “arbitration contracts”):

- Which demo/app reproduces it (prefer `docking_arbitration_demo`)?
- What is the expected invariant: where should the drag land / what should capture input / what should dismiss?
- Multi-window involvement: does it require tear-off / re-dock across windows?
- Viewport involvement: are embedded viewports (canvas/editor) capturing pointer/keyboard?
- Evidence needs: do we need a scripted repro + bundle, or also screenshots?

Defaults if unclear:

- Reproduce in `docking_arbitration_demo`, add/keep a `tools/diag-scripts/*.json` gate, and prefer invariants over pixels.

## Smallest starting point (one command)

- `cargo run -p fretboard -- dev native --bin docking_arbitration_demo`

## What to optimize for

- Deterministic drag + input arbitration (dock drag vs viewport capture).
- Multi-window correctness (tear-off and re-dock).
- Reproducible diagnostics (scripts + bundles) for regressions.

## Quick start (run the conformance harness)

- `cargo run -p fretboard -- dev native --bin docking_arbitration_demo`

## Code entry points

- Docking UI (policy-heavy): `ecosystem/fret-docking/src/dock/space.rs`
- Workspace/tab strip helpers: `ecosystem/fret-workspace/src/*`
- Demos / conformance harness:
  - `apps/fret-examples/src/docking_demo.rs`
  - `apps/fret-examples/src/docking_arbitration_demo.rs`

## Workflow

- Use scripted repros + bundles:
  - `tools/diag-scripts/docking-demo-drag-indicators.json`
  - `tools/diag-scripts/docking-arbitration-demo-split-viewports.json`
  - `tools/diag-scripts/docking-arbitration-demo-modal-dock-drag-viewport-capture.json`
- When a bug is fixed, keep the script as a gate (prefer invariants over pixels).

See: `fret-diag-workflow`.

## Definition of done (what to leave behind)

- Minimum deliverables (3-pack): Repro (docking demo), Gate (diag script), Evidence (bundle). See `fret-skills-playbook`.
- The issue reproduces in the smallest docking demo target and is captured by a stable `tools/diag-scripts/*.json`.
- IDs are stable across reorder/tear-off (persistence and state do not “stick to positions”).
- Multi-window behavior is correct (tear-off creates a new window; re-dock restores expected layout).
- Drag arbitration is deterministic (dock drag vs viewport capture vs modal barriers).
- At least one evidence artifact exists for review:
  - bundle + triage output, and optionally screenshots if they add signal.

## Evidence anchors

- Docking ops + persistence: `docs/adr/0013-docking-ops-and-persistence.md`
- Multi-window lifecycle + DPI: `docs/adr/0017-multi-window-display-and-dpi.md`
- Docking UI/policy: `ecosystem/fret-docking/src/dock/space.rs`
- Conformance demos:
  - `apps/fret-examples/src/docking_demo.rs`
  - `apps/fret-examples/src/docking_arbitration_demo.rs`

## Common pitfalls

- Fixing docking behavior by adding knobs to `crates/fret-ui` (usually the wrong layer; prefer `ecosystem/fret-docking` policy).
- No scripted repro for drag arbitration (regressions become “human-only” and hard to review).
- Unstable IDs for tabs/panes (state and persistence become flaky across reorder/tear-off).

## References

- Docking ops + persistence: `docs/adr/0013-docking-ops-and-persistence.md`
- Multi-window display + DPI: `docs/adr/0017-multi-window-display-and-dpi.md`
- Docking arbitration checklist: `docs/docking-arbitration-checklist.md`
- Docking multi-window parity tracker: `docs/workstreams/docking-multiwindow-imgui-parity.md`
- Viewport embedding contracts: `docs/viewport-panels.md`, `docs/adr/0007-viewport-surfaces.md`

## Related skills

- `fret-diag-workflow` (scripted repros + bundles)
- `fret-overlays-and-focus` (modal barriers and overlay interference with drags)
