pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret::UiCx;
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model_keyed("query", String::new);
    let control_id = ControlId::from("ui-gallery-combobox-label");

    let combobox = shadcn::Combobox::new(value, open)
        .a11y_label("Combobox label association")
        .query_model(query)
        .control_id(control_id.clone())
        .test_id_prefix("ui-gallery-combobox-label")
        .items([
            shadcn::ComboboxItem::new("next", "Next.js"),
            shadcn::ComboboxItem::new("svelte", "SvelteKit"),
            shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
            shadcn::ComboboxItem::new("remix", "Remix"),
            shadcn::ComboboxItem::new("astro", "Astro"),
        ])
        .into_element_parts(cx, |_cx| {
            vec![
                shadcn::ComboboxPart::from(shadcn::ComboboxTrigger::new().width_px(Px(260.0))),
                shadcn::ComboboxPart::from(
                    shadcn::ComboboxInput::new().placeholder("Select a framework"),
                ),
            ]
        });

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::Field::new([
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Framework")
                        .for_control(control_id.clone())
                        .test_id("ui-gallery-combobox-label-label")
                        .into_element(cx),
                    shadcn::FieldDescription::new("Click the label to focus the combobox trigger.")
                        .for_control(control_id.clone())
                        .into_element(cx),
                ])
                .into_element(cx),
                combobox,
            ]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-combobox-label")
}
// endregion: example
