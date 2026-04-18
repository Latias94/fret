pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed(
        "ui-gallery-native-select-disabled-value",
        || None::<Arc<str>>,
    );
    let open = cx.local_model_keyed("ui-gallery-native-select-disabled-open", || false);
    let select_layout = LayoutRefinement::default().max_w(Px(320.0)).min_w_0();

    let select = shadcn::native_select(value, open)
        .a11y_label("Native select: priority (disabled)")
        .placeholder("Select priority")
        .disabled(true)
        .trigger_test_id("ui-gallery-native-select-disabled-native-trigger")
        .test_id_prefix("ui-gallery-native-select-disabled-native")
        .options([
            shadcn::NativeSelectOption::placeholder("Select priority"),
            shadcn::NativeSelectOption::new("low", "Low"),
            shadcn::NativeSelectOption::new("medium", "Medium"),
            shadcn::NativeSelectOption::new("high", "High"),
            shadcn::NativeSelectOption::new("critical", "Critical"),
        ])
        .refine_layout(select_layout)
        .into_element(cx)
        .test_id("ui-gallery-native-select-disabled-native");

    ui::v_flex(|_cx| vec![select])
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-native-select-disabled")
}
// endregion: example
