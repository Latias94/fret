pub const SOURCE: &str = include_str!("date_and_time_picker.rs");

// region: example
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use time::Date;

#[derive(Default)]
struct Models {
    month: Option<Model<CalendarMonth>>,
    selected: Option<Model<Option<Date>>>,
    from: Option<Model<String>>,
    to: Option<Model<String>>,
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
    let (month, selected, from, to) = cx.with_state(Models::default, |st| {
        (
            st.month.clone(),
            st.selected.clone(),
            st.from.clone(),
            st.to.clone(),
        )
    });

    let today = today_from_env_or_now();
    let time_date =
        time::Date::from_calendar_date(today.year(), today.month(), 12).expect("valid time date");

    let month = match month {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(CalendarMonth::from_date(time_date));
            cx.with_state(Models::default, |st| st.month = Some(model.clone()));
            model
        }
    };
    let selected = match selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(time_date));
            cx.with_state(Models::default, |st| st.selected = Some(model.clone()));
            model
        }
    };
    let from = match from {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("10:30:00"));
            cx.with_state(Models::default, |st| st.from = Some(model.clone()));
            model
        }
    };
    let to = match to {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("12:30:00"));
            cx.with_state(Models::default, |st| st.to = Some(model.clone()));
            model
        }
    };

    let theme = Theme::global(&*cx.app).snapshot();
    let clock_fg = ColorRef::Color(theme.color_token("muted-foreground"));
    let clock_icon = |cx: &mut ElementContext<'_, H>| {
        shadcn::icon::icon_with(
            cx,
            IconId::new_static("lucide.clock-2"),
            None,
            Some(clock_fg.clone()),
        )
    };

    let calendar = shadcn::Calendar::new(month, selected)
        .test_id_prefix("ui-gallery.calendar.time")
        .refine_style(ChromeRefinement::default().p(Space::N0))
        .into_element(cx);

    let footer = shadcn::FieldGroup::new([
        shadcn::Field::new([
            shadcn::FieldLabel::new("Start Time").into_element(cx),
            shadcn::InputGroup::new(from)
                .a11y_label("Start Time")
                .trailing([clock_icon(cx)])
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Field::new([
            shadcn::FieldLabel::new("End Time").into_element(cx),
            shadcn::InputGroup::new(to)
                .a11y_label("End Time")
                .trailing([clock_icon(cx)])
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .into_element(cx);

    shadcn::Card::new(vec![
        shadcn::CardContent::new(vec![calendar])
            .size(shadcn::CardSize::Sm)
            .into_element(cx),
        shadcn::CardFooter::new(vec![footer])
            .size(shadcn::CardSize::Sm)
            .into_element(cx),
    ])
    .size(shadcn::CardSize::Sm)
    .refine_layout(LayoutRefinement::default().min_w_0().max_w(Px(360.0)))
    .into_element(cx)
}
// endregion: example
