---
title: Diagnostics Extensibility + Capabilities v1
status: draft
date: 2026-02-10
scope: diagnostics, automation, protocol, ecosystem, portability
---

# Diagnostics Extensibility + Capabilities v1

This workstream defines a forward-compatible “shape” for Fret diagnostics and scripted UI automation that:

- scales from in-repo debugging to ecosystem authors (external apps/components),
- supports multiple target surfaces over time (native, web runner, multi-window, embedded viewports/canvas, future mobile),
- evolves without turning missing support into “hang until timeout”.

This is intentionally contract-first and layered:

- `crates/fret-ui`: mechanism hooks only (no policy).
- `ecosystem/fret-bootstrap`: diagnostics service + script execution.
- Tooling (`apps/fretboard`, DevTools GUI, MCP): orchestration, packaging, gates, and UX.

Related:

- Living usage doc: `docs/ui-diagnostics-and-scripted-tests.md`
- Debug workflows: `docs/debugging-ui-with-inspector-and-scripts.md`, `docs/debugging-playbook.md`
- Workstream foundations: `docs/workstreams/ui-automation-and-debug-recipes-v1.md`, `docs/workstreams/diag-devtools-gui-v1.md`
- Contract ADRs: `docs/adr/0174-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`,
  `docs/adr/0196-ui-automation-and-debug-recipes-v1.md`,
  `docs/adr/0204-ui-diagnostics-extensibility-and-capabilities-v1.md`

## Why this workstream

The current `diag` workflow is already powerful (bundles + scripts + gates + packing). The scaling risk is not
“missing features”, it is uncontrolled evolution:

- Scripts depending on coordinates become fragile across DPI/window sizes and layout refactors.
- Optional features (screenshots, multi-window targeting, touch gestures) can silently turn into timeouts when unsupported.
- Ecosystem authors need a stable contract they can depend on without importing half the repo.

This workstream locks the extensibility rules so we can refactor fearlessly.

## Principles (borrowed from successful ecosystems)

1. **Portable artifact first**: the script JSON and the `bundle.json` are the reviewable, shareable units.
2. **Semantics-first selection**: prefer `test_id`, then role/name/path; coordinates are an explicit fallback mode.
3. **Intent-level actions**: scripts should read like “user intent” (`menu_select_path`, `scroll_into_view`, `click_stable`).
4. **Deterministic waits**: prefer `wait_until` predicates and action timeouts; avoid wall-clock sleeps.
5. **Capabilities are explicit**: missing support must fail fast with a structured reason (not as a timeout).
6. **Layering stays clean**: no “test-only policy” leaks into `fret-ui`.

## Contract surface: what ecosystem authors should rely on

### 1) Script JSON as the stable interface

Ecosystem authors should be able to:

- add `test_id` (or semantics labels) in their UI,
- write scripts as JSON (or generate JSON via typed helpers),
- run them through `fretboard diag run` (native) or the devtools WS export path (web runner),
- ship repro zips (`diag repro` / `diag pack`) to maintainers.

Typed helpers (Rust builders, script generators) are allowed, but MUST compile down to the same JSON schema.

### 2) Capability negotiation

The script runner/tooling must make “what is supported” explicit. Examples:

- `screenshot_png` (on-demand PNG screenshots used by `capture_screenshot`)
- `multi_window` (window targeting and cross-window assertions)
- `pointer_kind_touch` / `gesture_pinch` (future mobile-style input)

Tooling should:

- refuse to execute scripts that require missing capabilities,
- emit a machine-readable reason (for CI and AI triage).

### 3) Window targeting (future-proofing)

Multi-window is a first-class Fret goal; automation needs a stable way to:

- choose a window deterministically,
- scope selector resolution and actions to that window,
- assert per-window focus/barrier/capture invariants.

Key requirement: do not make “window index” the only selector; prefer stable identifiers where possible.

### 4) Embedded viewports / canvas

For canvas-like surfaces:

- preferred: project interactive objects into the semantics tree (with `test_id`) so scripts can stay selector-driven.
- fallback: coordinate injection must be expressed relative to a selector anchor (e.g. element-local normalized 0..1),
  not raw screen pixels.

### 5) Evolution rules

- Additive extensions (optional fields, new capabilities) must not break existing scripts.
- Breaking changes must increment `schema_version`.
- Tooling should provide “normalize/pretty-print” to keep diffs stable and reviewable.

## Deliverables

This workstream ships three “stable outcomes”:

1. A clear ADR-backed contract (`docs/adr/0204-...`) that defines extension points and versioning rules.
2. A small set of ergonomic authoring tools (typed builders + generators) that still emit JSON.
3. CI-friendly gates for contract health (schema validation, capability checks, and minimal smoke suites).

See:

- TODO tracker: `docs/workstreams/diag-extensibility-and-capabilities-v1-todo.md`
- Milestones: `docs/workstreams/diag-extensibility-and-capabilities-v1-milestones.md`

