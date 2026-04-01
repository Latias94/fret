use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::action::ActionCx;
use fret_ui::element::{AnyElement, SemanticsDecoration};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::current_color;
use fret_ui_kit::declarative::icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, LengthRefinement, Space};
use fret_ui_kit::{WidgetStateProperty, ui};
use time::{Date, Duration, OffsetDateTime, Weekday};

use crate::bool_model::IntoBoolModel;
use crate::button::{
    Button, ButtonSize, ButtonStyle, ButtonVariant, button_text_style,
    outline_trigger_invalid_style,
};
use crate::calendar::Calendar;
use crate::calendar_month_model::IntoCalendarMonthModel;
use crate::optional_date_model::IntoOptionalDateModel;
use crate::popover::{Popover, PopoverAlign, PopoverContent, PopoverSide};
use crate::select::{Select, SelectItem, SelectPosition, SelectValue};

/// shadcn/ui example: `date-picker-with-presets` (v4).
///
/// Upstream reference:
/// - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/date-picker-with-presets.tsx`
#[derive(Clone)]
pub struct DatePickerWithPresets {
    pub open: Model<bool>,
    pub month: Model<CalendarMonth>,
    pub selected: Model<Option<Date>>,
    control_id: Option<ControlId>,
    test_id_prefix: Option<Arc<str>>,
    preset_value: Option<Model<Option<Arc<str>>>>,
    week_start: Weekday,
    placeholder: Arc<str>,
    today_override: Option<Date>,
    required: bool,
    aria_invalid: bool,
    disabled: bool,
    show_outside_days: bool,
    disable_outside_days: bool,
    close_on_select: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for DatePickerWithPresets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatePickerWithPresets")
            .field("open", &"<model>")
            .field("month", &"<model>")
            .field("selected", &"<model>")
            .field("test_id_prefix", &self.test_id_prefix.as_deref())
            .field("preset_value", &self.preset_value.is_some())
            .field("week_start", &self.week_start)
            .field("placeholder", &self.placeholder)
            .field("today_override", &self.today_override)
            .field("required", &self.required)
            .field("aria_invalid", &self.aria_invalid)
            .field("disabled", &self.disabled)
            .field("show_outside_days", &self.show_outside_days)
            .field("disable_outside_days", &self.disable_outside_days)
            .field("close_on_select", &self.close_on_select)
            .finish()
    }
}

