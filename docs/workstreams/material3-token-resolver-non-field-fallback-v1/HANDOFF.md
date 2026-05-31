# Material3 Token Resolver Non-Field Fallback v1 Handoff

Status: Closed
Last updated: 2026-05-31

## Current State

This lane is closed as a narrow follow-on from `material3-token-resolver-fallback-v1`.

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

M3NF-055 is complete: Checkbox, Slider, and Switch residual color fallback chains now use
`MaterialTokenResolver` while preserving token visual and targeted choice-control state outcomes.

M3NF-060 is complete: final package/workstream gates passed, the residual color fallback audit has
no matches, and the closeout audit is recorded.

There are no executable tasks left in this lane. If future work wants to govern non-color direct
token lookups, start a narrower follow-on instead of reopening this scope.

## Guardrails

- Keep Material-specific fallback policy in `fret-ui-material3`.
- Do not edit generated v30 token data.
- Preserve public recipe behavior.
- Add resolver helpers only when a repeated pattern is proven by real migrated families.

## Suggested Next Gate

No gate is required for this closed lane. For a new follow-on, begin with the residual audit pattern
recorded in `EVIDENCE_AND_GATES.md`.
