---
title: Mobile Share + Clipboard (v1) — Milestones
status: draft
date: 2026-02-12
scope: contract-first share/export + open-in/import + clipboard portability (Android-first)
---

# Mobile Share + Clipboard (v1) — Milestones

Workstream entry:

- `docs/workstreams/mobile-share-and-clipboard-v1/design.md`

## M0 — ADRs landed (contract surfaces explicit)

Definition of done:

- ADR 0265 and ADR 0266 exist, are cross-referenced by the mobile contract map, and are consistent
  with the existing effect/token patterns.

## M1 — Minimal implementation (one runner)

Definition of done:

- One runner (desktop or web) implements:
  - share sheet show (best-effort) and a completion event, and
  - clipboard read failure paths are observable and do not crash.

## M2 — Diagnostics gates

Definition of done:

- A scripted diag scenario can:
  - trigger a clipboard paste request and observe success/unavailable behavior,
  - simulate an incoming-open token and validate bounded reads + explicit release semantics.

