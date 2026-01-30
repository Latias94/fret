use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, LengthRefinement, Space};
use time::{Date, Weekday};

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
    week_start: Weekday,
    placeholder: Arc<str>,
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
            .field("week_start", &self.week_start)
            .field("placeholder", &self.placeholder)
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
            week_start: Weekday::Monday,
            placeholder: Arc::from("Pick a date"),
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
            let week_start = self.week_start;
            let show_outside_days = self.show_outside_days;
            let disable_outside_days = self.disable_outside_days;
            let disabled = self.disabled;
            let placeholder = self.placeholder.clone();
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
                        let select =
                            Select::new_controllable::<H, Arc<str>>(cx, None, None, None, false)
                                .placeholder("Select")
                                .position(SelectPosition::Popper)
                                .items([
                                    SelectItem::new("0", "Today"),
                                    SelectItem::new("1", "Tomorrow"),
                                    SelectItem::new("3", "In 3 days"),
                                    SelectItem::new("7", "In a week"),
                                ])
                                .into_element(cx);

                        let calendar = Calendar::new(month.clone(), selected.clone())
                            .week_start(week_start)
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

    Arc::<str>::from(format!("{month} {}, {}", date.day(), date.year()))
}
