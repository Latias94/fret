---
title: Mobile Graphics Backend Selection (v1) — Design
status: draft
date: 2026-02-12
scope: Android-first + iOS parity
---

# Mobile Graphics Backend Selection (v1) — Workstream

Goal: define and implement a stable, debuggable policy for selecting and validating the graphics
backend on mobile targets (Android/iOS), without coupling UI code to backend quirks.

This workstream is intentionally aligned with “Fret as a UI framework engine” (Flutter-like shell):

- the runner owns lifecycle + surfaces + backend init (ADR 0262),
- apps/components must remain backend-agnostic (ADR 0066),
- diagnostics must make failures explainable and reproducible.

## Contract anchors

- Backend policy (normative): `docs/adr/0268-mobile-graphics-backend-selection-and-downlevel-policy-v1.md`
- Mobile lifecycle + surface policy: `docs/adr/0262-mobile-lifecycle-and-surface-policy-v1.md`

## Current observations (why this exists)

- Android emulators frequently run `wgpu` Vulkan through GFXStream/SwiftShader. This can be unstable
  for early renderer init and can crash the process.
- A “just try OpenGL” fallback is not automatically safe: downlevel device creation can fail due to
  missing limits/capabilities, and shaders/pipelines may not be portable.

Therefore:

- real devices are the primary acceptance gate for correctness/performance,
- emulators are best-effort, for non-GPU smoke tests and developer iteration.

## Policy sketch (v1)

- iOS: prefer Metal (platform-native).
- Android: prefer Vulkan (platform-native on real devices).
- Explicit override wins (`FRET_WGPU_BACKEND`).
- Default posture is fail-fast on init failure (CI/release).
- Fallback/downlevel is opt-in (debug/dev only) and must be fully diagnosable.

## Developer knobs (v1)

- `FRET_WGPU_BACKEND`: explicit backend override (fail-fast if invalid or unsupported).
- `FRET_WGPU_ALLOW_FALLBACK=1`: enables fallback backend attempts in **debug builds only**.
  Release builds remain fail-fast.

## Minimum renderer gate (v1)

The default renderer requires `wgpu::DownlevelFlags::VERTEX_STORAGE` (storage buffers in vertex
shaders). If the selected adapter does not satisfy this, initialization fails fast so we avoid
late validation panics during pipeline creation.

## Deliverables

- A stable “backend selection story” for mobile bring-up, documented and testable.
- A minimal diagnostic record for every run (backend + adapter + driver + override provenance).
- A reproducible device smoke test recipe (real device first).

## Non-goals (v1)

- Perfect emulator support across host GPUs/SDK versions.
- A complete Android GPU compatibility database.
