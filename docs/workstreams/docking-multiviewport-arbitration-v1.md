# Docking + Multi-Viewport Input Arbitration (v1)

Status: Draft (fearless refactor workstream)

This workstream defines the contracts, diagnostics, and regression harness needed to keep docking
and embedded engine viewports behaving deterministically under:

- multiple viewport panels in a single window (splits / tabs),
- multiple OS windows (tear-off),
- multi-root overlays (menus/popovers/tooltips/modals),
- pointer capture sessions (viewport tools, drags),
- view-cache reuse (producer subtrees may not rerender every frame).

The goal is to avoid a late-stage rewrite by locking the “hard-to-change” arbitration seams early.

## Scope (What “viewport” means here)

This document focuses on **engine viewport panels** (render targets embedded via `RenderTargetId`,
ADR 0007) and their forwarded input (`ViewportInputEvent`, ADR 0132).

UI clipping/scroll “viewports” are a separate concern.

## Contract Gates (Hard Boundaries)

- Engine viewport surfaces: `docs/adr/0007-viewport-surfaces.md`
- Viewport input forwarding (explicit units): `docs/adr/0132-viewport-input-forwarding-explicit-units.md`
- Focus + command routing (modal gating): `docs/adr/0020-focus-and-command-routing.md`
- Multi-root overlays: `docs/adr/0011-overlays-and-multi-root.md`
- Docking layering split (B route): `docs/adr/0075-docking-layering-b-route-and-retained-bridge.md`
- Docking × overlays × viewport capture arbitration: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Multi-pointer identity + sessions: `docs/adr/0150-pointer-identity-and-multi-pointer-capture.md`,
  `docs/adr/0151-multi-pointer-drag-sessions-and-routing-keys.md`

## Current State (Baseline)

- The UI runtime publishes a **window-scoped** arbitration snapshot via `InputContext.window_arbitration`
  (`WindowInputArbitrationSnapshot`, `crates/fret-runtime/src/window_input_arbitration.rs`).
- Docking implements viewport forwarding inside the docking UI layer
  (`ecosystem/fret-docking/src/dock/viewport.rs`, `.../space.rs`) and owns a local viewport-capture
  session (currently single-pointer).
- Overlay policy (menus/popovers/tooltips) lives in `ecosystem/fret-ui-kit/window_overlays` and
  already has guardrails for view-cache reuse (cached synthesis + TTL).

## Problem Statement (Why v1 is needed)

Once docking + multiple embedded viewports are present, “hand feel” regressions tend to show up as
boundary failures:

- a viewport tool capture unexpectedly loses move/up when a dock drag starts,
- a modal barrier blocks overlays but still forwards viewport input (or vice versa),
- two visible viewports fight for focus/capture ownership,
- behavior depends on whether a subtree rerendered (view-cache hit/miss),
- regressions are hard to diagnose because “who owned the pointer” is not observable.

Most of these failures are not renderer bugs; they are arbitration bugs. v1 treats diagnostics and
tests as the primary delivery.

## v1 Deliverables

1) **Diagnostics seam**
   - Export “viewport input forwarding + capture ownership + suppression reason” into diagnostic bundles.
   - Make scripted regressions able to gate on “we actually forwarded viewport input” and “capture existed”.

2) **Conformance tests**
   - Unit tests in `fret-docking` for routing rules across multiple viewports and capture transitions.
   - Scripted regressions (UI Gallery) that cover: split viewports, dock drags, modal barriers, and view-cache reuse.

3) **Refactor guardrails (multi-pointer ready)**
   - Move docking’s viewport capture from a single `Option<...>` to pointer-keyed state (`PointerId → capture`),
     aligning with ADR 0072/0150/0151 (even if demos remain mouse-only).

## Non-goals (v1)

- A full editor tool API (ADR 0049 remains deferred).
- Redesigning viewport overlay rendering hooks (stay in app/eco scope).
- Replacing docking UI authoring style (retained bridge vs declarative).

## Next

See the executable tracker:

- `docs/workstreams/docking-multiviewport-arbitration-v1-todo.md`

