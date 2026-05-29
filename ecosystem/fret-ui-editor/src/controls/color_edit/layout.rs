use std::sync::Arc;

use fret_core::{Axis, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::readout::editor_inline_error_text_props;

pub(super) struct ColorEditRootLayoutArgs {
    pub(super) swatch: AnyElement,
    pub(super) input: AnyElement,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) layout: LayoutStyle,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) row_height: Px,
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
    } = args;

    let error_msg = cx
        .get_model_cloned(&error, Invalidation::Paint)
        .unwrap_or(None);
    let error_el = error_msg.map(|msg| {
        cx.text_props(editor_inline_error_text_props(
            msg,
            Theme::global(&*cx.app).color_token("destructive"),
            row_height,
        ))
    });

    if layout.size.min_height.is_none() {
        layout.size.min_height = Some(Length::Px(row_height));
    }

    let mut el = cx.flex(
        FlexProps {
            layout,
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(4.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            let row = cx.flex(
                FlexProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    direction: Axis::Horizontal,
                    gap: SpacingLength::Px(Px(8.0)),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| vec![swatch, input],
            );

            let mut out = vec![row];
            if let Some(err) = error_el {
                out.push(err);
            }
            out
        },
    );

    if let Some(test_id) = test_id.as_ref() {
        el = el.test_id(test_id.clone());
    }
    el
}
