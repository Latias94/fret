# Material 3 IconButton Diagnostics Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The matrix still marked IconButton as a known-follow-on row. The promoted centered-chrome script
also targeted the aggregate Material3 gallery page, while the stable centered IconButton selector
now lives on the dedicated Icon Button page.

## Truth

- IconButton recipe owns variants, toggle semantics, expressive selected/pressed shape morphing, and
  stable selectors.
- Material foundation owns shared state-layer/ripple and minimum interactive target sizing.
- Diagnostics should prove the 48px interaction target and centered visual chrome for a dedicated
  IconButton page target.
- No kit-policy or mechanism change is justified by the current evidence.

## Boundaries

- Do not change IconButton implementation unless repaired diagnostics prove drift.
- Do not move shape morphing to kit policy in this packet.
- Treat stale page navigation as diagnostics harness drift.
