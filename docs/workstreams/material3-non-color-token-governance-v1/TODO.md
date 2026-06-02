# Material3 Non-Color Token Governance v1 TODO

Status: Closed
Last updated: 2026-05-31

Task IDs use `M3NC-*`.

## Tasks

- [x] M3NC-010: Open non-color token governance lane.
  - Scope: workstream docs, catalog state, and baseline audit.
  - Expected result: lane exists with clear ownership rules and first executable slice.
  - Gate: JSON/catalog/diff hygiene.

- [x] M3NC-020: Centralize typography weight reads.
  - Scope: chip-family label text styles and Slider value-indicator label style.
  - Expected result: component modules use `tokens::typography` helpers for weight normalization
    instead of hand-coding `theme.number_by_key(...)` and `FontWeight(...)`.
  - Gate: token visual fixture plus `chip_state` and `slider_state`.
  - Note: Assist, Filter, Input, Suggestion chip label styles and Slider value-indicator label style
    now route weight normalization through `tokens::typography::text_style_with_weight`; targeted
    gates passed.

- [x] M3NC-030: Migrate choice-control numeric residuals.
  - Scope: Radio disabled icon opacity and any adjacent selection-control number fallback.
  - Expected result: component-to-system state/disabled opacity reads use `MaterialTokenResolver`.
  - Gate: token visual fixture plus Radio/choice-control state tests.
  - Note: Radio disabled icon opacity now uses `MaterialTokenResolver::number_optional`; the
    focused Radio tests and Checkbox/Slider/Switch state gates passed. The broad `radio_alignment`
    binary still contains unrelated overlay/navigation golden suites, so this lane uses a Radio
    filter for the Radio-specific proof.

- [x] M3NC-040: Classify motion/easing direct reads.
  - Scope: Dialog, Snackbar, ModalNavigationDrawer easing and duration reads.
  - Expected result: repeated motion fallback chains either use a shared helper or are documented as
    intentional motion-scheme calls.
  - Gate: targeted dialog/snackbar/drawer motion/state tests.
  - Note: Material duration/easing token reads now use `MaterialTokenResolver` helpers for Dialog,
    Snackbar, and ModalNavigationDrawer; targeted motion/state gates passed.

- [x] M3NC-050: Classify time picker/input numeric fallback chains.
  - Scope: TimeInput and TimePicker numeric fallback helpers plus residual low-risk non-color
    direct reads found during closeout audit.
  - Expected result: migrate high-confidence number chains or split a time-field follow-on with
    exact residual paths and tests.
  - Gate: automation surface time picker tests plus package check.
  - Note: TimeInput/TimePicker state-layer opacity fallback chains, TimePicker/DatePicker modal
    duration/easing/scrim reads, DatePicker outside-month opacity, Dropdown/Tooltip durations, and
    indication ripple duration/easing reads now use `MaterialTokenResolver`. Remaining direct reads
    are owned by resolver, typography, fixture utilities, token-key registration, or context flags.

- [x] M3NC-060: Verify and close or split.
  - Scope: docs, package gates, catalog, layering, diff hygiene.
  - Expected result: lane closes with residuals classified or a narrower follow-on opened.
  - Gate: all commands in `EVIDENCE_AND_GATES.md`.
  - Note: Final gates passed on 2026-05-31. No new follow-on is required for the migrated
    non-color fallback chains; remaining direct reads have explicit owners.

## Notes

- Preserve visual and behavioral outcomes.
- Keep Material-specific token policy in `ecosystem/fret-ui-material3`.
- Do not treat every scalar metric fallback as debt.
