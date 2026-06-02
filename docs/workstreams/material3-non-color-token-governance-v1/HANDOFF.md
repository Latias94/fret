# Material3 Non-Color Token Governance v1 Handoff

Status: Closed
Last updated: 2026-05-31

## Current State

This lane is closed as a follow-on from `material3-token-resolver-non-field-fallback-v1`.

M3NC-010 is complete: the lane exists, baseline direct-read audits are recorded, and the first
executable task is M3NC-020.

M3NC-020 is complete: chip-family and Slider label weight normalization now routes through
`tokens::typography::text_style_with_weight` while preserving token visual, `chip_state`, and
`slider_state` outcomes.

M3NC-030 is complete: Radio disabled icon opacity now uses `MaterialTokenResolver::number_optional`
and focused Radio/choice-control gates passed.

M3NC-040 is complete: Dialog, Snackbar, and ModalNavigationDrawer duration/easing direct reads now
route through `MaterialTokenResolver` motion helpers while preserving targeted state/motion
outcomes.

M3NC-050 is complete: TimeInput/TimePicker state-layer opacity chains and residual low-risk
DatePicker/TimePicker/Dropdown/Tooltip/indication non-color reads now use `MaterialTokenResolver`.

M3NC-060 is complete: final package/workstream gates passed, closeout evidence is recorded, and
remaining direct reads are classified as owned by resolver, typography, context, registration, or
fixture utilities.

There are no executable tasks left in this lane.

## Guardrails

- Do not reopen the closed color fallback lanes.
- Keep Material-specific token policy in `ecosystem/fret-ui-material3`.
- Centralize repeated policy only when a real cross-component pattern exists.
- Keep component-local scalar defaults in component token modules when no fallback chain exists.

## Suggested Next Gate

No gate is required for this closed lane. Start a narrower follow-on only if future evidence shows
one of the classified residual owners is the wrong boundary.
