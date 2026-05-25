---
title: Shadcn Component Parity Matrix v1 TODO
status: active
date: 2026-05-25
---

# TODO

- [x] SCPM-010: Open a narrow workstream for shadcn component harness coverage, separate from the
      closed component fact harness v1 lane.
- [x] SCPM-020: Add a matrix generator that reads the canonical progress doc, coverage manifest,
      current suite report, and extra component packet artifacts.
- [x] SCPM-030: Generate the first machine-readable and human-readable component harness matrix.
- [ ] SCPM-040: Pick the next P0 `inventory_only` or `coverage_targeted` component and promote it
      to a full harness seed with source refs, upstream snapshot, Fret `test_id`s, diagnostics
      script, and packet checks.
- [ ] SCPM-050: Add a stricter depth model for states that are not visible in the current binary
      axes: disabled, hover, focus-visible, pressed, open, keyboard, mobile, RTL, text metrics, and
      paint/token output.
