pub const SOURCE: &str = include_str!("settings_panel.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn radius_input<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    test_id: &'static str,
    a11y: &'static str,
    value: Model<String>,
) -> AnyElement {
    shadcn::Input::new(value)
        .a11y_label(a11y)
        .placeholder("0")
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id(test_id)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.local_model_keyed("open", || false);
    let radius_x = cx.local_model_keyed("radius_x", || String::from("0"));
    let radius_y = cx.local_model_keyed("radius_y", || String::from("0"));
    let radius_bl = cx.local_model_keyed("radius_bl", || String::from("8"));
    let radius_br = cx.local_model_keyed("radius_br", || String::from("8"));

    let settings_collapsible = shadcn::Collapsible::new(open.clone())
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element_with_open_model(
            cx,
            |cx, open, is_open| {
                let icon = fret_icons::IconId::new_static(if is_open {
                    "lucide.minimize"
                } else {
                    "lucide.maximize"
                });
                let toggle = shadcn::Button::new("")
                    .a11y_label("Toggle details")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::IconSm)
                    .icon(icon)
                    .toggle_model(open)
                    .test_id("ui-gallery-collapsible-settings-trigger")
                    .into_element(cx);

                let fields = ui::h_flex(|cx| {
                    vec![
                        radius_input(
                            cx,
                            "ui-gallery-collapsible-settings-radius-x",
                            "Radius X",
                            radius_x.clone(),
                        ),
                        radius_input(
                            cx,
                            "ui-gallery-collapsible-settings-radius-y",
                            "Radius Y",
                            radius_y.clone(),
                        ),
                    ]
                })
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

                ui::h_flex(|_cx| vec![fields, toggle])
                    .gap(Space::N2)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx)
            },
            |cx| {
                let fields = ui::h_flex(|cx| {
                    vec![
                        radius_input(
                            cx,
                            "ui-gallery-collapsible-settings-radius-bl",
                            "Bottom-left",
                            radius_bl.clone(),
                        ),
                        radius_input(
                            cx,
                            "ui-gallery-collapsible-settings-radius-br",
                            "Bottom-right",
                            radius_br.clone(),
                        ),
                    ]
                })
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

                shadcn::CollapsibleContent::new([fields])
                    .refine_layout(LayoutRefinement::default().w_full().mt(Space::N2))
                    .into_element(cx)
                    .test_id("ui-gallery-collapsible-settings-content")
            },
        )
        .test_id("ui-gallery-collapsible-settings");

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Radius").into_element(cx),
            shadcn::CardDescription::new("Set the corner radius of the element.").into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([settings_collapsible]).into_element(cx),
    ])
    .size(shadcn::CardSize::Sm)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-collapsible-settings-panel")
}
// endregion: example
