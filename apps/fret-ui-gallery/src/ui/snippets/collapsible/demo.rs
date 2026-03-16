pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui::Theme;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use shadcn::raw::collapsible::primitives as shadcn_col;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    // Mirrors the current shadcn/ui v4 base `collapsible-demo.tsx`: a free-form order-details
    // layout with `Trigger(asChild)` in the header and `Content` later in the tree.
    cx.scope(|cx| {
        let open = cx.local_model_keyed("demo_open", || false);

        let detail_card = |cx: &mut UiCx<'_>,
                           test_id: &'static str,
                           title: &'static str,
                           detail: &'static str| {
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
        };

        shadcn_col::Collapsible::new()
            .open(open.clone())
            .gap(Space::N2)
            .refine_layout(LayoutRefinement::default().w_px(Px(350.0)).min_w_0())
            .into_element(cx, move |cx| {
                let header = {
                    let title = shadcn::raw::typography::small("Order #4189").into_element(cx);
                    let button = shadcn::Button::new("")
                        .a11y_label("Toggle details")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Icon)
                        .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
                        .icon(fret_icons::IconId::new_static("lucide.chevrons-up-down"))
                        .test_id("ui-gallery-collapsible-demo-trigger")
                        .into_element(cx);
                    let trigger = shadcn_col::CollapsibleTrigger::new([button])
                        .as_child(true)
                        .into_element(cx);

                    let row = ui::h_flex(move |_cx| vec![title, trigger])
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
                                    shadcn::raw::typography::muted("Status").into_element(cx),
                                    shadcn::raw::typography::small("Shipped").into_element(cx),
                                ]
                            })
                            .gap(Space::N2)
                            .items_center()
                            .justify_between()
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .into_element(cx),
                        ]
                    })
                    .test_id("ui-gallery-collapsible-demo-status")
                };

                let content = shadcn_col::CollapsibleContent::new([
                    detail_card(
                        cx,
                        "ui-gallery-collapsible-demo-shipping-address",
                        "Shipping address",
                        "100 Market St, San Francisco",
                    ),
                    detail_card(
                        cx,
                        "ui-gallery-collapsible-demo-items",
                        "Items",
                        "2x Studio Headphones",
                    ),
                ])
                .gap(Space::N2)
                .test_id("ui-gallery-collapsible-demo-content")
                .into_element(cx);

                vec![header, status, content]
            })
            .test_id("ui-gallery-collapsible-demo")
    })
}
// endregion: example
