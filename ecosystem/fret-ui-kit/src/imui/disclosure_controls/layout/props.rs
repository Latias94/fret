use fret_core::Px;
use fret_ui::element::{
    ColumnProps, ContainerProps, LayoutStyle, Length, Overflow, SizeStyle, SpacingLength,
};

use super::super::{spec::DisclosureSpec, visual};

pub(super) fn disclosure_content_container_props(spec: &DisclosureSpec) -> ContainerProps {
    let mut props = ContainerProps::default();
    props.layout = fill_auto_visible_layout();
    props.padding = visual::disclosure_content_padding(spec).into();
    props
}

pub(super) fn disclosure_content_column_props() -> ColumnProps {
    disclosure_column_props()
}

pub(super) fn disclosure_root_column_props() -> ColumnProps {
    disclosure_column_props()
}

fn disclosure_column_props() -> ColumnProps {
    ColumnProps {
        layout: fill_auto_visible_layout(),
        gap: SpacingLength::Px(Px(0.0)),
        ..Default::default()
    }
}

fn fill_auto_visible_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Auto,
            ..Default::default()
        },
        overflow: Overflow::Visible,
        ..Default::default()
    }
}
