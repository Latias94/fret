# ImUi Kit Owner Split v1 - M2 Pressable Response Assembly Slice

Status: shared pressable response assembly helper landed
Date: 2026-05-13

## Result

- Shared `populate_pressable_response(...)` now owns the common pressable response assembly path
  for `active_trigger_behavior.rs`, `item_behavior.rs`, and `slider_controls.rs`.
- No public IMUI names changed.
- No `crates/fret-ui` runtime contract changed.
- `active_trigger_behavior.rs` and `item_behavior.rs` now keep only their unique event/signal
  writes; `slider_controls.rs` now uses the same private response core.

## Evidence

- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/pressable_response.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/active_trigger_behavior.rs`
- `ecosystem/fret-ui-kit/src/imui/item_behavior.rs`
- `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`

## Gates

- `cargo fmt --package fret-ui-kit -- --check`
- `cargo check -p fret-ui-kit --features imui`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast`
- `python tools/check_layering.py`
- `python tools/gate_imui_workstream_source.py`
- `git diff --check`
