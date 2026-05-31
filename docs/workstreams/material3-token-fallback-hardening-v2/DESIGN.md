# Material3 Token Fallback Hardening v2 - Design

Status: closed
Date: 2026-05-31

## Intent

The v1 Material3 token visual matrix closed fixture coverage, but its inventory still records a
large amount of component-local fallback logic and visual constants. This lane reduces that
long-term maintenance cost without reopening the closed matrix lane or widening core mechanisms.

The first execution slice targets the chip token family because AssistChip, FilterChip, InputChip,
and SuggestionChip all have real consumers and duplicate the same fallback patterns:

- disabled on-surface color plus opacity composition,
- 32px container height,
- small shape fallback,
- 18px icon-size fallbacks,
- state-layer opacity fallback,
- elevated container elevation fallback,
- outline width and disabled-outline opacity fallback.

## Source And Boundary Truth

- Material Web v30 remains the token source for `md.comp.*` scalar/color keys.
- Compose Material3 remains the reference for shared Material policy ownership: shape, disabled
  alpha, state-layer, and interaction fallback should be shared policy, not recipe-local drift.
- `crates/fret-ui` stays untouched. This is Material policy inside `ecosystem/fret-ui-material3`.
- Recipe files such as `chip.rs`, `filter_chip.rs`, `input_chip.rs`, and `suggestion_chip.rs`
  should continue to consume typed token functions. They should not gain token-resolution logic.

## Scope

In scope:

- Add a chip-family shared token helper module under `ecosystem/fret-ui-material3/src/tokens`.
- Migrate the four chip token modules to that helper where the fallback behavior is identical.
- Update inventory tooling so the new helper is counted as a shared helper, not an unmapped
  component token module.
- Generate a v2 inventory artifact for this lane.
- Add focused unit coverage for the shared helper and run chip-focused gates.

Out of scope:

- Changing public chip component APIs.
- Changing visual token values.
- Removing all Material3 fallback sites in one pass.
- Moving Material policy into `crates/*`.

## Refactor Brief

Intent: remove repeated chip-family fallback code before it hardens into four slightly different
implementations.

Scope: `ecosystem/fret-ui-material3/src/tokens/{chip,filter_chip,input_chip,suggestion_chip}.rs`,
new shared token helper, inventory tooling, and this workstream evidence.

Deletion plan: delete duplicate per-module `ChipOutline`, `disabled_on_surface_color`, and common
metric/shape/state-layer/elevation/outline fallback blocks where the behavior is equivalent.

Boundary plan: keep helper private to the Material3 token layer; component recipes continue to call
their existing token module APIs.

Testing plan: helper unit tests, chip state tests, token visual fixture validation where practical,
inventory regeneration, catalog/layering checks, Rust check/clippy.

Risk plan: visual drift is the main risk. Keep existing public token functions and preserve exact
fallback keys and fallback values.

Scale plan: bounded workstream with one production slice and closeout.
