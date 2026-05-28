# Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/chip_set.rs`
- `ecosystem/fret-ui-material3/src/chip.rs`
- `ecosystem/fret-ui-material3/src/filter_chip.rs`
- `ecosystem/fret-ui-material3/src/input_chip.rs`
- `ecosystem/fret-ui-material3/src/suggestion_chip.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/state_matrix.rs`
- `docs/workstreams/material3-chip-visual-diagnostics-packet-v1/artifacts/chip_visual_diagnostics_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_choice_controls_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/samples/src/main/java/androidx/compose/material3/samples/ChipSamples.kt`
- `repo-ref/base-ui/packages/react/src/toolbar/root/ToolbarRoot.tsx`
- `repo-ref/base-ui/packages/react/src/internals/composite/root/useCompositeRoot.ts`

## Canonical Gates

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_segmented_buttons_and_chips_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment chip_set_roving_treats_trailing_action_focus_as_active_chip
python -m json.tool docs/workstreams/material3-chip-set-roving-packet-v1/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- The chip visual diagnostics packet remains supporting gallery evidence; this packet's focused
  closure gate is the roving handoff test.
- If `C:` temp space is exhausted, set `TEMP`/`TMP` to `target/tmp` before running cargo.
