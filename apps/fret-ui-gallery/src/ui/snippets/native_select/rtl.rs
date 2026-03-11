pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::NativeSelect::new_controllable(cx, None, None, None, false)
            .placeholder("Choose language")
            .a11y_label("RTL native select")
            .trigger_test_id("ui-gallery-native-select-rtl-trigger")
            .test_id_prefix("ui-gallery-native-select-rtl")
            .options([
                shadcn::NativeSelectOption::placeholder("Choose language"),
                shadcn::NativeSelectOption::new("rust", "Rust"),
                shadcn::NativeSelectOption::new("zig", "Zig"),
                shadcn::NativeSelectOption::new("go", "Go"),
            ])
            .refine_layout(LayoutRefinement::default().max_w(Px(320.0)).min_w_0())
            .into_element(cx)
    })
    .test_id("ui-gallery-native-select-rtl")
}
// endregion: example
