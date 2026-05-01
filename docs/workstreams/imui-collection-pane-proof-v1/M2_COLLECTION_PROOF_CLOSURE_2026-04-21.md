# M2 Collection Proof Closure - 2026-04-21

Status: closed on 2026-04-21

## Decision

1. Keep `apps/fret-examples/src/imui_editor_proof_demo.rs` as the collection-first M2 proof surface.
2. Close M2 with an in-demo asset-browser/file-browser proof instead of a new dedicated demo.
3. Marquee / box-select stays deferred for M2.

## What changed

- `apps/fret-examples/src/imui_editor_proof_demo.rs` now carries a collection-first asset browser
  proof with:
  - stable `ui.id(...)` keyed tiles,
  - `ImUiMultiSelectState<Arc<str>>` multi-select over asset ids,
  - visible-order flipping without losing selection membership,
  - and selected-set drag/drop payloads delivered to an app-owned import target.
- `ecosystem/fret-imui/src/tests/interaction.rs` now proves selected collection drag payloads survive visible order flips.
- `tools/gate_imui_workstream_source.py` now locks the M2 collection proof markers with
  source-policy assertions.

## Why this closes M2

- The repo now has one first-party collection proof that goes beyond row-list multi-select.
- The box-select question now has an explicit lane-local answer: defer it until a narrower proof
  shows click / range / toggle selection is insufficient.
- Focused interaction proof now covers the collection-specific combination this lane needed:
  multi-select breadth, selected-set drag/drop, and keyed persistence across visible-order changes.

## Evidence

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `tools/gate_imui_workstream_source.py`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-ui-kit/src/imui/multi_select.rs`

## Gates

- `python tools/gate_imui_workstream_source.py`
- `cargo nextest run -p fret-imui collection_drag_payload_preserves_selected_keys_across_order_flip --no-fail-fast`
