---
title: Renderer Paint GPU Storage Unification v1 — Milestones
status: active
date: 2026-02-16
---

# Renderer Paint GPU Storage Unification v1 — Milestones

This board tracks a renderer-internal refactor (no contract changes).

## Definition of done

We consider this workstream shipped when:

1. A shared “storage ring buffer” utility backs both path and text paint uploads.
2. Behavior and WGSL binding surfaces are unchanged.
3. The standard renderer gates remain green.

## Milestones

### M0 — Docs baseline

Status: Landed.

### M1 — Storage ring utility

Status: Landed.

### M2 — Adoption (path + text paints)

Status: Landed.

### M3 — Gates + evidence

Status: Landed.
