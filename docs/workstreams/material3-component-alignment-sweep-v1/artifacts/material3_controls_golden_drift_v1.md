# Material 3 Controls Golden Drift v1

Date: 2026-05-27
Task: M3CAS-020
Status: Classified

## Verdict

The `material3-controls.*.json` drift is classified as stale golden plus an underspecified test
harness alignment assumption.

It is not classified as a Material recipe regression and does not require moving behavior into
`crates/*`.

## Reproduction

Initial gate:

```powershell
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1
```

Observed failure:

- failing file:
  `goldens/material3-headless/v1/material3-controls.scale1_0.dark.tonal_spot.json`
- same failing gate had already been observed in a clean detached `HEAD` worktree during the prior
  Material harness lane

## Diagnosis

The current rendered scene is stable. A temporary detached worktree was created at:

```text
F:\SourceCodes\Rust\fret-worktrees\m3cas020-golden-check
```

In that worktree, regenerating controls goldens passed:

```powershell
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1
```

The regenerated files changed all twelve `material3-controls.*.json` files. Structural comparison
showed:

- scene operation signatures are unchanged,
- quad counts are unchanged,
- changed values are primarily quad rectangles,
- `idle_select_supporting_text_insets` was unchanged in the representative
  `scale1_0.dark.tonal_spot` file.

The representative drift pattern before the harness alignment fix:

- button and disabled button chrome resolved from about `64px` width to available container width,
- checkbox, switch, and chip chrome shifted with the available column width,
- state/focus scenes preserved the same operation structure.

## Ownership Decision

Material recipe defaults should not encode caller/container width negotiation.

The controls golden is intended to protect intrinsic Material control chrome and aggregate visual
coverage, not the default `FlexProps::align = Stretch` behavior of the test column. Therefore the
test harness now explicitly sets the controls suite column to `CrossAlign::Start`.

This keeps the golden focused on intrinsic component output while leaving full-width layout as
caller-owned behavior for app and gallery surfaces.

## Action Taken

- Added explicit `CrossAlign::Start` to the controls-suite vertical flex in
  `ecosystem/fret-ui-material3/tests/radio_alignment.rs`.
- Refreshed all twelve `goldens/material3-headless/v1/material3-controls.*.json` files after the
  harness alignment was made explicit.
- Re-ran the same controls golden gate without `FRET_UPDATE_GOLDENS`; it passed.

## Proof

Passing gate after refresh:

```powershell
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1
```

Result:

```text
1 test run: 1 passed, 71 skipped
```

## Residual Risk

- This classification only covers the aggregate controls golden drift.
- It does not prove every control component has complete Material parity.
- The next sweep tasks should still create packet evidence for Tabs/Navigation, field family,
  overlays, and remaining choice controls.
