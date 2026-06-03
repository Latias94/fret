//! TransformEdit row section chrome owner.

use std::sync::Arc;

use fret_core::{Axis, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::controls::{Checkbox, CheckboxOptions};
use crate::primitives::EditorDensity;
use crate::primitives::colors::{editor_border, editor_muted_foreground, editor_subtle_bg};
use crate::primitives::readout::{
    editor_inline_control_label_text_props, editor_section_badge_text_props,
};

pub(in crate::controls::transform_edit) fn section_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    short: &'static str,
    a11y: &'static str,
    show_link: bool,
    link: Option<(Model<bool>, Option<Arc<str>>)>,
    content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let label_fg = editor_muted_foreground(theme);
    let badge_bg = editor_subtle_bg(theme);
    let badge_border = editor_border(theme);
    let badge_w = Px(density.row_height.0.max(density.hit_thickness.0));

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
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(6.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |cx| {
            let mut out = Vec::new();
            out.push(cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(badge_w),
                            height: Length::Px(density.row_height),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    background: Some(badge_bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(badge_border),
                    corner_radii: Corners::all(Px(4.0)),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.text_props(editor_section_badge_text_props(
                        Arc::from(short),
                        label_fg,
                        density.row_height,
                    ))]
                },
            ));

            if show_link && let Some((linked, test_id)) = link {
                let mut el = Checkbox::new(linked)
                    .options(CheckboxOptions {
                        a11y_label: Some(Arc::from(a11y)),
                        focusable: true,
                        enabled: true,
                        ..Default::default()
                    })
                    .into_element(cx);
                if let Some(test_id) = test_id.as_ref() {
                    el = el.test_id(test_id.clone());
                }
                out.push(cx.flex(
                    FlexProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Auto,
                                height: Length::Px(density.row_height),
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
                                Arc::from("Link"),
                                label_fg,
                                Px(11.0),
                                density.row_height,
                            )),
                        ]
                    },
                ));
            }

            out.push(content(cx));
            out
        },
    )
}
