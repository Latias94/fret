# Material 3 Conformance Gates v1

Date: 2026-06-01

This note records the architecture pass that deepens Material 3 conformance gates after the
shadcn-level completeness work exposed two shallow test surfaces.

## Truth

- Minimum touch target conformance must be proven through rendered Headless Surfaces, not source
  string scans.
- Literal `md.*` token coverage remains an audit concern, but the source inventory belongs in a
  structured manifest and the source text parser should only guard manifest drift.
- Fret-specific Material token aliases are valid when they preserve a variant-shaped authoring path
  for recipes while still resolving to Material system roles or explicit Material defaults.

## Changes

- `minimum_touch_target` now consumes `material3_touch_target_cases_v1.json`, renders Tabs,
  NavigationBar, NavigationRail, NavigationDrawer, Menu, and List rows, and asserts their semantic
  layout bounds meet the 48dp minimum touch target.
- `material3_token_usage_manifest_v1.json` records literal Material token usage for 121 recipe,
  foundation, interaction, and token-module source files.
- `tokens::coverage` now parses the manifest into typed Rust structs, validates source drift, and
  checks v30 theme resolution from the manifest rather than from crate-root source inventory.
- v30 token injection now fills the previously missing button, menu, navigation, search, date, and
  time-picker Material/Fret-specific token aliases used by recipe token modules.

## Proof

- `cargo test -p fret-ui-material3 --features diagnostics --lib`
- `cargo test -p fret-ui-material3 --features diagnostics --test minimum_touch_target`
- `cargo test -p fret-ui-material3 --features diagnostics --lib material3_token_usage_manifest_matches_literal_md_sources`
- `cargo test -p fret-ui-material3 --features diagnostics --lib material3_literal_md_tokens_resolve_in_v30_theme`

## Residual Risk

- The literal token coverage gate still reads Rust source text to detect manifest drift. It no
  longer uses source scanning as the coverage source of truth. The next deepening step is an
  importer/generator that refreshes the manifest from a checked upstream/token-module export.
