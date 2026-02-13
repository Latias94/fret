---
title: Mobile Graphics Backend Selection (v1) — Milestones
status: draft
date: 2026-02-12
scope: Android-first + iOS parity
---

# Mobile Graphics Backend Selection (v1) — Milestones

Workstream entry:

- `docs/workstreams/mobile-gfx-backend-v1/design.md`

## M0 — Scope + contract drafted

Definition of done:

- ADR 0268 exists with clear default policy + override + diagnostics requirements:
  - `docs/adr/0268-mobile-graphics-backend-selection-and-downlevel-policy-v1.md`
- The workstream docs exist (design + TODO + milestones).

## M1 — Observable backend selection

Definition of done:

- Every mobile run logs:
  - selected backend,
  - adapter/vendor/device,
  - driver_info,
  - override provenance (explicit vs automatic).

Evidence:

- Log snippet in a diag bundle or scripted capture showing the above fields.

## M2 — Fail-fast default + opt-in fallback (dev only)

Definition of done:

- CI/release posture is fail-fast on backend init failure.
- A developer-only switch can enable fallback attempts (if desired), and records attempt history.

Evidence:

- A reproducible failure mode captures “attempted backends + errors” in logs/diag metadata.

## M3 — Real-device acceptance gate

Definition of done:

- Android Vulkan-first path runs on at least one real device without crashes during init.
- The device recipe is documented and repeatable.

Evidence:

- A short “device smoke test” log/diag record attached to the workstream.
- Real-device smoke recipe + bundle anchors:
  - `docs/workstreams/mobile-gfx-backend-v1/m3-real-device-smoke-oppo-plg110.md`
