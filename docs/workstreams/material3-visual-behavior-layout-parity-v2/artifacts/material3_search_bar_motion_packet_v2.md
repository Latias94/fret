# Material3 SearchBar Motion Packet v2

Date: 2026-05-28
Task: M3PV2-039

## Truth

- Ordinary SearchBar motion is input-field indication policy, not SearchView expansion motion.
- Compose Material3 `SearchBarDefaults.InputField` routes the text field interaction source into
  the input container indication. The state layer/ripple must cover the full rounded search field,
  including presses that begin over the editable text area.
- Default focused/unfocused SearchBar container colors are equal in Compose, so default container
  color animation is a no-op; the observable standalone motion packet is hover state-layer fade
  plus press ripple expansion.

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt`
  - `SearchBarDefaults.InputField` applies `Modifier.sizeIn(..., minHeight = InputFieldHeight)`.
  - The `BasicTextField` decorator provides a `container` that applies
    `Modifier.textFieldBackground(...).then(Modifier.indication(interactionSource, ripple(...)))`.
  - Default `inputFieldColors` resolve focused and unfocused container colors to
    `SearchBarTokens.ContainerColor`.

## Artifacts

- `ecosystem/fret-ui-material3/src/foundation/indication.rs`
- `ecosystem/fret-ui-material3/src/search_bar.rs`
- `ecosystem/fret-ui-material3/tests/search_bar_motion.rs`

## Wiring

- Added `material_ink_layer_for_pressable_with_last_down` so a policy component can provide a
  captured pointer-down origin when the press begins in a descendant text input.
- SearchBar now keeps a component-local pointer-down interaction cell from its full-size pointer
  region and feeds it into the shared Material ink runtime.
- SearchBar chrome was split into:
  - an outer full-size rounded container that owns background, focus ring, state layer, and ripple;
  - an inner padded content layer for the row/text input/icons.
- This fixes the previous padding leak where the state layer and ripple only covered the content
  box instead of the full 56px rounded SearchBar container.

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --test search_bar_motion
```

It failed because hover state-layer quads were inset to the padded content rect
(`x = 48px`, `w = 688px`) instead of the full chrome rect (`x = 32px`, `w = 720px`), and pressing
the editable text area did not start a SearchBar ripple.

Green gate:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test search_bar_motion
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids material3_search_view_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_bar_suite_goldens_v1 material3_headless_search_view_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --test search_view_behavior
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json
python tools/check_workstream_catalog.py
git diff --check
```

## Matrix Impact

- `search_bar.motion`: `covered_v2`.
- SearchView motion remains separately covered by `material3_search_view_motion_packet_v2.md`.

## Residual Risk

- This packet proves ordinary standalone SearchBar indication motion. It does not add predictive
  back or SearchView expansion behavior.
- Default container color animation is documented as a no-op because upstream focused/unfocused
  default colors match; custom color overrides would need a future style-override packet.
