use fret_core::{Edges, Px};
use fret_ui::element::{ContainerProps, Length, RowProps, SpacingLength};

pub(super) fn menu_item_panel_props() -> ContainerProps {
    let mut panel = ContainerProps::default();
    panel.layout.size.width = Length::Fill;
    panel.layout.size.height = Length::Auto;
    panel.padding = Edges {
        left: Px(6.0),
        right: Px(6.0),
        top: Px(2.0),
        bottom: Px(2.0),
    }
    .into();
    panel
}

pub(super) fn menu_item_row_props() -> RowProps {
    let mut row = RowProps::default();
    row.layout.size.width = Length::Fill;
    row.layout.size.height = Length::Auto;
    row.gap = SpacingLength::Px(Px(6.0));
    row
}
