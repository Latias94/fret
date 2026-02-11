---
title: Diagnostics Extensibility + Capabilities v1 - Component Conformance
status: draft
date: 2026-02-11
scope: diagnostics, scripted-tests, conformance, components, self-drawn-ui
---

# Diagnostics Extensibility + Capabilities v1 - Component Conformance

This document is a sub-part of `docs/workstreams/diag-extensibility-and-capabilities-v1/README.md`.

Self-drawn UI frameworks often regress in ways that screenshots and logs alone do not explain:

- text editing + IME composition (caret/selection/preedit lifecycle),
- input routing (hit-test, capture, focus barriers, occlusion),
- overlay behavior (placement, collision, focus trap/restore),
- layout correctness (overflow, clipping, bounds outside the window),
- list virtualization (items “exist” visually but not in the semantics tree yet),
- “uncanny valley” interaction feel (wheel/pointer latency, damping, jitter).

The diagnostics contract should enable **end-to-end component tests** that are:

- *scriptable* (no human timing),
- *explainable* (structured reasons + traces),
- *gateable* (CI-friendly checks),
- *portable* (bundle + script zip is the unit of review).

## The goal: invariants, not “track every internal state”

For complex widgets (e.g. shadcn `Select`), the goal is not to expose or snapshot every internal state.
Instead, define a small set of **stable invariants** per feature, and make failures self-diagnosing.

Examples of “good invariants”:

- open/close lifecycle: trigger opens, Escape/outside-press dismisses,
- focus behavior: close restores focus predictably,
- selection outcome: commit updates the trigger label/value; disabled items do not commit,
- roving/typeahead: active item updates as expected (and skips disabled items),
- placement sanity: content bounds stay within window bounds; chosen side/align is explainable under collisions,
- routing correctness: injected clicks/keys either land or emit a trace explaining barriers/capture/occlusion.

## Recommended testing layers (stack, not one tool)

1. **Policy / state machine tests (unit-level)**
   - test the interaction policy (dismiss, roving, typeahead, hover intent, timers),
   - use deterministic time/frames when possible.
2. **Scripted end-to-end tests (`fretboard diag`)**
   - validate routing + focus + overlay placement + virtualization,
   - assert against semantics and structured evidence, not pixels by default.
3. **Visual / feel probes (optional, targeted)**
   - use screenshots and pixel diffs to catch rendering regressions,
   - use perf gates for wheel/resize/pointer-move “feel” regressions.

The diagnostic design is primarily in service of layer (2), and should *feed* layers (1) and (3) with evidence.

## Practical playbook for a component suite (example: `Select`)

### A) Put stable targets in semantics

Add stable `test_id` on:

- trigger,
- overlay content root,
- each item (stable by value, not by index),
- optional: scroll viewport, search/typeahead input (if any).

Avoid duplicate `test_id` across the window; treat duplicates as error-level findings (`diag lint`).

### B) Write scripts that read like user intent

Prefer:

- `click` / `click_stable` over coordinate injection,
- `wait_until` predicates over wall-clock sleeps,
- step-local `capture_bundle` at “interesting points” (to make failures explainable without reruns).

Declare `meta.required_capabilities` for non-trivial evidence requirements (screenshots, multi-window, etc.).

### C) Gate with evidence-first checks

Run (native):

- `cargo run -p fretboard -- diag run <script.json> --launch -- <cmd...>`

Then:

- `fretboard diag lint <bundle_dir|bundle.json>` (sanity checks, emits `check.lint.json`)
- `fretboard diag triage <bundle_dir|bundle.json> --json` (small summary)

Use `diag compare` for before/after regressions when the semantics contract is expected to stay compatible.

### D) Expand coverage with matrices without making tests unreadable

When you need a placement/collision matrix (many cases):

- prefer fixture-driven generation (JSON fixtures + thin harness) over dozens of hand-authored scripts,
- keep scripts normalized and reviewable (`diag script normalize` once implemented).

## Layout and virtualization debugging tips (diag-first)

- Prefer semantics bounds checks (in-window, non-empty) over pixel diffs.
- When virtualization is involved, add `wait_until` that asserts the target item exists in semantics before clicking.
- Use `click_stable` for jittery overlays and scrolling targets.
- Use `diag lint --all-test-ids` when you want out-of-window hints for all targeted nodes, not only focused ones.

## References

- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/workstreams/diag-extensibility-and-capabilities-v1/text-and-ime.md`
- `tools/diag-scripts/ui-gallery-select-*.json`
