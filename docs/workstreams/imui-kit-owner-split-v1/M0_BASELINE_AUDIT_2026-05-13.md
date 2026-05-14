# M0 Baseline Audit - 2026-05-13

Status: active baseline

## Scope

This audit establishes the first structural target for `imui-kit-owner-split-v1`. It is source
evidence for a private owner split, not a behavior or API-change proposal.

## Current Source Snapshot

`ecosystem/fret-ui-kit/src/imui` currently has several large policy files. The largest source files
by line count at lane start are:

| File | Lines | Read |
| --- | ---: | --- |
| `facade_writer.rs` | 1801 | Largest remaining public facade owner; mixes facade methods, private glue, and response assembly. |
| `debug_draw_controls.rs` | 1263 | Already split by `imui-debug-draw-owner-split-v1`; remaining size is accepted for the closed lane. |
| `floating_window_on_area.rs` | 817 | Specialized floating-window owner. |
| `table_controls.rs` | 814 | Specialized table owner. |
| `menu_family_controls.rs` | 808 | Specialized menu family owner. |
| `disclosure_controls.rs` | 699 | Specialized disclosure/tree/collapsing owner. |
| `text_controls.rs` | 597 | Specialized text input and textarea owner. |
| `popup_overlay.rs` | 512 | Specialized popup overlay owner. |

The first structural target is therefore `facade_writer.rs`, not because file size alone is a bug,
but because it is both the app-facing method hub and a mixing point for unrelated policy glue.

## Source-Backed Facts

- `fret-imui` remains policy-light and only exports the minimal immediate authoring frontend.
- `fret-ui-kit::imui` owns the policy-heavy widget, response, options, floating, table, text,
  drag/drop, and debug draw surface.
- `fret::imui` is the app-facing optional lane; it re-exports `kit`, `editor`, and `docking`
  submodules.
- `fret-ui-editor::imui` is a thin adapter over declarative editor controls and is not the target
  for this structural split.
- The debug draw owner split is already closed and should not be reopened for generic cleanup.

## Hazards

1. **Public facade hub pressure**
   - `facade_writer.rs` must continue to carry the public `ImUiFacade` methods, but private
     implementation glue does not all need to live there.
   - Risk: moving too much at once can make public API review difficult.

2. **Transient response signal sprawl**
   - `facade_support.rs`, `interaction_runtime/*`, and `response/hover.rs` collectively define the
     response-status path.
   - Risk: future Dear ImGui-style status expansion becomes hard to audit if the private signal
     path stays scattered.

3. **Canonical helper deletion risk**
   - `*_with_options(...)` helpers are canonical explicit-options entry points, not duplicate
     aliases.
   - Risk: over-aggressive deletion would remove valid public API.

4. **Wrong-layer refactor risk**
   - Runtime mechanisms belong in `crates/fret-ui`; policy belongs here.
   - Risk: moving behavior down to runtime would violate ADR 0066.

## First Recommended Slice

Start with a private facade support split:

1. Identify clusters in `facade_writer.rs` where public methods are thin wrappers around repeated
   private construction logic.
2. Move one cluster behind a private owner module while keeping public `ImUiFacade` method names
   and behavior unchanged.
3. Run focused `fret-ui-kit` IMUI smoke gates.
4. Record the result as `M1_*_SLICE_YYYY-MM-DD.md`.

The likely first cluster is command/action button and menu item glue or response/status assembly.
Pick the smaller cluster after reading the code in detail.

## Not First

- Do not start by adding list-box, image item, table advanced flags, or child-region resize.
- Do not reopen debug draw owner split.
- Do not add porting sugar.
- Do not move policy into `fret-imui`.

## Baseline Gates

```bash
python -m json.tool docs/workstreams/imui-kit-owner-split-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
python tools/gate_imui_facade_teaching_source.py
python tools/gate_imui_workstream_source.py
cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast
cargo check -p fret-ui-kit --features imui
cargo fmt --package fret-ui-kit -- --check
git diff --check
```
