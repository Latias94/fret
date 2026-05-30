# Material3 Token Resolver Fallback v1

Status: Active
Last updated: 2026-05-31

## Problem

Material3 token visual coverage is broad, but fallback policy is still shallow in many token
modules. Repeated helpers such as alpha multiplication, color blending, component-to-system color
fallbacks, and literal defaults appear inside component token modules instead of one Material token
resolver seam.

This weakens locality: a fallback bug can require auditing TextField, Select, Autocomplete,
Checkbox, Radio, Switch, Slider, List, and visual fixtures separately.

## Target State

- `foundation::token_resolver` owns common Material fallback and color-composition policy.
- Component token modules express Material token roles and state branches, not repeated primitive
  fallback mechanics.
- The token visual fixture runner continues to prove unchanged outcomes.
- Public recipe behavior remains unchanged; this lane deepens token implementation only.

## Scope

In scope:

- `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`
- `ecosystem/fret-ui-material3/src/tokens/{text_field,select,autocomplete,checkbox,radio,switch,slider,list}.rs`
- `ecosystem/fret-ui-material3/src/tokens/visual_fixtures.rs` when shared fixture color helpers need
  to use the same primitive behavior.
- Material3 token visual fixture tests and targeted package gates.
- Workstream docs and catalog.

Out of scope:

- Changing Material Web v30 generated token data.
- Changing recipe public surfaces.
- Moving Material-specific fallback policy into `crates/fret-ui`.
- Reopening `material3-token-visual-matrix-v1` or `material3-foundation-deepening-v1`.
- Field chrome, menu layout policy, or indication orchestration follow-ons.

## Architecture Direction

`MaterialTokenResolver` should become the Material-facing module for repeated fallback mechanics.
Component token modules remain the adapters that know component prefixes, variants, and interaction
state names.

The deletion test should improve: deleting the resolver should make duplicated alpha/blend/fallback
policy reappear across token modules, while deleting one component token module should not delete
the shared fallback vocabulary.

## Assumptions

- Confident: Material-specific token policy belongs in `fret-ui-material3`, not the Runtime
  Substrate.
- Confident: The existing token visual fixture matrix is the right regression surface for this
  refactor.
- Likely: Alpha and blend helpers are the safest first slice because they are pure and already
  duplicated.
- Likely: Color/number/metric/corner fallback helper expansion should proceed after the pure helper
  migration so each step has small diffs and stable fixture evidence.

## Source References

- `CONTEXT.md`
- `docs/adr/0032-style-tokens-and-theme-resolution.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/workstreams/material3-token-visual-matrix-v1/WORKSTREAM.json`
- `docs/workstreams/material3-foundation-deepening-v1/CLOSEOUT_AUDIT_2026-05-31.md`
