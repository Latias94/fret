# Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/radio.rs`
- `ecosystem/fret-ui-material3/src/foundation/indication.rs`
- `ecosystem/fret-ui-material3/src/foundation/interactive_size.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_choice_controls_packet_v1.md`
- `docs/workstreams/material3-switch-diagnostics-packet-v1/artifacts/switch_diagnostics_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/RadioButton.kt`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/RadioButtonTokens.kt`
- `repo-ref/base-ui/packages/react/src/radio-group/RadioGroup.tsx`
- `repo-ref/base-ui/packages/react/src/radio/root/RadioRoot.tsx`

## Canonical Gates

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_choice_controls_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment radio_selected_dot_is_centered_in_outline radio_ripple_origin_tracks_pointer_down_position radio_pressed_scene_structure_is_stable
python -m json.tool docs/workstreams/material3-radio-scene-semantics-packet-v1/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- The focused Rust gates cover the current Radio closure claim.
- This packet intentionally does not add a Radio gallery diagnostics script.
- If `C:` temp space is exhausted, set `TEMP`/`TMP` to `target/tmp` before running cargo.
