# Material 3 Radio Scene Semantics Packet v1

Date: 2026-05-28
Status: Closed

## Problem

The Material3 component matrix still marked `Radio` as a known follow-on because the broader choice
controls packet deferred the final Radio packet until Switch foundation evidence existed. Switch now
has a closed diagnostics packet proving the shared indication and minimum target split, so Radio can
be closed with its existing focused selector, scene, ripple, and pressed-state gates.

## Target State

- Radio and RadioGroup expose stable `radio_group`, `radio`, and `radio.chrome` selector surfaces.
- The selected dot is centered inside the outline at supported scale factors.
- Pointer-origin ripple remains routed through the shared Material indication path.
- Pressed scene structure is stable across tonal/expressive and light/dark schemes.
- Roving/typeahead remains recipe-owned until another design-system consumer proves a shared kit
  policy need.

## Source Truth

- Compose Material3 `RadioButton.kt` for selected state, `Role.RadioButton`, minimum interactive
  component sizing, ripple, icon drawing, and state/touch behavior.
- Compose Material3 `RadioButtonTokens.kt` for state-layer and icon sizing.
- Base UI `RadioGroup.tsx` and `RadioRoot.tsx` for headless group/item semantics, checked state,
  disabled/read-only handling, and composite item wiring.
- Fret `Radio`/`RadioGroup` for GPU-first recipe rendering and Fret-specific test-id surfaces.

## Layer Ownership

- `ecosystem/fret-ui-material3/src/radio.rs`: radio group semantics, item selection, dot geometry,
  roving/typeahead wiring, and root/chrome selectors.
- `ecosystem/fret-ui-material3/src/foundation/indication.rs`: shared Material state-layer/ripple
  behavior.
- `ecosystem/fret-ui-material3/src/foundation/interactive_size.rs`: minimum interactive target and
  root-derived `.chrome` selector stamping.
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`: selector proof.
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`: scene geometry, ripple, and pressed-state
  proof.

## In Scope

- Close the Radio matrix residual.
- Record the source and gate evidence in a dedicated follow-on packet.
- Preserve the current layer split.

## Out Of Scope

- New gallery diagnostics for Radio.
- Moving RadioGroup roving/typeahead policy into `fret-ui-kit`.
- New `crates/*` focus or semantics mechanisms.
- Pixel-perfect upstream screenshot diffing.

## Closeout Condition

This lane is complete once the dedicated packet exists, the matrix row points at it, and the focused
selector/scene gates pass.
