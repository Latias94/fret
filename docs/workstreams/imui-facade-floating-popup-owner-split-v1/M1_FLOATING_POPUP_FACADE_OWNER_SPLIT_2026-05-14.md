# M1 Floating Popup Facade Owner Split - 2026-05-14

Status: floating/popup facade owner split landed

## Change

Moved the floating/popup/tooltip/drag-drop/window trait default implementation bodies into:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup.rs`

`facade_writer.rs` now declares `mod floating_popup;` and keeps `UiWriterImUiFacadeExt` as the
public trait hub with thin forwarding methods.

## Evidence

| File | Role |
| --- | --- |
| `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` | public trait hub and thin forwarders |
| `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup.rs` | private floating/popup default-body owner |
| `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs` | popup policy implementation |
| `ecosystem/fret-ui-kit/src/imui/tooltip_overlay.rs` | tooltip policy implementation |
| `ecosystem/fret-ui-kit/src/imui/drag_drop.rs` | typed drag/drop policy implementation |
| `ecosystem/fret-ui-kit/src/imui/floating_surface.rs` | floating area/layer substrate |
| `ecosystem/fret-ui-kit/src/imui/floating_window.rs` | in-window floating-window wrapper |

## Contract Check

- Public trait method names and signatures remain unchanged.
- No `fret::imui` re-export path changed.
- No `fret-imui` dependency or public surface changed.
- No `crates/fret-ui` runtime contract changed.
