pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
    month: Option<Model<CalendarMonth>>,
    selected: Option<Model<Option<Date>>>,
}

fn ensure_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<bool>, Model<CalendarMonth>, Model<Option<Date>>) {
    let state = cx.with_state(Models::default, |st| st.clone());

    let today = time::OffsetDateTime::now_utc().date();
    let open = state.open.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(false);
        cx.with_state(Models::default, |st| st.open = Some(model.clone()));
        model
    });
    let month = state.month.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(CalendarMonth::from_date(today));
        cx.with_state(Models::default, |st| st.month = Some(model.clone()));
        model
    });
    let selected = state.selected.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(None::<Date>);
        cx.with_state(Models::default, |st| st.selected = Some(model.clone()));
        model
    });

    (open, month, selected)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (open, month, selected) = ensure_models(cx);

    shadcn::DatePicker::new(open, month, selected)
        .placeholder("Pick a date")
        .test_id_prefix("ui-gallery-date-picker-usage")
        .refine_layout(LayoutRefinement::default().w_px(Px(280.0)))
        .into_element(cx)
}
// endregion: example
