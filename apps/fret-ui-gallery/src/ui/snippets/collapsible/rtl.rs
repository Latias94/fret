pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui::Theme;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn detail_card<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    test_id: &'static str,
    title: &'static str,
    detail: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let theme = Theme::global(&*cx.app).snapshot();
    let props = decl_style::container_props(
        &theme,
        ChromeRefinement::default()
            .border_1()
            .rounded(Radius::Md)
            .px(Space::N4)
            .py(Space::N2),
        LayoutRefinement::default().w_full().min_w_0(),
    );

    cx.container(props, move |cx| {
        vec![
            ui::v_flex(|cx| {
                vec![
                    shadcn::raw::typography::small(title).into_element(cx),
                    shadcn::raw::typography::muted(detail).into_element(cx),
                ]
            })
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx),
        ]
    })
    .test_id(test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("rtl_open", || false);

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Collapsible::new(open.clone())
            .refine_layout(LayoutRefinement::default().w_px(Px(350.0)).min_w_0())
            .into_element_with_open_model(
                cx,
                |cx, open, _is_open| {
                    let title = shadcn::raw::typography::small("الطلب #4189").into_element(cx);
                    let button = shadcn::Button::new("")
                        .a11y_label("Toggle details")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Icon)
                        .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
                        .icon(fret_icons::IconId::new_static("lucide.chevrons-up-down"))
                        .toggle_model(open)
                        .test_id("ui-gallery-collapsible-rtl-trigger")
                        .into_element(cx);

                    let header = {
                        let row = ui::h_flex(move |_cx| vec![title, button])
                            .gap(Space::N4)
                            .items_center()
                            .justify_between()
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .into_element(cx);
                        let theme = Theme::global(&*cx.app).snapshot();
                        let props = decl_style::container_props(
                            &theme,
                            ChromeRefinement::default().px(Space::N4),
                            LayoutRefinement::default().w_full().min_w_0(),
                        );
                        cx.container(props, move |_cx| vec![row])
                    };

                    let status = {
                        let theme = Theme::global(&*cx.app).snapshot();
                        let props = decl_style::container_props(
                            &theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .px(Space::N4)
                                .py(Space::N2),
                            LayoutRefinement::default().w_full().min_w_0(),
                        );
                        cx.container(props, move |cx| {
                            vec![
                                ui::h_flex(|cx| {
                                    vec![
                                        shadcn::raw::typography::muted("الحالة").into_element(cx),
                                        shadcn::raw::typography::small("تم الشحن").into_element(cx),
                                    ]
                                })
                                .gap(Space::N2)
                                .items_center()
                                .justify_between()
                                .layout(LayoutRefinement::default().w_full().min_w_0())
                                .into_element(cx),
                            ]
                        })
                        .test_id("ui-gallery-collapsible-rtl-status")
                    };

                    ui::v_flex(move |_cx| vec![header, status])
                        .gap(Space::N2)
                        .items_stretch()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx)
                },
                |cx| {
                    shadcn::CollapsibleContent::new([
                        detail_card(
                            cx,
                            "ui-gallery-collapsible-rtl-shipping-address",
                            "عنوان الشحن",
                            "100 Market St, San Francisco",
                        )
                        .into_element(cx),
                        detail_card(
                            cx,
                            "ui-gallery-collapsible-rtl-items",
                            "العناصر",
                            "2x سماعات الاستوديو",
                        )
                        .into_element(cx),
                    ])
                    .refine_layout(LayoutRefinement::default().w_full().min_w_0().mt(Space::N2))
                    .into_element(cx)
                    .test_id("ui-gallery-collapsible-rtl-content")
                },
            )
            .test_id("ui-gallery-collapsible-rtl-card")
    })
    .test_id("ui-gallery-collapsible-rtl-demo")
}
// endregion: example
