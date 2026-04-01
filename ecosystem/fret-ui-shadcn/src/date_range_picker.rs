use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::{AnyElement, SemanticsDecoration};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::current_color;
use fret_ui_kit::declarative::icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, LengthRefinement, Space};
use fret_ui_kit::{WidgetStateProperty, ui};
use time::{Date, Weekday};

use crate::bool_model::IntoBoolModel;
use crate::button::{Button, ButtonSize, ButtonStyle, ButtonVariant, button_text_style};
use crate::calendar_month_model::IntoCalendarMonthModel;
use crate::calendar_range::CalendarRange;
use crate::date_range_selection_model::IntoDateRangeSelectionModel;
use crate::popover::{Popover, PopoverAlign, PopoverContent, PopoverSide};

use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};

#[derive(Clone)]
pub struct DateRangePicker {
    pub open: Model<bool>,
    pub month: Model<CalendarMonth>,
    pub selected: Model<DateRangeSelection>,
    control_id: Option<ControlId>,
    test_id_prefix: Option<Arc<str>>,
    pub placeholder: Arc<str>,
    pub week_start: Weekday,
    required: bool,
    aria_invalid: bool,
    pub show_outside_days: bool,
    pub disable_outside_days: bool,
    pub disabled_predicate: Option<Arc<dyn Fn(Date) -> bool + Send + Sync + 'static>>,
    pub disabled: bool,
    close_on_select: bool,
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
            .field("test_id_prefix", &self.test_id_prefix.as_deref())
            .field("placeholder", &self.placeholder)
            .field("week_start", &self.week_start)
            .field("required", &self.required)
            .field("aria_invalid", &self.aria_invalid)
            .field("show_outside_days", &self.show_outside_days)
            .field("disable_outside_days", &self.disable_outside_days)
            .field("disabled_predicate", &self.disabled_predicate.is_some())
            .field("disabled", &self.disabled)
            .field("close_on_select", &self.close_on_select)
            .finish()
    }
}

