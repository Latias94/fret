use std::sync::Arc;

use fret_core::{Axis, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, MarginEdge, SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::colors::editor_invalid_foreground;
use crate::primitives::readout::editor_status_badge_text_props;

pub(super) struct ColorEditRootLayoutArgs {
    pub(super) swatch: AnyElement,
    pub(super) input: AnyElement,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) layout: LayoutStyle,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) row_height: Px,
    pub(super) control_height: Px,
}

pub(super) fn color_edit_root_layout<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: ColorEditRootLayoutArgs,
) -> AnyElement {
    let ColorEditRootLayoutArgs {
        swatch,
        input,
        error,
        mut layout,
        test_id,
        row_height,
        control_height,
    } = args;

    let error_msg = cx
        .get_model_cloned(&error, Invalidation::Paint)
        .unwrap_or(None);
    let error_el = error_msg.as_ref().map(|msg| {
        let theme = Theme::global(&*cx.app);
        let invalid_fg = editor_invalid_foreground(theme);
        let mut props = editor_status_badge_text_props(msg.clone(), invalid_fg, row_height);
        props.layout.margin.left = MarginEdge::Px(Px(4.0));
        cx.text_props(props).a11y_label(msg.clone())
    });

    if layout.size.min_height.is_none() {
        layout.size.min_height = Some(Length::Px(control_height));
    }

    let row_height = if matches!(layout.size.height, Length::Auto) {
        Length::Px(control_height)
    } else {
        layout.size.height
    };

    if error_msg.is_none() && matches!(layout.size.height, Length::Auto) {
        layout.size.height = row_height;
    }

    let mut row_layout = layout;
    row_layout.size.width = Length::Fill;
    row_layout.size.height = row_height;

    let row = cx.flex(
        FlexProps {
            layout: row_layout,
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(8.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |_cx| vec![swatch, input],
    );

    let mut el = if let Some(error_el) = error_el {
        cx.flex(
            FlexProps {
                layout,
                direction: Axis::Vertical,
                gap: SpacingLength::Px(Px(4.0)),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| vec![row, error_el],
        )
    } else {
        row
    };

    if let Some(test_id) = test_id.as_ref() {
        el = el.test_id(test_id.clone());
    }
    el
}
