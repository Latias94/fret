# Material3 Slider Token Defaults v1 - Design

Status: closed
Date: 2026-05-31

## Intent

Continue Material3 token fallback hardening by targeting the highest fallback-density component
token module in the current inventory: `tokens::slider`. Slider has repeated default matrices for
state-layer size, tick marks, stop indicators, track heights, handle size/shape/width, and disabled
or selected opacities.

This lane moves those stable Material defaults into a private helper so `slider.rs` can focus on
token key mapping and fallback order.

## Source And Boundary Truth

- Material Web v30 remains the source for `md.comp.slider.*` token keys.
- Material spec/Compose Material3 define the visual intent for Slider defaults and state layers.
- Runtime Slider APIs and interaction behavior stay unchanged.
- `crates/*` stay untouched.

## Scope

In scope:

- Add a private Slider token default helper module.
- Move stable Slider fallback/default matrices out of `slider.rs`.
- Update inventory tooling so the helper is counted as token helper policy.
- Generate a v1 inventory artifact for this lane.
- Add focused helper tests and run existing Slider behavior/golden gates.

Out of scope:

- Changing Slider public APIs.
- Changing Slider interaction state machines, semantics, or geometry.
- Changing Material token values.

## Refactor Brief

Intent: lower Slider token-module fallback/default density before more Slider variants or expressive
tokens make the module harder to audit.

Scope: `ecosystem/fret-ui-material3/src/tokens/{slider,slider_common}.rs`, inventory tooling,
Slider tests, and this workstream evidence.

Deletion plan: remove inline `Px(...)` and opacity default literals from Slider resolver functions
where they represent stable Material defaults.

Boundary plan: keep the helper private to Material3 tokens; `Slider` recipes continue to call the
existing `slider_tokens::*` API.

Testing plan: helper unit tests, `slider_state`, `material3_headless_slider_suite_goldens_v1`, crate
check/clippy, inventory regeneration, catalog/layering checks.

Risk plan: visual drift in handle/track/tick geometry is the main risk. Keep wrapper functions
unchanged and gate existing Slider behavior/golden tests.

Scale plan: bounded fearless-refactor workstream with one implementation slice and closeout.
