# Token groups (high leverage)

This is a “what to tweak” cheat sheet for theme-based styling in Fret.

## Density

- `metric.padding.sm`, `metric.padding.md`
- `component.space.*` (optional; `Space::*` falls back to padding tokens)
- `component.size.md.input.h`, `.px`, `.py`
- `component.size.md.button.h`, `component.size.sm.button.h`, `component.size.lg.button.h`
- `component.table.row_min_h`
- `component.list.row_height`
- `metric.scrollbar.width`

## Radius

- `metric.radius.sm|md|lg`
- `component.radius.sm|md|lg|full`

## Elevation (shadows)

- `shadow` (color)
- `component.shadow.{xs,sm,md,lg,xl}.offset_x|offset_y|spread|softness`

## Focus ring

- `ring`, `ring-offset-background`
- `component.ring.width`, `component.ring.offset`

## Typography

- `metric.font.size`, `metric.font.line_height`
- `component.text.sm_px`, `component.text.sm_line_height`
- `component.text.base_px`, `component.text.base_line_height`

## Component-specific tuning (only when needed)

- NavigationMenu: `component.navigation_menu.viewport.side_offset`
- HoverCard: `component.hover_card.side_offset`, `component.hover_card.window_margin`
- Slider: `component.slider.track_height`, `component.slider.thumb_size`
