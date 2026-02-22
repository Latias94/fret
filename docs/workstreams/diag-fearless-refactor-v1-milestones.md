---
title: Diag Fearless Refactor v1 (Milestones)
status: draft
date: 2026-02-21
scope: diagnostics, automation, bundle-schema, refactor
---

# Diag Fearless Refactor v1 (Milestones)

This file tracks milestones for `docs/workstreams/diag-fearless-refactor-v1.md`.

Conventions:

- keep changes additive and compatibility-first (fearless refactor prerequisite),
- prefer bounded artifacts (index/slice/packet) over grepping full `bundle.json`,
- keep a fast gate green after each milestone (recommended: `cargo check -p fret-ui-gallery`).

## Milestones

### M0: First extraction (filesystem triggers)

- [x] Split filesystem-trigger concerns out of `ui_diagnostics.rs`.

Definition of done:

- `UiDiagnosticsService` can still run with filesystem triggers enabled.
- The parent module can call the extracted methods without widening public API beyond `pub(super)`.

### M1: Bundle export modularization

- [x] Extract bundle schema selection + JSON writing into a cohesive module (`bundle_dump.rs`).
- [x] Make schema v2 + semantics-mode defaults explicit and documented.

Definition of done:

- Bundle export logic no longer spans the core `UiDiagnosticsService` file.
- Manual dumps and script dumps produce the same artifact layout with only schema/mode differences.

### M2: Script runner modularization

- [ ] Extract the scripted interaction runner (v1/v2 parsing, step loop, evidence capture) into its own module.
  - [x] Extract runner helpers (`script_runner.rs`).
  - [x] Extract step handlers into dedicated modules (pointer sessions, drag playback, menu, scroll, visibility, etc.).

Definition of done:

- The runner can be reasoned about and tested without navigating the entire diagnostics service file.

### M3: Inspect/pick modularization

- [ ] Extract inspect/pick trigger polling + state into modules.
- [ ] Keep transport concerns isolated (filesystem vs DevTools WS).

Definition of done:

- Inspect/pick code paths are independently editable without touching bundle/schema code.

### M4: Plan 1 closure for AI loops

- [ ] Ensure “AI packet” is the default shareable artifact path for triage.
- [ ] Ensure sidecars (`bundle.meta.json`, `bundle.index.json`, `test_ids.index.json`) are consistently available
  in pack/repro flows.
  - [x] Runtime writes canonical sidecars on native dumps.
  - [ ] Ensure `diag pack --include-all` always includes sidecars under `_root/` (even when the bundle dir is relocated).

Definition of done:

- A typical scripted failure can be debugged from an AI packet without opening the full `bundle.json`.

### M5: Plan 2 prototype (deferred)

- [ ] Prototype a manifest-first chunked bundle layout + compatibility materializer.

Definition of done:

- At least one workflow produces a manifest bundle and can materialize `bundle.json` on demand.
