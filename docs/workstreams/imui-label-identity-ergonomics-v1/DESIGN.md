# ImUi Label Identity Ergonomics v1

Status: closed
Last updated: 2026-04-28

## Why This Lane Exists

Dear ImGui makes identity cheap at the callsite:

- `Label##suffix` renders `Label` while distinguishing the item from another visible `Label`.
- `Label###stable_id` renders `Label` while using `stable_id` as the identity.
- `##hidden_id` renders no label while still providing a usable identity.

Fret IMUI already has explicit `ui.id(...)`, `ui.push_id(...)`, and `for_each_keyed(...)`, but the
label-bearing control helpers do not yet understand this label grammar. That is a real ergonomics
gap for IMUI parity, and the old historical note should not be revived directly.

## Scope

In scope:

- a small label parser in `ecosystem/fret-ui-kit::imui`
- label-bearing IMUI controls that currently render the raw label
- authoring tests in `ecosystem/fret-imui`
- local unit gates proving both display-label behavior and identity behavior

Out of scope:

- changing `crates/fret-ui` runtime identity contracts
- changing `test_id` semantics
- changing accessibility labels unless the caller opts into the label grammar path
- localization policy
- docking, multi-window, or OS-window identity

## Target Semantics

- No marker: visible label is the full string; identity key is the full string.
- `Label##suffix`: visible label is `Label`; identity key is the full string.
- `##suffix`: visible label is empty; identity key is the full string.
- `Label###stable_id`: visible label is `Label`; identity key is `stable_id`.
- `###stable_id`: visible label is empty; identity key is `stable_id`.

`###` takes identity precedence over `##`, matching Dear ImGui's stable-ID use case. Display text is
still clipped at the first `##` marker.

## Layer Decision

This belongs in `ecosystem/fret-ui-kit::imui`, not `crates/fret-ui`.

The runtime already has keyed subtrees. The missing piece is policy-layer string grammar and
consistent use by immediate-mode controls.

## Starting Assumptions

- Area: existing identity mechanism
  - Assumption: `ImUi::id`, `ImUi::push_id`, and `for_each_keyed` are enough runtime mechanism for
    this lane.
  - Evidence: `ecosystem/fret-imui/src/frontend.rs`.
  - Confidence: Confident.
  - Consequence if wrong: promote a focused runtime identity gap into an ADR-backed lane.

- Area: upstream parity
  - Assumption: the first useful parity target is Dear ImGui's label grammar, not its full ID-stack
    debug tooling.
  - Evidence: `repo-ref/imgui/imgui.h`, `repo-ref/imgui/imgui.cpp`,
    `repo-ref/imgui/imgui_widgets.cpp`.
  - Confidence: Confident.
  - Consequence if wrong: this lane may under-serve debugging, but still improves core authoring.

- Area: accessibility and test IDs
  - Assumption: `test_id` remains explicit and should not be inferred from label identity.
  - Evidence: Fret's current diagnostics and semantics tests rely on explicit test IDs.
  - Confidence: Likely.
  - Consequence if wrong: a separate diagnostics/test-id policy lane should decide that mapping.

## Exit Criteria

- The parser has direct unit coverage for no marker, `##`, hidden-label, and `###` cases.
- At least one stateful control proof shows `###stable_id` keeps identity stable across visible
  label changes.
- Label-bearing controls in the admitted first set no longer render `##` / `###` suffixes.
- No `fret-ui` runtime contract is widened without a new ADR/alignment update.
