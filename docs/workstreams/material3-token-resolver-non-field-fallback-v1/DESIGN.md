# Material3 Token Resolver Non-Field Fallback v1

Status: Active
Last updated: 2026-05-31

## Problem

`material3-token-resolver-fallback-v1` centralized alpha/blend helpers, migrated state-layer
fallbacks for selected controls, and hardened the field-family token modules. Non-field Material3
token modules still contain repeated component-to-system color fallback chains.

This keeps primitive fallback semantics distributed across component token modules such as Button,
Chip, IconButton, FAB, Tabs, Card, Dialog, Snackbar, List, Tooltip, and navigation surfaces.

## Target State

- `foundation::token_resolver` remains the Material-specific fallback seam.
- Non-field component token modules express role/state/variant key selection.
- Shared component-to-system color fallback, optional opacity lookup, and color composition use the
  resolver vocabulary already proven by the previous lane.
- Token visual fixture outcomes and component behavior tests remain unchanged.

## Scope

In scope:

- `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs` only when a helper is proven by
  repeated migrated code and fits the resolver vocabulary for later families.
- Non-field token modules with existing visual fixture coverage, starting with Button.
- Component-focused tests for migrated families.
- Workstream docs, evidence, and closeout.

Out of scope:

- TextField, Select, and Autocomplete field-family fallback paths already handled by
  `material3-token-resolver-fallback-v1`.
- Generated Material Web v30 token data.
- Recipe public API changes.
- Layout, motion, overlay placement, or indication orchestration policy.
- Moving Material fallback policy into `crates/fret-ui`.

## Architecture Direction

Use `MaterialTokenResolver` as the only owner of repeated Material fallback mechanics. Component
token modules may still own key selection functions because they encode Material role/state names.

Do not add a helper for a one-off token path. Prefer using existing resolver primitives unless a
migrated family proves a repeated fallback shape that should remain consistent across later
families.

## Assumptions

- Confident: remaining work is broad enough to deserve a new follow-on instead of reopening the
  closed field/state-layer fallback lane.
- Confident: Button is the best first slice because it is a single module with high fallback count
  and existing tests.
- Likely: Chip-family migration should happen after Button proves the non-field pattern.

## Source References

- `docs/workstreams/material3-token-resolver-fallback-v1/CLOSEOUT_AUDIT_2026-05-31.md`
- `docs/adr/0032-style-tokens-and-theme-resolution.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0226-material3-state-layer-and-ripple-primitives.md`
