---
title: Diagnostics Extensibility + Capabilities v1 - Evidence & Trace
status: draft
date: 2026-02-10
scope: diagnostics, automation, evidence, trace, triage
---

# Diagnostics Extensibility + Capabilities v1 - Evidence & Trace

This document is a sub-part of `docs/workstreams/diag-extensibility-and-capabilities-v1/README.md`.

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
- `hit_node_path` (debug-only root→leaf path for `hit_node_id`; treat as in-run references only)
- `hit_semantics_node_id` / `hit_semantics_test_id` (best-effort semantics node observed at that position)
- `includes_intended` (best-effort: whether the hit semantics matches the intended target)
- `hit_path_contains_intended` (best-effort: whether the hit-test path contains the intended node id)
- `barrier_root` / `focus_barrier_root` (in-run references)
- `pointer_occlusion` / `pointer_occlusion_layer_id` (input arbitration snapshot; explains occlusion)
- `pointer_capture_active` / `pointer_capture_layer_id` / `pointer_capture_multiple_layers` (input arbitration snapshot; explains capture)
- `blocking_reason` / `blocking_root` / `blocking_layer_id` (best-effort attribution for fast triage; prefer raw fields when debugging novel cases)
  - when available, `blocking_root` points to the responsible `layer_root` (in-run reference only)
- `scope_roots` (layer roots + pointer occlusion hints; bounded, intended to explain “why input did not reach underlay”)
- `note` (action kind / phase, e.g. `click`, `drag_pointer.start`, `scroll_into_view.wheel`)

Click-stable trace entry fields (v2 `click_stable` only):

- `step_index`
- `stable_required` / `stable_count` / `moved_px` / `max_move_px` / `remaining_frames`
- `hit_test` (a full `UiHitTestTraceEntryV1` captured at the would-be click point)

Focus trace entry fields:

- `step_index`
- `note` (phase marker, e.g. `type_text_into.wait_focus`)
- `reason_code` (best-effort inference, e.g. `focus.blocked_by_focus_barrier`)
- `text_input_snapshot` (best-effort, redactable, from `WindowTextInputSnapshot`):
  - `focus_is_text_input` / `is_composing`
  - `text_len_utf16`
  - `selection_utf16` / `marked_utf16`
  - `ime_cursor_area` (best-effort candidate/caret placement rectangle)
- `expected_node_id` / `expected_test_id` (when waiting for a specific focus target)
- `modal_barrier_root` / `focus_barrier_root` + pointer capture/occlusion hints (from the UI input arbitration snapshot)
- `focused_element` / `focused_element_path` (element-runtime view)
- `focused_node_id` / `focused_test_id` / `focused_role` (best-effort semantics view)
- `matches_expected` (best-effort)

Shortcut routing trace entry fields (native + web, runner-driven, no raw text):

- `step_index`
- `frame_id`
- `phase` (`pre_dispatch`, `post_dispatch`)
- `deferred` (whether shortcut matching was deferred until after widget dispatch, ADR 0012)
- `focus_is_text_input` / `ime_composing`
- `key` / `modifiers` / `repeat`
- `outcome` (`reserved_for_ime`, `consumed_by_widget`, `command_dispatched`, `command_disabled`, ...)
- `command` / `command_enabled` (when applicable)
- `pending_sequence_len` (multi-keystroke shortcuts)

Overlay placement trace entry fields (runner + component-layer hooks; structured geometry evidence):

- `kind`: `anchored_panel` (popper-like) or `placed_rect` (higher-level placement policies)
- `step_index` / `note` / `frame_id`
- `overlay_root_name` (best-effort stable overlay identity, when known)
- `anchor_element` / `anchor_test_id` (best-effort)
- `content_element` / `content_test_id` (best-effort)

For `kind=anchored_panel` (placement solver trace):

- `outer_input` (pre-collision outer bounds)
- `outer_collision` + `collision_padding` / `collision_boundary` (effective boundary)
- `anchor` / `desired`
- `preferred_side` / `align` / `direction` / `sticky` / `shift` / `offset` / `gap_px`
- `preferred_rect` / `flipped_rect` + fit booleans + available space hints
- `chosen_side` / `chosen_rect` / `rect_after_shift` / `shift_delta` / `final_rect`
- `arrow` (optional; side + offset + center offset)

For `kind=placed_rect` (non-solver placement):

- `outer` / `anchor` / `placed` (+ optional `side`)

Web IME trace entry fields (wasm only, ADR 0180; debug-only, redactable):

- `step_index`
- `note` (phase marker)
- `enabled` / `composing` / `suppress_next_input`
- `textarea_has_focus` / `active_element_tag`
- `position_mode` / `mount_kind` / `device_pixel_ratio`
- `textarea_selection_start_utf16` / `textarea_selection_end_utf16`
- `last_cursor_area` / `last_cursor_anchor_px`
- `last_input_type`
- `last_preedit_len` / `last_preedit_cursor_utf16` / `last_commit_len` (no raw text by default)
- `beforeinput_seen` / `input_seen` / `suppressed_input_seen` / `composition_*_seen` / `cursor_area_set_seen`

IME event trace entry fields (native + web, runner-driven, no raw text):

- `step_index`
- `note`
- `kind` (`enabled`, `disabled`, `preedit`, `commit`, `delete_surrounding`)
- `preedit_len` / `preedit_cursor` (cursor range in UTF-8 bytes within the preedit string)
- `commit_len`
- `delete_surrounding` (before_bytes, after_bytes)

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
