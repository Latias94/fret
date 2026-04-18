pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("ui-gallery-native-select-demo-value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("ui-gallery-native-select-demo-open", || false);

    // shadcn NativeSelect is `w-fit` at the wrapper level; keep the gallery close to that default
    // (content-driven width), while still clamping to a reasonable max.
    let select_layout = LayoutRefinement::default().max_w(Px(320.0)).min_w_0();

    let select = shadcn::native_select(value, open)
        .a11y_label("Native select: status")
        .placeholder("Select status")
        .trigger_test_id("ui-gallery-native-select-basic-native-trigger")
        .test_id_prefix("ui-gallery-native-select-basic-native")
        .options([
            shadcn::NativeSelectOption::placeholder("Select status"),
            shadcn::NativeSelectOption::new("todo", "Todo"),
            shadcn::NativeSelectOption::new("in-progress", "In Progress"),
            shadcn::NativeSelectOption::new("done", "Done"),
            shadcn::NativeSelectOption::new("cancelled", "Cancelled"),
        ])
        .refine_layout(select_layout)
        .into_element(cx)
        .test_id("ui-gallery-native-select-basic-native");

    ui::v_flex(|_cx| vec![select])
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-native-select-demo")
}
// endregion: example
