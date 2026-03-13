pub const SOURCE: &str = include_str!("invalid.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value =
        cx.local_model_keyed("ui-gallery-native-select-invalid-value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("ui-gallery-native-select-invalid-open", || false);
    let select_layout = LayoutRefinement::default().max_w(Px(320.0)).min_w_0();

    let select = shadcn::native_select(value, open)
        .a11y_label("Native select: role (invalid)")
        .placeholder("Select role")
        .aria_invalid(true)
        .trigger_test_id("ui-gallery-native-select-error-native-trigger")
        .test_id_prefix("ui-gallery-native-select-error-native")
        .options([
            shadcn::NativeSelectOption::placeholder("Select role"),
            shadcn::NativeSelectOption::new("admin", "Admin"),
            shadcn::NativeSelectOption::new("editor", "Editor"),
            shadcn::NativeSelectOption::new("viewer", "Viewer"),
            shadcn::NativeSelectOption::new("guest", "Guest"),
        ])
        .refine_layout(select_layout)
        .into_element(cx)
        .test_id("ui-gallery-native-select-error-native");

    ui::v_flex(|_cx| vec![select])
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-native-select-error")
}
// endregion: example
