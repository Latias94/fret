# ImUi Debug Draw Cookbook Proof v1

Status: Closed narrow P1 teaching-surface follow-on
Last updated: 2026-05-05

The debug-draw feature slices added Dear ImGui-style shape, channel, triangle mesh, and metadata
capability to `fret-ui-kit::imui`, but the cookbook still lacked a public app-facing example. That
left a consumer gap: an app author could read the API from tests, but could not copy a runnable
`fret::imui` example that used the current facade.

## Ownership

- `apps/fret-cookbook` owns the runnable teaching surface.
- `apps/fretboard` owns feature-hint discoverability for `fretboard-dev dev native --example ...`.
- `fret::imui::kit` remains the public authoring facade for policy-heavy debug-draw helpers.
- `crates/*`, render backends, and `fret-imui` stay unchanged.

## Must-Be-True Outcomes

- `imui_debug_draw_basics` builds with `--features cookbook-imui`.
- The example imports debug-draw types through `fret::imui::{prelude::*, kit::*}`.
- The example exercises clip stack, channel split/merge, multi-color rects, triangle meshes,
  image triangle meshes, and source-level metadata summaries.
- Cookbook authoring tests pin the public teaching markers and reject raw internal imports or raw
  Dear ImGui buffer/callback vocabulary.
- The cookbook index and first-hour examples doc make the proof discoverable.

## Non-Goals

- No renderer callback / `ImDrawCallback` surface.
- No mutable raw command, vertex, or index buffer exposure.
- No asset pipeline tutorial; the image mesh proof uses `ImageId::default()` as an API proof.
- No changes to debug-draw paint behavior, clipping behavior, or scene contracts.
