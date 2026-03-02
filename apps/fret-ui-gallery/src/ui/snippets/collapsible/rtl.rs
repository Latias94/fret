// region: example
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
}

fn open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
            model
        }
    }
}

fn details_collapsible<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    test_id_prefix: &'static str,
    open: Option<Model<bool>>,
    label: &'static str,
    status: &'static str,
) -> AnyElement {
    let container_props =
        |cx: &mut ElementContext<'_, H>, chrome: ChromeRefinement, layout: LayoutRefinement| {
            cx.with_theme(|theme| decl_style::container_props(theme, chrome, layout))
        };

    let details_content = |cx: &mut ElementContext<'_, H>| {
        shadcn::CollapsibleContent::new(vec![
            {
                let props = container_props(
                    cx,
                    ChromeRefinement::default()
                        .border_1()
                        .rounded(Radius::Sm)
                        .px(Space::N4)
                        .py(Space::N2),
                    LayoutRefinement::default().w_full(),
                );
                cx.container(props, |cx| {
                    vec![stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .justify_between(),
                        |cx| {
                            vec![
                                shadcn::typography::muted(cx, "Shipping address"),
                                cx.text("100 Market St, San Francisco"),
                            ]
                        },
                    )]
                })
            },
            {
                let props = container_props(
                    cx,
                    ChromeRefinement::default()
                        .border_1()
                        .rounded(Radius::Sm)
                        .px(Space::N4)
                        .py(Space::N2),
                    LayoutRefinement::default().w_full(),
                );
                cx.container(props, |cx| {
                    vec![stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .justify_between(),
                        |cx| {
                            vec![
                                shadcn::typography::muted(cx, "Items"),
                                cx.text("2x Studio Headphones"),
                            ]
                        },
                    )]
                })
            },
        ])
        .refine_layout(LayoutRefinement::default().w_full().mt(Space::N2))
        .into_element(cx)
        .test_id(format!("{test_id_prefix}-content"))
    };

    let collapsible = match open {
        Some(open_model) => shadcn::Collapsible::new(open_model)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element_with_open_model(
                cx,
                |cx, open, _is_open| {
                    shadcn::Button::new("")
                        .a11y_label("Toggle")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Icon)
                        .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
                        .icon(fret_icons::IconId::new_static("lucide.chevrons-up-down"))
                        .toggle_model(open)
                        .test_id(format!("{test_id_prefix}-trigger"))
                        .into_element(cx)
                },
                |cx| details_content(cx),
            ),
        None => shadcn::Collapsible::uncontrolled(false)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element_with_open_model(
                cx,
                |cx, open, _is_open| {
                    shadcn::Button::new("")
                        .a11y_label("Toggle")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Icon)
                        .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
                        .icon(fret_icons::IconId::new_static("lucide.chevrons-up-down"))
                        .toggle_model(open)
                        .test_id(format!("{test_id_prefix}-trigger"))
                        .into_element(cx)
                },
                |cx| details_content(cx),
            ),
    };

    let wrapper_props = container_props(
        cx,
        ChromeRefinement::default().px(Space::N3).py(Space::N2),
        LayoutRefinement::default().w_full(),
    );
    cx.container(wrapper_props, |cx| {
        vec![stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_stretch()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| {
                vec![
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .justify_between(),
                        |cx| vec![cx.text(label), cx.text(status)],
                    ),
                    collapsible,
                ]
            },
        )]
    })
    .test_id(test_id_prefix)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = open_model(cx);
    fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            details_collapsible(
                cx,
                "ui-gallery-collapsible-rtl",
                Some(open.clone()),
                "Order #4189",
                "Shipped",
            )
        },
    )
    .test_id("ui-gallery-collapsible-rtl")
}
// endregion: example
