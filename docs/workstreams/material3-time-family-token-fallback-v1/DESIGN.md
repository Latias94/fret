# Material3 Time Family Token Fallback v1 - Design

Status: closed
Date: 2026-05-31

## Intent

Continue the Material3 token fallback hardening work after the chip-family slice by targeting the
time-family period selector. `time_picker` and `time_input` both expose a period selector and repeat
the same token fallback policy for shape, outline, selected container, label color, label text
style, and state-layer color/opacity.

This lane moves that repeated Material policy into one private token helper without changing the
component APIs or interaction behavior.

## Source And Boundary Truth

- Material Web v30 remains the source for `md.comp.time-picker.*` and `md.comp.time-input.*`
  scalar/color keys.
- Material spec and Compose Material3 are the behavioral reference for period selector state
  outcomes; this lane only touches typed token fallback access.
- `crates/*` stay untouched. The shared policy belongs in `ecosystem/fret-ui-material3/src/tokens`.
- Recipe code continues to call `time_picker_tokens::*` and `time_input_tokens::*`.

## Scope

In scope:

- Add a private shared token helper for time-family period selector fallback policy.
- Migrate duplicated `period_selector_*` token functions in `time_picker.rs` and `time_input.rs`.
- Preserve existing token function names consumed by recipes and visual fixtures.
- Update inventory tooling so the helper is counted as shared policy.
- Generate a v1 inventory artifact for this lane.
- Add focused unit tests for the shared helper and run time-picker/token visual gates.

Out of scope:

- Changing TimePicker or TimeInput interaction state machines.
- Changing period selector layout behavior in recipes.
- Changing token values or public component APIs.
- Moving policy into `crates/fret-ui`.

## Refactor Brief

Intent: prevent two time-family modules from drifting in their period-selector fallback chains.

Scope: `ecosystem/fret-ui-material3/src/tokens/{time_picker,time_input}.rs`, a new shared helper,
inventory tooling, and this workstream evidence.

Deletion plan: remove duplicated period-selector shape, outline, selected container, label, and
state-layer fallback blocks where the behavior is equivalent.

Boundary plan: keep the helper private to Material3 tokens; consumers keep their current token
module APIs.

Testing plan: helper unit tests, time picker interaction smoke where practical, Material3 token
visual golden suite, inventory regeneration, catalog/layering checks, Rust check/clippy.

Risk plan: the main risk is token key drift between `time-picker` and `time-input`. The helper
accepts component prefixes and height key variants so the differing token keys stay explicit.

Scale plan: bounded fearless-refactor workstream with one implementation slice and closeout.
