---
title: Mobile Contracts (v1) — Milestones
status: draft
date: 2026-02-12
scope: contract-first mobile readiness (Android-first, iOS follow-up)
---

# Mobile Contracts (v1) — Milestones

Workstream entry:

- `docs/workstreams/mobile-contracts-v1/design.md`

## M0 — ADRs landed (contract surfaces explicit)

Definition of done:

- ADR 0261 (TextInputClient interop), ADR 0262 (lifecycle/surface), ADR 0263 (pointer/touch baseline)
  exist and cross-reference the relevant accepted ADRs.
- ADR 0260 (mobile shell bridge) references these ADRs as contract anchors.

## M1 — Evidence captured (diag-first)

Definition of done:

- A scripted diag scenario can simulate keyboard occlusion (via insets) and prove “focused input is
  visible” for at least one UI-gallery screen.
- A diagnostics bundle can report:
  - committed insets,
  - primary pointer type,
  - text-input focus/composition snapshot.

## M2 — Next hard contracts outlined (drafts only)

Definition of done:

- ADR drafts exist (even if not accepted) for:
  - file picker + sandbox handle semantics,
  - share sheet / intent bridge,
  - clipboard portability (mobile).

