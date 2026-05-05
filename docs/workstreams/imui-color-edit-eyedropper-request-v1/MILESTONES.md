# ImUi Color Edit Eyedropper Request v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M0 - Contract Check

- Confirmed eyedropper cannot honestly be implemented as framework-owned screen sampling with the
  current `Effect` surface.
- Chose app-owned policy in `fret-ui-editor`.

## M1 - Request Hook

- Added `ColorEditEyedropperRequest` and `OnColorEditEyedropper`.
- Added `ColorEditOptions::on_eyedropper` and `eyedropper_test_id`.

## M2 - Popup Command

- Added `popup/eyedropper.rs`.
- Wired the command into `request_popup_overlay` without changing picker, side-preview, numeric,
  palette, history, tooltip, copy, or drag/drop ownership.

## M3 - Gates

- Added focused unit coverage for alpha rules and default opt-in behavior.
- Added source-policy anchors proving the feature stays out of runtime effects.
