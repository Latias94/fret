# Material3 Token Visual Matrix v1 - TODO

Status: Active
Last updated: 2026-05-30

Task IDs use `M3TVM-*`.

## M0 - Lane And Matrix Schema

- [x] M3TVM-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-token-visual-matrix-v1,docs/workstreams/README.md]
  Goal: Open the token visual matrix lane and seed the schema/source map from the closed M3PV2
  component-axis matrix and current Material Web v30 token injection surface.
  Validation: `python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/WORKSTREAM.json | Out-Null`; `python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_v1.json | Out-Null`; `python tools/check_workstream_catalog.py`; `git diff --check`.
  Review: DONE. The workstream exists, the initial matrix covers all 39 M3PV2 components, and the
  schema records dimensions, source precedence, family ownership, and first implementation slices.
  Handoff: Start M3TVM-020 by turning token/fallback inventory into an automated report before
  changing component recipe code.

## M1 - Token Inventory And Fallback Audit

- [x] M3TVM-020 [owner=codex] [deps=M3TVM-010] [scope=ecosystem/fret-ui-material3/src/tokens,ecosystem/fret-ui-material3/src/foundation,tools/parity-discovery,docs/workstreams/material3-token-visual-matrix-v1]
  Goal: Build a token inventory report that classifies generated Material Web tokens, manual v30
  aliases, component token modules, fallback chains, and magic visual constants.
  Validation: `python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-30 --output docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json`; generated report plus JSON/schema gates; targeted `cargo nextest run -p fret-ui-material3 --lib tokens::v30`.
  Review: DONE. The generated report covers all 38 component token modules mapped by the 39
  component rows, classifies 3,580 generated Material Web keys, 67 non-generated v30 manual writes,
  1,136 component token fallback sites, and 504 component token visual constants after the
  M3TVM-030 TextField disabled-opacity cleanup.
  Handoff: Start M3TVM-030 by turning the report's heaviest fallback families into declarative
  fixture rows before refactoring fallback logic.

- [x] M3TVM-030 [owner=codex] [deps=M3TVM-020] [scope=ecosystem/fret-ui-material3/tests,ecosystem/fret-ui-material3/tests/fixtures,docs/workstreams/material3-token-visual-matrix-v1]
  Goal: Add a fixture-driven visual-token harness for token outcomes and rendered scene assertions.
  Validation: a small fixture suite proving color/alpha/shape/elevation/outline outcomes for at
  least Button and TextField.
  Validation: `python -m json.tool ecosystem/fret-ui-material3/tests/fixtures/material3_token_visual_cases_v1.json | Out-Null`; `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures`; `cargo nextest run -p fret-ui-material3 --lib tokens::v30`; report/catalog/diff gates.
  Review: DONE. The crate-internal runner consumes a stable JSON fixture suite and covers Button
  and TextField token outcomes for color, alpha, shape, outline, elevation, and typography aliases.
  It also caught and fixed a TextField disabled input-text opacity double-application.
  Handoff: Start M3TVM-040 or M3TVM-050 by extending the same fixture schema before changing more
  fallback logic.

## M2 - Family Token Matrix Packets

- [ ] M3TVM-040 [owner=codex] [deps=M3TVM-030] [scope=ecosystem/fret-ui-material3/src/{tokens,foundation,text_field.rs,select.rs,autocomplete.rs,exposed_dropdown.rs,search_bar.rs,search_view.rs,date_picker.rs,time_picker.rs},ecosystem/fret-ui-material3/tests]
  Goal: Close field-family token visual matrix rows for TextField, Select, Autocomplete,
  ExposedDropdown, SearchBar, SearchView, DatePicker, and TimePicker.
  Validation: fixture rows plus existing field-family M3PV2 gates.
  Review: Pending.

- [ ] M3TVM-050 [owner=codex] [deps=M3TVM-030] [scope=ecosystem/fret-ui-material3/src/{tokens,foundation,button.rs,checkbox.rs,radio.rs,switch.rs,slider.rs,segmented_button.rs,icon_button.rs,chip*.rs,*chip.rs},ecosystem/fret-ui-material3/tests]
  Goal: Close action/control/chip token visual matrix rows for state layers, checked/selected
  visuals, disabled opacity, shape, elevation, and icon/content color.
  Validation: fixture rows plus existing choice/control M3PV2 gates.
  Review: Pending.

- [ ] M3TVM-060 [owner=codex] [deps=M3TVM-030] [scope=ecosystem/fret-ui-material3/src/{tokens,foundation,tabs.rs,navigation_bar.rs,navigation_rail.rs,navigation_drawer.rs,top_app_bar.rs},ecosystem/fret-ui-material3/tests]
  Goal: Close navigation/app-chrome token visual matrix rows for active indicators, destination
  labels/icons, drawer surfaces, and app-bar container/title states.
  Validation: fixture rows plus existing navigation M3PV2 gates.
  Review: Pending.

- [ ] M3TVM-070 [owner=codex] [deps=M3TVM-030] [scope=ecosystem/fret-ui-material3/src/{tokens,foundation,dialog.rs,bottom_sheet.rs,tooltip.rs,snackbar.rs,menu.rs,dropdown_menu.rs,card.rs,badge.rs,fab.rs,list.rs,progress_indicator.rs,divider.rs,carousel_item.rs},ecosystem/fret-ui-material3/tests,goldens/material3-headless/v1]
  Goal: Close overlay/surface/data-display token visual matrix rows for container color, shape,
  elevation, scrim, outline, typography, and draw-region colors.
  Validation: fixture rows plus existing overlay/surface M3PV2 gates.
  Review: Pending.

## M3 - Consolidation And Closeout

- [ ] M3TVM-080 [owner=codex] [deps=M3TVM-040,M3TVM-050,M3TVM-060,M3TVM-070] [scope=ecosystem/fret-ui-material3/src/tokens,ecosystem/fret-ui-material3/tests,docs/workstreams/material3-token-visual-matrix-v1]
  Goal: Delete redundant fallback helpers/tests made obsolete by typed token outcomes and fixtures.
  Validation: diff review plus focused nextest/check/clippy gates.
  Review: Pending.

- [ ] M3TVM-090 [owner=codex] [deps=M3TVM-080] [scope=docs/workstreams/material3-token-visual-matrix-v1]
  Goal: Close the lane or split residual visual-matrix breadth into narrow follow-ons.
  Validation: matrix rows have explicit state; all residuals have source-backed notes.
  Review: Pending.
