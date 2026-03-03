pub const SOURCE: &str = include_str!("date_and_time_picker.rs");

// region: example
use fret_core::Px;
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_kit::declarative::stack;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use time::Date;

const FIELD_W_PX: Px = Px(128.0);

#[derive(Default)]
struct Models {
    open: Option<Model<bool>>,
    month: Option<Model<CalendarMonth>>,
    selected: Option<Model<Option<Date>>>,
    time: Option<Model<String>>,
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

fn format_date_locale_short_en_us(date: Date) -> String {
    let month = u8::from(date.month());
    format!("{month}/{}/{}", date.day(), date.year())
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (open, month, selected, time_value) = cx.with_state(Models::default, |st| {
        (
            st.open.clone(),
            st.month.clone(),
            st.selected.clone(),
            st.time.clone(),
        )
    });

    let today = today_from_env_or_now();
    let time_date =
        time::Date::from_calendar_date(today.year(), today.month(), 12).expect("valid time date");

    let open = match open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
            model
        }
    };

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
    let time_value = match time_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("10:30:00"));
            cx.with_state(Models::default, |st| st.time = Some(model.clone()));
            model
        }
    };

    let open_for_calendar = open.clone();
    let selected_for_calendar = selected.clone();

    let button_text = cx
        .app
        .models()
        .read(&selected, |v| *v)
        .ok()
        .flatten()
        .map(format_date_locale_short_en_us)
        .unwrap_or_else(|| String::from("Select date"));

    let calendar = shadcn::Calendar::new(month, selected_for_calendar)
        .test_id_prefix("ui-gallery.calendar.time")
        .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
        .close_on_select(open_for_calendar.clone())
        .into_element(cx)
        .test_id("ui-gallery.calendar.time.calendar");

    let popover = shadcn::Popover::new(open.clone())
        .align(shadcn::PopoverAlign::Start)
        .into_element(
            cx,
            move |cx| {
                let trigger = shadcn::Button::new(button_text.clone())
                    .variant(shadcn::ButtonVariant::Outline)
                    .content_justify(fret_ui_kit::Justify::Between)
                    .text_weight(fret_core::FontWeight::NORMAL)
                    .trailing_icon(fret_icons::IconId::new_static("lucide.chevron-down"))
                    .refine_layout(LayoutRefinement::default().w_px(FIELD_W_PX))
                    .into_element(cx)
                    .test_id("ui-gallery.calendar.time.date-trigger");

                shadcn::PopoverTrigger::new(trigger).into_element(cx)
            },
            move |cx| {
                shadcn::PopoverContent::new([calendar])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .refine_layout(
                        LayoutRefinement::default()
                            .w(fret_ui_kit::LengthRefinement::Auto)
                            .overflow_hidden(),
                    )
                    .into_element(cx)
                    .test_id("ui-gallery.calendar.time.popover-content")
            },
        );

    let date_column = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_px(FIELD_W_PX)),
        |cx| vec![shadcn::FieldLabel::new("Date").into_element(cx), popover],
    );

    let time_column = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_px(FIELD_W_PX)),
        |cx| {
            vec![
                shadcn::FieldLabel::new("Time").into_element(cx),
                shadcn::InputGroup::new(time_value)
                    .a11y_label("Time")
                    .into_element(cx)
                    .test_id("ui-gallery.calendar.time.time-input"),
            ]
        },
    );

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        |_cx| vec![date_column, time_column],
    )
    .test_id("ui-gallery.calendar.time.picker")
}
// endregion: example
