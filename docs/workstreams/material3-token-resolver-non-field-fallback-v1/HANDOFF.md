# Material3 Token Resolver Non-Field Fallback v1 Handoff

Status: Active
Last updated: 2026-05-31

## Current State

This lane is open as a narrow follow-on from `material3-token-resolver-fallback-v1`.

M3NF-010 is complete: the workstream exists, the residual fallback surface is documented, and Button
is selected as the first executable migration slice.

M3NF-020 is complete: Button token fallback chains now use `MaterialTokenResolver` while preserving
visual fixture and `button_state` outcomes.

M3NF-030 is complete: Assist, Filter, Input, and Suggestion chip color/disabled fallback chains now
use `MaterialTokenResolver` while preserving visual fixture and `chip_state` outcomes.

M3NF-040 is complete: IconButton, FAB, SegmentedButton, and Tabs color/opacity fallback chains now
use `MaterialTokenResolver` while preserving token visual and targeted state-test outcomes.

M3NF-050 is complete: the surface/navigation/overlay residual set now uses
`MaterialTokenResolver` for repeated component-to-system color fallback and state-layer opacity
paths while preserving token visual and targeted state-test outcomes.

The next executable task is M3NF-055: migrate the remaining selection-control residuals in
`checkbox.rs`, `slider.rs`, and `switch.rs`.

## Guardrails

- Keep Material-specific fallback policy in `fret-ui-material3`.
- Do not edit generated v30 token data.
- Preserve public recipe behavior.
- Add resolver helpers only when a repeated pattern is proven by real migrated families.

## Suggested Next Gate

`cargo nextest run -p fret-ui-material3 --features diagnostics --test checkbox_state --test slider_state --test switch_state`
