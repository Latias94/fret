# M0 Baseline Audit - 2026-05-14

Status: baseline audit closed

## Source Read

`ecosystem/fret-ui-kit/src/imui/facade_writer.rs` still carried policy-heavy trait default bodies
for:

- floating layers and floating areas,
- popup open/close and popup menu/modal rendering entry points,
- tooltip entry points,
- typed drag/drop entry points,
- context popup entry points,
- in-window floating windows.

The surrounding implementation owners already existed in narrower modules:

- `ecosystem/fret-ui-kit/src/imui/floating_surface.rs`
- `ecosystem/fret-ui-kit/src/imui/floating_window.rs`
- `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
- `ecosystem/fret-ui-kit/src/imui/tooltip_overlay.rs`
- `ecosystem/fret-ui-kit/src/imui/drag_drop.rs`

## Decision

This is a private owner split, not a public API widening. Rust requires the public trait methods to
stay declared together in `facade_writer.rs`, so the correct split is to move method bodies into a
private `facade_writer/floating_popup.rs` owner and leave the trait as thin forwarding glue.

## Non-Goals

- Do not add list-box, image, plot, or advanced table helpers from this lane.
- Do not copy Dear ImGui begin/end mutable grammar.
- Do not move popup/floating policy into `fret-imui`.
- Do not widen `crates/fret-ui`.
