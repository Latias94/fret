use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};
use time::{Date, Weekday};

use crate::button::{Button, ButtonVariant};
use crate::calendar_range::CalendarRange;
use crate::popover::{Popover, PopoverAlign, PopoverSide};

use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};

#[derive(Clone)]
pub struct DateRangePicker {
    pub open: Model<bool>,
    pub month: Model<CalendarMonth>,
    pub selected: Model<DateRangeSelection>,
    pub placeholder: Arc<str>,
    pub week_start: Weekday,
    pub show_outside_days: bool,
    pub disable_outside_days: bool,
    pub disabled_predicate: Option<Arc<dyn Fn(Date) -> bool + Send + Sync + 'static>>,
    pub disabled: bool,
    pub format_selected: Arc<dyn Fn(DateRangeSelection) -> Arc<str> + Send + Sync + 'static>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for DateRangePicker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DateRangePicker")
            .field("open", &"<model>")
            .field("month", &"<model>")
            .field("selected", &"<model>")
            .field("placeholder", &self.placeholder)
            .field("week_start", &self.week_start)
            .field("show_outside_days", &self.show_outside_days)
            .field("disable_outside_days", &self.disable_outside_days)
            .field("disabled_predicate", &self.disabled_predicate.is_some())
            .field("disabled", &self.disabled)
            .finish()
    }
}

impl DateRangePicker {
    pub fn new(
        open: Model<bool>,
        month: Model<CalendarMonth>,
        selected: Model<DateRangeSelection>,
    ) -> Self {
        Self {
            open,
            month,
            selected,
            placeholder: Arc::from("Pick a date range"),
            week_start: Weekday::Monday,
            show_outside_days: true,
            disable_outside_days: true,
            disabled_predicate: None,
            disabled: false,
            format_selected: Arc::new(format_selected_lll_dd_y_range),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn format_selected(
        mut self,
        f: impl Fn(DateRangeSelection) -> Arc<str> + Send + Sync + 'static,
    ) -> Self {
        self.format_selected = Arc::new(f);
        self
    }

    pub fn format_selected_iso(mut self) -> Self {
        self.format_selected = Arc::new(format_selected_iso);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn week_start(mut self, week_start: Weekday) -> Self {
        self.week_start = week_start;
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

    pub fn disabled_by(mut self, f: impl Fn(Date) -> bool + Send + Sync + 'static) -> Self {
        self.disabled_predicate = Some(Arc::new(f));
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
            let disabled_predicate = self.disabled_predicate.clone();
            let open_trigger = open.clone();
            let open_content = open.clone();
            let initial_focus_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
                Rc::new(Cell::new(None));
            let trigger_chrome = self.chrome.clone();
            let trigger_layout = self.layout.clone();

            let selected_value = cx.watch_model(&selected).cloned().unwrap_or_default();
            let button_text: Arc<str> =
                if selected_value.from.is_some() || selected_value.to.is_some() {
                    (self.format_selected)(selected_value)
                } else {
                    self.placeholder.clone()
                };

            Popover::new(open.clone())
                .side(PopoverSide::Bottom)
                .align(PopoverAlign::Start)
                .initial_focus_from_cell(initial_focus_out.clone())
                .into_element(
                    cx,
                    move |cx| {
                        Button::new(button_text)
                            .variant(ButtonVariant::Outline)
                            .toggle_model(open_trigger.clone())
                            .disabled(self.disabled)
                            .refine_style(trigger_chrome.clone())
                            .refine_layout(
                                LayoutRefinement::default()
                                    .w_full()
                                    .merge(trigger_layout.clone()),
                            )
                            .into_element(cx)
                    },
                    move |cx| {
                        let props = decl_style::container_props(
                            &theme,
                            ChromeRefinement::default().p(Space::N2),
                            LayoutRefinement::default(),
                        );
                        cx.container(props, move |cx| {
                            let mut calendar = CalendarRange::new(month.clone(), selected.clone())
                                .week_start(self.week_start)
                                .show_outside_days(self.show_outside_days)
                                .disable_outside_days(self.disable_outside_days)
                                .close_on_select(open_content.clone())
                                .initial_focus_out(initial_focus_out.clone());

                            if let Some(pred) = disabled_predicate.clone() {
                                calendar = calendar.disabled_by(move |d| pred(d));
                            }

                            vec![calendar.into_element(cx)]
                        })
                    },
                )
        })
    }
}

fn format_selected_iso(sel: DateRangeSelection) -> Arc<str> {
    match (sel.from, sel.to) {
        (Some(from), Some(to)) => Arc::<str>::from(format!("{from} – {to}")),
        (Some(from), None) => Arc::<str>::from(from.to_string()),
        (None, Some(to)) => Arc::<str>::from(to.to_string()),
        (None, None) => Arc::<str>::from(""),
    }
}

fn format_date_lll_dd_y_en(date: Date) -> String {
    use time::Month;

    let month = match date.month() {
        Month::January => "Jan",
        Month::February => "Feb",
        Month::March => "Mar",
        Month::April => "Apr",
        Month::May => "May",
        Month::June => "Jun",
        Month::July => "Jul",
        Month::August => "Aug",
        Month::September => "Sep",
        Month::October => "Oct",
        Month::November => "Nov",
        Month::December => "Dec",
    };

    let day = format!("{:02}", date.day());
    format!("{month} {day}, {}", date.year())
}

fn format_selected_lll_dd_y_range(sel: DateRangeSelection) -> Arc<str> {
    match (sel.from, sel.to) {
        (Some(from), Some(to)) => Arc::<str>::from(format!(
            "{} - {}",
            format_date_lll_dd_y_en(from),
            format_date_lll_dd_y_en(to)
        )),
        (Some(from), None) => Arc::<str>::from(format_date_lll_dd_y_en(from)),
        (None, Some(to)) => Arc::<str>::from(format_date_lll_dd_y_en(to)),
        (None, None) => Arc::<str>::from(""),
    }
}
