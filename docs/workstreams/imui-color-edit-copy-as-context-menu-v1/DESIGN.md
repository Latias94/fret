# ImUi Color Edit Copy-As Context Menu v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui exposes `Copy as..` from `ColorEditOptionsPopup()` so users can quickly copy the current
color as float tuple, integer tuple, RGB hex, or RGBA hex when alpha is visible. This lane adds the
same workflow to editor `ColorEdit` through a swatch context menu while preserving Fret's
effect-driven clipboard boundary.

## Ownership

- `ColorEditOptions::copy` is a per-control opt-out surface for context copy behavior.
- `popup/copy.rs` owns copy-as payload formatting, the context menu overlay, and clipboard effects.
- Clipboard writes go through `Effect::ClipboardWriteText`; no direct platform clipboard handle is
  exposed to editor controls.
- `crates/fret-ui`, `fret-imui`, and global `SetColorEditOptions()`-style state are not widened.

## Must-Be-True Outcomes

- Right-clicking the root swatch can open a compact context menu for copy-as actions.
- Shift+F10 and the ContextMenu key can open the same menu when the swatch is focusable.
- Copy payloads match Dear ImGui's payload set: float tuple, integer tuple, `#RRGGBB`, and
  `#RRGGBBAA` only when alpha is visible.
- Selecting a menu item emits `ClipboardWriteText` and closes the menu.
- Apps can disable the behavior per control through `ColorEditCopyOptions`.

## Non-Goals

- No eyedropper behavior.
- No platform clipboard contract changes.
- No nested shadcn context-menu recipe dependency from `fret-ui-editor`.
- No full picker thumbnail options popup polish.
