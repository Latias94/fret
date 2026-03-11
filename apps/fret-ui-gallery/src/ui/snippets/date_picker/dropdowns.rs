pub const SOURCE: &str = include_str!("dropdowns.rs");

// region: example
use fret_app::App;
use fret_ui::Invalidation;
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
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

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let today = std::env::var("FRET_UI_GALLERY_FIXED_TODAY")
        .ok()
        .and_then(|raw| parse_iso_date_ymd(&raw))
        .unwrap_or_else(|| time::OffsetDateTime::now_utc().date());

    let (open, month, selected) = cx.with_state(Models::default, |st| {
        (st.open.clone(), st.month.clone(), st.selected.clone())
    });

    let (open, month, selected) = match (open, month, selected) {
        (Some(open), Some(month), Some(selected)) => (open, month, selected),
        _ => {
            let open = cx.app.models_mut().insert(false);
            let month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
            let selected = cx.app.models_mut().insert(None::<Date>);

            cx.with_state(Models::default, |st| {
                st.open = Some(open.clone());
                st.month = Some(month.clone());
                st.selected = Some(selected.clone());
            });

            (open, month, selected)
        }
    };

    let is_desktop = fret_ui_kit::declarative::viewport_queries::viewport_width_at_least(
        cx,
        Invalidation::Layout,
        fret_ui_kit::declarative::viewport_queries::tailwind::MD,
        fret_ui_kit::declarative::viewport_queries::ViewportQueryHysteresis::default(),
    );

    let content_month = month.clone();
    let content_selected = selected.clone();
    let content = move |cx: &mut ElementContext<'_, App>| {
        shadcn::Calendar::new(content_month.clone(), content_selected.clone())
            .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
            .test_id_prefix("ui-gallery-date-picker-dropdowns-calendar")
            .into_element(cx)
    };

    let trigger_open = open.clone();
    let trigger = move |cx: &mut ElementContext<'_, App>| {
        shadcn::Button::new("Pick a date")
            .variant(shadcn::ButtonVariant::Outline)
            .toggle_model(trigger_open.clone())
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
            .into_element(cx)
            .test_id("ui-gallery-date-picker-dropdowns-trigger")
    };

    let overlay = if is_desktop {
        shadcn::Popover::new(open.clone())
            .side(shadcn::PopoverSide::Bottom)
            .align(shadcn::PopoverAlign::Start)
            .into_element(
                cx,
                move |cx| trigger(cx),
                move |cx| {
                    shadcn::PopoverContent::new([content(cx)])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            LayoutRefinement::default()
                                .w(fret_ui_kit::LengthRefinement::Auto)
                                .min_w_0()
                                .min_h_0(),
                        )
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-dropdowns-popover-content")
                },
            )
    } else {
        let done_open = open.clone();
        shadcn::Drawer::new(open.clone()).into_element(
            cx,
            move |cx| trigger(cx),
            move |cx| {
                shadcn::DrawerContent::new([
                    content(cx),
                    shadcn::DrawerFooter::new([shadcn::Button::new("Done")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(done_open.clone())
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-dropdowns-done")])
                    .into_element(cx),
                ])
                .into_element(cx)
                .test_id("ui-gallery-date-picker-dropdowns-drawer-content")
            },
        )
    };

    shadcn::Field::new([
        shadcn::FieldLabel::new("With dropdowns").into_element(cx),
        overlay,
    ])
    .into_element(cx)
    .test_id("ui-gallery-date-picker-dropdowns")
}
// endregion: example
