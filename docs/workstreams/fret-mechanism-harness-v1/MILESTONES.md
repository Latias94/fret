---
title: Fret Mechanism Harness v1 Milestones
status: active
date: 2026-05-11
---

# Milestones

## M0: Lane Shape

Status: complete

- Dedicated workstream exists.
- Coverage map names current coverage and gaps.
- First-open gates are listed in `WORKSTREAM.json`.

## M1: Layout and Invalidation Slice

Status: complete

- Layout primitive fixture remains the baseline geometry suite.
- Layout dirty invalidation gains fixture-driven scalar metric coverage.
- Suppressed-boundary child dirty deltas and subtree removal are case-id-addressable.

## M2: Runtime Gate Connection

Status: complete

- UI Gallery checkbox underflow script is the first real runtime proof.
- The script remains part of `diag-hardening-smoke`.
- Synthetic dirty invalidation cases explain the same class of mechanism risk.

## M3: Findings Loop

Status: complete for the first slice

- Run the gate set.
- Classify defects by layer.
- Fix confirmed defects.
- Leave evidence, regression coverage, and the next-slice recommendation.

## M4: View-Cache and Root-Boundary Invalidation Slice

Status: complete

- The layout dirty invalidation fixture now covers retained contained relayout, dirty frontier
  coverage at a wrapper/direct-child boundary, detached dirty-cache-root pruning, and view-cache
  layout-dirty expansion attribution.
- The harness runner can capture intermediate metrics so frontier state can be asserted before
  layout consumes it.
- Focused gates for `view_cache`, scroll-contained frontier behavior, and layout request build-root
  attribution passed.
