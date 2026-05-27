use fret_core::Px;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, LayoutStyle, Length, Overflow, SizeStyle,
    SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use super::{spec::DisclosureSpec, visual};
use crate::imui::ImUiFacade;

pub(super) fn disclosure_content_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    spec: &DisclosureSpec,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let mut build = Some(f);
    let mut content = cx.named("content", |cx| {
        let mut props = ContainerProps::default();
        props.layout = LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                ..Default::default()
            },
            overflow: Overflow::Visible,
            ..Default::default()
        };
        props.padding = visual::disclosure_content_padding(spec).into();

        cx.container(props, move |cx| {
            vec![cx.column(
                ColumnProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        overflow: Overflow::Visible,
                        ..Default::default()
                    },
                    gap: SpacingLength::Px(Px(0.0)),
                    ..Default::default()
                },
                move |cx| {
                    let mut out = Vec::new();
                    let mut body_ui = ImUiFacade {
                        cx,
                        out: &mut out,
                        build_focus: None,
                    };
                    if let Some(build) = build.take() {
                        build(&mut body_ui);
                    }
                    out
                },
            )]
        })
    });
    if let Some(test_id) = spec.content_test_id.as_ref() {
        content = content.test_id(test_id.clone());
    }
    content
}

pub(super) fn disclosure_root_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    spec: &DisclosureSpec,
    root_children: Vec<AnyElement>,
) -> AnyElement {
    let mut root = cx.column(
        ColumnProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                overflow: Overflow::Visible,
                ..Default::default()
            },
            gap: SpacingLength::Px(Px(0.0)),
            ..Default::default()
        },
        move |_cx| root_children,
    );
    if let Some(test_id) = spec.root_test_id.as_ref() {
        root = root.test_id(test_id.clone());
    }
    root
}
