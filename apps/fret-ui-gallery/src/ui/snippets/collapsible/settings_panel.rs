pub const SOURCE: &str = include_str!("settings_panel.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use shadcn::raw::collapsible::primitives as shadcn_col;

fn radius_input<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    test_id: &'static str,
    a11y: &'static str,
    value: Model<String>,
) -> impl IntoUiElement<H> + use<H> {
    shadcn::Input::new(value)
        .a11y_label(a11y)
        .placeholder("0")
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id(test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("settings_open", || false);
    let radius_x = cx.local_model_keyed("radius_x", || String::from("0"));
    let radius_y = cx.local_model_keyed("radius_y", || String::from("0"));
    let radius_bl = cx.local_model_keyed("radius_bl", || String::from("0"));
    let radius_br = cx.local_model_keyed("radius_br", || String::from("0"));

    let settings_collapsible = shadcn_col::Collapsible::new()
        .open(open.clone())
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx, |cx| {
            let is_open = cx.watch_model(&open).layout().copied().unwrap_or(false);
            let icon = fret_icons::IconId::new_static(if is_open {
                "lucide.minimize"
            } else {
                "lucide.maximize"
            });

            let top_row = ui::h_flex(|cx| {
                vec![
                    radius_input(
                        cx,
                        "ui-gallery-collapsible-settings-radius-x",
                        "Radius X",
                        radius_x.clone(),
                    )
                    .into_element(cx),
                    radius_input(
                        cx,
                        "ui-gallery-collapsible-settings-radius-y",
                        "Radius Y",
                        radius_y.clone(),
                    )
                    .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);

            let bottom_row = ui::h_flex(|cx| {
                vec![
                    radius_input(
                        cx,
                        "ui-gallery-collapsible-settings-radius-bl",
                        "Bottom-left",
                        radius_bl.clone(),
                    )
                    .into_element(cx),
                    radius_input(
                        cx,
                        "ui-gallery-collapsible-settings-radius-br",
                        "Bottom-right",
                        radius_br.clone(),
                    )
                    .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);

            let content = shadcn_col::CollapsibleContent::new([bottom_row])
                .refine_layout(LayoutRefinement::default().w_full())
                .test_id("ui-gallery-collapsible-settings-content")
                .into_element(cx);

            let fields = ui::v_flex(|_cx| vec![top_row, content])
                .gap(Space::N2)
                .items_stretch()
                .layout(LayoutRefinement::default().w_full().min_w_0().flex_1())
                .into_element(cx);

            let button = shadcn::Button::new("")
                .a11y_label("Toggle details")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .icon(icon)
                .test_id("ui-gallery-collapsible-settings-trigger")
                .into_element(cx);
            let trigger = shadcn_col::CollapsibleTrigger::new([button])
                .as_child(true)
                .into_element(cx);

            vec![
                ui::h_flex(|_cx| vec![fields, trigger])
                    .gap(Space::N2)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-collapsible-settings");

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Radius"),
                    shadcn::card_description("Set the corner radius of the element."),
                ]
            }),
            shadcn::card_content(|cx| ui::children![cx; settings_collapsible]),
        ]
    })
    .size(shadcn::CardSize::Sm)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-collapsible-settings-panel")
}
// endregion: example
