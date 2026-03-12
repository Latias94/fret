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
