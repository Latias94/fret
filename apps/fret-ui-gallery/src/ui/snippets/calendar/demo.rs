pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use time::Date;

#[derive(Default)]
struct Models {
    month: Option<Model<CalendarMonth>>,
    selected: Option<Model<Option<Date>>>,
}

fn parse_iso_date_ymd(raw: &str) -> Option<Date> {
    let raw = raw.trim();
    let (year, rest) = raw.split_once('-')?;
    let (month, day) = rest.split_once('-')?;

    let year: i32 = year.parse().ok()?;
    let month: u8 = month.parse().ok()?;
    let day: u8 = day.parse().ok()?;

    let month = time::Month::try_from(month).ok()?;
    Date::from_calendar_date(year, month, day).ok()
}

fn today_from_env_or_now() -> Date {
    std::env::var("FRET_UI_GALLERY_FIXED_TODAY")
        .ok()
        .and_then(|raw| parse_iso_date_ymd(&raw))
        .unwrap_or_else(|| time::OffsetDateTime::now_utc().date())
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (month, selected) = cx.with_state(Models::default, |st| {
        (st.month.clone(), st.selected.clone())
    });

    let today = today_from_env_or_now();

    let month = match month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(CalendarMonth::from_date(today));
            cx.with_state(Models::default, |st| st.month = Some(model.clone()));
            model
        }
    };

    let selected = match selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(today));
            cx.with_state(Models::default, |st| st.selected = Some(model.clone()));
            model
        }
    };

    shadcn::Calendar::new(month, selected)
        .test_id_prefix("ui-gallery.calendar.demo")
        .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
        .refine_style(
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .shadow_sm(),
        )
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_w(Px(980.0)),
        )
        .into_element(cx)
}
// endregion: example
