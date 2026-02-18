# Material 3 Expressive Alignment v1 (TODO)

Milestones: `docs/workstreams/material3-expressive-alignment-v1-milestones.md`  
Plan: `docs/workstreams/material3-expressive-alignment-v1-refactor-plan.md`

This TODO list is **Expressive-variant focused**. For general Material3 parity work, use:
`docs/workstreams/material3-todo.md`.

## Foundation (policy + infra)

- [ ] Confirm which upstream (Compose vs MUI vs Material Web) is normative for each mismatch class.
- [ ] Standardize per-component evidence anchors in PR templates (owner path + symbol + gate command).
- [ ] Expand typed token coverage where Expressive introduces new keys upstream (no placeholders).

## Controls (selection / toggles)

- [ ] Checkbox: tri-state (`indeterminate`) outcome + gates.
- [ ] Switch: thumb/track icons + with-icon sizing (verify disabled/pressed edge cases).
- [ ] Icon toggle button (Expressive): shape morph + semantics gate (done; keep in sync with upstream).
- [ ] Segmented buttons: selection indicator geometry + keyboard nav + state layer.
- [ ] Radio: verify pressed/hover/focus outcome parity in Expressive.
- [ ] Slider: thumb/track sizing + tick marks + semantics/value exposure.

## Inputs

- [ ] Text field: leading/trailing icons + supporting text + error/disabled layers.
- [ ] Search: expressive tokens + focus/keyboard affordances.
- [ ] Select/autocomplete: menu anchoring + typeahead + scroll/virtualization stability.

## Navigation / surfaces

- [ ] Tabs: indicator motion + expressive shape behavior (if upstream defines it).
- [ ] Navigation bar/rail/drawer: selected indicator geometry + icon/label sizes.
- [ ] Dialog/bottom sheet/snackbar/tooltip: elevation + scrim + motion + dismissal policy boundaries.

## Tooling / automation

- [ ] For each aligned component, add:
  - `apps/fret-ui-gallery`: demo section + stable `test_id`
  - `tools/diag-scripts`: script v2 that asserts invariants + packs bundle/screenshots
  - `ecosystem/fret-ui-material3/tests`: headless gates for deterministic invariants

