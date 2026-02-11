---
title: Diagnostics Extensibility + Capabilities v1 (TODO)
status: draft
date: 2026-02-10
scope: diagnostics, automation, protocol, ecosystem, portability
---

# Diagnostics Extensibility + Capabilities v1 (TODO)

This file tracks tasks for `docs/workstreams/diag-extensibility-and-capabilities-v1/README.md`.

Guiding idea: keep `diag` useful for day-to-day debugging *and* safe to depend on as an ecosystem contract surface.

## Foundations (contract hygiene)

- [x] Write an extensibility-focused ADR:
  - [x] `docs/adr/0189-ui-diagnostics-extensibility-and-capabilities-v1.md`
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
- [x] Filesystem discovery:
  - [x] runner writes deterministic `capabilities.json` under `FRET_DIAG_DIR`.
- [x] DevTools WS discovery:
  - [x] runner/session descriptors advertise `diag.*` capabilities.
- [ ] Tooling gating:
  - [x] fail fast when required capabilities are missing,
  - [x] emit evidence file `check.capabilities.json` (machine-readable),
  - [x] `diag repro` includes gating failures in `repro.summary.json`.

## Evidence & trace (debuggability surfaces)

Doc: `docs/workstreams/diag-extensibility-and-capabilities-v1/evidence-and-trace.md`

- [x] Add a stable `reason_code` surface to `script.result.json` (`UiScriptResultV1.reason_code`).
- [x] Add selector resolution evidence to `script.result.json` (`evidence.selector_resolution_trace`).
- [x] Add hit-test / routing evidence to `script.result.json` (`evidence.hit_test_trace`).
- [ ] Expand the reason-code taxonomy as new evidence surfaces land (avoid premature over-taxonomy).
- [ ] Add bounded trace evidence (ring buffer) dumped on failure:
  - [x] hit-test + routing evidence (capture/barriers/occlusion) with deeper explainability:
    - hit node path (`hit_node_path`) for “hit the right region but got blocked” cases,
    - best-effort attribution (`blocking_reason` / `blocking_root` / `blocking_layer_id`),
    - best-effort explanation string (`routing_explain`),
    - best-effort capture + occlusion owners (semantics node + bounds),
    - best-effort capture owner element-path (`pointer_capture_element_path`),
  - [x] overlay placement evidence in `script.result.json` (`evidence.overlay_placement_trace`),
  - [x] focus + IME evidence snapshots in `script.result.json` (`evidence.focus_trace`, `evidence.web_ime_trace`),
  - [x] IME event summaries in `script.result.json` (`evidence.ime_event_trace`),
  - [x] shortcut routing evidence in `script.result.json` (`evidence.shortcut_routing_trace`),
  - [x] text input snapshot evidence in `script.result.json` (via `evidence.focus_trace[].text_input_snapshot`),
  - [ ] focus change evidence with reasons beyond barrier inference,
  - [ ] predicate evaluation deltas (what changed, what did not).
- [x] Add `diag lint` mode for captured bundles:
  - [x] semantics lint (duplicate `test_id`, missing labels, dangling `active_descendant`),
  - [x] layout lint (focused/active bounds outside window, empty bounds for focused/test-id nodes),
  - [x] emit `check.lint.json` (exit 1 when error findings exist).

## Tooling tasks (authoring ergonomics)

- [x] Add typed Rust helpers for building Script v2:
  - [x] `crates/fret-diag-protocol/src/builder.rs`
- [x] Add an internal script generator tool that emits JSON from typed templates:
  - [x] `apps/fret-diag-scriptgen`

Doc: `docs/workstreams/diag-extensibility-and-capabilities-v1/script-tooling.md`

- [x] Add `diag script normalize` (pretty-print, stable diffs).
- [x] Add `diag script validate` (schema/parse, clear error paths, `check.script_schema.json`).
- [x] Add `diag script lint` (capability inference, discouraged patterns, `check.script_lint.json`).
- [ ] Add CI-friendly “generate + check” workflow:
  - [ ] ensure generated scripts match checked-in scripts (when applicable),
  - [ ] prefer `.fret/diag/scripts` for local generation (avoid accidental churn in `tools/diag-scripts/`).
- [ ] Add `diag script shrink` (delta debugging) to minimize flaky/large repros:
  - [ ] emit `repro.min.json` + summary.

## Text & IME (self-drawn UI pain point)

Doc: `docs/workstreams/diag-extensibility-and-capabilities-v1/text-and-ime.md`

- [x] Define the minimum redaction-friendly evidence surface for focused text inputs:
  - [x] selection range (UTF-16),
  - [x] caret/candidate rect (best-effort `ime_cursor_area`),
  - [x] IME composition state summary (`is_composing` + `marked_utf16`).
- [ ] Add at least one stable script gate for:
  - [x] word boundary (double click),
  - [x] line boundary (triple click),
  - [ ] “composition not stolen by shortcuts” (requires trace + reason codes).

## Determinism (flake triage)

Doc: `docs/workstreams/diag-extensibility-and-capabilities-v1/determinism.md`

- [x] Define and capture an environment fingerprint in bundles (runner kind, target triple, flags, scale factors, capabilities).
- [x] Add a repeat-run triage workflow:
  - [x] run a script N times,
  - [x] classify diffs (semantics/layout/perf; routing classification TBD),
  - [x] emit `repeat.summary.json`.

## Component conformance (self-drawn UI pain points)

Doc: `docs/workstreams/diag-extensibility-and-capabilities-v1/component-conformance.md`

- [x] Add a component conformance playbook document (invariants-first, evidence-first).
- [ ] Turn the shadcn `Select` suite into a reference-quality example:
  - [ ] ensure stable `test_id` for trigger/content/items (value-stable, not index-stable),
  - [ ] cover: open/close, selection commit, disabled option, roving/typeahead, outside-press/Escape dismiss,
    focus restore, wheel scroll stability, collision/placement sanity.
- [ ] Make suites run `diag lint` automatically (and fail on error-level findings):
  - [x] `fretboard diag suite` runs bundle lint by default (use `--no-lint` to disable).

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