impl DatePickerWithPresets {
    pub fn new(
        open: impl IntoBoolModel,
        month: impl IntoCalendarMonthModel,
        selected: impl IntoOptionalDateModel,
    ) -> Self {
        Self {
            open: open.into_bool_model(),
            month: month.into_calendar_month_model(),
            selected: selected.into_optional_date_model(),
            control_id: None,
            test_id_prefix: None,
            preset_value: None,
            week_start: Weekday::Sunday,
            placeholder: Arc::from("Pick a date"),
            today_override: None,
            required: false,
            aria_invalid: false,
            disabled: false,
            show_outside_days: true,
            disable_outside_days: false,
            close_on_select: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Binds the outer trigger button to a logical form control id (similar to HTML `id`).
    ///
    /// This enables `Label::for_control(ControlId)` to focus the trigger and populate
    /// `aria-labelledby` / `aria-describedby`-like semantics via the control registry.
    pub fn control_id(mut self, id: impl Into<ControlId>) -> Self {
        self.control_id = Some(id.into());
        self
    }

    /// Sets a stable automation prefix for the presets trigger/content/select/calendar surfaces.
    ///
    /// Derived ids include `{prefix}-trigger`, `{prefix}-content`, `{prefix}-select-*`, and
    /// `{prefix}-calendar`. The inner `Select` and `Calendar` receive forwarded prefixes so their
    /// nested anchors remain stable for diagnostics.
    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
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

    pub fn show_outside_days(mut self, show: bool) -> Self {
        self.show_outside_days = show;
        self
    }

    pub fn disable_outside_days(mut self, disable: bool) -> Self {
        self.disable_outside_days = disable;
        self
    }

    /// Closes the popover after selecting a day from the embedded calendar.
    ///
    /// Default: `false`, matching the upstream presets example.
    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
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
            let theme = Theme::global(&*cx.app).snapshot();
            let open = self.open.clone();
            let month = self.month.clone();
            let selected = self.selected.clone();
            let control_id = self.control_id.clone();
            let test_id_prefix = self.test_id_prefix.clone();
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
            let close_on_select_open = self.close_on_select.then(|| open.clone());
            let required = self.required;
            let aria_invalid = self.aria_invalid;
            let initial_focus_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
                Rc::new(Cell::new(None));
            let trigger_test_id = test_id_prefix
                .as_ref()
                .map(|prefix| Arc::<str>::from(format!("{prefix}-trigger")));
            let content_test_id = test_id_prefix
                .as_ref()
                .map(|prefix| Arc::<str>::from(format!("{prefix}-content")));
            let select_test_id_prefix = test_id_prefix
                .as_ref()
                .map(|prefix| Arc::<str>::from(format!("{prefix}-select")));
            let calendar_test_id = test_id_prefix
                .as_ref()
                .map(|prefix| Arc::<str>::from(format!("{prefix}-calendar")));
            let calendar_icon = fret_icons::IconId::new_static("lucide.calendar");

            let selected_value = cx.watch_model(&selected).copied().flatten();
            let selected_empty = selected_value.is_none();
            let button_text: Arc<str> = match selected_value {
                Some(date) => format_selected_ppp_en(date),
                None => placeholder,
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
                            .toggle_model(open_trigger.clone())
                            .disabled(disabled)
                            .refine_style(chrome.clone())
                            .leading_icon(calendar_icon)
                            .children([content])
                            .refine_layout(layout.clone());
                        if let Some(control_id) = control_id.clone() {
                            button = button.control_id(control_id);
                        }

                        if selected_empty {
                            button = button.style(
                                ButtonStyle::default()
                                    .foreground(WidgetStateProperty::new(Some(muted_fg))),
                            );
                        }
                        if aria_invalid {
                            button = button.style(outline_trigger_invalid_style(&theme));
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
                        .value(SelectValue::new().placeholder("Select"))
                        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
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
                        ]);
                        let select = if let Some(prefix) = select_test_id_prefix.clone() {
                            select.test_id_prefix(prefix)
                        } else {
                            select
                        }
                        .into_element(cx);

                        let mut calendar = Calendar::new(month.clone(), selected.clone())
                            .week_start(week_start)
                            .today(today)
                            .required(required)
                            .show_outside_days(show_outside_days)
                            .disable_outside_days(disable_outside_days)
                            .initial_focus_out(initial_focus_out.clone());
                        if let Some(open) = close_on_select_open.clone() {
                            calendar = calendar.close_on_select(open);
                        }
                        if let Some(prefix) = calendar_test_id.clone() {
                            calendar = calendar.test_id_prefix(prefix);
                        }
                        let mut calendar = calendar.into_element(cx);
                        if let Some(test_id) = calendar_test_id.clone() {
                            calendar = calendar.test_id(test_id);
                        }

                        let border = theme.color_token("border");
                        let base_radius = theme.metric_token("metric.radius.lg");
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

                        let body = ui::v_stack(move |_cx| vec![select, calendar_container])
                            .gap(Space::N2)
                            .items_stretch()
                            .into_element(cx);

                        let mut content = PopoverContent::new([body])
                            .refine_style(ChromeRefinement::default().p(Space::N2))
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
        if (11..=13).contains(&mod_100) {
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
                    DatePickerWithPresets::new(open.clone(), month.clone(), selected.clone())
                        .today(Date::from_calendar_date(2026, Month::March, 15).unwrap())
                        .test_id_prefix("date-presets")
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
    fn date_picker_with_presets_trigger_width_is_intrinsic_unless_caller_overrides_it() {
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
            "date-picker-with-presets-trigger-intrinsic-width",
            |cx| {
                let open = cx.app.models_mut().insert(false);
                let month = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::March));
                let selected = cx.app.models_mut().insert(None::<Date>);

                DatePickerWithPresets::new(open, month, selected).into_element(cx)
            },
        );
        let fill = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "date-picker-with-presets-trigger-fill-width",
            |cx| {
                let open = cx.app.models_mut().insert(false);
                let month = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::March));
                let selected = cx.app.models_mut().insert(None::<Date>);

