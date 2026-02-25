---
title: Shadcn Semantic Drift Sweep (v1) — Milestones
status: draft
date: 2026-02-24
---

# Shadcn Semantic Drift Sweep (v1) — Milestones

Workstream entry:

- `docs/workstreams/shadcn-semantic-drift-sweep-v1.md`

## M0 — Seed docs + inventory (landable)

- Workstream + TODO + milestones documents exist.
- Seed drift list includes at least:
  - responsive semantics example (DataTable LG),
  - theme name heuristic inventory,
  - token-read snapshot sweep plan.

## M1 — Responsive policy decisions (viewport vs container)

- A written decision for each “unclear” responsive case in `fret-ui-shadcn`.
- For DataTable:
  - Decision recorded (parity-first vs editor-first vs dual-mode).
  - Evidence anchors point to upstream `repo-ref/ui` sources.
- Status (2026-02-25):
  - Seed decision table covers all current viewport/container query callsites in
    `ecosystem/fret-ui-shadcn/src/` (see TODO doc). Remaining work is upstream evidence collection
    and any behavior changes / gates.

## M2 — Theme metadata strategy (remove name heuristics)

- A stable, documented strategy replaces `theme.name.contains("/dark")`.
- If new theme metadata is introduced:
  - it is app/theme-owned (ADR 0032),
  - it does not leak policy into `crates/fret-ui`,
  - it has at least one regression test.

## M3 — Token read sweep (low-risk refactor)

- High-confidence sweep completed:
  - convert safe callsites to `ThemeSnapshot`,
  - avoid borrow pitfalls called out in `docs/component-author-guide.md`.
- No new public runtime contracts introduced.
- Status (2026-02-25):
  - `Theme::global(&*cx.app).clone()` reduced to 0 callsites in `ecosystem/fret-ui-shadcn/src/`.

## M4 — Regression gates for behavior changes

- For each behavior-changing migration (responsive decisions, theme metadata, motion outcomes),
  at least one gate exists:
  - unit tests for invariants, and/or
  - diag scripts with stable `test_id` targets.

## M5 — Closure pass

- Drift inventory updated to reflect remaining known gaps.
- Any follow-up workstreams spawned if scope grows (e.g. “DataTable editor-first container
  responsiveness” as a separate workstream).
