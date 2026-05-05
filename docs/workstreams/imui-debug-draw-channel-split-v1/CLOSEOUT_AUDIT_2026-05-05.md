# ImUi Debug Draw Channel Split v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Added `channels_split`, `channels_set_current`, and `channels_merge` to `ImUiDebugDrawList`.
- Implemented channel buffering by swapping the active `commands` vector with saved channel buffers.
- Flattened merged channels in channel order before Canvas painting.
- Auto-merged an open split at the end of `debug_draw(...)`.
- Kept the feature entirely in `fret-ui-kit::imui`; no renderer or runtime contract changed.

## Evidence

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_draw.cpp`

## Gates Run

```bash
cargo nextest run -p fret-ui-kit --features imui debug_draw_channels --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_list_records --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_options_default_to_clipped_canvas --no-fail-fast
```

## Residual Gaps

- Callback/user draw commands.
- Raw mesh/primitive writer API.
- Per-command metadata beyond the existing high-level command enum.
