---
title: Diagnostics Extensibility + Capabilities v1 (TODO)
status: draft
date: 2026-02-10
scope: diagnostics, automation, protocol, ecosystem, portability
---

# Diagnostics Extensibility + Capabilities v1 (TODO)

This file tracks tasks for `docs/workstreams/diag-extensibility-and-capabilities-v1.md`.

Guiding idea: keep `diag` useful for day-to-day debugging *and* safe to depend on as an ecosystem contract surface.

## Foundations (contract hygiene)

- [x] Write an extensibility-focused ADR:
  - [x] `docs/adr/0204-ui-diagnostics-extensibility-and-capabilities-v1.md`
- [x] Split the workstream into small sub-documents (capabilities/evidence/script tooling/text+IME/determinism).
- [x] Enforce “single source of truth” for script protocol types:
  - [x] runner MUST not fork protocol structs/enums (parse/execute against `crates/fret-diag-protocol`).
  - [x] add an explicit note in docs about this rule (prevents silent drift).
  - [ ] delete the legacy forked-protocol block (currently under `#[cfg(any())]`) once confident.

## Capabilities (vocabulary, discovery, gating)

Doc: `docs/workstreams/diag-extensibility-and-capabilities-v1/capabilities.md`

- [x] Decide capability namespaces and initial stable vocabulary:
  - [x] `devtools.*` vs `diag.*` naming,
  - [x] minimum `diag.*` list for v1.
- [x] Script metadata surface:
  - [x] define `meta` object shape (name/tags/required_capabilities/target_hints),
  - [x] rule: tooling ignores unknown `meta` keys.
- [ ] Filesystem discovery:
  - [x] runner writes deterministic `capabilities.json` under `FRET_DIAG_DIR`.
- [ ] DevTools WS discovery:
  - [x] runner/session descriptors advertise `diag.*` capabilities.
- [ ] Tooling gating:
  - [x] fail fast when required capabilities are missing,
  - [x] emit evidence file `check.capabilities.json` (machine-readable),
  - [x] `diag repro` includes gating failures in `repro.summary.json`.

## Evidence & trace (debuggability surfaces)

Doc: `docs/workstreams/diag-extensibility-and-capabilities-v1/evidence-and-trace.md`

- [ ] Define a stable reason-code taxonomy for script failures (“why did this fail?”).
- [ ] Add bounded trace evidence (ring buffer) dumped on failure:
  - [ ] selector resolution evidence,
  - [ ] input routing evidence (hit-test, capture/barriers, focus changes),
  - [ ] predicate evaluation deltas.
- [ ] Add `diag lint` mode for captured bundles:
  - [ ] semantics lint (duplicate `test_id`, missing labels, inconsistent flags),
  - [ ] layout lint (bounds outside window, overlap invariants, basic stability checks),
  - [ ] emit `check.lint.json`.

## Tooling tasks (authoring ergonomics)

- [x] Add typed Rust helpers for building Script v2:
  - [x] `crates/fret-diag-protocol/src/builder.rs`
- [x] Add an internal script generator tool that emits JSON from typed templates:
  - [x] `apps/fret-diag-scriptgen`

Doc: `docs/workstreams/diag-extensibility-and-capabilities-v1/script-tooling.md`

- [ ] Add `diag script normalize` (pretty-print, stable diffs).
- [ ] Add `diag script validate` (schema/parse, clear error paths, `check.script_schema.json`).
- [ ] Add `diag script lint` (capability inference, discouraged patterns, `check.script_lint.json`).
- [ ] Add CI-friendly “generate + check” workflow:
  - [ ] ensure generated scripts match checked-in scripts (when applicable),
  - [ ] prefer `.fret/diag/scripts` for local generation (avoid accidental churn in `tools/diag-scripts/`).
- [ ] Add `diag script shrink` (delta debugging) to minimize flaky/large repros:
  - [ ] emit `repro.min.json` + summary.

## Text & IME (self-drawn UI pain point)

Doc: `docs/workstreams/diag-extensibility-and-capabilities-v1/text-and-ime.md`

- [ ] Define the minimum redaction-friendly evidence surface for focused text inputs:
  - [ ] selection range (UTF-16),
  - [ ] caret rect,
  - [ ] IME composition state summary.
- [ ] Add at least one stable script gate for:
  - [ ] word boundary (double click),
  - [ ] line boundary (triple click),
  - [ ] “composition not stolen by shortcuts” (requires trace + reason codes).

## Determinism (flake triage)

Doc: `docs/workstreams/diag-extensibility-and-capabilities-v1/determinism.md`

- [ ] Define and capture an environment fingerprint in bundles (versions, DPI, font fallback summary, flags).
- [ ] Add a repeat-run triage workflow:
  - [ ] run a script N times,
  - [ ] classify diffs (semantics/layout/routing/perf),
  - [ ] emit `repeat.summary.json`.

## Scenario coverage (future-proofing)

- [ ] Multi-window:
  - [ ] one deterministic “open window B, assert focus/barrier invariants” smoke.
- [ ] Embedded viewport/canvas:
  - [ ] one demo that projects objects into semantics (`test_id`) for stable automation.
  - [ ] one fallback demo using anchored normalized coordinates (capability-gated).
- [ ] Mobile alignment (future):
  - [ ] define touch pointer kind surface and basic gestures in protocol (capability-gated).

## CI tasks (guardrails)

- [ ] Add a small smoke suite for CI that:
  - [ ] avoids pixel assertions by default,
  - [ ] uses only `test_id`/role selectors,
  - [ ] runs with predictable timeouts.
- [ ] Add checks that protect contract evolution:
  - [ ] script schema validation for the smoke suite,
  - [ ] normalization check (avoid “random diff churn”).
