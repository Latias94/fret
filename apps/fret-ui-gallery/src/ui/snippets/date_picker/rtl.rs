pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use super::{default_month, fixed_today};
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let today = fixed_today();
    let open = cx.local_model_keyed("open", || false);
    let month = cx.local_model_keyed("month", || default_month(today));
    let selected = cx.local_model_keyed("selected", || Some(today));

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::DatePicker::new(open, month, selected)
            .placeholder("Pick a date")
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
            .into_element(cx)
    })
    .test_id("ui-gallery-date-picker-rtl")
}
// endregion: example
