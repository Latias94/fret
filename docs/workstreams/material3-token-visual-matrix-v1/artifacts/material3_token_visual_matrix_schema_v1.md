# Material3 Token Visual Matrix Schema v1

Date: 2026-05-30
Task: M3TVM-010

## Purpose

This schema defines the first machine-readable shape for exhaustive Material3 token visual parity.
It complements the closed M3PV2 component-axis matrix by tracking token-driven visual outcomes
instead of broad component behavior axes.

## Matrix Dimensions

- `component`: Material3 component or recipe surface.
- `family`: field, navigation, choice control, chip, overlay/feedback, surface/data display.
- `variant_axis`: public or recipe-level visual variants.
- `state_axis`: visual states such as enabled, hovered, focused, pressed, disabled, selected,
  error, open, and closed.
- `scheme_axis`: Material dynamic color axes currently exposed by `tokens::v30`:
  `light_tonal_spot`, `dark_tonal_spot`, `light_expressive`, and `dark_expressive`.
- `part_axis`: visible subparts such as container, label, icon, outline, active-indicator,
  state-layer, handle, track, scrim, panel, and supporting text.
- `token_roles`: color, alpha, shape, elevation, outline width, metric, typography, motion, and
  shadow.
- `owner_layer`: generated token snapshot, manual v30 alias, component token module, Material
  foundation, component recipe, or caller-owned layout.
- `gate_state`: the current proof state for the component's token visual matrix.

## Initial Gate States

- `inventory_seeded`: component is in the matrix and mapped to likely token owners, but no dedicated
  visual-token fixture packet has closed it yet.
- `inventory_audited`: the generated inventory report has mapped the component to current token
  modules, fallback chains, manual v30 writes, and magic visual constants, but no fixture packet has
  closed exact token outcomes yet.
- `covered_fixture`: fixture-driven token/scene assertions cover the row.
- `covered_scene`: focused Rust scene assertions cover the row.
- `covered_golden`: representative headless golden coverage exists but exact token assertions are
  not exhaustive.
- `not_applicable`: the token role does not apply to the current recipe surface.
- `split_follow_on`: the row requires future API breadth outside this lane.

## Source Map

- Generated token source: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`
- Public injection surface: `ecosystem/fret-ui-material3/src/tokens/v30.rs`
- Component token modules: `ecosystem/fret-ui-material3/src/tokens/*.rs`
- Shared fallback helper: `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`
- Generated inventory report:
  `docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json`
- Inventory generator: `tools/parity-discovery/material3_token_inventory.py`
- Closed component-axis reference:
  `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_v2_closeout_audit.md`
- Local source references: `repo-ref/compose-multiplatform-core`, `repo-ref/base-ui`

## First Implementation Slices

1. M3TVM-020 inventories generated tokens, manual aliases, fallback chains, and magic constants.
2. M3TVM-030 adds fixture-driven token visual assertions.
3. M3TVM-040 through M3TVM-070 close family rows.
4. M3TVM-080 deletes redundant fallback/test code only after replacement gates exist.