impl DateRangePicker {
    pub fn new(
        open: impl IntoBoolModel,
        month: impl IntoCalendarMonthModel,
        selected: impl IntoDateRangeSelectionModel,
    ) -> Self {
        Self {
            open: open.into_bool_model(),
            month: month.into_calendar_month_model(),
            selected: selected.into_date_range_selection_model(),
            control_id: None,
            test_id_prefix: None,
            placeholder: Arc::from("Pick a date"),
            week_start: Weekday::Sunday,
            required: false,
            aria_invalid: false,
            show_outside_days: true,
            disable_outside_days: false,
            disabled_predicate: None,
            disabled: false,
            close_on_select: false,
            format_selected: Arc::new(format_selected_lll_dd_y_range),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
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

    /// Sets a stable automation prefix for the date-range trigger/content/calendar surfaces.
    ///
    /// Derived ids include `{prefix}-trigger`, `{prefix}-content`, and `{prefix}-calendar`. The
    /// inner range calendar also forwards `{prefix}-calendar` into `CalendarRange::test_id_prefix(...)`
    /// so day/caption anchors remain stable for diagnostics.
    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
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

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
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

    /// Closes the popover once the range selection is complete.
    ///
    /// Default: `false`, matching the upstream shadcn range example.
    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
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
            let close_on_select_open = self.close_on_select.then(|| open.clone());
            let required = self.required;
            let aria_invalid = self.aria_invalid;
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

            let selected_value = cx.watch_model(&selected).cloned().unwrap_or_default();
            let selected_empty = selected_value.from.is_none() && selected_value.to.is_none();
            let button_text: Arc<str> =
                if selected_value.from.is_some() || selected_value.to.is_some() {
                    (self.format_selected)(selected_value)
                } else {
                    self.placeholder.clone()
                };

            Popover::from_open(open.clone())
                .side(PopoverSide::Bottom)
                .align(PopoverAlign::Start)
                .initial_focus_from_cell(initial_focus_out.clone())
                .into_element_with(
                    cx,
                    move |cx| {
                        let theme = Theme::global(&*cx.app).snapshot();
                        let calendar_icon_for_content = calendar_icon.clone();
                        let button_text_for_content = button_text.clone();

                        let fg_fallback = ColorRef::Color(theme.color_token("foreground"));
                        let muted_fg = ColorRef::Color(theme.color_token("muted-foreground"));
                        let (text_size, text_weight, line_height) = {
                            let theme_full = Theme::global(&*cx.app);
                            let mut text_style =
                                button_text_style(theme_full, ButtonSize::default());
                            text_style.weight = fret_core::FontWeight::NORMAL;
                            let line_height = text_style
                                .line_height
                                .unwrap_or_else(|| theme_full.metric_token("font.line_height"));
                            (text_style.size, text_style.weight, line_height)
                        };

                        let content = ui::h_row(move |cx| {
                            let fg = current_color::inherited_current_color(cx)
                                .unwrap_or_else(|| fg_fallback.clone());

                            vec![
                                icon::icon(cx, calendar_icon_for_content),
                                ui::text(button_text_for_content.clone())
                                    .text_size_px(text_size)
                                    .fixed_line_box_px(line_height)
                                    .line_box_in_bounds()
                                    .font_weight(text_weight)
                                    .nowrap()
                                    .text_color(fg)
                                    .into_element(cx),
                            ]
                        })
                        .justify_start()
                        .items_center()
                        .gap(Space::N2)
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx);

                        let mut button = Button::new(button_text.clone())
                            .variant(ButtonVariant::Outline)
                            .leading_icon(calendar_icon)
                            .children([content])
                            .toggle_model(open_trigger.clone())
                            .disabled(self.disabled)
                            .refine_style(trigger_chrome.clone())
                            .refine_layout(trigger_layout.clone());
                        if let Some(control_id) = control_id.clone() {
                            button = button.control_id(control_id);
                        }

                        if selected_empty {
                            button = button.style(
                                ButtonStyle::default()
                                    .foreground(WidgetStateProperty::new(Some(muted_fg))),
                            );
                        }
                        if let Some(test_id) = trigger_test_id.clone() {
                            button = button.test_id(test_id);
                        }

                        let mut trigger = button.into_element(cx);
                        if required || aria_invalid {
                            let mut decoration = SemanticsDecoration::default();
                            if required {
                                decoration = decoration.required(true);
                            }
                            if aria_invalid {
                                decoration = decoration.invalid(fret_core::SemanticsInvalid::True);
                            }
                            trigger = trigger.attach_semantics(decoration);
                        }
                        trigger
                    },
                    move |cx| {
                        let mut calendar = CalendarRange::new(month.clone(), selected.clone())
                            .week_start(self.week_start)
                            .required(required)
                            .show_outside_days(self.show_outside_days)
                            .disable_outside_days(self.disable_outside_days)
                            // Upstream `date-picker-with-range` renders two months in the popover.
                            .number_of_months(2)
                            // Keep the popover open after the first click so the user can pick an end date.
                            .min_days(1)
                            .initial_focus_out(initial_focus_out.clone());
                        if let Some(open) = close_on_select_open.clone() {
                            calendar = calendar.close_on_select(open);
                        }

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
                            // shadcn/ui DatePickerWithRange demo uses `PopoverContent` with `w-auto p-0`.
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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size as CoreSize};
    use fret_ui::UiTree;
    use fret_ui_kit::OverlayController;
    use fret_ui_kit::primitives::control_registry::ControlId;
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
        selected: Model<DateRangeSelection>,
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
                    DateRangePicker::new(open.clone(), month.clone(), selected.clone())
                        .test_id_prefix("date-range")
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
    fn date_range_picker_trigger_width_is_intrinsic_unless_caller_overrides_it() {
        let mut app = App::new();
        let window = AppWindowId::default();

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(360.0), Px(240.0)),
        );

        let intrinsic = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "date-range-picker-trigger-intrinsic-width",
            |cx| {
                let open = cx.app.models_mut().insert(false);
                let month = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::March));
                let selected = cx.app.models_mut().insert(DateRangeSelection::default());

