use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::action::ActionCx;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, LengthRefinement, Space};
use time::{Date, Duration, OffsetDateTime, Weekday};

use crate::button::{Button, ButtonVariant};
use crate::calendar::Calendar;
use crate::popover::{Popover, PopoverAlign, PopoverContent, PopoverSide};
use crate::select::{Select, SelectItem, SelectPosition};

/// shadcn/ui example: `date-picker-with-presets` (v4).
///
/// Upstream reference:
/// - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/date-picker-with-presets.tsx`
#[derive(Clone)]
pub struct DatePickerWithPresets {
    pub open: Model<bool>,
    pub month: Model<CalendarMonth>,
    pub selected: Model<Option<Date>>,
    preset_value: Option<Model<Option<Arc<str>>>>,
    week_start: Weekday,
    placeholder: Arc<str>,
    today_override: Option<Date>,
    disabled: bool,
    show_outside_days: bool,
    disable_outside_days: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for DatePickerWithPresets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatePickerWithPresets")
            .field("open", &"<model>")
            .field("month", &"<model>")
            .field("selected", &"<model>")
            .field("preset_value", &self.preset_value.is_some())
            .field("week_start", &self.week_start)
            .field("placeholder", &self.placeholder)
            .field("today_override", &self.today_override)
            .field("disabled", &self.disabled)
            .field("show_outside_days", &self.show_outside_days)
            .field("disable_outside_days", &self.disable_outside_days)
            .finish()
    }
}

impl DatePickerWithPresets {
    pub fn new(
        open: Model<bool>,
        month: Model<CalendarMonth>,
        selected: Model<Option<Date>>,
    ) -> Self {
        Self {
            open,
            month,
            selected,
            preset_value: None,
            week_start: Weekday::Monday,
            placeholder: Arc::from("Pick a date"),
            today_override: None,
            disabled: false,
            show_outside_days: true,
            disable_outside_days: true,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Overrides the "today" date used by preset selection (Today/Tomorrow/...).
    ///
    /// Default: `OffsetDateTime::now_utc().date()`.
    pub fn today(mut self, today: Date) -> Self {
        self.today_override = Some(today);
        self
    }

    /// Controls the internal Select value model (Radix `value`).
    ///
    /// When omitted, a local model is created and stored in element state at the call site.
    pub fn preset_value_model(mut self, model: Model<Option<Arc<str>>>) -> Self {
        self.preset_value = Some(model);
        self
    }

    pub fn week_start(mut self, week_start: Weekday) -> Self {
        self.week_start = week_start;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn show_outside_days(mut self, show: bool) -> Self {
        self.show_outside_days = show;
        self
    }

    pub fn disable_outside_days(mut self, disable: bool) -> Self {
        self.disable_outside_days = disable;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let open = self.open.clone();
            let month = self.month.clone();
            let selected = self.selected.clone();
            let preset_value = self.preset_value.clone();
            let week_start = self.week_start;
            let show_outside_days = self.show_outside_days;
            let disable_outside_days = self.disable_outside_days;
            let disabled = self.disabled;
            let placeholder = self.placeholder.clone();
            let today = self
                .today_override
                .unwrap_or_else(|| OffsetDateTime::now_utc().date());
            let chrome = self.chrome.clone();
            let layout = self.layout.clone();
            let open_trigger = open.clone();
            let open_content = open.clone();
            let initial_focus_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
                Rc::new(Cell::new(None));

            let selected_value = cx.watch_model(&selected).copied().flatten();
            let button_text: Arc<str> = match selected_value {
                Some(date) => format_selected_ppp_en(date),
                None => placeholder,
            };

            Popover::new(open.clone())
                .side(PopoverSide::Bottom)
                .align(PopoverAlign::Start)
                .initial_focus_from_cell(initial_focus_out.clone())
                .into_element(
                    cx,
                    move |cx| {
                        Button::new(button_text.clone())
                            .variant(ButtonVariant::Outline)
                            .toggle_model(open_trigger.clone())
                            .disabled(disabled)
                            .refine_style(chrome.clone())
                            .refine_layout(
                                LayoutRefinement::default().w_full().merge(layout.clone()),
                            )
                            .into_element(cx)
                    },
                    move |cx| {
                        let preset_value = controllable_state::use_controllable_model(
                            cx,
                            preset_value.clone(),
                            || None::<Arc<str>>,
                        )
                        .model();

                        let select = Select::new_controllable::<H, Arc<str>>(
                            cx,
                            Some(preset_value.clone()),
                            None,
                            None,
                            false,
                        )
                        .placeholder("Select")
                        .position(SelectPosition::Popper)
                        .on_value_change({
                            let selected = selected.clone();
                            let month = month.clone();
                            move |host, action_cx: ActionCx, raw: Arc<str>| {
                                let Ok(days) = raw.parse::<i64>() else {
                                    return;
                                };

                                let next_date = today + Duration::days(days);
                                let _ = host.models_mut().update(&selected, |v| {
                                    *v = Some(next_date);
                                });
                                let _ = host.models_mut().update(&month, |m| {
                                    *m = CalendarMonth::from_date(next_date);
                                });
                                host.request_redraw(action_cx.window);
                            }
                        })
                        .items([
                            SelectItem::new("0", "Today"),
                            SelectItem::new("1", "Tomorrow"),
                            SelectItem::new("3", "In 3 days"),
                            SelectItem::new("7", "In a week"),
                        ])
                        .into_element(cx);

                        let calendar = Calendar::new(month.clone(), selected.clone())
                            .week_start(week_start)
                            .today(today)
                            .show_outside_days(show_outside_days)
                            .disable_outside_days(disable_outside_days)
                            .close_on_select(open_content.clone())
                            .initial_focus_out(initial_focus_out.clone())
                            .into_element(cx);

                        let border = theme.color_required("border");
                        let base_radius = theme.metric_required("metric.radius.lg");
                        let rounded_md = Px((base_radius.0 - 2.0).max(0.0));
                        let calendar_container = {
                            let props = decl_style::container_props(
                                &theme,
                                ChromeRefinement::default()
                                    .radius(rounded_md)
                                    .border_1()
                                    .bg(ColorRef::Color(Color::TRANSPARENT))
                                    .border_color(ColorRef::Color(border)),
                                LayoutRefinement::default().w(LengthRefinement::Auto),
                            );
                            cx.container(props, move |_cx| vec![calendar])
                        };

                        let body = stack::vstack(
                            cx,
                            stack::VStackProps::default().gap(Space::N2).items_stretch(),
                            move |_cx| vec![select, calendar_container],
                        );

                        PopoverContent::new([body])
                            .refine_style(ChromeRefinement::default().p(Space::N2))
                            .refine_layout(LayoutRefinement::default().w(LengthRefinement::Auto))
                            .into_element(cx)
                    },
                )
        })
    }
}

fn format_selected_ppp_en(date: Date) -> Arc<str> {
    use time::Month;

    let month = match date.month() {
        Month::January => "January",
        Month::February => "February",
        Month::March => "March",
        Month::April => "April",
        Month::May => "May",
        Month::June => "June",
        Month::July => "July",
        Month::August => "August",
        Month::September => "September",
        Month::October => "October",
        Month::November => "November",
        Month::December => "December",
    };

    fn ordinal_suffix(day: u8) -> &'static str {
        let mod_100 = day % 100;
        if mod_100 >= 11 && mod_100 <= 13 {
            return "th";
        }
        match day % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    }

    let day = date.day();
    let suffix = ordinal_suffix(day);

    Arc::<str>::from(format!("{month} {day}{suffix}, {}", date.year()))
}
