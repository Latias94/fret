# Fret Interaction Kernel (v1)

Status: Draft / proposed.

This workstream introduces an ecosystem-level crate that consolidates interaction primitives shared
across editor-grade surfaces:

- `imui` in-window floating windows (title bar drag, resize, activation choreography).
- `fret-node` canvas interactions (pan/zoom, selection drag, semantic zoom sizing).
- docking/multi-window parity (ImGui-style "peek behind moving window" and transparent hit-testing
  while dragging tear-off windows).

The goal is to converge on one set of small, testable state machines and math helpers so "hand feel"
does not drift across subsystems.

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

The crate should provide:

- drag thresholding helpers,
- a single explicit 2D viewport transform convention (pan/zoom mapping),
- reusable (but parameterized) interaction state machines for:
  - drag capture,
  - activation vs focus (bring-to-front gating),
  - resize handles (hitbox + cursor policy),
  - optional animated viewport changes (fit-view style requests).

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

The initial v1 API should cover screen-space and canvas-space explicitly. OS/window-space integration
is a milestone (see milestones doc).

## Evidence / related workstreams

- ImGui parity tracking: `docs/workstreams/imui-imgui-parity-audit-v1.md`
- Docking multi-window parity: `docs/workstreams/docking-multiwindow-imgui-parity.md`
- XyFlow parity notes: `docs/workstreams/fret-node-xyflow-parity.md`

## Proposed acceptance criteria

This workstream is considered successful when:

1. `imui` floating window drag/resize/activation uses the shared kernel helpers with no regressions.
2. `fret-node` viewport math and animation helpers use the shared kernel helpers with conformance
   tests preserved.
3. `fretboard diag` scripts exist to prevent regressions for:
   - fractional DPI (150%) correctness,
   - stale paint checks during drags,
   - multi-window hover arbitration behaviors (docking parity track).

For a concrete checklist and gates, see:

- `docs/workstreams/fret-interaction-kernel-v1-todo.md`
- `docs/workstreams/fret-interaction-kernel-v1-milestones.md`