                DatePickerWithPresets::new(open, month, selected)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
            },
        );

        let intrinsic_pressable = find_first_pressable(&intrinsic).unwrap_or_else(|| {
            panic!("expected DatePickerWithPresets trigger to render a Pressable: {intrinsic:#?}")
        });
        let fill_pressable = find_first_pressable(&fill).unwrap_or_else(|| {
            panic!("expected DatePickerWithPresets trigger to render a Pressable: {fill:#?}")
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
    fn date_picker_with_presets_control_id_uses_registry_labelled_by_and_described_by_elements() {
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
            "presets-control-id",
            |cx| {
                let id = ControlId::from("invoice-date");
                let open = cx.app.models_mut().insert(false);
                let month = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::March));
                let selected = cx.app.models_mut().insert(None::<Date>);

                cx.column(fret_ui::element::ColumnProps::default(), move |cx| {
                    vec![
                        crate::field::FieldLabel::new("Invoice date")
                            .for_control(id.clone())
                            .into_element(cx),
                        crate::field::FieldDescription::new("Choose a date or a preset shortcut.")
                            .for_control(id.clone())
                            .into_element(cx),
                        DatePickerWithPresets::new(open.clone(), month.clone(), selected.clone())
                            .control_id(id)
                            .into_element(cx),
                    ]
                })
            },
        );

        let label_id = root.children[0].id;
        let description_id = root.children[1].id;
        let trigger = find_first_pressable(root.children.get(2).expect("presets picker child"))
            .expect("expected DatePickerWithPresets trigger pressable");

        assert_eq!(trigger.a11y.labelled_by_element, Some(label_id.0));
        assert_eq!(trigger.a11y.described_by_element, Some(description_id.0));
    }

    #[test]
    fn date_picker_with_presets_test_id_prefix_stamps_trigger_content_select_and_calendar() {
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
        let selected = app.models_mut().insert(None::<Date>);

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

        assert!(ids.iter().copied().any(|id| id == "date-presets-trigger"));
        assert!(ids.iter().copied().any(|id| id == "date-presets-content"));
        assert!(
            ids.iter()
                .copied()
                .any(|id| id == "date-presets-select-trigger")
        );
        assert!(ids.iter().copied().any(|id| id == "date-presets-calendar"));
        assert!(
            ids.iter()
                .copied()
                .any(|id| id == "date-presets-calendar:2026-03-01")
        );
    }

    #[test]
    fn date_picker_with_presets_required_exposes_required_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(480.0), Px(260.0)),
        );

        let open = app.models_mut().insert(false);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::March));
        let selected = app.models_mut().insert(None::<Date>);

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
            "date-picker-with-presets-required-semantics",
            |cx| {
                vec![
                    DatePickerWithPresets::new(open.clone(), month.clone(), selected.clone())
                        .required(true)
                        .test_id_prefix("required-date-picker-with-presets")
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
            .find(|n| n.test_id.as_deref() == Some("required-date-picker-with-presets-trigger"))
            .expect("date picker with presets trigger semantics");
        assert!(node.flags.required);
    }

    #[test]
    fn date_picker_with_presets_aria_invalid_exposes_invalid_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(480.0), Px(260.0)),
        );

        let open = app.models_mut().insert(false);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::March));
        let selected = app.models_mut().insert(None::<Date>);

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
            "date-picker-with-presets-invalid-semantics",
            |cx| {
                vec![
                    DatePickerWithPresets::new(open.clone(), month.clone(), selected.clone())
                        .aria_invalid(true)
                        .test_id_prefix("invalid-date-picker-with-presets")
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
            .find(|n| n.test_id.as_deref() == Some("invalid-date-picker-with-presets-trigger"))
            .expect("date picker with presets trigger semantics");
        assert_eq!(node.flags.invalid, Some(fret_core::SemanticsInvalid::True));
    }
}
