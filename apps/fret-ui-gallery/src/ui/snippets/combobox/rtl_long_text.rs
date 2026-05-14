pub const SOURCE: &str = include_str!("rtl_long_text.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

const LONG_VALUE: &str = "enterprise-observability-platform";
const LONG_LABEL: &str = "Enterprise Observability Platform With Extremely Long Product Name";

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("rtl-long-text-value", || Some(Arc::<str>::from(LONG_VALUE)));
    let open = cx.local_model_keyed("rtl-long-text-open", || false);
    let query = cx.local_model_keyed("rtl-long-text-query", String::new);

    ui::v_flex(move |cx| {
        vec![with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
            shadcn::Combobox::new(value.clone(), open.clone())
                .a11y_label("Combobox RTL long text")
                .query_model(query.clone())
                .refine_layout(
                    LayoutRefinement::default()
                        .w_full()
                        .max_w(Px(220.0))
                        .min_w_0(),
                )
                .test_id_prefix("ui-gallery-combobox-rtl-long-text")
                .items([
                    shadcn::ComboboxItem::new(LONG_VALUE, LONG_LABEL),
                    shadcn::ComboboxItem::new("short", "Short"),
                    shadcn::ComboboxItem::new("compact", "Compact"),
                ])
                .trigger(
                    shadcn::ComboboxTrigger::new()
                        .variant(shadcn::ComboboxTriggerVariant::Button)
                        .width_px(Px(180.0)),
                )
                .input(
                    shadcn::ComboboxInput::new()
                        .placeholder("اختر المنتج")
                        .show_trigger(true),
                )
                .content(
                    shadcn::ComboboxContent::new([
                        shadcn::ComboboxContentPart::input(
                            shadcn::ComboboxInput::new().placeholder("ابحث عن المنتج..."),
                        ),
                        shadcn::ComboboxContentPart::empty(shadcn::ComboboxEmpty::new(
                            "No product found.",
                        )),
                    ])
                    .width_px(Px(240.0)),
                )
                .into_element(cx)
        })]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().max_w(Px(260.0)))
    .into_element(cx)
}
// endregion: example
