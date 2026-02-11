use fret_app::App;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub(super) fn maybe_push_debug_hud(
    cx: &mut ElementContext<'_, App>,
    theme: Theme,
    show_debug_hud: bool,
    debug_hud_lines: Vec<Arc<str>>,
    content: &mut Vec<AnyElement>,
) {
    if !show_debug_hud {
        return;
    }

    content.push(cx.keyed("ui_gallery.debug_hud", |cx| {
        let hud_layout = fret_ui::element::LayoutStyle {
            position: fret_ui::element::PositionStyle::Absolute,
            inset: fret_ui::element::InsetStyle {
                top: Some(Px(8.0)),
                right: Some(Px(8.0)),
                ..Default::default()
            },
            size: fret_ui::element::SizeStyle {
                width: fret_ui::element::Length::Px(Px(520.0)),
                height: fret_ui::element::Length::Px(Px(220.0)),
                ..Default::default()
            },
            ..Default::default()
        };

        let gate = fret_ui::element::InteractivityGateProps {
            layout: hud_layout,
            present: true,
            interactive: false,
        };

        cx.interactivity_gate_props(gate, |cx| {
            let mut container_props = decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(theme.color_required("background")))
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N3),
                LayoutRefinement::default().w_full().h_full(),
            );
            container_props.layout.size.width = fret_ui::element::Length::Fill;
            container_props.layout.size.height = fret_ui::element::Length::Fill;
            container_props.layout.overflow = fret_ui::element::Overflow::Clip;

            let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N1),
                |cx| {
                    debug_hud_lines
                        .iter()
                        .map(|line| {
                            cx.text_props(TextProps {
                                layout: Default::default(),
                                text: line.clone(),
                                style: None,
                                color: Some(theme.color_required("foreground")),
                                wrap: TextWrap::Word,
                                overflow: TextOverflow::Clip,
                            })
                        })
                        .collect::<Vec<_>>()
                },
            );

            [cx.container(container_props, |cx| {
                [shadcn::ScrollArea::new([body])
                    .refine_layout(LayoutRefinement::default().w_full().h_full())
                    .into_element(cx)]
            })]
        })
    }));
}
