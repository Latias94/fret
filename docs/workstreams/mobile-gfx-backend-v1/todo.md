---
title: Mobile Graphics Backend Selection (v1) — TODO
status: draft
date: 2026-02-12
---

# Mobile Graphics Backend Selection (v1) — TODO

Workstream entry:

- `docs/workstreams/mobile-gfx-backend-v1/design.md`

## Docs / contracts

- [ ] Accept ADR 0268 (or revise) once device evidence exists:
  - `docs/adr/0268-mobile-graphics-backend-selection-and-downlevel-policy-v1.md`
- [ ] Add this workstream to `docs/README.md` and `docs/workstreams/README.md`.

## Runner behavior (Android/iOS)

- [x] Record backend selection provenance:
  - explicit override vs automatic,
  - attempted backends and failure reasons (when fallback is enabled).
- [x] Capture adapter/backend metadata into diag bundles (when enabled).
- [x] Define a “fail-fast by default” posture for CI and release builds.

## Developer knobs

- [x] Document and standardize the env var(s):
  - `FRET_WGPU_BACKEND` (explicit override).
  - `FRET_WGPU_ALLOW_FALLBACK=1` (debug-only fallback attempts).
- [ ] (Optional) Add a structured runner config policy for mobile:
  - `VulkanOnly`, `PreferVulkan`, `PreferGl`, `AllowFallback` (names TBD).

## Evidence / acceptance

- [ ] Android real-device smoke test recipe (Pixel + one non-Pixel GPU class if possible).
- [ ] iOS simulator + real-device notes (backend selection is mostly Metal, but still record metadata).