                DateRangePicker::new(open, month, selected).into_element(cx)
            },
        );
        let fill = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "date-range-picker-trigger-fill-width",
            |cx| {
                let open = cx.app.models_mut().insert(false);
                let month = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::March));
                let selected = cx.app.models_mut().insert(DateRangeSelection::default());

                DateRangePicker::new(open, month, selected)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
            },
        );

        let intrinsic_pressable = find_first_pressable(&intrinsic).unwrap_or_else(|| {
            panic!("expected DateRangePicker trigger to render a Pressable: {intrinsic:#?}")
        });
        let fill_pressable = find_first_pressable(&fill).unwrap_or_else(|| {
            panic!("expected DateRangePicker trigger to render a Pressable: {fill:#?}")
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

    #[test]
    fn date_range_picker_control_id_uses_registry_labelled_by_and_described_by_elements() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Slate,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(360.0), Px(180.0)),
        );

        let root = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "range-control-id",
            |cx| {
                let id = ControlId::from("travel-dates");
                let open = cx.app.models_mut().insert(false);
                let month = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::March));
                let selected = cx.app.models_mut().insert(DateRangeSelection::default());

                cx.column(fret_ui::element::ColumnProps::default(), move |cx| {
                    vec![
                        crate::field::FieldLabel::new("Travel dates")
                            .for_control(id.clone())
                            .into_element(cx),
                        crate::field::FieldDescription::new("Choose outbound and return dates.")
                            .for_control(id.clone())
                            .into_element(cx),
                        DateRangePicker::new(open.clone(), month.clone(), selected.clone())
                            .control_id(id)
                            .into_element(cx),
                    ]
                })
            },
        );

        let label_id = root.children[0].id;
        let description_id = root.children[1].id;
        let trigger = find_first_pressable(root.children.get(2).expect("range picker child"))
            .expect("expected DateRangePicker trigger pressable");

        assert_eq!(trigger.a11y.labelled_by_element, Some(label_id.0));
        assert_eq!(trigger.a11y.described_by_element, Some(description_id.0));
    }

    #[test]
    fn date_range_picker_test_id_prefix_stamps_trigger_content_and_calendar() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        let open = app.models_mut().insert(true);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::March));
        let selected = app.models_mut().insert(DateRangeSelection::default());

        for frame in 1..=3 {
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                month.clone(),
                selected.clone(),
                frame,
            );
        }

        let snapshot = ui.semantics_snapshot().expect("semantics snapshot");
        let ids: Vec<&str> = snapshot
            .nodes
            .iter()
            .filter_map(|node| node.test_id.as_deref())
            .collect();

        assert!(ids.iter().copied().any(|id| id == "date-range-trigger"));
        assert!(ids.iter().copied().any(|id| id == "date-range-content"));
        assert!(ids.iter().copied().any(|id| id == "date-range-calendar"));
        assert!(
            ids.iter()
                .copied()
                .any(|id| id == "date-range-calendar:2026-03-01")
        );
    }

    #[test]
    fn date_range_picker_required_exposes_required_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(480.0), Px(240.0)),
        );

        let open = app.models_mut().insert(false);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::March));
        let selected = app.models_mut().insert(DateRangeSelection::default());

        app.set_frame_id(FrameId(1));
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );
        OverlayController::begin_frame(&mut app, window);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "date-range-picker-required-semantics",
            |cx| {
                vec![
                    DateRangePicker::new(open.clone(), month.clone(), selected.clone())
                        .required(true)
                        .test_id_prefix("required-date-range-picker")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("required-date-range-picker-trigger"))
            .expect("date range picker trigger semantics");
        assert!(node.flags.required);
    }

    #[test]
    fn date_range_picker_aria_invalid_exposes_invalid_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(480.0), Px(240.0)),
        );

        let open = app.models_mut().insert(false);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::March));
        let selected = app.models_mut().insert(DateRangeSelection::default());

        app.set_frame_id(FrameId(1));
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );
        OverlayController::begin_frame(&mut app, window);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "date-range-picker-invalid-semantics",
            |cx| {
                vec![
                    DateRangePicker::new(open.clone(), month.clone(), selected.clone())
                        .aria_invalid(true)
                        .test_id_prefix("invalid-date-range-picker")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("invalid-date-range-picker-trigger"))
            .expect("date range picker trigger semantics");
        assert_eq!(node.flags.invalid, Some(fret_core::SemanticsInvalid::True));
    }
}
