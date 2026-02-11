---
title: Mobile Bring-up (v1) — Milestones
status: draft
date: 2026-02-11
scope: Android-first MVP (ui-gallery), iOS follow-up
---

# Mobile Bring-up (v1) — Milestones

Workstream entry:

- `docs/workstreams/mobile-bringup-v1.md`

This milestone plan defines “done” in terms of observable outcomes (tests / diagnostics evidence),
not internal implementation details.

## M0 — Scope locked (Android-first MVP)

Definition of done:

- Workstream docs exist (this file + TODO list).
- Target demo and acceptance criteria are explicit:
  - `fret-ui-gallery` runs on Android (device/emulator),
  - can scroll with touch,
  - can type with IME,
  - focused input is not obscured by keyboard.

## M1 — Touch pan-to-scroll baseline (runtime)

Definition of done:

- Touch pan-to-scroll works for core scroll surfaces:
  - `crates/fret-ui` `Scroll`
  - `crates/fret-ui` `VirtualList`
- The implementation is touch-only and does not change layout/semantics in steady-state frames (no snapshot churn).

Evidence:

- Unit test proves touch dragging updates the bound `ScrollHandle` offset (via `fret-ui-shadcn::ScrollArea`).
- Optional: a UI-gallery scripted diag scenario demonstrates scroll on a real device.

## M2 — Keyboard avoidance seam (occlusion insets)

Definition of done:

- `fret-ui-gallery` applies a keyboard avoidance policy driven by environment queries:
  - uses `environment_occlusion_insets` (ADR 0232) and adds scrollable bottom slack (or padding)
    sufficient to keep focused inputs visible.

Evidence:

- Unit/integration test demonstrates the policy applies when `occlusion_insets.bottom > 0`.
- A diag bundle snapshot captures the committed insets and the chosen avoidance behavior.
- A scripted diag scenario simulates occlusion and asserts the focused input remains visible:
  - `tools/diag-scripts/ui-gallery-ai-chat-demo-keyboard-occlusion-focus-visible.json`

## M3 — Android insets + lifecycle plumbing (runner)

Definition of done:

- Android runner commits:
  - safe-area insets (best-effort),
  - keyboard occlusion insets (best-effort; must include IME bottom inset when visible),
  into the `WindowMetricsService` seam that `fret-ui` commits each frame.
- Android runner handles `Resumed` / `Suspended` so GPU surfaces are dropped and rebuilt safely.

Evidence:

- A debug view/diag snapshot shows non-zero insets when keyboard is visible.
- A device run can background/foreground without a crash or runaway rendering.

## M4 — iOS follow-up (contract parity)

Definition of done:

- iOS runner commits safe-area + keyboard occlusion insets into the same seam.
- iOS lifecycle and surface rebuild follow the same policy matrix as Android.

Evidence:

- Same acceptance criteria as M3, but on iOS.
