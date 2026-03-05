pub const SOURCE: &str = include_str!("expandable.rs");

// region: example
use fret_app::App;
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign};
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    expandable_selected: Option<Model<Option<usize>>>,
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_xs = Px(320.0);

    let state = cx.with_state(Models::default, |st| st.clone());
    let expandable_selected = match state.expandable_selected {
        Some(model) => model,
        None => {
            let model: Model<Option<usize>> = cx.app.models_mut().insert(None);
            cx.with_state(Models::default, |st| {
                st.expandable_selected = Some(model.clone())
            });
            model
        }
    };
    let expandable_selected_now = cx
        .watch_model(&expandable_selected)
        .copied()
        .unwrap_or(None);

    let set_expandable_selected = |next: Option<usize>| {
        let expandable_selected = expandable_selected.clone();
        Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  _reason: fret_ui::action::ActivateReason| {
                let next = next;
                let _ = host
                    .models_mut()
                    .update(&expandable_selected, |cur| *cur = next);
                host.request_redraw(action_cx.window);
            },
        ) as fret_ui::action::OnActivate
    };

    let items = (1..=5)
        .map(|idx| {
            let expanded = expandable_selected_now == Some(idx);
            let height = if expanded { Px(260.0) } else { Px(140.0) };

            let theme = Theme::global(&*cx.app).clone();
            let gap = decl_style::space(&theme, Space::N2);

            let body = cx.flex(
                FlexProps {
                    layout: decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default().w_full().h_px(height),
                    ),
                    direction: fret_core::Axis::Vertical,
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    gap: gap.into(),
                    padding: Edges::all(Px(24.0)).into(),
                    ..Default::default()
                },
                move |cx| {
                    let theme = Theme::global(&*cx.app).snapshot();

                    let header = cx.flex(
                        FlexProps {
                            layout: decl_style::layout_style(
                                &theme,
                                LayoutRefinement::default().w_full(),
                            ),
                            direction: fret_core::Axis::Horizontal,
                            justify: MainAlign::SpaceBetween,
                            align: CrossAlign::Center,
                            wrap: false,
                            ..Default::default()
                        },
                        move |cx| {
                            let title = ui::text(format!("Item\u{00A0}{idx}"))
                                .text_base()
                                .font_semibold()
                                .nowrap()
                                .into_element(cx);
                            let toggle =
                                shadcn::Button::new(if expanded { "Collapse" } else { "Expand" })
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .test_id(format!(
                                        "ui-gallery-carousel-expandable-item-{}-toggle",
                                        idx
                                    ))
                                    .on_activate(set_expandable_selected(Some(idx)))
                                    .into_element(cx);
                            vec![title, toggle]
                        },
                    );

                    let mut out = vec![header];

                    if expanded {
                        out.push(ui::text("Expandable body").text_sm().into_element(cx));
                    }

                    out
                },
            );

            let card = shadcn::Card::new([body]).into_element(cx);
            ui::container(move |_cx| vec![card])
                .w_full()
                .p_1()
                .into_element(cx)
        })
        .map(shadcn::CarouselItem::new)
        .collect::<Vec<_>>();

    shadcn::Carousel::default()
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_xs)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-expandable")
        .into_element_parts(
            cx,
            |_cx| shadcn::CarouselContent::new(items),
            shadcn::CarouselPrevious::new(),
            shadcn::CarouselNext::new(),
        )
}
// endregion: example
