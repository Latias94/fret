# Material3 Select Chevron And Overlay Motion Packet v2

Task: M3PV2-037
Date: 2026-05-28
Status: Complete

## Truth

Select open/close motion has two visible pieces beyond field-trigger label motion:

- the trailing chevron rotates between closed and open states;
- the listbox overlay fades and scales on enter/exit.

The first frame after open/close should already show intermediate motion. Settled-only overlay
goldens are not enough because they cannot detect a one-frame delay or snap.

## Sources

- Base UI Select demos use open-state icon rotation (`rotate: 180deg`) as a visible trigger-state
  motion outcome.
- Fret Material3 `foundation::overlay_motion` centralizes menu-like overlay alpha/scale motion via
  Material `FastSpatial` and `FastEffects` springs.
- M3PV2-036 established shared field-motion for Select trigger label/field chrome; this packet
  covers the remaining Select-specific trigger chevron plus the Select overlay wrapper.

## Findings

This found one Select component bug and one already-correct shared-helper path:

- Select chevron used the legacy `StateLayerAnimator`, so the first open frame stayed at the
  closed transform and only started rotating on a later frame.
- Select overlay alpha/scale already used `drive_overlay_open_close_motion`; SceneOp probing
  confirmed open and close first frames include intermediate opacity and scale.

## Changes

- Switched `SelectChevronRuntime` from `StateLayerAnimator` to `SpringAnimator` using the same
  `FastSpatial` spring already resolved for field motion.
- Removed Select trigger-local dropdown duration/easing plumbing that only existed for the old
  chevron tween.
- Added a SceneOp fixed-frame gate proving:
  - chevron rotates on the first open frame;
  - overlay fades/scales on the first open frame;
  - open chevron settles at a half-turn;
  - chevron rotates on the first close frame;
  - overlay fades/scales on the first close frame.

## Gates

Red before fix:

```powershell
cargo nextest run -p fret-ui-material3 --test select_behavior select_chevron_rotates_on_first_open_frame
```

Failed with:

```text
expected Select chevron to rotate on the first open frame
```

Green:

```powershell
cargo nextest run -p fret-ui-material3 --test select_behavior select_chevron_rotates_on_first_open_frame
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test select_behavior
cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
```

## Residual Risk

- The test asserts transform/opacity classes in the renderer-agnostic SceneOp stream, not pixel
  screenshots. That is intentional for fixed-frame determinism.
- This closes Select motion together with M3PV2-036. Future overlay-family packets should still
  add cross-component overlay motion probes for Menu, Tooltip, SearchView, Dialog, and BottomSheet.
