pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Point, Transform2D};
use fret_ui::Theme;
use fret_ui::element::VisualTransformProps;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn rotated_lucide<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &'static str,
    rotation_deg: f32,
) -> impl IntoUiElement<H> + use<H> {
    let size = Px(16.0);
    let center = Point::new(Px(8.0), Px(8.0));
    let transform = Transform2D::rotation_about_degrees(rotation_deg, center);

    cx.visual_transform_props(
        VisualTransformProps {
            layout: {
                let theme = Theme::global(&*cx.app);
                decl_style::layout_style(
                    theme,
                    LayoutRefinement::default()
                        .w_px(size)
                        .h_px(size)
                        .flex_shrink_0(),
                )
            },
            transform,
        },
        move |cx| {
            vec![icon::icon_with(
                cx,
                fret_icons::IconId::new_static(id),
                Some(size),
                None,
            )]
        },
    )
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    cx.scope(|cx| {
        let open = cx.local_model_keyed("basic_open", || false);
        let is_open = cx.watch_model(&open).layout().copied().unwrap_or(false);
        let snapshot = Theme::global(&*cx.app).snapshot();
        let chrome = if is_open {
            ChromeRefinement::default()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(snapshot.color_token("muted")))
        } else {
            ChromeRefinement::default().rounded(Radius::Md)
        };

        let props = decl_style::container_props(
            &snapshot,
            chrome,
            LayoutRefinement::default().w_full().min_w_0(),
        );

        let basic_collapsible = cx
            .container(props, |cx| {
                vec![
                    shadcn::Collapsible::new(open.clone())
                        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element_with_open_model(
                            cx,
                            |cx, open, is_open| {
                                let chevron = rotated_lucide(
                                    cx,
                                    "lucide.chevron-down",
                                    if is_open { 180.0 } else { 0.0 },
                                )
                                .into_element(cx);
                                let row = ui::h_flex(|cx| vec![cx.text("Product details"), chevron])
                                    .layout(LayoutRefinement::default().w_full().min_w_0())
                                    .justify_between()
                                    .items_center()
                                    .into_element(cx);

                                shadcn::Button::new("Product details")
                                    .variant(shadcn::ButtonVariant::Ghost)
                                    .refine_layout(LayoutRefinement::default().w_full())
                                    .children([row])
                                    .toggle_model(open)
                                    .test_id("ui-gallery-collapsible-basic-trigger")
                                    .into_element(cx)
                            },
                            |cx| {
                                let body = ui::v_flex(|cx| {
                                    vec![
                                        shadcn::raw::typography::p(
                                            "This panel can be expanded or collapsed to reveal additional content.",
                                        )
                                        .into_element(cx),
                                        shadcn::Button::new("Learn More")
                                            .size(shadcn::ButtonSize::Xs)
                                            .into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .items_start()
                                .layout(LayoutRefinement::default().w_full().min_w_0())
                                .into_element(cx);

                                shadcn::CollapsibleContent::new([body])
                                    .refine_style(
                                        ChromeRefinement::default().p(Space::N2p5).pt(Space::N0),
                                    )
                                    .refine_layout(LayoutRefinement::default().w_full())
                                    .into_element(cx)
                                    .test_id("ui-gallery-collapsible-basic-content")
                            },
                        )
                ]
            })
            .test_id("ui-gallery-collapsible-basic");

        shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_content(|cx| ui::children![cx; basic_collapsible]),
            ]
        })
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx)
    })
}
// endregion: example
