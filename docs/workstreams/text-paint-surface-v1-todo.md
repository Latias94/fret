---
title: Text Paint Surface v1 — TODO
status: active
date: 2026-02-16
---

# Text Paint Surface v1 — TODO Tracker

Status: Active (workstream tracker)

Workstream narrative: `docs/workstreams/text-paint-surface-v1.md`  
Milestone board: `docs/workstreams/text-paint-surface-v1-milestones.md`

## Tracking format

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `TPS-{area}-{nnn}`

Leave 1–3 evidence anchors when completing an item (paths + key functions/tests), and prefer
renderer conformance tests for correctness-sensitive semantics.

## M0 — Contract lock-in (bounded + portable)

- [ ] TPS-contract-010 Extend `SceneOp::Text` to accept `Paint` instead of solid `Color`.
- [ ] TPS-contract-020 Define paint coordinate semantics for text (origin + glyph local pos).
- [ ] TPS-adr-030 Add an ADR that locks semantics + degradation policy.

## M1 — Renderer implementation (wgpu default)

- [ ] TPS-render-100 Encode `Paint` for text draws (bounded, deterministic).
- [ ] TPS-render-110 Implement gradient paint evaluation in the text shader/pipeline.
- [ ] TPS-render-120 Ensure material paint is capability-gated and degrades deterministically.

## M2 — Conformance (required)

- [ ] TPS-test-200 Add GPU readback conformance for text paint:
  - linear gradient has expected left/right coverage on glyph shapes
  - stability across scale factors
  - uses a deterministic font source (avoid system-font flakiness)

## M3 — Adoption (optional)

- [ ] TPS-adopt-300 Wire one real consumer to use non-solid text paint:
  - pick a small demo surface (ui-gallery / editor diagnostics) to validate ergonomics.

