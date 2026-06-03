//! TransformEdit column section chrome owner.

use std::sync::Arc;

use fret_core::{Axis, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::controls::{Checkbox, CheckboxOptions};
use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::readout::{
    editor_inline_control_label_text_props, editor_section_heading_text_props,
};

pub(in crate::controls::transform_edit) fn section_col<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let label_fg = editor_muted_foreground(theme);

    cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(4.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            vec![
                cx.text_props(editor_section_heading_text_props(
                    Arc::from(label),
                    label_fg,
                )),
                content(cx),
            ]
        },
    )
}

pub(in crate::controls::transform_edit) fn section_col_with_link<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    show_link: bool,
    linked_scale: Model<bool>,
    link_test_id: Option<Arc<str>>,
    content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    let mut col = section_col(cx, label, content);
    if show_link {
        let theme = Theme::global(&*cx.app);
        let label_fg = editor_muted_foreground(theme);

        col = cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: Axis::Vertical,
                gap: SpacingLength::Px(Px(4.0)),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |cx| {
                let mut out = vec![col];

                let mut el = Checkbox::new(linked_scale)
                    .options(CheckboxOptions {
                        a11y_label: Some(Arc::from("Link scale")),
                        focusable: true,
                        enabled: true,
                        ..Default::default()
                    })
                    .into_element(cx);
                if let Some(test_id) = link_test_id.as_ref() {
                    el = el.test_id(test_id.clone());
                }

                out.push(cx.flex(
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
                        gap: SpacingLength::Px(Px(4.0)),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        vec![
                            el,
                            cx.text_props(editor_inline_control_label_text_props(
                                Arc::from("Uniform"),
                                label_fg,
                                Px(10.0),
                                Px(12.0),
                            )),
                        ]
                    },
                ));

                out
            },
        );
    }
    col
}
