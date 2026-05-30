# Material3 Layout Direction Provider Bridge v1 TODO

Status: Closed
Last updated: 2026-05-30

## Tasks

- [x] M3-DIR-001: Add Material3 foundation helpers that bridge resolved layout direction into the
  core `LayoutDirection` provider.
  - Scope: `ecosystem/fret-ui-material3/src/foundation/context.rs`.
  - Gate: `cargo nextest run -p fret-ui-material3 --lib layout_direction_inherits_masks_and_restores material_resolved_layout_direction_provides_core_direction_for_elements`.

- [x] M3-DIR-002: Wire the resolved direction provider into Tabs as the first consumer proof.
  - Scope: `ecosystem/fret-ui-material3/src/tabs.rs`,
    `ecosystem/fret-ui-material3/tests/tabs_state.rs`.
  - Gate: `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state rtl_tabs_theme_direction_mirrors_physical_tab_order`.

- [x] M3-DIR-003: Run targeted quality gates and close the lane with evidence.
  - Scope: formatting, check, clippy, layering, workstream state validation.
