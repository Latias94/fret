pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let w_fit = LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto);

    shadcn::RadioGroup::uncontrolled(Some("comfortable"))
        .a11y_label("Options")
        .refine_layout(w_fit)
        .item(shadcn::RadioGroupItem::new("default", "Default"))
        .item(shadcn::RadioGroupItem::new("comfortable", "Comfortable"))
        .item(shadcn::RadioGroupItem::new("compact", "Compact"))
        .into_element(cx)
        .test_id("ui-gallery-radio-group-demo")
}
// endregion: example
