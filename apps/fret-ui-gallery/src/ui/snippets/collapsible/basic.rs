pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret_core::{Point, Transform2D};
use fret_ui::Theme;
use fret_ui::element::VisualTransformProps;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn rotated_lucide<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &'static str,
    rotation_deg: f32,
) -> AnyElement {
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
            vec![shadcn::icon::icon_with(
                cx,
                fret_icons::IconId::new_static(id),
                Some(size),
                None,
            )]
        },
    )
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let basic_collapsible = shadcn::Collapsible::uncontrolled(false)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element_with_open_model(
            cx,
            |cx, open, is_open| {
                let chevron = rotated_lucide(
                    cx,
                    "lucide.chevron-down",
                    if is_open { 180.0 } else { 0.0 },
                );
                let row = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .justify_between()
                        .items_center(),
                    |cx| vec![cx.text("Product details"), chevron],
                );

                shadcn::Button::new("Product details")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .children([row])
                    .toggle_model(open)
                    .test_id("ui-gallery-collapsible-basic-trigger")
                    .into_element(cx)
            },
            |cx| {
                let body = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N2)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().min_w_0()),
                    |cx| {
                        vec![
                            shadcn::typography::p(
                                cx,
                                "This panel can be expanded or collapsed to reveal additional content.",
                            ),
                            shadcn::Button::new("Learn more")
                                .size(shadcn::ButtonSize::Sm)
                                .variant(shadcn::ButtonVariant::Secondary)
                                .into_element(cx),
                        ]
                    },
                );

                shadcn::CollapsibleContent::new([body])
                    .refine_style(ChromeRefinement::default().p(Space::N2p5).pt(Space::N0))
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
                    .test_id("ui-gallery-collapsible-basic-content")
            },
        )
        .test_id("ui-gallery-collapsible-basic");

    shadcn::Card::new([shadcn::CardContent::new([basic_collapsible]).into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx)
}
// endregion: example
