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
- `tokens::usage` now owns the audited source inventory, literal `md.*` classification, internal
  test-token filtering, and format-string template expansion rules used by both conformance tests
  and `material3_token_audit`.
- `tokens::coverage` now parses the manifest into typed Rust structs, validates source drift, and
  checks v30 theme resolution from the manifest rather than from crate-root source inventory. It is
  now a thin conformance Adapter over `tokens::usage`.
- `material3_token_audit` now audits the same 121-source usage set as the manifest gate instead of
  recursively scanning generated token preset files or maintainer binaries.
- `material3_token_audit -- --check-usage-manifest` now regenerates the expected manifest from
  `tokens::usage` in memory and fails when the checked fixture is stale; `--update-usage-manifest`
  rewrites the fixture from the same Interface.
- v30 token injection now fills the previously missing button, menu, navigation, search, date, and
  time-picker Material/Fret-specific token aliases used by recipe token modules.
- v30 token injection now separates the generated Material Web baseline (`tokens::material_web_v30`)
  from the curated Fret overlay Module (`tokens::v30_overlay`) that owns Fret markers, aliases,
  defaults, and hand-authored backfills.
- `tokens::v30_overlay_metadata` now owns the classification of non-Material-Web overlay/backfill
  keys. `material3_token_audit` reuses that metadata instead of keeping a local allowlist, and
  fails `--check` when a used key is neither present in Material Web v30 sassvars nor classified by
  the overlay metadata Module.

## Proof

- `cargo test -p fret-ui-material3 --features diagnostics --lib`
- `cargo test -p fret-ui-material3 --features diagnostics --test minimum_touch_target`
- `cargo test -p fret-ui-material3 --features diagnostics --lib material3_token_usage_manifest_matches_literal_md_sources`
- `cargo test -p fret-ui-material3 --features diagnostics --lib material3_literal_md_tokens_resolve_in_v30_theme`
- `cargo run -p fret-ui-material3 --bin material3_token_audit -- --no-material-missing --limit 5`
- `cargo run -p fret-ui-material3 --bin material3_token_audit -- --check --no-material-missing --limit 5`
- `cargo run -p fret-ui-material3 --bin material3_token_audit -- --check-usage-manifest`

## Residual Risk

- The literal token coverage gate still reads Rust source text to detect manifest drift, but source
  discovery and manifest regeneration now share one Module. The remaining risk is that the scanner
  intentionally handles only normal Rust string literals; if recipes begin using raw strings or
  macro-expanded token keys, the usage Interface should be extended before relying on those forms.
