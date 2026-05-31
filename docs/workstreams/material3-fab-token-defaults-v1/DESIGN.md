# Material3 FAB Token Defaults v1 - Design

Status: closed
Date: 2026-05-31

## Intent

Continue Material3 token fallback hardening by targeting the highest magic-constant module in the
current inventory: `tokens::fab`. FAB and extended FAB have many Material default matrices for
size, icon size, shape, spacing, disabled opacity, and state-layer opacity. Those values are
currently mixed directly into token resolver functions.

This lane moves the default matrices into a private Material3 token helper so `fab.rs` can focus on
token key mapping and fallback order.

## Source And Boundary Truth

- Material Web v30 remains the source for `md.comp.fab.*` and `md.comp.extended-fab.*` token keys.
- Material spec/Compose Material3 define the visual intent for FAB sizes and state layers.
- This is Material policy inside `ecosystem/fret-ui-material3`; `crates/*` stay untouched.
- Existing `fab_tokens::*` function names and FAB component APIs stay unchanged.

## Scope

In scope:

- Add a private FAB token helper for default matrices.
- Move FAB/extended-FAB size, shape, spacing, disabled opacity, and state-layer fallback defaults
  out of `fab.rs`.
- Update inventory tooling so the helper is counted as token policy helper code.
- Generate a v1 inventory artifact for this lane.
- Add focused helper tests and run existing FAB behavior tests.

Out of scope:

- Changing FAB public APIs.
- Changing rendered FAB geometry, interaction, motion, or semantics.
- Changing token values.
- Full cleanup of every remaining Material3 token module.

## Refactor Brief

Intent: make FAB token defaults explicit and governed before more FAB variants or expressive
variants multiply local fallback constants.

Scope: `ecosystem/fret-ui-material3/src/tokens/{fab,fab_common}.rs`, inventory tooling, FAB tests,
and this workstream evidence.

Deletion plan: remove inline `Px(...)` and opacity/default literals from FAB resolver functions
where they represent stable Material token defaults.

Boundary plan: keep `fab_common` private to Material3 tokens; recipe code continues to call the
existing `fab_tokens` API.

Testing plan: helper unit tests for size/shape/spacing/default opacity matrices, existing
`fab_state` tests, crate check/clippy, inventory regeneration, catalog/layering checks.

Risk plan: visual drift from a wrong matrix move is the main risk. Keep wrapper functions unchanged
and gate existing FAB geometry/motion tests.

Scale plan: bounded fearless-refactor workstream with one implementation slice and closeout.
