# Material3 Token Resolver Non-Field Fallback v1 Milestones

Status: Closed
Last updated: 2026-05-31

## M0 - Lane Opened

Status: Complete

- Follow-on scope is split from the closed field/state-layer fallback lane.
- Button is selected as the first executable migration target.

## M1 - Button Baseline Proved

Status: Complete

- Button token fallback mechanics use the resolver vocabulary.
- Button visual fixture outcomes and state tests remain green.

## M2 - Chip Family Hardened

Status: Complete

- Chip, FilterChip, InputChip, and SuggestionChip repeated fallback chains are migrated or split
  with evidence.

## M3 - Action Components Hardened

Status: Complete

- IconButton, FAB, SegmentedButton, and Tabs use shared fallback vocabulary where repeated paths
  justify it.

## M4 - Residual Surfaces Closed Or Split

Status: Complete

- Remaining non-field surface/navigation fallback families are migrated or split into a narrower
  follow-on.
- Surface/navigation/overlay and small residual token modules now use `MaterialTokenResolver` for
  repeated component-to-system color fallback and state-layer opacity paths.
- The remaining broad residual set is isolated to selection controls: `checkbox.rs`, `slider.rs`,
  and `switch.rs`.

## M4.5 - Selection Controls Remaining

Status: Complete

- Checkbox, Slider, and Switch completed a focused choice-control migration and state-test pass.
- Checkbox, Slider, and Switch residual color fallback chains now use `MaterialTokenResolver`; the
  remaining direct Slider number read is a label-text weight override, not a color fallback chain.

## M5 - Lane Verified

Status: Complete

- Formatting, token fixtures, targeted component tests, check/clippy, catalog, layering, and diff
  hygiene pass.
- The residual color fallback audit has no matches across non-generated Material3 token modules.
- Non-color direct token lookup governance is explicitly split to future work if needed.
