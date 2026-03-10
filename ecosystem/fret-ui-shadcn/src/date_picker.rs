use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{FontWeight, Px};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_kit::primitives::popover as radix_popover;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, LengthRefinement, Space,
    WidgetStateProperty,
};
use time::{Date, OffsetDateTime, Weekday};

use crate::button::{Button, ButtonStyle, ButtonVariant};
use crate::calendar::Calendar;
use crate::popover::{Popover, PopoverAlign, PopoverContent, PopoverSide};

#[derive(Clone)]
pub struct DatePicker {
    pub open: Model<bool>,
    pub month: Model<CalendarMonth>,
    pub selected: Model<Option<Date>>,
    control_id: Option<ControlId>,
    test_id_prefix: Option<Arc<str>>,
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
            .field("test_id_prefix", &self.test_id_prefix.as_deref())
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
            control_id: None,
            test_id_prefix: None,
            week_start: Weekday::Sunday,
            placeholder: Arc::from("Pick a date"),
            format_selected: Arc::new(format_selected_ppp_en),
            disabled: false,
            show_outside_days: true,
            disable_outside_days: false,
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
    /// Default: `January 15th, 2026` (English, shadcn-aligned).
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

    /// Binds the trigger button to a logical form control id (similar to HTML `id`).
    ///
    /// This enables `Label::for_control(ControlId)` to focus the trigger and populate
    /// `aria-labelledby` / `aria-describedby`-like semantics via the control registry.
    pub fn control_id(mut self, id: impl Into<ControlId>) -> Self {
        self.control_id = Some(id.into());
        self
    }

    /// Sets a stable automation prefix for the date picker trigger/content/calendar surfaces.
    ///
    /// Derived ids include `{prefix}-trigger`, `{prefix}-content`, and `{prefix}-calendar`. The
    /// calendar also forwards `{prefix}-calendar` into `Calendar::test_id_prefix(...)` so its inner
    /// navigation/day anchors remain stable for diagnostics.
    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let open = self.open.clone();
            let month = self.month.clone();
            let selected = self.selected.clone();
            let control_id = self.control_id.clone();
            let test_id_prefix = self.test_id_prefix.clone();
            let disabled_predicate = self.disabled_predicate.clone();
            let open_trigger = open.clone();
            let open_content = open.clone();
            let initial_focus_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
                Rc::new(Cell::new(None));
            let trigger_chrome = self.chrome.clone();
            let trigger_layout = self.layout.clone();
            let trigger_test_id = test_id_prefix
                .as_ref()
                .map(|prefix| Arc::<str>::from(format!("{prefix}-trigger")));
            let content_test_id = test_id_prefix
                .as_ref()
                .map(|prefix| Arc::<str>::from(format!("{prefix}-content")));
            let calendar_test_id = test_id_prefix
                .as_ref()
                .map(|prefix| Arc::<str>::from(format!("{prefix}-calendar")));
            let calendar_icon = fret_icons::IconId::new_static("lucide.calendar");

            let selected_value = cx.watch_model(&selected).copied().flatten();
            let label_is_placeholder = selected_value.is_none();
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
                        let mut button = Button::new(button_text.clone())
                            .variant(ButtonVariant::Outline)
                            .toggle_model(open_trigger.clone())
                            .disabled(self.disabled)
                            .leading_icon(calendar_icon)
                            .leading_icon_size(Px(16.0))
                            .content_justify_start()
                            .text_weight(FontWeight::NORMAL)
                            .refine_style(trigger_chrome.clone())
                            .refine_layout(trigger_layout.clone());
                        if let Some(control_id) = control_id.clone() {
                            button = button.control_id(control_id);
                        }

                        if label_is_placeholder {
                            button = button.style(ButtonStyle::default().foreground(
                                WidgetStateProperty::new(Some(ColorRef::Token {
                                    key: "muted-foreground",
                                    fallback: ColorFallback::ThemeTextMuted,
                                })),
                            ));
                        }
                        if let Some(test_id) = trigger_test_id.clone() {
                            button = button.test_id(test_id);
                        }

                        button.into_element(cx)
                    },
                    move |cx| {
                        let mut calendar = Calendar::new(month.clone(), selected.clone())
                            .week_start(self.week_start)
                            .show_outside_days(self.show_outside_days)
                            .disable_outside_days(self.disable_outside_days)
                            .close_on_select(open_content.clone())
                            .initial_focus_out(initial_focus_out.clone());

                        if let Some(pred) = disabled_predicate.clone() {
                            calendar = calendar.disabled_by(move |d| pred(d));
                        }
                        if let Some(prefix) = calendar_test_id.clone() {
                            calendar = calendar.test_id_prefix(prefix);
                        }

                        let mut calendar = calendar.into_element(cx);
                        if let Some(test_id) = calendar_test_id.clone() {
                            calendar = calendar.test_id(test_id);
                        }
                        let mut content = PopoverContent::new([calendar])
                            // shadcn/ui DatePicker demo uses `PopoverContent` with `w-auto p-0`.
                            .refine_style(ChromeRefinement::default().p(Space::N0))
                            .refine_layout(LayoutRefinement::default().w(LengthRefinement::Auto))
                            .into_element(cx);
                        if let Some(test_id) = content_test_id.clone() {
                            content = content.test_id(test_id);
                        }
                        content
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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size as CoreSize};
    use fret_ui::UiTree;
    use fret_ui_kit::OverlayController;
    use time::Month;

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

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
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
        crate::shadcn_themes::apply_shadcn_new_york(
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

    fn find_first_pressable(el: &AnyElement) -> Option<fret_ui::element::PressableProps> {
        match &el.kind {
            fret_ui::element::ElementKind::Pressable(props) => Some(props.clone()),
            _ => el.children.iter().find_map(find_first_pressable),
        }
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

        let trigger_label: Arc<str> = Arc::from("2026-01-15");
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap1
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some(trigger_label.as_ref()))
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
        assert_eq!(focused_sem.test_id.as_deref(), Some(trigger_label.as_ref()));
        assert!(
            focused_sem.flags.selected,
            "expected focused day to be selected"
        );
    }

    #[test]
    fn date_picker_trigger_width_is_intrinsic_unless_caller_overrides_it() {
        let mut app = App::new();
        let window = AppWindowId::default();

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(320.0), Px(200.0)),
        );

        let intrinsic = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "date-picker-trigger-intrinsic-width",
            |cx| {
                let open = cx.app.models_mut().insert(false);
                let month = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::March));
                let selected = cx.app.models_mut().insert(None::<Date>);

                DatePicker::new(open, month, selected).into_element(cx)
            },
        );
        let fill = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "date-picker-trigger-fill-width",
            |cx| {
                let open = cx.app.models_mut().insert(false);
                let month = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::March));
                let selected = cx.app.models_mut().insert(None::<Date>);

                DatePicker::new(open, month, selected)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
            },
        );

        let intrinsic_pressable = find_first_pressable(&intrinsic).unwrap_or_else(|| {
            panic!("expected DatePicker trigger to render a Pressable: {intrinsic:#?}")
        });
        let fill_pressable = find_first_pressable(&fill).unwrap_or_else(|| {
            panic!("expected DatePicker trigger to render a Pressable: {fill:#?}")
        });

        assert_eq!(
            intrinsic_pressable.layout.size.width,
            fret_ui::element::Length::Auto
        );
        assert_eq!(
            fill_pressable.layout.size.width,
            fret_ui::element::Length::Fill
        );
    }
}
