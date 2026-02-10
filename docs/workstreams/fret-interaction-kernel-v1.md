# Fret Interaction Kernel (v1)

Status: Draft (v1 direction locked)
Last updated: 2026-02-10

This workstream introduces an ecosystem-level crate that consolidates interaction primitives shared
across editor-grade surfaces:

- `imui` in-window floating windows (title bar drag, resize, activation choreography).
- `fret-node` canvas interactions (pan/zoom, selection drag, semantic zoom sizing).
- docking/multi-window parity (ImGui-style "peek behind moving window" and transparent hit-testing
  while dragging tear-off windows).

The goal is to converge on one set of small, testable interaction state machines (and a single
source of truth for math/coordinate conventions) so "hand feel" does not drift across subsystems.

## Motivation

Today, interaction logic is spread across multiple crates:

- `ecosystem/fret-ui-kit/src/imui/*`
- `ecosystem/fret-node/src/interaction/*` + `ecosystem/fret-node/src/ui/*`
- docking/runner integration surfaces (multi-window hit-test and hover arbitration)

Each area is individually correct, but the overlap creates:

- duplicated gesture threshold math and coordinate transforms,
- subtle DPI/rounding differences,
- inconsistent activation / capture choreography (especially under fractional DPI and multi-window).

## Scope

Create a new ecosystem crate:

- Path: `ecosystem/fret-interaction`
- Name: `fret-interaction`

The crate should provide (v1):

- interaction state machines and choreography for editor-grade gestures:
  - drag capture,
  - activation vs focus (bring-to-front gating),
  - resize handles (hitbox + cursor policy),
  - optional animated viewport changes (fit-view style requests).
- shared, policy-light helpers used by those state machines:
  - drag threshold evaluation (ImGui-aligned defaults where applicable),
  - DPI / device-pixel snapping helpers (when necessary).

### v1 design decision: math lives in `fret-canvas`

We will not maintain a second set of viewport/pan/zoom mapping helpers in `fret-interaction`.

- Canonical math + coordinate conventions for canvas-style viewports live in `ecosystem/fret-canvas`:
  - `fret_canvas::view::PanZoom2D`
  - `fret_canvas::view::CanvasViewport2D`
- `fret-node` already depends on `fret-canvas` and has conformance tests that act as the stability
  harness for these conventions.

`fret-interaction` may re-export those types (or provide thin adapters) to avoid dependency sprawl,
but it must not introduce competing mapping semantics.

## Non-goals

- No node-graph domain types (nodes/edges/ports) in this crate.
- No styling/tokens. Visuals remain owned by the host surface (`fret-ui-kit`, `fret-node`).
- No `crates/fret-ui` contract changes in v1 (keep this ecosystem-only unless proven stable).

## Layering and ownership rules

- This is an ecosystem crate. It may depend on `fret-core` and (when required) `fret-ui`/`fret-runtime`,
  but it must not force the mechanism layer to adopt policy.
- Call sites must remain the source of truth for:
  - coordinate space selection (screen-space vs canvas-space),
  - policy toggles (e.g. ImGui flags, XyFlow knobs),
  - effect wiring (commands, model updates, OS window actions).

## Coordinate spaces (v1 contract)

The kernel must be explicit about transforms, and avoid implicit assumptions:

- screen-space: UI logical pixels within an app window.
- canvas-space: world coordinates affected by pan/zoom (node graphs).
- OS/window-space: multi-window hover routing and hit-testing.

The initial v1 API must be explicit about screen-space vs canvas-space and must reuse `fret-canvas`
for canvas mapping. OS/window-space integration is a milestone (see milestones doc).

## M3 notes — docking / multi-window touchpoints (v1 sketch)

This section is intentionally *non-normative* until we land a concrete diag repro and wire it into a
gate.

The goal of M3 is not to move docking policy into the kernel. It is to standardize the minimum set
of primitives that multiple subsystems need to achieve consistent hand-feel when dragging across
windows.

Proposed split:

- `fret-interaction` owns:
  - small helpers/state machines for drag capture choreography (when to capture/release; how to
    update phases consistently),
  - cross-window drag session helpers that operate on `fret-runtime::DragSession`:
    - prefer using `begin_cross_window_drag_with_kind(...)` for gestures that must participate in
      cross-window hover arbitration,
    - ensure `DragSession.current_window` updates are consistent when the host reports a window
      transition.
- docking/runner owns:
  - OS-window hit-testing, hover routing, and “transparent moving window” policies,
  - translating docking effects into `WindowRequest::Create` / tear-off lifecycle,
  - platform capability checks and fallbacks.

Deliverable definition (M3 “done”):

- We have at least one deterministic `fretboard diag` repro that exercises multi-window hover
  arbitration while a drag is active.
- The kernel provides a small, reusable helper surface that the docking runtime and at least one
  other subsystem can share without re-implementing the same choreography.

## Evidence / related workstreams

- ImGui parity tracking: `docs/workstreams/imui-imgui-parity-audit-v1.md`
- Docking multi-window parity: `docs/workstreams/docking-multiwindow-imgui-parity.md`
- XyFlow parity notes: `docs/workstreams/fret-node-xyflow-parity.md`

## Proposed acceptance criteria

This workstream is considered successful when:

1. `imui` floating window drag/resize/activation uses shared kernel state machines with no
   regressions (including fractional DPI behavior).
2. `fret-node` continues to use `fret-canvas` viewport math, and no duplicate viewport mapping math
   is re-introduced under a new type in `fret-interaction`.
3. `fretboard diag` scripts exist to prevent regressions for:
   - fractional DPI (150%) correctness,
   - stale paint checks during drags,
   - multi-window hover arbitration behaviors (docking parity track).

For a concrete checklist and gates, see:

- `docs/workstreams/fret-interaction-kernel-v1-todo.md`
- `docs/workstreams/fret-interaction-kernel-v1-milestones.md`

## Gates (executable)

Preferred one-shot gate script for this workstream:

- `pwsh tools/diag_gate_interaction_kernel_v1.ps1`
  - Optional (stronger M3 check): `-StrongDockHover` (enforces `--check-dock-drag-current-windows-min 2`).
