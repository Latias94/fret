use super::super::*;

pub(in super::super) fn context_menu_size_at_zoom(
    style: &NodeGraphStyle,
    item_count: usize,
    zoom: f32,
) -> Size {
    let w = style.paint.context_menu_width / zoom;
    let item_h = style.paint.context_menu_item_height / zoom;
    let pad = style.paint.context_menu_padding / zoom;
    let h = (2.0 * pad + item_h * item_count.max(1) as f32).max(item_h + 2.0 * pad);
    Size::new(Px(w), Px(h))
}

pub(in super::super) fn context_menu_rect_at(
    style: &NodeGraphStyle,
    origin: Point,
    item_count: usize,
    zoom: f32,
) -> Rect {
    Rect::new(origin, context_menu_size_at_zoom(style, item_count, zoom))
}

pub(in super::super) fn hit_context_menu_item(
    style: &NodeGraphStyle,
    menu: &ContextMenuState,
    pos: Point,
    zoom: f32,
) -> Option<usize> {
    let rect = context_menu_rect_at(style, menu.origin, menu.items.len(), zoom);
    if !rect.contains(pos) {
        return None;
    }

    let pad = style.paint.context_menu_padding / zoom;
    let item_h = style.paint.context_menu_item_height / zoom;
    let inner_top = rect.origin.y.0 + pad;
    let y = pos.y.0 - inner_top;
    if y < 0.0 {
        return None;
    }

    let ix = (y / item_h).floor() as isize;
    if ix < 0 {
        return None;
    }
    let ix = ix as usize;
    (ix < menu.items.len()).then_some(ix)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_menu(item_count: usize) -> ContextMenuState {
        ContextMenuState {
            origin: Point::new(Px(10.0), Px(20.0)),
            invoked_at: Point::new(Px(10.0), Px(20.0)),
            target: ContextMenuTarget::Background,
            items: (0..item_count)
                .map(|ix| NodeGraphContextMenuItem {
                    label: std::sync::Arc::<str>::from(format!("Item {ix}")),
                    enabled: true,
                    action: NodeGraphContextMenuAction::Custom(ix as u64),
                })
                .collect(),
            candidates: Vec::new(),
            hovered_item: None,
            active_item: 0,
            typeahead: String::new(),
        }
    }

    #[test]
    fn context_menu_size_keeps_minimum_single_row_height() {
        let style = NodeGraphStyle::default();
        let size = context_menu_size_at_zoom(&style, 0, 1.0);
        assert_eq!(size.width, Px(style.paint.context_menu_width));
        assert_eq!(
            size.height,
            Px(style.paint.context_menu_item_height + 2.0 * style.paint.context_menu_padding)
        );
    }

    #[test]
    fn hit_context_menu_item_maps_pointer_to_item_index() {
        let style = NodeGraphStyle::default();
        let menu = test_menu(3);
        let hit = hit_context_menu_item(&style, &menu, Point::new(Px(15.0), Px(58.0)), 1.0);
        assert_eq!(hit, Some(1));
    }
}
