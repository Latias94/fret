# ImUi Debug Draw Channel Split v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui's `ImDrawList::ChannelsSplit`, `ChannelsSetCurrent`, and `ChannelsMerge` let callers
submit primitives in one order and later flatten them by channel order. This is useful for custom
debug overlays that discover foreground primitives before background primitives or that want to
avoid repeatedly switching clip/texture state.

Fret already has a declarative scene order. This lane only adds an IMUI debug draw ordering helper:
channels are buffered in `fret-ui-kit::imui` and flattened before the `Canvas` painter lowers
commands into scene ops.

## Ownership

- `fret-ui-kit::imui` owns the channel split facade and command buffering.
- `crates/fret-core`, `crates/fret-ui`, `fret-imui`, and the WGPU renderer are not widened.
- Scene order remains the renderer-facing contract after channels are merged.

## Must-Be-True Outcomes

- `channels_split(count)` starts a non-nested split for `count > 1`.
- `channels_set_current(index)` switches the active command buffer when the index is valid.
- Invalid switches are ignored rather than panicking in app code.
- `channels_merge()` flattens channel 0, channel 1, and so on into the final command stream.
- Leaving a split open at the end of `debug_draw(...)` still auto-merges before painting.

## Non-Goals

- No nested channel split stack.
- No raw `ImDrawCmd` metadata or callback command support.
- No renderer batching changes.
- No attempt to expose Dear ImGui's raw vertex/index buffers.
