# Material 3 Conformance Gates v1

Date: 2026-06-01

This note records the architecture pass that deepens Material 3 conformance gates after the
shadcn-level completeness work exposed two shallow test surfaces.

## Truth

- Minimum touch target conformance must be proven through rendered Headless Surfaces, not source
  string scans.
- Literal `md.*` token coverage remains an audit concern, but the source inventory and extraction
  logic belong in the token layer rather than in crate-root tests.
- Fret-specific Material token aliases are valid when they preserve a variant-shaped authoring path
  for recipes while still resolving to Material system roles or explicit Material defaults.

## Changes

- `minimum_touch_target` now renders Tabs, NavigationBar, NavigationRail, NavigationDrawer, Menu,
  and List rows and asserts their semantic layout bounds meet the 48dp minimum touch target.
- `tokens::coverage` owns literal Material token source inventory, extraction, de-duplication, and
  theme resolution checks for crate-level conformance tests.
- v30 token injection now fills the previously missing button, menu, search bar, and date picker
  Material/Fret-specific token aliases used by recipe token modules.

## Proof

- `cargo test -p fret-ui-material3 --features diagnostics --lib`
- `cargo test -p fret-ui-material3 --features diagnostics --test minimum_touch_target`

## Residual Risk

- The literal token coverage gate still audits Rust source text because the recipe token modules do
  not yet expose a generated typed token manifest. The parser and inventory are now localized in the
  token layer, so the next deepening step can replace that implementation without touching crate-root
  tests.
