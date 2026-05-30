# Material3 Foundation Deepening v1 TODO

Status: Active
Last updated: 2026-05-30

## Tasks

- [x] M3FD-010: Deepen Material context direction consumption.
  - Scope: `foundation::context`, residual Material recipes that still read kit direction directly.
  - Expected result: Material theme/default direction flows through one Material-facing interface.
  - Gate: `cargo nextest run -p fret-ui-material3 --lib material_layout_direction_in_scope_uses_theme_default_and_local_override`.

- [x] M3FD-020: Add context-level regression coverage for Material direction in overlay/popup
  consumers.
  - Scope: Autocomplete, DropdownMenu, SearchView, Tooltip where applicable.
  - Expected result: popup placement and logical start/end behavior do not diverge from Select/Tabs.
  - Gate: source audit for migrated consumers plus `DropdownMenu` RTL popup geometry.

- [x] M3FD-030: Define and implement a private Material field family module.
  - Scope: `foundation::field*`, `text_field.rs`, `select.rs`, `autocomplete.rs`,
    `exposed_dropdown.rs`.
  - Expected result: shared field chrome has one recipe-facing interface.
  - Gate: existing TextField/Select RTL label/supporting-text tests remain green.

- [x] M3FD-040: Migrate field-family recipes to the new field interface and delete duplicated
  field chrome math.
  - Scope: TextField, Select trigger, Autocomplete field bridge, ExposedDropdown adapter.
  - Expected result: recipes provide token namespace/state inputs instead of reimplementing label,
    slot, supporting-text, and indicator placement.
  - Gate: field family tests plus final package check/clippy.

- [x] M3FD-050: Introduce a typed token registry/outcome matrix seam.
  - Scope: `tokens::{material_web_v30,v30,visual_fixtures}`, token audit/import bins.
  - Expected result: generated source data and fixture runners become adapters behind a smaller
    token outcome interface.
  - Gate: token visual fixture tests.

- [x] M3FD-060: Split large token visual fixture logic into fixture-driven families.
  - Scope: `tokens/visual_fixtures.rs`, `tests/fixtures`, goldens if needed.
  - Expected result: adding a component family does not require editing one giant outcome module.
  - Gate: `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`.
  - Note: outcome runners now live under `tokens/visual_fixtures/{fields,selection,navigation,overlays,surfaces}.rs`.

- [ ] M3FD-070: Verify and close the lane.
  - Scope: formatting, check, clippy, layering, workstream catalog, diff hygiene.
  - Gate: all commands in `EVIDENCE_AND_GATES.md`.

## Notes

- Keep public recipe churn justified by depth and deletion, not cosmetic cleanup.
- Do not duplicate generic `fret-ui-kit` primitives in Material3.
- Split narrow follow-ons when a task discovers a real mechanism gap in `crates/fret-ui`.
