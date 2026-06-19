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
    let heading = cx.text_props(editor_section_heading_text_props(
        Arc::from(label),
        label_fg,
    ));
    let body = content(cx);

    section_col_shell(cx, heading, vec![body])
}

pub(in crate::controls::transform_edit) fn section_col_with_link<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    show_link: bool,
    linked_scale: Model<bool>,
    link_test_id: Option<Arc<str>>,
    content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let label_fg = editor_muted_foreground(theme);
    let heading = cx.text_props(editor_section_heading_text_props(
        Arc::from(label),
        label_fg,
    ));
    let body = content(cx);

    if show_link {
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

        let link_row = cx.flex(
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
        );

        section_col_shell(cx, heading, vec![body, link_row])
    } else {
        section_col_shell(cx, heading, vec![body])
    }
}

fn section_col_shell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    heading: AnyElement,
    body_children: Vec<AnyElement>,
) -> AnyElement {
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
        move |_cx| {
            let mut out = Vec::with_capacity(1 + body_children.len());
            out.push(heading);
            out.extend(body_children);
            out
        },
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_runtime::Model;
    use fret_ui::elements::with_element_cx;

    use super::section_col_with_link;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        )
    }

    #[test]
    fn section_col_with_link_keeps_link_row_inside_one_column_shell() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let linked_scale: Model<bool> = app.models_mut().insert(false);

        let element = with_element_cx(
            &mut app,
            window,
            bounds(),
            "transform-edit-section-col",
            |cx| {
                section_col_with_link(
                    cx,
                    "Scale",
                    true,
                    linked_scale.clone(),
                    Some(Arc::from("transform.edit.link-scale")),
                    |cx| cx.text("Scale content"),
                )
            },
        );

        assert_eq!(element.children.len(), 3);
        assert_eq!(element.children[2].children.len(), 2);
    }
}
