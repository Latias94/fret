use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::headless::calendar::CalendarMonth;
use fret_ui_kit::primitives::popover as radix_popover;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};
use time::{Date, OffsetDateTime, Weekday};

use crate::button::{Button, ButtonVariant};
use crate::calendar::Calendar;
use crate::popover::{Popover, PopoverAlign, PopoverSide};

#[derive(Clone)]
pub struct DatePicker {
    pub open: Model<bool>,
    pub month: Model<CalendarMonth>,
    pub selected: Model<Option<Date>>,
    week_start: Weekday,
    placeholder: Arc<str>,
    disabled: bool,
    show_outside_days: bool,
    disable_outside_days: bool,
    disabled_predicate: Option<Arc<dyn Fn(Date) -> bool + Send + Sync + 'static>>,
}

impl std::fmt::Debug for DatePicker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatePicker")
            .field("open", &"<model>")
            .field("month", &"<model>")
            .field("selected", &"<model>")
            .field("week_start", &self.week_start)
            .field("placeholder", &self.placeholder)
            .field("disabled", &self.disabled)
            .field("show_outside_days", &self.show_outside_days)
            .field("disable_outside_days", &self.disable_outside_days)
            .field("disabled_predicate", &self.disabled_predicate.is_some())
            .finish()
    }
}

impl DatePicker {
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
            disabled_predicate: None,
        }
    }

    /// Creates a date picker with Radix-style controlled/uncontrolled models.
    ///
    /// - `selected` / `default_selected` controls the selected date.
    /// - `open` / `default_open` controls the popover visibility.
    ///
    /// Note: if `selected`/`open` are `None`, internal models are stored in element state at the
    /// call site. Call this from a stable subtree (key the parent node if needed).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        selected: Option<Model<Option<Date>>>,
        default_selected: Option<Date>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let open = radix_popover::PopoverRoot::new()
            .open(open)
            .default_open(default_open)
            .open_model(cx);

        let initial_selected = default_selected.or_else(|| {
            selected
                .as_ref()
                .and_then(|m| m.read_ref(&*cx.app, |v| *v).ok().flatten())
        });
        let selected =
            controllable_state::use_controllable_model(cx, selected, || initial_selected).model();

        let today = OffsetDateTime::now_utc().date();
        let default_month = initial_selected
            .map(CalendarMonth::from_date)
            .unwrap_or_else(|| CalendarMonth::from_date(today));
        let month =
            controllable_state::use_controllable_model(cx, None::<Model<CalendarMonth>>, || {
                default_month
            })
            .model();

        Self::new(open, month, selected)
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

    pub fn disabled_by(mut self, f: impl Fn(Date) -> bool + Send + Sync + 'static) -> Self {
        self.disabled_predicate = Some(Arc::new(f));
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

            let selected_value = cx.watch_model(&selected).copied().flatten();
            let button_text: Arc<str> = match selected_value {
                Some(date) => Arc::<str>::from(date.to_string()),
                None => self.placeholder.clone(),
            };

            Popover::new(open.clone())
                .side(PopoverSide::Bottom)
                .align(PopoverAlign::Start)
                .into_element(
                    cx,
                    move |cx| {
                        Button::new(button_text)
                            .variant(ButtonVariant::Outline)
                            .toggle_model(open_trigger.clone())
                            .disabled(self.disabled)
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx)
                    },
                    move |cx| {
                        let props = decl_style::container_props(
                            &theme,
                            ChromeRefinement::default().p(Space::N2),
                            LayoutRefinement::default(),
                        );
                        cx.container(props, move |cx| {
                            let mut calendar = Calendar::new(month.clone(), selected.clone())
                                .week_start(self.week_start)
                                .show_outside_days(self.show_outside_days)
                                .disable_outside_days(self.disable_outside_days)
                                .close_on_select(open_content.clone());

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
