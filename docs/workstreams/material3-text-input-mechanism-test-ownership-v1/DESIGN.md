# Material3 TextInput Mechanism Test Ownership v1 Design

Status: Closed
Last updated: 2026-05-31

## Problem

After the Material3 interaction-regression family splits, the only remaining test in
`material3_interaction_regressions.rs` was `text_input_text_input_event_updates_model`. That test
used `fret_ui::element::TextInputProps` and `Event::TextInput` directly; no Material3 component,
token, or recipe participated. Keeping it in `fret-ui-material3` blurred the mechanism/policy
boundary.

## Decision

Move the positive editable TextInput event coverage to the `fret-ui` mechanism test suite by adding
`text_input_text_input_event_updates_model` to
`crates/fret-ui/src/declarative/tests/interactions/text_input.rs`.

Delete `ecosystem/fret-ui-material3/tests/material3_interaction_regressions.rs` because no
Material3-owned tests remain in that binary.

## Boundaries

- `fret-ui` owns the primitive TextInput model update contract.
- Material3 owns component and recipe policy tests, not raw `TextInputProps` mechanism coverage.
- No public API or runtime behavior changes are introduced.

## Non-Goals

- Do not refactor TextInput internals.
- Do not move Material3 field-family, TimePicker, overlay, navigation, or choice/action tests again.
- Do not add a shared cross-crate test harness.

## Follow-On Shape

The interaction-regression ownership cleanup chain is closed. Future Material3 work should start
from component-family binaries or from explicit source-alignment packets, not from the deleted
residual binary.
