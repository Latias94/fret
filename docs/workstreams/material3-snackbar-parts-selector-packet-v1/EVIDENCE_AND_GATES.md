# Material 3 Snackbar Parts Selector Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-kit/src/window_overlays/render.rs`
- `ecosystem/fret-ui-kit/src/window_overlays/tests/toast.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_overlay_feedback_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Gates

Fresh gates for closeout:

```powershell
cargo fmt --package fret-ui-kit --package fret-ui-material3 -- --check
python -m json.tool docs\workstreams\material3-snackbar-parts-selector-packet-v1\WORKSTREAM.json
python -m json.tool docs\workstreams\material3-component-alignment-sweep-v1\artifacts\component_alignment_matrix_v1.json
python tools\check_workstream_catalog.py
cargo nextest run -p fret-ui-kit toast_action_cancel_and_close_test_ids_derive_from_root_test_id
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tooltip_and_snackbar_expose_stable_part_test_ids
cargo check -p fret-ui-kit --tests
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-kit --tests --no-deps -- -D warnings
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
git diff --check
```

## Residual Risk

This packet does not add message/supporting-text selectors for shared toasts. The previous Material3
follow-on named only action/close automation, and the current Material Snackbar recipe does not
expose a separate custom content slot that requires text-part targeting.
