---
type: Work Progress
title: GenUI shadcn resolver moved off raw shadcn modules
timestamp: 2026-07-08T04:49:42Z
tags:
  - fret-genui-shadcn
  - fret-ui-shadcn
  - public-surface
  - facade
status: verified
---

# GenUI shadcn resolver raw-free facade lane

## Summary

`ecosystem/fret-genui-shadcn` no longer uses `shadcn::raw::*` inside its resolver layer.

Migrated resolver callsites to the curated facade:

- typography rendering now uses `shadcn::typography::*`;
- dropdown menu item variants now use `shadcn::DropdownMenuItemVariant`;
- radio group orientation now uses `shadcn::RadioGroupOrientation`.

## Facade Gap Closed

The migration exposed that `RadioGroupOrientation` was public from `fret-ui-shadcn::radio_group`
but missing from `fret_ui_shadcn::facade`. The facade now re-exports it so generated UI resolvers do
not need to reopen `shadcn::raw::radio_group`.

## Gates

Added two source-policy checks:

- `fret-genui-shadcn::surface_policy_tests::resolver_uses_curated_shadcn_facade_not_raw_modules`
  scans resolver sources and rejects `shadcn::raw::`.
- `fret-ui-shadcn::surface_policy_tests::radio_group_orientation_is_available_on_curated_facade`
  locks `RadioGroupOrientation` onto the facade.

## Verification

- `cargo nextest run -p fret-genui-shadcn --no-fail-fast`
- `cargo nextest run -p fret-ui-shadcn --lib radio_group_orientation_is_available_on_curated_facade --no-fail-fast`
- `rg -n "shadcn::raw::|fret_ui_shadcn::raw::|fret::shadcn::raw::" ecosystem/fret-genui-shadcn/src/resolver ecosystem/fret-genui-shadcn/tests`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `git diff --check`

## Residual Risk

Other first-party ecosystem crates still have raw shadcn usage, especially `fret-ui-ai` style imports
and shadcn crate-local conformance tests. Those should be handled as separate slices so this resolver
change remains narrowly reviewable.
