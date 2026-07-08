---
type: Work Progress
title: AI elements shadcn raw imports closed
timestamp: 2026-07-08T05:03:18Z
tags:
  - fret-ui-ai
  - fret-ui-shadcn
  - public-surface
  - facade
status: verified
---

# AI elements shadcn raw imports closed

## Summary

`ecosystem/fret-ui-ai` element sources no longer import `fret_ui_shadcn::raw::*`.

Migrated direct raw style imports to the curated facade:

- `ButtonStyle` now comes from `fret_ui_shadcn::facade::ButtonStyle`;
- sandbox tabs styling now uses `fret_ui_shadcn::facade::{TabsListHeightOverride, TabsListVariant,
  TabsStyle}`.

## Facade Gap Closed

`TabsStyle` and `TabsListHeightOverride` were already public tabs configuration types but were not
available on `fret_ui_shadcn::facade`. They are now exported beside `TabsListVariant`, so first-party
AI elements can keep advanced-looking visual tuning on the curated component facade instead of raw
tabs modules.

## Gates

`ecosystem/fret-ui-ai/tests/shadcn_import_surface.rs` now accepts only
`fret_ui_shadcn::facade::*` imports and `fret_ui_shadcn::prelude::*` in AI elements. Raw shadcn
imports are no longer allowed by the AI element import policy.

The shadcn facade policy now checks the tabs facade exports include `TabsListHeightOverride`,
`TabsListVariant`, and `TabsStyle`.

## Verification

- `cargo nextest run -p fret-ui-ai --test shadcn_import_surface --no-fail-fast`
- `cargo nextest run -p fret-ui-shadcn --lib authoring_critical_family_exports_live_on_curated_facade_only --no-fail-fast`
- `rg -n "fret_ui_shadcn::raw::|shadcn::raw::|fret::shadcn::raw::" ecosystem/fret-ui-ai/src ecosystem/fret-ui-ai/tests`

Also ran `cargo nextest run -p fret-ui-ai --lib --no-fail-fast`; it compiled the crate and ran 236
tests, with 232 passing and 4 unrelated text-role assertions failing:

- `elements::agent::tests::agent_header_label_uses_chrome_title_text_role`
- `elements::environment_variables::tests::environment_variables_title_text_uses_chrome_title_text_role`
- `elements::shimmer::tests::shimmer_resolved_mode_keeps_wrap_overflow_and_baseline_aligned`
- `elements::terminal::tests::terminal_title_label_uses_chrome_title_text_role`

## Residual Risk

The remaining raw shadcn usage after this slice is primarily shadcn crate-local conformance/tests,
documented raw escape-hatch policy strings, cookbook/source-policy tests, and the retained
`fret::shadcn::raw::advanced::sync_theme_from_environment(...)` examples service seam.
