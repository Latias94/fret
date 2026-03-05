pub const SOURCE: &str = include_str!("settings_panel.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
    radius_x: Option<Model<String>>,
    radius_y: Option<Model<String>>,
    radius_bl: Option<Model<String>>,
    radius_br: Option<Model<String>>,
}

fn get_or_init_models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Models {
    let state = cx.with_state(Models::default, |st| st.clone());
    if state.open.is_some()
        && state.radius_x.is_some()
        && state.radius_y.is_some()
        && state.radius_bl.is_some()
        && state.radius_br.is_some()
    {
        return state;
    }

    let open = cx.app.models_mut().insert(false);
    let radius_x = cx.app.models_mut().insert(String::from("0"));
    let radius_y = cx.app.models_mut().insert(String::from("0"));
    let radius_bl = cx.app.models_mut().insert(String::from("8"));
    let radius_br = cx.app.models_mut().insert(String::from("8"));

    let out = Models {
        open: Some(open.clone()),
        radius_x: Some(radius_x.clone()),
        radius_y: Some(radius_y.clone()),
        radius_bl: Some(radius_bl.clone()),
        radius_br: Some(radius_br.clone()),
    };

    cx.with_state(Models::default, |st| *st = out.clone());
    out
}

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
    let models = get_or_init_models(cx);
    let open = models.open.clone().expect("models.init sets open");
    let radius_x = models.radius_x.clone().expect("models.init sets radius_x");
    let radius_y = models.radius_y.clone().expect("models.init sets radius_y");
    let radius_bl = models
        .radius_bl
        .clone()
        .expect("models.init sets radius_bl");
    let radius_br = models
        .radius_br
        .clone()
        .expect("models.init sets radius_br");

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
