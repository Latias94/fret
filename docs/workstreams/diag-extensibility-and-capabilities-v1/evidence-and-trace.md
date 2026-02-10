---
title: Diagnostics Extensibility + Capabilities v1 - Evidence & Trace
status: draft
date: 2026-02-10
scope: diagnostics, automation, evidence, trace, triage
---

# Diagnostics Extensibility + Capabilities v1 - Evidence & Trace

This document is a sub-part of `docs/workstreams/diag-extensibility-and-capabilities-v1.md`.

Thesis: screenshots and logs are helpful, but self-drawn UI debugging only scales when failures produce
**structured evidence** with **explainable reasons**.

## Artifacts (portable evidence units)

Required:

- `bundle.json`: the primary portable artifact (semantics + layout + frame stats + debug surfaces).
- `script.json`: the portable repro recipe.
- `script.result.json`: the structured outcome (stage, step index, reason, last bundle dir).

Optional but recommended:

- `triage.json`: a machine-readable summary derived from a bundle (small, AI-friendly).
- screenshots (bundle-scoped or on-demand): useful as supporting evidence, not the primary truth.
- `check.*.json`: CI/automation evidence files (capabilities gating, schema validation, lint results).

## Structured failure reasons (reason codes)

Define a stable taxonomy of reason codes for script failures.

Design constraints:

- reason codes MUST be stable across refactors,
- reason codes MUST be specific enough to triage without reading source,
- reason codes MUST be machine-readable.

Reason codes are stable strings (prefer dotted namespaces). Start small and expand only when the
new code is clearly more actionable than existing evidence surfaces.

Initial v1 codes (implemented):

- `semantics.missing` (runner has no semantics snapshot for the step)
- `selector.not_found` (selector resolution produced zero matches)
- `timeout` (the step timed out waiting for a condition)
- `assert.failed` (an explicit `assert` predicate failed)

Planned namespaces (future expansion):

- `capability.*`
- `selector.*`
- `routing.*` (hit-test / capture / barrier / occlusion)
- `focus.*`
- `text_ime.*`

This workstream treats “why did it fail?” as a first-class contract surface.

## Script result evidence surface (`script.result.json`)

`script.result.json` is the machine-readable outcome of a run.

Current additions:

- `UiScriptResultV1.reason_code` (optional): stable reason code string.
- `UiScriptResultV1.evidence.selector_resolution_trace` (optional, bounded): a compact per-step
  explanation of selector resolution.

Selector resolution trace entry fields:

- `step_index`
- `selector` (the input selector)
- `match_count` (number of matching nodes)
- `chosen_node_id` (the runner-chosen node id, if any)
- `candidates` (top-N ranked candidates with `role` / optional `name` / optional `test_id`)
- `note` (optional hint, e.g. `invalid_role`, `fallback_hidden_nodes`)

Redaction: when `redact_text` is enabled, candidate `name` is omitted.

Hit-test / routing trace entry fields:

- `step_index`
- `selector` (the selector associated with the injected action)
- `position` (the injected pointer position)
- `hit_node_id` (the immediate hit-test result, if any; not stable across runs)
- `hit_semantics_node_id` / `hit_semantics_test_id` (best-effort semantics node observed at that position)
- `includes_intended` (best-effort: whether the hit semantics matches the intended target)
- `barrier_root` / `focus_barrier_root` (in-run references)
- `scope_roots` (layer roots + pointer occlusion hints; bounded, intended to explain “why input did not reach underlay”)
- `note` (action kind / phase, e.g. `click`, `drag_pointer.start`, `scroll_into_view.wheel`)

## Trace surface (ring buffer, dumped on failure)

When a script fails (or when tooling requests it), the runner SHOULD emit a trace slice for the last K frames:

- step start/end markers (step index, step kind, window scope),
- selector resolution (how many candidates, why rejected),
- hit-test chain and routing decisions (capture/barrier/occlusion),
- focus change events with reasons,
- predicate evaluation deltas (what changed, what did not).

Properties:

- bounded size (ring buffer),
- redactable (no raw text unless explicitly enabled),
- actionable without screenshots.

Recommended output: `trace.json` (or embedded under bundle debug payload).

## “Lint” as a diagnostics mode (no interaction required)

Add a script-independent lint mode that runs on a captured `bundle.json`:

Semantics lint examples:

- duplicate `test_id`,
- focusable without label,
- disabled nodes exposing actions,
- inconsistent role/name across frames for the same id.

Layout lint examples:

- bounds outside window,
- non-overlapping invariants violated for known layout structures,
- text bounds overlap or baseline anomalies (when evidence exists).

Output: `check.lint.json` (machine-readable), plus a human-readable summary.

## ROI screenshots (optional, but high leverage)

When screenshots are needed, prefer “ROI-by-selector” (cropped to a node bounds and stabilized by `test_id`)
over full-frame diffs. This reduces flake and makes diffs reviewable.

This should be capability-gated (`diag.screenshot_png`) and never required for basic correctness gates.
