---
title: Fret Mechanism Harness v1 Design
status: active
date: 2026-05-11
---

# Design

This lane is the execution track for the existing `docs/mechanism-harness-v2.md` architecture. The
name is v1 because this is the first end-to-end self-drawn UI mechanism workstream; the harness
crate and architecture note are already on their v2 artifact vocabulary.

## Scope

Build a mechanism-first defect loop for self-drawn UI:

- synthetic fixtures for controlled mechanism states;
- reusable observed facts and predicates in `crates/fret-mechanism-harness`;
- `crates/fret-ui` runners that adapt real runtime state into the harness observation model;
- UI Gallery diagnostics gates for the highest-risk real app paths.

## Owns

- Coverage map for layout primitives, layout dirty invalidation, hit-test routing, overlays, focus,
  and semantics.
- Fixture-driven mechanism suites where the runner is shared and cases differ by data.
- Mechanism predicates that describe runtime facts without importing component policy.
- Evidence notes that classify each finding as mechanism, recipe policy, app demo, diagnostics, or
  reference-data drift.

## Does Not Own

- shadcn or Material recipe policy decisions such as padding, row height, dismissal policy, hover
  intent, or focus restoration.
- Diagnostics launch, screenshot, sidecar, or bundle plumbing.
- A second harness crate.
- Broad component library parity sweeps. Those stay in recipe workstreams and consume this harness
  only when the issue is mechanism-shaped.

## Layer Split

- `crates/fret-mechanism-harness`: fixture schema, observation model, oracle predicates, thin
  runner.
- `crates/fret-ui`: controlled mechanism scenarios and adapters from `UiTree` state into observed
  facts.
- `ecosystem/*`: recipe-level scenarios that prove component composition satisfies mechanism
  oracles.
- `tools/diag-scripts`: runtime evidence for real UI Gallery paths.

## First Slice

The first slice starts with layout and invalidation because they are the source of many self-drawn
UI parity failures:

- layout primitive fixture coverage already exists for sizing, stretch, transparent wrappers, grid,
  and transform spaces;
- layout dirty invalidation now needs scalar mechanism facts because dirty counts and underflow
  repair are not bounds;
- the checkbox "Enable notifications" UI Gallery script is the runtime proof that a mechanism bug can
  surface as a component-looking parity issue.

