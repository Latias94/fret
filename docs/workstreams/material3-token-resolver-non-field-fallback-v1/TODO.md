# Material3 Token Resolver Non-Field Fallback v1 TODO

Status: Active
Last updated: 2026-05-31

Task IDs use `M3NF-*`.

## Tasks

- [x] M3NF-010: Open the non-field fallback follow-on.
  - Scope: `docs/workstreams/material3-token-resolver-non-field-fallback-v1`.
  - Expected result: lane exists with clear scope, gates, and first migration target.
  - Gate: JSON/catalog/diff hygiene.

- [x] M3NF-020: Migrate Button token fallback chains.
  - Scope: `ecosystem/fret-ui-material3/src/tokens/button.rs` and resolver helpers only if Button
    proves a reusable pattern.
  - Expected result: Button keeps variant/state key selection while repeated color fallback and
    opacity lookup use `MaterialTokenResolver`.
  - Gate: token visual fixtures plus `button_state` tests.
  - Note: Button color/opacity fallback chains now use resolver primitives; visual fixture and
    `button_state` gates passed.

- [x] M3NF-030: Migrate chip-family token fallback chains.
  - Scope: Chip, FilterChip, InputChip, SuggestionChip.
  - Expected result: chip-family color fallback and disabled opacity paths use the resolver
    vocabulary without visual drift.
  - Gate: token visual fixtures plus chip state tests.
  - Note: Assist, Filter, Input, and Suggestion chip color/disabled fallback chains now use
    resolver primitives; visual fixture and `chip_state` gates passed.

- [x] M3NF-040: Migrate icon/action token fallback chains.
  - Scope: IconButton, FAB, SegmentedButton, Tabs.
  - Expected result: action/control token modules use shared fallback vocabulary for repeated color
    fallback paths while retaining state/variant key selection.
  - Gate: token visual fixtures plus targeted state tests.
  - Note: IconButton, FAB, SegmentedButton, and Tabs color/opacity fallback chains now use resolver
    primitives; token visual fixture and targeted state gates passed.

- [x] M3NF-050: Migrate surface/navigation fallback chains or split them.
  - Scope: Card, Dialog, Snackbar, List, Tooltip, navigation surfaces, and small residual modules.
  - Expected result: either migrated residual families or a narrower follow-on with evidence.
  - Gate: token visual fixtures plus targeted behavior/state tests for touched families.
  - Note: Badge, Card, CarouselItem, Dialog, Divider, List, Menu, NavigationBar,
    NavigationDrawer, NavigationRail, ProgressIndicator, SearchBar, SearchView, BottomSheet,
    Snackbar, and Tooltip scoped fallback chains now use `MaterialTokenResolver`; broad
    selection-control residuals are split to M3NF-055.

- [ ] M3NF-055: Migrate selection-control residual fallback chains.
  - Scope: Checkbox, Slider, Switch.
  - Expected result: the remaining color fallback residuals move to `MaterialTokenResolver`
    without regressing choice-control visual/state outcomes.
  - Gate: token visual fixtures plus `checkbox_state`, `slider_state`, and `switch_state`.

- [ ] M3NF-060: Verify and close.
  - Scope: docs, package gates, catalog, layering, diff hygiene.
  - Expected result: lane closes or splits any remaining non-field fallback work.
  - Gate: all commands in `EVIDENCE_AND_GATES.md`.

## Notes

- Preserve public recipe behavior.
- Do not edit generated Material Web v30 token data.
- Keep migrations fixture-first and family-bounded.
