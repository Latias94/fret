# Material3 TextField Multiline Line Limits Packet v2

Task: M3PV2-034
Date: 2026-05-28
Status: Complete

## Truth

Material multiline text fields own a visible line-limit layout contract. Compose Material3 exposes
`minLines` and `maxLines` on `TextField`, forwards them to `BasicTextField`, and keeps the base
TextField minimum height at `56.dp`. For Fret Material3, that means multiline visible line limits
must affect the outer TextField chrome, not only the inner editable text node.

The observable rule used by this packet is:

- 1 visible line keeps the existing Material TextField base height.
- Additional visible lines add the input text line height.
- `min_lines` sets the minimum visible line count.
- `max_lines` clamps the visible line count and the TextArea measurement height.

With Fret Material type scale, `body-large` is `16px / 24px`, so a filled multiline TextField with
`min_lines(3)` resolves to `56px + 2 * 24px = 104px`.

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TextField.kt`
  - `TextField` exposes `singleLine`, `maxLines`, and `minLines`.
  - The implementation forwards `maxLines` and `minLines` to `BasicTextField`.
  - The field applies `defaultMinSize(minWidth = TextFieldDefaults.MinWidth, minHeight = TextFieldDefaults.MinHeight)`.
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TextFieldDefaults.kt`
  - `TextFieldDefaults.MinHeight = 56.dp`.
  - `TextFieldDefaults.MinWidth = 280.dp`.
- Fret Material3 theme tokens:
  - `md.sys.typescale.body-large` resolves to a 24px line height in the current harness.

## Findings

This was not only a Material recipe bug.

Core mechanism gap:

- Declarative `TextAreaProps` had `min_height` but no `max_height`.
- Declarative `measure_text_area` measured `"M"` instead of the bound model text, so explicit
  multiline content could not drive layout.
- Bound/widget TextArea layout could not clamp measured height to a caller-provided maximum.

Material recipe gap:

- `TextField` had multiline mode but no Compose-aligned visible line-limit API.
- The multiline TextArea stayed fixed to the base 56px field height even when a caller wanted a
  taller visible editor.
- `maxLines` semantics had no Material chrome-height mapping.

## Changes

- Added `TextAreaProps::max_height` and propagated it through declarative host layout, paint,
  semantics, text event, and bound widget paths.
- Updated declarative TextArea measurement to observe and measure the bound string model, falling
  back to `"M"` only for empty content.
- Added `TextField::min_lines`, `TextField::max_lines`, and `TextField::line_limits`.
- Centralized TextField input style resolution so single-line and multiline branches share the same
  Material type intent mapping.
- Mapped multiline visible line limits to TextField container/TextArea min/max heights.
- Added red/green layout gates for multiline min-lines expansion and max-lines clamping.
- Refreshed TextField headless goldens for the current filled active-indicator layer split.

## Gates

Red before fix:

```powershell
cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_multiline_min_lines_expands_container_height
```

Failed with the multiline filled TextField chrome stuck at `56px`; expected `104px`.

Green:

```powershell
cargo fmt --package fret-ui --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_multiline_min_lines_expands_container_height text_field_multiline_max_lines_clamps_container_height
cargo nextest run -p fret-ui-material3 --test text_field_hover
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_text_field_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_text_field_suite_goldens_v1
cargo nextest run -p fret-ui --lib text_area_semantics_labelled_and_described_elements_are_exposed
cargo nextest run -p fret-ui --lib declarative_text_area_updates_model_on_text_input
cargo check -p fret-ui --lib
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
```

On Windows, a cold `fret-ui` lib-test relink was rerun with `CARGO_PROFILE_TEST_DEBUG=0` after a
standard-profile relink timeout corrupted incremental artifacts. The same two `fret-ui` filters
passed under that lower-debug test profile, while `cargo check -p fret-ui --lib` passed under the
normal profile.

## Residual Risk

- Soft-wrap-derived line counts are still approximate at the Material recipe layer. This packet
  handles explicit newline counts and clamps TextArea measurement, but exact soft-wrap visible-line
  ownership should move into richer TextArea intrinsic measurement when available.
- TextArea still has symmetric `padding_y`; exact asymmetric Material TextField top/bottom input
  padding can be split in a later TextArea style packet if visual evidence demands it.
- TextField motion remains open. This packet proves settled layout, not fixed-timestep transition
  behavior.
