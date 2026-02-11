---
title: Mobile Bring-up (v1)
status: draft
date: 2026-02-11
scope: Android-first MVP (ui-gallery), iOS follow-up
---

# Mobile Bring-up (v1) — Workstream

Goal: run `fret-ui-gallery` on mobile with acceptable MVP UX:

- can scroll (touch pan),
- can type (IME works),
- focused inputs are not obscured by the on-screen keyboard (keyboard avoidance via occlusion insets).

This workstream is intentionally **Android-first**:

- Android is the fastest path to a real device smoke test for `winit + wgpu`.
- iOS has the same contract needs, but requires separate platform glue for insets and lifecycle.

## iOS simulator run (no Xcode project)

For quick iteration on iOS without committing an Xcode project, we can bundle the Rust-built
executable into a minimal `.app` and run it via `simctl`.

Prereqs:

- Xcode (or Xcode Command Line Tools) with an iOS Simulator runtime installed.
- Rust targets installed:
  - `rustup target add aarch64-apple-ios-sim`

Run:

- `tools/mobile/ios_sim_run.sh --release`

Notes:

- Set `IOS_SIM_UDID=<udid>` to force a specific simulator device.
- This is a development loop only; real device packaging/signing is tracked separately.

## Contract anchors (already accepted)

- Mechanism vs policy split: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Platform backends split: `docs/adr/0090-platform-backends-native-web.md`
- Environment queries + insets: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`

## Non-goals (v1)

- Multi-window on mobile.
- Full gesture arena parity (Flutter-grade). We only need a reliable pan-to-scroll baseline.
- Perfect keyboard avoidance across all OEMs and IMEs on day one (we want a stable seam + evidence).

## Layering rules (non-negotiable)

- `crates/fret-ui`: mechanism only (pointer routing, capture, scroll containers, scroll handles, environment queries).
- `ecosystem/fret-ui-kit`: gesture/policy glue (gesture arena, capture-steal, inertial scrolling, keyboard avoidance helpers).
- `ecosystem/fret-ui-shadcn`: recipe adoption (ScrollArea inherits runtime scroll behavior; gallery uses keyboard avoidance).
- `crates/fret-launch` / `crates/fret-platform-*`: platform glue (lifecycle + safe-area/keyboard insets commits).

## Tracking

- Milestones: `docs/workstreams/mobile-bringup-v1-milestones.md`
- TODO list: `docs/workstreams/mobile-bringup-v1-todo.md`
