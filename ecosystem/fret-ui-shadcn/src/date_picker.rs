use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
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
    format_selected: Arc<dyn Fn(Date) -> Arc<str> + Send + Sync + 'static>,
    disabled: bool,
    show_outside_days: bool,
    disable_outside_days: bool,
    disabled_predicate: Option<Arc<dyn Fn(Date) -> bool + Send + Sync + 'static>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for DatePicker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatePicker")
            .field("open", &"<model>")
            .field("month", &"<model>")
            .field("selected", &"<model>")
            .field("week_start", &self.week_start)
            .field("placeholder", &self.placeholder)
            .field("format_selected", &"<fn>")
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
            format_selected: Arc::new(format_selected_ppp_en),
            disabled: false,
            show_outside_days: true,
            disable_outside_days: true,
            disabled_predicate: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
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

    /// Overrides how the selected date is shown on the trigger button.
    ///
    /// Default: `Jan 15, 2026` (English, shadcn-aligned).
    pub fn format_selected_by(
        mut self,
        f: impl Fn(Date) -> Arc<str> + Send + Sync + 'static,
    ) -> Self {
        self.format_selected = Arc::new(f);
        self
    }

    /// Uses ISO format (`YYYY-MM-DD`) via `Date::to_string()`.
    pub fn format_selected_iso(self) -> Self {
        self.format_selected_by(|d| Arc::<str>::from(d.to_string()))
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

            let selected_value = cx.watch_model(&selected).copied().flatten();
            let button_text: Arc<str> = match selected_value {
                Some(date) => (self.format_selected)(date),
                None => self.placeholder.clone(),
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
                            let mut calendar = Calendar::new(month.clone(), selected.clone())
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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size as CoreSize};
    use fret_ui::UiTree;
    use fret_ui_kit::OverlayController;
    use time::{Date, Month};

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

    fn shadcn_calendar_day_aria_label(date: Date) -> String {
        let day = date.day();
        format!(
            "{:?}, {:?} {day}{}, {}",
            date.weekday(),
            date.month(),
            ordinal_suffix(day),
            date.year()
        )
    }

    use fret_core::{
        PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, SvgId,
        SvgService, TextBlobId, TextConstraints, TextMetrics, TextService,
    };
    use fret_runtime::FrameId;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: CoreSize::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        month: Model<CalendarMonth>,
        selected: Model<Option<Date>>,
        frame_id: u64,
    ) {
        app.set_frame_id(FrameId(frame_id));
        crate::shadcn_themes::apply_shadcn_new_york_v4(
            app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );
        OverlayController::begin_frame(app, window);

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![
                    DatePicker::new(open.clone(), month.clone(), selected.clone())
                        .format_selected_iso()
                        .placeholder("Pick a test date")
                        .into_element(cx),
                ]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
    }

    #[test]
    fn date_picker_focuses_selected_day_on_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        let open = app.models_mut().insert(false);
        let selected_date = Date::from_calendar_date(2026, Month::January, 15).unwrap();
        let selected = app.models_mut().insert(Some(selected_date));
        let month = app
            .models_mut()
            .insert(CalendarMonth::from_date(selected_date));

        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            month.clone(),
            selected.clone(),
            1,
        );

        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_label = selected_date.to_string();
        let selected_label = format!(
            "{}, selected",
            shadcn_calendar_day_aria_label(selected_date)
        );
        let trigger_node = snap1
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some(trigger_label.as_str()))
            .map(|n| n.id)
            .expect("trigger semantics node");
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            month.clone(),
            selected.clone(),
            2,
        );

        let focused = ui.focus().expect("focused node");
        let snap2 = ui.semantics_snapshot().expect("semantics snapshot");
        let focused_sem = snap2
            .nodes
            .iter()
            .find(|n| n.id == focused)
            .expect("focused semantics node");
        assert_eq!(focused_sem.label.as_deref(), Some(selected_label.as_str()));
        assert!(
            focused_sem.flags.selected,
            "expected focused day to be selected"
        );
    }
}
