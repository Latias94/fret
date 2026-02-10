use std::sync::{Arc, Mutex};

use fret_core::{Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextStyle};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::combobox as kit_combobox;
use fret_ui_kit::primitives::controllable_state;
use fret_ui_kit::primitives::popover as radix_popover;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverrideSlot, Size, Space,
    WidgetState, WidgetStateProperty, WidgetStates, resolve_override_slot, ui,
};

use crate::{
    CommandItem, CommandList, CommandPalette, Drawer, DrawerContent, Popover, PopoverContent,
};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn test_id_slug(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

#[derive(Debug, Clone, Default)]
pub struct ComboboxStyle {
    pub trigger_background: OverrideSlot<ColorRef>,
    pub trigger_foreground: OverrideSlot<ColorRef>,
    pub trigger_border_color: OverrideSlot<ColorRef>,
}

impl ComboboxStyle {
    pub fn trigger_background(mut self, background: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.trigger_background = Some(background);
        self
    }

    pub fn trigger_foreground(mut self, foreground: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.trigger_foreground = Some(foreground);
        self
    }

    pub fn trigger_border_color(mut self, border: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.trigger_border_color = Some(border);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.trigger_background.is_some() {
            self.trigger_background = other.trigger_background;
        }
        if other.trigger_foreground.is_some() {
            self.trigger_foreground = other.trigger_foreground;
        }
        if other.trigger_border_color.is_some() {
            self.trigger_border_color = other.trigger_border_color;
        }
        self
    }
}

#[derive(Debug, Clone)]
pub struct ComboboxItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub disabled: bool,
}

impl ComboboxItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Default)]
struct ComboboxState {
    query: Option<Model<String>>,
    open_change_reason: Option<Model<Option<ComboboxOpenChangeReason>>>,
    clear_query_on_close: kit_combobox::ClearQueryOnCloseState,
    focus_restore_target: Option<Arc<Mutex<Option<GlobalElementId>>>>,
}

pub use kit_combobox::ComboboxOpenChangeReason;

type OnOpenChange = kit_combobox::OnOpenChange;
type OnOpenChangeWithReason = kit_combobox::OnOpenChangeWithReason;
type OnValueChange = Arc<dyn Fn(Option<Arc<str>>) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct Combobox {
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Option<Model<String>>,
    items: Vec<ComboboxItem>,
    test_id_prefix: Option<Arc<str>>,
    width: Option<Px>,
    responsive: bool,
    responsive_device_md_breakpoint: Px,
    placeholder: Arc<str>,
    search_placeholder: Arc<str>,
    empty_text: Arc<str>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    search_enabled: bool,
    consume_outside_pointer_events: bool,
    selection_commit_policy: kit_combobox::SelectionCommitPolicy,
    close_auto_focus_policy: kit_combobox::ComboboxCloseAutoFocusPolicy,
    on_value_change: Option<OnValueChange>,
    on_open_change: Option<OnOpenChange>,
    on_open_change_with_reason: Option<OnOpenChangeWithReason>,
    on_open_change_complete: Option<OnOpenChange>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: ComboboxStyle,
}

impl Combobox {
    pub fn new(model: Model<Option<Arc<str>>>, open: Model<bool>) -> Self {
        Self {
            model,
            open,
            query: None,
            items: Vec::new(),
            test_id_prefix: None,
            width: None,
            responsive: false,
            responsive_device_md_breakpoint: fret_ui_kit::declarative::viewport_tailwind::MD,
            placeholder: Arc::from("Select..."),
            search_placeholder: Arc::from("Search..."),
            empty_text: Arc::from("No results."),
            disabled: false,
            a11y_label: None,
            search_enabled: true,
            // shadcn/ui Combobox is a Popover + Command recipe; Popover is click-through by default.
            // (ADR 0069)
            consume_outside_pointer_events: false,
            selection_commit_policy: kit_combobox::SelectionCommitPolicy::default(),
            close_auto_focus_policy: kit_combobox::ComboboxCloseAutoFocusPolicy::default(),
            on_value_change: None,
            on_open_change: None,
            on_open_change_with_reason: None,
            on_open_change_complete: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: ComboboxStyle::default(),
        }
    }

    /// Creates a combobox with controlled/uncontrolled models:
    /// - `value` / `default_value` (selected item value)
    /// - `open` / `default_open` (popover visibility)
    ///
    /// This matches the Radix-style controlled vs uncontrolled contract (but note that upstream
    /// shadcn Combobox is a recipe, not a dedicated Radix primitive).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        value: Option<Model<Option<Arc<str>>>>,
        default_value: Option<Arc<str>>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let open = radix_popover::PopoverRoot::new()
            .open(open)
            .default_open(default_open)
            .open_model(cx);
        let value = controllable_state::use_controllable_model(cx, value, || default_value).model();
        Self::new(value, open)
    }

    /// When set, applies a fixed width to both the trigger and the popover content (shadcn demo
    /// uses `w-[200px]`).
    pub fn width(mut self, width: Px) -> Self {
        self.width = Some(width);
        self
    }

    /// When enabled, follows the upstream shadcn "responsive combobox" recipe: it uses a Drawer on
    /// narrow viewports (mobile) and a Popover on desktop.
    pub fn responsive(mut self, responsive: bool) -> Self {
        self.responsive = responsive;
        self
    }

    /// Overrides the device-level viewport breakpoint used by [`Combobox::responsive`].
    ///
    /// This is intentionally **viewport-driven** (mobile vs desktop), not container-query-driven.
    /// For panel-width responsiveness, prefer container queries (ADR 1170).
    pub fn responsive_device_md_breakpoint(mut self, breakpoint: Px) -> Self {
        self.responsive_device_md_breakpoint = breakpoint;
        self
    }

    pub fn query_model(mut self, query: Model<String>) -> Self {
        self.query = Some(query);
        self
    }

    pub fn item(mut self, item: ComboboxItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ComboboxItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn search_placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.search_placeholder = placeholder.into();
        self
    }

    pub fn empty_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.empty_text = text.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn search_enabled(mut self, enabled: bool) -> Self {
        self.search_enabled = enabled;
        self
    }

    pub fn consume_outside_pointer_events(mut self, consume: bool) -> Self {
        self.consume_outside_pointer_events = consume;
        self
    }

    pub fn selection_commit_policy(mut self, policy: kit_combobox::SelectionCommitPolicy) -> Self {
        self.selection_commit_policy = policy;
        self
    }

    pub fn close_auto_focus_policy(
        mut self,
        policy: kit_combobox::ComboboxCloseAutoFocusPolicy,
    ) -> Self {
        self.close_auto_focus_policy = policy;
        self
    }

    /// Called when selected value changes (Base UI `onValueChange`).
    pub fn on_value_change(mut self, on_value_change: Option<OnValueChange>) -> Self {
        self.on_value_change = on_value_change;
        self
    }

    /// Called when the open state changes (Base UI `onOpenChange`).
    pub fn on_open_change(mut self, on_open_change: Option<OnOpenChange>) -> Self {
        self.on_open_change = on_open_change;
        self
    }

    /// Called when the open state changes, with reason metadata.
    pub fn on_open_change_with_reason(
        mut self,
        on_open_change_with_reason: Option<OnOpenChangeWithReason>,
    ) -> Self {
        self.on_open_change_with_reason = on_open_change_with_reason;
        self
    }

    /// Called when open/close transition settles (Base UI `onOpenChangeComplete`).
    pub fn on_open_change_complete(
        mut self,
        on_open_change_complete: Option<OnOpenChange>,
    ) -> Self {
        self.on_open_change_complete = on_open_change_complete;
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

    pub fn style(mut self, style: ComboboxStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        combobox_with_patch(
            cx,
            self.model,
            self.open,
            self.query,
            &self.items,
            self.test_id_prefix,
            self.width,
            self.placeholder,
            self.search_placeholder,
            self.empty_text,
            self.disabled,
            self.a11y_label,
            self.responsive,
            self.responsive_device_md_breakpoint,
            self.search_enabled,
            self.consume_outside_pointer_events,
            self.selection_commit_policy,
            self.close_auto_focus_policy,
            self.on_value_change,
            self.on_open_change,
            self.on_open_change_with_reason,
            self.on_open_change_complete,
            self.chrome,
            self.layout,
            self.style,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn combobox<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Option<Model<String>>,
    items: &[ComboboxItem],
    width: Option<Px>,
    placeholder: Arc<str>,
    search_placeholder: Arc<str>,
    empty_text: Arc<str>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    search_enabled: bool,
    consume_outside_pointer_events: bool,
) -> AnyElement {
    combobox_with_patch(
        cx,
        model,
        open,
        query,
        items,
        None,
        width,
        placeholder,
        search_placeholder,
        empty_text,
        disabled,
        a11y_label,
        false,
        fret_ui_kit::declarative::viewport_tailwind::MD,
        search_enabled,
        consume_outside_pointer_events,
        kit_combobox::SelectionCommitPolicy::default(),
        kit_combobox::ComboboxCloseAutoFocusPolicy::default(),
        None,
        None,
        None,
        None,
        ChromeRefinement::default(),
        LayoutRefinement::default(),
        ComboboxStyle::default(),
    )
}

#[allow(clippy::too_many_arguments)]
fn combobox_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Option<Model<String>>,
    items: &[ComboboxItem],
    test_id_prefix: Option<Arc<str>>,
    width: Option<Px>,
    placeholder: Arc<str>,
    search_placeholder: Arc<str>,
    empty_text: Arc<str>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    responsive: bool,
    responsive_device_md_breakpoint: Px,
    search_enabled: bool,
    consume_outside_pointer_events: bool,
    selection_commit_policy: kit_combobox::SelectionCommitPolicy,
    close_auto_focus_policy: kit_combobox::ComboboxCloseAutoFocusPolicy,
    on_value_change: Option<OnValueChange>,
    on_open_change: Option<OnOpenChange>,
    on_open_change_with_reason: Option<OnOpenChangeWithReason>,
    on_open_change_complete: Option<OnOpenChange>,
    chrome_patch: ChromeRefinement,
    layout_patch: LayoutRefinement,
    style_override: ComboboxStyle,
) -> AnyElement {
    cx.scope(|cx| {
        let theme = Theme::global(&*cx.app).clone();
        let open_change_reason_model = {
            let existing =
                cx.with_state(ComboboxState::default, |st| st.open_change_reason.clone());
            if let Some(model) = existing {
                model
            } else {
                let model = cx.app.models_mut().insert(None::<ComboboxOpenChangeReason>);
                cx.with_state(ComboboxState::default, |st| {
                    st.open_change_reason = Some(model.clone())
                });
                model
            }
        };
        let focus_restore_target = {
            let existing =
                cx.with_state(ComboboxState::default, |st| st.focus_restore_target.clone());
            if let Some(cell) = existing {
                cell
            } else {
                let cell: Arc<Mutex<Option<GlobalElementId>>> = Arc::new(Mutex::new(None));
                cx.with_state(ComboboxState::default, |st| {
                    st.focus_restore_target = Some(cell.clone());
                });
                cell
            }
        };
        let close_auto_focus = kit_combobox::on_close_auto_focus_with_reason(
            open_change_reason_model.clone(),
            focus_restore_target.clone(),
            close_auto_focus_policy,
        );
        let selected = cx.watch_model(&model).cloned().unwrap_or_default();
        if let Some(handler) = on_value_change.as_ref() {
            let value_change = cx.with_state(
                kit_combobox::ValueChangeCallbackState::<Arc<str>>::default,
                |state| kit_combobox::value_change_event(state, selected.clone()),
            );
            if let Some(value) = value_change {
                handler(value);
            }
        }
        let is_open = cx.watch_model(&open).layout().copied().unwrap_or(false);
        let open_change_reason = cx
            .watch_model(&open_change_reason_model)
            .layout()
            .copied()
            .unwrap_or(None)
            .unwrap_or(ComboboxOpenChangeReason::None);
        let (open_change, open_change_complete) = cx
            .with_state(kit_combobox::OpenChangeCallbackState::default, |state| {
                kit_combobox::open_change_events(state, is_open, is_open, false)
            });
        if let (Some(open), Some(handler)) = (open_change, on_open_change.as_ref()) {
            handler(open);
        }
        if let (Some(open), Some(handler)) = (open_change, on_open_change_with_reason.as_ref()) {
            handler(open, open_change_reason);
        }
        if let (Some(open), Some(handler)) =
            (open_change_complete, on_open_change_complete.as_ref())
        {
            handler(open);
        }

        let query_model = if let Some(q) = query {
            cx.with_state(ComboboxState::default, |st| st.query = Some(q.clone()));
            q
        } else {
            let existing = cx.with_state(ComboboxState::default, |st| st.query.clone());
            if let Some(m) = existing {
                m
            } else {
                let m = cx.app.models_mut().insert(String::new());
                cx.with_state(ComboboxState::default, |st| st.query = Some(m.clone()));
                m
            }
        };

        let should_clear_query = cx.with_state(ComboboxState::default, |st| {
            kit_combobox::should_clear_query_on_close(&mut st.clear_query_on_close, is_open)
        });
        if should_clear_query {
            let _ = cx.app.models_mut().update(&query_model, |v| v.clear());
        }

        let size = Size::default();
        let radius = chrome_patch
            .radius
            .as_ref()
            .map(|m| m.resolve(&theme))
            .unwrap_or_else(|| size.control_radius(&theme));
        let ring = decl_style::focus_ring(&theme, radius);

        let resolved_label = selected
            .as_ref()
            .and_then(|v| items.iter().find(|it| it.value.as_ref() == v.as_ref()))
            .map(|it| it.label.clone())
            .unwrap_or(placeholder.clone());

        let text_style = TextStyle {
            font: FontId::default(),
            size: size.control_text_px(&theme),
            weight: FontWeight::MEDIUM,
            slant: Default::default(),
            line_height: theme
                .metric_by_key("font.line_height")
                .or(Some(theme.metric_required("font.line_height"))),
            letter_spacing_em: None,
        };

        let min_h = chrome_patch
            .min_height
            .as_ref()
            .map(|m| m.resolve(&theme))
            .unwrap_or_else(|| size.button_h(&theme));
        let pad_x = size.button_px(&theme);
        let pad_y = size.button_py(&theme);
        let border_w = chrome_patch
            .border_width
            .as_ref()
            .map(|m| m.resolve(&theme))
            .unwrap_or(Px(1.0));

        let mut trigger_layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .min_h(min_h)
                .merge(if let Some(w) = width {
                    LayoutRefinement::default().w_px(w)
                } else {
                    LayoutRefinement::default().w_full()
                })
                .merge(layout_patch),
        );
        trigger_layout.size.height = Length::Auto;
        trigger_layout.size.min_height = Some(min_h);

        let bg_base = chrome_patch
            .background
            .as_ref()
            .map(|c| c.resolve(&theme))
            .unwrap_or_else(|| {
                theme
                    .color_by_key("background")
                    .unwrap_or_else(|| theme.color_required("background"))
            });
        let bg_hover = theme
            .color_by_key("accent")
            .or_else(|| theme.color_by_key("accent.background"))
            .unwrap_or_else(|| theme.color_required("accent"));
        let bg_pressed = theme.color_required("accent");
        let fg_base = chrome_patch
            .text_color
            .as_ref()
            .map(|c| c.resolve(&theme))
            .unwrap_or_else(|| {
                theme
                    .color_by_key("foreground")
                    .unwrap_or_else(|| theme.color_required("foreground"))
            });
        let fg_hover = theme
            .color_by_key("accent-foreground")
            .or_else(|| theme.color_by_key("accent.foreground"))
            .unwrap_or(fg_base);
        let border_base = chrome_patch
            .border_color
            .as_ref()
            .map(|c| c.resolve(&theme))
            .unwrap_or_else(|| {
                theme
                    .color_by_key("input")
                    .or_else(|| theme.color_by_key("border"))
                    .unwrap_or_else(|| theme.color_required("border"))
            });

        let default_trigger_bg = WidgetStateProperty::new(ColorRef::Color(bg_base))
            .when(WidgetStates::HOVERED, ColorRef::Color(bg_hover))
            .when(WidgetStates::ACTIVE, ColorRef::Color(bg_pressed));
        let default_trigger_fg = WidgetStateProperty::new(ColorRef::Color(fg_base))
            .when(WidgetStates::HOVERED, ColorRef::Color(fg_hover))
            .when(WidgetStates::ACTIVE, ColorRef::Color(fg_hover));
        let default_trigger_border = WidgetStateProperty::new(ColorRef::Color(border_base));

        let enabled = !disabled;
        let items: Vec<ComboboxItem> = items.to_vec();
        let open_for_trigger = open.clone();
        let trigger_gap = MetricRef::space(Space::N2).resolve(&theme);
        let a11y_label_for_trigger = a11y_label.clone();

        let padding = chrome_patch.padding.clone().unwrap_or_default();
        let pad_top = padding.top.map(|m| m.resolve(&theme)).unwrap_or(pad_y);
        let pad_right = padding.right.map(|m| m.resolve(&theme)).unwrap_or(pad_x);
        let pad_bottom = padding.bottom.map(|m| m.resolve(&theme)).unwrap_or(pad_y);
        let pad_left = padding.left.map(|m| m.resolve(&theme)).unwrap_or(pad_x);

        let theme_for_trigger = theme.clone();

        // Device-level responsiveness: shadcn's "responsive combobox" uses Drawer on mobile.
        // This is a viewport breakpoint by design (not a container query).
        let is_desktop = fret_ui_kit::declarative::viewport_width_at_least(
            cx,
            Invalidation::Layout,
            responsive_device_md_breakpoint,
            fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
        );
        if responsive && !is_desktop {
            let open_change_reason_model_for_trigger = open_change_reason_model.clone();
            let open_change_reason_model_for_content = open_change_reason_model.clone();
            let test_id_prefix_for_content = test_id_prefix.clone();
            let focus_restore_target_for_trigger = focus_restore_target.clone();
            return Drawer::new(open.clone())
                .on_dismiss_request(Some(
                    kit_combobox::set_open_change_reason_on_dismiss_request(
                        open_change_reason_model.clone(),
                    ),
                ))
                .on_close_auto_focus(Some(close_auto_focus.clone()))
                .into_element(
                    cx,
                    move |cx| {
                        let open_change_reason_model = open_change_reason_model_for_trigger.clone();
                        let focus_restore_target = focus_restore_target_for_trigger.clone();
                        cx.pressable_with_id_props(|cx, st, trigger_id| {
                            *focus_restore_target
                                .lock()
                                .unwrap_or_else(|e| e.into_inner()) = Some(trigger_id);
                            let mut states = WidgetStates::from_pressable(cx, st, enabled);
                            states.set(WidgetState::Open, is_open);

                            let bg_ref = resolve_override_slot(
                                style_override.trigger_background.as_ref(),
                                &default_trigger_bg,
                                states,
                            );
                            let fg_ref = resolve_override_slot(
                                style_override.trigger_foreground.as_ref(),
                                &default_trigger_fg,
                                states,
                            );
                            let border_ref = resolve_override_slot(
                                style_override.trigger_border_color.as_ref(),
                                &default_trigger_border,
                                states,
                            );

                            let bg = bg_ref.resolve(&theme_for_trigger);
                            let fg = fg_ref.resolve(&theme_for_trigger);
                            let border = border_ref.resolve(&theme_for_trigger);
                            let icon_fg = alpha_mul(fg, 0.5);

                            cx.pressable_add_on_activate(
                                kit_combobox::set_open_change_reason_on_activate(
                                    open_change_reason_model.clone(),
                                    ComboboxOpenChangeReason::TriggerPress,
                                ),
                            );
                            cx.pressable_toggle_bool(&open_for_trigger);

                            let props = PressableProps {
                                layout: trigger_layout,
                                enabled,
                                focusable: true,
                                focus_ring: Some(ring),
                                a11y: PressableA11y {
                                    role: Some(SemanticsRole::ComboBox),
                                    label: a11y_label_for_trigger
                                        .clone()
                                        .or_else(|| Some(resolved_label.clone())),
                                    expanded: Some(is_open),
                                    ..Default::default()
                                },
                                ..Default::default()
                            };

                            let children = vec![cx.container(
                                ContainerProps {
                                    layout: LayoutStyle::default(),
                                    padding: Edges {
                                        top: pad_top,
                                        right: pad_right,
                                        bottom: pad_bottom,
                                        left: pad_left,
                                    },
                                    background: Some(bg),
                                    shadow: None,
                                    border: Edges::all(border_w),
                                    border_color: Some(border),
                                    corner_radii: Corners::all(radius),
                                    ..Default::default()
                                },
                                move |cx| {
                                    vec![cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap: trigger_gap,
                                            padding: Edges::all(Px(0.0)),
                                            justify: MainAlign::SpaceBetween,
                                            align: CrossAlign::Center,
                                            wrap: false,
                                        },
                                        move |cx| {
                                            let label_style = text_style.clone();
                                            vec![
                                                {
                                                    let mut label =
                                                        ui::label(cx, resolved_label.clone())
                                                            .w_full()
                                                            .min_w_0()
                                                            .flex_1()
                                                            .basis_0()
                                                            .text_size_px(label_style.size)
                                                            .font_weight(label_style.weight)
                                                            .text_color(fg_ref.clone())
                                                            .truncate();
                                                    if let Some(line_height) =
                                                        label_style.line_height
                                                    {
                                                        label = label.line_height_px(line_height);
                                                    }
                                                    if let Some(letter_spacing_em) =
                                                        label_style.letter_spacing_em
                                                    {
                                                        label = label
                                                            .letter_spacing_em(letter_spacing_em);
                                                    }
                                                    label.into_element(cx)
                                                },
                                                decl_icon::icon_with(
                                                    cx,
                                                    ids::ui::CHEVRON_DOWN,
                                                    Some(Px(16.0)),
                                                    Some(ColorRef::Color(icon_fg)),
                                                ),
                                            ]
                                        },
                                    )]
                                },
                            )];

                            (props, children)
                        })
                    },
                    move |cx| {
                        let test_id_prefix = test_id_prefix_for_content.clone();
                        let open_change_reason_model = open_change_reason_model_for_content.clone();
                        let theme_max_list_h = theme
                            .metric_by_key("component.combobox.max_list_height")
                            .or_else(|| theme.metric_by_key("component.select.max_list_height"))
                            .unwrap_or(Px(280.0));

                        let transparent = Color::TRANSPARENT;
                        let list = if search_enabled {
                            let max_list_h = Px(theme_max_list_h.0.max(0.0));

                            let mut command_items: Vec<CommandItem> =
                                Vec::with_capacity(items.len());
                            for item in items.iter().cloned() {
                                let item_disabled = disabled || item.disabled;
                                let is_selected = selected
                                    .as_ref()
                                    .is_some_and(|v| v.as_ref() == item.value.as_ref());

                                let model_for_select = model.clone();
                                let open_for_select = open.clone();
                                let query_for_select = query_model.clone();
                                let open_change_reason_model_for_select =
                                    open_change_reason_model.clone();
                                let value_for_select = item.value.clone();
                                let on_select = kit_combobox::commit_selection_on_activate(
                                    selection_commit_policy,
                                    model_for_select,
                                    open_for_select,
                                    query_for_select,
                                    open_change_reason_model_for_select,
                                    value_for_select,
                                );

                                let mut cmd_item = CommandItem::new(item.label.clone())
                                    .value(item.value.clone())
                                    .disabled(item_disabled)
                                    .checkmark(is_selected)
                                    .on_select_action(on_select);
                                if let Some(prefix) = test_id_prefix.as_deref() {
                                    cmd_item = cmd_item.test_id(format!(
                                        "{prefix}-item-{}",
                                        test_id_slug(item.value.as_ref())
                                    ));
                                }
                                command_items.push(cmd_item);
                            }

                            CommandPalette::new(query_model.clone(), command_items)
                                .a11y_label("Combobox list")
                                .input_role(SemanticsRole::ComboBox)
                                .input_expanded(true)
                                .placeholder(search_placeholder.clone())
                                .disabled(disabled)
                                .empty_text(empty_text.clone())
                                .refine_style(
                                    ChromeRefinement::default()
                                        .radius(Px(0.0))
                                        .border_width(Px(0.0))
                                        .bg(ColorRef::Color(transparent))
                                        .border_color(ColorRef::Color(transparent)),
                                )
                                .refine_scroll_layout(LayoutRefinement::default().max_h(max_list_h))
                                .into_element(cx)
                        } else {
                            let max_list_h = Px(theme_max_list_h.0.max(0.0));

                            let fg = theme
                                .color_by_key("foreground")
                                .unwrap_or_else(|| theme.color_required("foreground"));
                            let fg_disabled = alpha_mul(fg, 0.5);
                            let item_text_style = crate::command::item_text_style(&theme);

                            let mut command_items: Vec<CommandItem> =
                                Vec::with_capacity(items.len());
                            for item in items.iter().cloned() {
                                let item_disabled = disabled || item.disabled;
                                let is_selected = selected
                                    .as_ref()
                                    .is_some_and(|v| v.as_ref() == item.value.as_ref());

                                let model_for_select = model.clone();
                                let open_for_select = open.clone();
                                let query_for_select = query_model.clone();
                                let open_change_reason_model_for_select =
                                    open_change_reason_model.clone();
                                let value_for_select = item.value.clone();
                                let on_select = kit_combobox::commit_selection_on_activate(
                                    selection_commit_policy,
                                    model_for_select,
                                    open_for_select,
                                    query_for_select,
                                    open_change_reason_model_for_select,
                                    value_for_select,
                                );

                                let label_text = item.label.clone();
                                let label_style = item_text_style.clone();
                                let icon = decl_icon::icon_with(
                                    cx,
                                    ids::ui::CHECK,
                                    Some(Px(16.0)),
                                    Some(ColorRef::Color(if item_disabled {
                                        fg_disabled
                                    } else {
                                        fg
                                    })),
                                );
                                let icon = cx
                                    .opacity(if is_selected { 1.0 } else { 0.0 }, move |_cx| {
                                        vec![icon]
                                    });

                                let text = {
                                    let mut label = ui::label(cx, label_text.clone())
                                        .text_size_px(label_style.size)
                                        .font_weight(label_style.weight)
                                        .text_color(ColorRef::Color(if item_disabled {
                                            fg_disabled
                                        } else {
                                            fg
                                        }))
                                        .truncate();
                                    if let Some(line_height) = label_style.line_height {
                                        label = label.line_height_px(line_height);
                                    }
                                    if let Some(letter_spacing_em) = label_style.letter_spacing_em {
                                        label = label.letter_spacing_em(letter_spacing_em);
                                    }
                                    label.into_element(cx)
                                };

                                let mut cmd_item = CommandItem::new(label_text)
                                    .value(item.value.clone())
                                    .disabled(item_disabled)
                                    .on_select_action(on_select)
                                    .children(vec![text, icon]);
                                if let Some(prefix) = test_id_prefix.as_deref() {
                                    cmd_item = cmd_item.test_id(format!(
                                        "{prefix}-item-{}",
                                        test_id_slug(item.value.as_ref())
                                    ));
                                }
                                command_items.push(cmd_item);
                            }

                            CommandList::new(command_items)
                                .disabled(disabled)
                                .empty_text(empty_text.clone())
                                .refine_scroll_layout(LayoutRefinement::default().max_h(max_list_h))
                                .into_element(cx)
                        };

                        DrawerContent::new(vec![list])
                            .refine_style(ChromeRefinement::default().p(Space::N0))
                            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                            .into_element(cx)
                    },
                );
        }

        let open_change_reason_model_for_trigger = open_change_reason_model.clone();
        let open_change_reason_model_for_content = open_change_reason_model.clone();
        let test_id_prefix_for_content = test_id_prefix.clone();
        let focus_restore_target_for_trigger = focus_restore_target.clone();
        Popover::new(open.clone())
            .auto_focus(true)
            .consume_outside_pointer_events(consume_outside_pointer_events)
            .on_dismiss_request(Some(
                kit_combobox::set_open_change_reason_on_dismiss_request(
                    open_change_reason_model.clone(),
                ),
            ))
            .on_close_auto_focus(Some(close_auto_focus.clone()))
            .into_element_with_anchor(
                cx,
                move |cx| {
                    let open_change_reason_model = open_change_reason_model_for_trigger.clone();
                    let focus_restore_target = focus_restore_target_for_trigger.clone();
                    cx.pressable_with_id_props(|cx, st, trigger_id| {
                        *focus_restore_target
                            .lock()
                            .unwrap_or_else(|e| e.into_inner()) = Some(trigger_id);
                        let mut states = WidgetStates::from_pressable(cx, st, enabled);
                        states.set(WidgetState::Open, is_open);

                        let bg_ref = resolve_override_slot(
                            style_override.trigger_background.as_ref(),
                            &default_trigger_bg,
                            states,
                        );
                        let fg_ref = resolve_override_slot(
                            style_override.trigger_foreground.as_ref(),
                            &default_trigger_fg,
                            states,
                        );
                        let border_ref = resolve_override_slot(
                            style_override.trigger_border_color.as_ref(),
                            &default_trigger_border,
                            states,
                        );

                        let bg = bg_ref.resolve(&theme_for_trigger);
                        let fg = fg_ref.resolve(&theme_for_trigger);
                        let border = border_ref.resolve(&theme_for_trigger);
                        let icon_fg = alpha_mul(fg, 0.5);

                        cx.pressable_add_on_activate(
                            kit_combobox::set_open_change_reason_on_activate(
                                open_change_reason_model.clone(),
                                ComboboxOpenChangeReason::TriggerPress,
                            ),
                        );
                        cx.pressable_toggle_bool(&open_for_trigger);

                        let props = PressableProps {
                            layout: trigger_layout,
                            enabled,
                            focusable: true,
                            focus_ring: Some(ring),
                            a11y: PressableA11y {
                                role: Some(SemanticsRole::ComboBox),
                                label: a11y_label_for_trigger
                                    .clone()
                                    .or_else(|| Some(resolved_label.clone())),
                                expanded: Some(is_open),
                                ..Default::default()
                            },
                            ..Default::default()
                        };

                        let children = vec![cx.container(
                            ContainerProps {
                                layout: LayoutStyle::default(),
                                padding: Edges {
                                    top: pad_top,
                                    right: pad_right,
                                    bottom: pad_bottom,
                                    left: pad_left,
                                },
                                background: Some(bg),
                                shadow: None,
                                border: Edges::all(border_w),
                                border_color: Some(border),
                                corner_radii: Corners::all(radius),
                                ..Default::default()
                            },
                            move |cx| {
                                vec![cx.flex(
                                    FlexProps {
                                        layout: LayoutStyle::default(),
                                        direction: fret_core::Axis::Horizontal,
                                        gap: trigger_gap,
                                        padding: Edges::all(Px(0.0)),
                                        justify: MainAlign::SpaceBetween,
                                        align: CrossAlign::Center,
                                        wrap: false,
                                    },
                                    move |cx| {
                                        let label_style = text_style.clone();
                                        vec![
                                            {
                                                let mut label =
                                                    ui::label(cx, resolved_label.clone())
                                                        .w_full()
                                                        .min_w_0()
                                                        .flex_1()
                                                        .basis_0()
                                                        .text_size_px(label_style.size)
                                                        .font_weight(label_style.weight)
                                                        .text_color(fg_ref.clone())
                                                        .truncate();
                                                if let Some(line_height) = label_style.line_height {
                                                    label = label.line_height_px(line_height);
                                                }
                                                if let Some(letter_spacing_em) =
                                                    label_style.letter_spacing_em
                                                {
                                                    label =
                                                        label.letter_spacing_em(letter_spacing_em);
                                                }
                                                label.into_element(cx)
                                            },
                                            decl_icon::icon_with(
                                                cx,
                                                ids::ui::CHEVRON_DOWN,
                                                Some(Px(16.0)),
                                                Some(ColorRef::Color(icon_fg)),
                                            ),
                                        ]
                                    },
                                )]
                            },
                        )];

                        (props, children)
                    })
                },
                move |cx, anchor| {
                    let test_id_prefix = test_id_prefix_for_content.clone();
                    let open_change_reason_model = open_change_reason_model_for_content.clone();
                    let theme_max_list_h = theme
                        .metric_by_key("component.combobox.max_list_height")
                        .or_else(|| theme.metric_by_key("component.select.max_list_height"))
                        .unwrap_or(Px(280.0));
                    let desired_w = width.unwrap_or_else(|| Px(anchor.size.width.0.max(180.0)));

                    let transparent = Color::TRANSPARENT;
                    let list = if search_enabled {
                        let max_list_h = Px(theme_max_list_h.0.max(0.0));

                        let mut command_items: Vec<CommandItem> = Vec::with_capacity(items.len());
                        for item in items.iter().cloned() {
                            let item_disabled = disabled || item.disabled;
                            let is_selected = selected
                                .as_ref()
                                .is_some_and(|v| v.as_ref() == item.value.as_ref());

                            let model_for_select = model.clone();
                            let open_for_select = open.clone();
                            let query_for_select = query_model.clone();
                            let open_change_reason_model_for_select =
                                open_change_reason_model.clone();
                            let value_for_select = item.value.clone();
                            let on_select = kit_combobox::commit_selection_on_activate(
                                selection_commit_policy,
                                model_for_select,
                                open_for_select,
                                query_for_select,
                                open_change_reason_model_for_select,
                                value_for_select,
                            );

                            let mut cmd_item = CommandItem::new(item.label.clone())
                                .value(item.value.clone())
                                .disabled(item_disabled)
                                .checkmark(is_selected)
                                .on_select_action(on_select);
                            if let Some(prefix) = test_id_prefix.as_deref() {
                                cmd_item = cmd_item.test_id(format!(
                                    "{prefix}-item-{}",
                                    test_id_slug(item.value.as_ref())
                                ));
                            }
                            command_items.push(cmd_item);
                        }

                        CommandPalette::new(query_model.clone(), command_items)
                            .a11y_label("Combobox list")
                            .input_role(SemanticsRole::ComboBox)
                            .input_expanded(true)
                            .placeholder(search_placeholder.clone())
                            .disabled(disabled)
                            .empty_text(empty_text)
                            .refine_style(
                                ChromeRefinement::default()
                                    .radius(Px(0.0))
                                    .border_width(Px(0.0))
                                    .bg(ColorRef::Color(transparent))
                                    .border_color(ColorRef::Color(transparent)),
                            )
                            .refine_scroll_layout(LayoutRefinement::default().max_h(max_list_h))
                            .into_element(cx)
                    } else {
                        let max_list_h = Px(theme_max_list_h.0.max(0.0));

                        let fg = theme
                            .color_by_key("foreground")
                            .unwrap_or_else(|| theme.color_required("foreground"));
                        let fg_disabled = alpha_mul(fg, 0.5);
                        let item_text_style = crate::command::item_text_style(&theme);

                        let mut command_items: Vec<CommandItem> = Vec::with_capacity(items.len());
                        for item in items.iter().cloned() {
                            let item_disabled = disabled || item.disabled;
                            let is_selected = selected
                                .as_ref()
                                .is_some_and(|v| v.as_ref() == item.value.as_ref());

                            let model_for_select = model.clone();
                            let open_for_select = open.clone();
                            let query_for_select = query_model.clone();
                            let open_change_reason_model_for_select =
                                open_change_reason_model.clone();
                            let value_for_select = item.value.clone();
                            let on_select = kit_combobox::commit_selection_on_activate(
                                selection_commit_policy,
                                model_for_select,
                                open_for_select,
                                query_for_select,
                                open_change_reason_model_for_select,
                                value_for_select,
                            );

                            let label_text = item.label.clone();
                            let label_style = item_text_style.clone();
                            let icon = decl_icon::icon_with(
                                cx,
                                ids::ui::CHECK,
                                Some(Px(16.0)),
                                Some(ColorRef::Color(if item_disabled {
                                    fg_disabled
                                } else {
                                    fg
                                })),
                            );
                            let icon = cx
                                .opacity(if is_selected { 1.0 } else { 0.0 }, move |_cx| {
                                    vec![icon]
                                });

                            let text = {
                                let mut label = ui::label(cx, label_text.clone())
                                    .text_size_px(label_style.size)
                                    .font_weight(label_style.weight)
                                    .text_color(ColorRef::Color(if item_disabled {
                                        fg_disabled
                                    } else {
                                        fg
                                    }))
                                    .truncate();
                                if let Some(line_height) = label_style.line_height {
                                    label = label.line_height_px(line_height);
                                }
                                if let Some(letter_spacing_em) = label_style.letter_spacing_em {
                                    label = label.letter_spacing_em(letter_spacing_em);
                                }
                                label.into_element(cx)
                            };

                            let mut cmd_item = CommandItem::new(label_text)
                                .value(item.value.clone())
                                .disabled(item_disabled)
                                .on_select_action(on_select)
                                .children(vec![text, icon]);
                            if let Some(prefix) = test_id_prefix.as_deref() {
                                cmd_item = cmd_item.test_id(format!(
                                    "{prefix}-item-{}",
                                    test_id_slug(item.value.as_ref())
                                ));
                            }
                            command_items.push(cmd_item);
                        }

                        CommandList::new(command_items)
                            .disabled(disabled)
                            .empty_text(empty_text)
                            .refine_scroll_layout(LayoutRefinement::default().max_h(max_list_h))
                            .into_element(cx)
                    };

                    PopoverContent::new(vec![list])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(LayoutRefinement::default().w_px(desired_w).min_w_0())
                        .into_element(cx)
                },
            )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::{Cell, RefCell};

    use fret_app::App;
    use fret_core::{
        AppWindowId, Point, Px, Rect, SemanticsRole, Size, SvgId, SvgService, TextBlobId,
        TextConstraints, TextMetrics, TextService, UiServices,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_runtime::FrameId;
    use fret_ui::tree::UiTree;
    use fret_ui_kit::primitives::popover as radix_popover;

    #[derive(Default)]
    struct FakeServices;

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
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
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        open: Model<bool>,
        items: Vec<ComboboxItem>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "combobox",
            |cx| vec![Combobox::new(model, open).items(items).into_element(cx)],
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_with_underlay(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        open: Model<bool>,
        items: Vec<ComboboxItem>,
        underlay_clicked: Model<bool>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "combobox-underlay",
            move |cx| {
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        a11y: PressableA11y {
                            role: Some(SemanticsRole::Button),
                            label: Some(Arc::from("Underlay")),
                            test_id: Some(Arc::from("underlay")),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx, _st| {
                        cx.pressable_toggle_bool(&underlay_clicked);
                        Vec::new()
                    },
                );
                vec![
                    underlay,
                    Combobox::new(model, open).items(items).into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn combobox_new_controllable_creates_internal_models_and_applies_defaults() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let model_id_out = Cell::new(None);
        let open_id_out = Cell::new(None);
        let model_out: RefCell<Option<Model<Option<Arc<str>>>>> = RefCell::new(None);
        let open_out: RefCell<Option<Model<bool>>> = RefCell::new(None);

        let items = vec![
            ComboboxItem::new("alpha", "Alpha"),
            ComboboxItem::new("beta", "Beta"),
        ];

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices,
                      model_id_out: &Cell<Option<fret_runtime::ModelId>>,
                      open_id_out: &Cell<Option<fret_runtime::ModelId>>,
                      model_out: &RefCell<Option<Model<Option<Arc<str>>>>>,
                      open_out: &RefCell<Option<Model<bool>>>| {
            let next_frame = FrameId(app.frame_id().0.saturating_add(1));
            app.set_frame_id(next_frame);

            fret_ui_kit::OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "combobox-controllable",
                |cx| {
                    vec![cx.keyed("combobox", |cx| {
                        let combobox = Combobox::new_controllable(
                            cx,
                            None,
                            Some(Arc::from("beta")),
                            None,
                            false,
                        )
                        .items(items.clone());
                        model_id_out.set(Some(combobox.model.id()));
                        open_id_out.set(Some(combobox.open.id()));
                        *model_out.borrow_mut() = Some(combobox.model.clone());
                        *open_out.borrow_mut() = Some(combobox.open.clone());
                        combobox.into_element(cx)
                    })]
                },
            );
            ui.set_root(root);
            fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
            ui.layout_all(app, services, bounds, 1.0);
        };

        render(
            &mut ui,
            &mut app,
            &mut services,
            &model_id_out,
            &open_id_out,
            &model_out,
            &open_out,
        );
        let first_model = model_id_out.get().expect("value model id");
        let first_open = open_id_out.get().expect("open model id");
        let value = model_out
            .borrow()
            .as_ref()
            .expect("value model")
            .read_ref(&app, |v| v.clone())
            .expect("read value");
        assert_eq!(value.as_deref(), Some("beta"));
        let is_open = open_out
            .borrow()
            .as_ref()
            .expect("open model")
            .read_ref(&app, |v| *v)
            .expect("read open");
        assert!(!is_open);

        render(
            &mut ui,
            &mut app,
            &mut services,
            &model_id_out,
            &open_id_out,
            &model_out,
            &open_out,
        );
        let second_model = model_id_out.get().expect("value model id (second render)");
        let second_open = open_id_out.get().expect("open model id (second render)");
        assert_eq!(first_model, second_model);
        assert_eq!(first_open, second_open);
    }

    #[test]
    fn combobox_new_controllable_prefers_controlled_models() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let value = app.models_mut().insert(Some(Arc::from("alpha")));
        let open = app.models_mut().insert(true);

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );

        let seen = Cell::new(false);
        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "combobox-controlled",
            |cx| {
                vec![cx.keyed("combobox", |cx| {
                    let combobox = Combobox::new_controllable(
                        cx,
                        Some(value.clone()),
                        Some(Arc::from("beta")),
                        Some(open.clone()),
                        false,
                    );
                    assert_eq!(combobox.model, value);
                    assert_eq!(combobox.open, open);
                    seen.set(true);
                    combobox.into_element(cx)
                })]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        assert!(seen.get());
    }

    #[test]
    fn combobox_open_change_builder_sets_handler() {
        let mut app = App::new();
        let value = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let combobox = Combobox::new(value, open).on_open_change(Some(Arc::new(|_open| {})));

        assert!(combobox.on_open_change.is_some());
    }

    #[test]
    fn combobox_open_change_with_reason_builder_sets_handler() {
        let mut app = App::new();
        let value = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let combobox = Combobox::new(value, open)
            .on_open_change_with_reason(Some(Arc::new(|_open, _reason| {})));

        assert!(combobox.on_open_change_with_reason.is_some());
    }

    #[test]
    fn combobox_open_change_complete_builder_sets_handler() {
        let mut app = App::new();
        let value = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let combobox =
            Combobox::new(value, open).on_open_change_complete(Some(Arc::new(|_open| {})));

        assert!(combobox.on_open_change_complete.is_some());
    }

    #[test]
    fn combobox_on_value_change_builder_sets_handler() {
        let mut app = App::new();
        let value = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let combobox = Combobox::new(value, open).on_value_change(Some(Arc::new(|_value| {})));

        assert!(combobox.on_value_change.is_some());
    }

    #[test]
    fn combobox_search_input_exposes_combobox_role_active_descendant_and_value() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            ComboboxItem::new("alpha", "Alpha"),
            ComboboxItem::new("beta", "Beta"),
            ComboboxItem::new("gamma", "Gamma"),
        ];

        // First frame: establish stable trigger bounds.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Second frame: open the popover.
        //
        // `active_descendant` depends on stable element<->node mapping, so we render one extra
        // frame before asserting it (see cmdk tests).
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox && n.value.is_some())
            .expect("combobox search input node");
        assert!(
            input.flags.expanded,
            "combobox search input should report expanded=true while open"
        );
        assert_eq!(input.value.as_deref(), Some(""));

        let list = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("listbox node");
        assert!(
            list.labelled_by.iter().any(|id| *id == input.id),
            "listbox should be labelled by the combobox input"
        );
        assert!(
            input.controls.iter().any(|id| *id == list.id),
            "combobox input should control the listbox"
        );

        let active = input
            .active_descendant
            .expect("active_descendant should be set");
        let active_node = snap
            .nodes
            .iter()
            .find(|n| n.id == active)
            .expect("active_descendant should reference a node in the snapshot");
        assert_eq!(active_node.role, SemanticsRole::ListBoxOption);

        let input_id = input.id;
        ui.set_focus(Some(input_id));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::TextInput("a".to_string()),
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open,
            items,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.id == input_id)
            .expect("combobox search input node after typing");
        assert_eq!(input.role, SemanticsRole::ComboBox);
        assert_eq!(input.value.as_deref(), Some("a"));
    }

    #[test]
    fn combobox_pointer_open_auto_focuses_search_input() {
        use fret_core::{Event, Modifiers, MouseButton};

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("beta")));
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            ComboboxItem::new("alpha", "Alpha"),
            ComboboxItem::new("beta", "Beta"),
            ComboboxItem::new("gamma", "Gamma"),
        ];

        // Frame 1: establish stable trigger bounds.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox && n.value.is_none())
            .expect("combobox trigger semantics");
        let trigger_bounds = ui
            .debug_node_visual_bounds(trigger.id)
            .expect("trigger bounds");
        let trigger_center = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: trigger_center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert!(
            app.models().get_copied(&open).unwrap_or(false),
            "expected pointer click to open combobox"
        );

        // Frame 2: open; autofocus should move focus into the search input.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open,
            items,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox && n.value.is_some())
            .expect("combobox search input node");
        assert_eq!(
            ui.focus(),
            Some(input.id),
            "expected pointer-open to autofocus the search input"
        );
    }

    #[test]
    fn combobox_keyboard_open_auto_focuses_search_input() {
        use fret_core::{Event, KeyCode, Modifiers};

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("beta")));
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            ComboboxItem::new("alpha", "Alpha"),
            ComboboxItem::new("beta", "Beta"),
            ComboboxItem::new("gamma", "Gamma"),
        ];

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("combobox trigger node");
        ui.set_focus(Some(trigger));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyUp {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
            },
        );
        assert!(
            app.models().get_copied(&open).unwrap_or(false),
            "expected Enter to open combobox"
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open,
            items,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox && n.value.is_some())
            .expect("combobox search input node");
        assert_eq!(
            ui.focus(),
            Some(input.id),
            "expected keyboard-open to autofocus the search input"
        );
    }

    #[test]
    fn combobox_close_transition_disables_pointer_move_and_timer_events() {
        use fret_core::{Event, Modifiers, MouseButton, MouseButtons};

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            ComboboxItem::new("alpha", "Alpha"),
            ComboboxItem::new("beta", "Beta"),
            ComboboxItem::new("gamma", "Gamma"),
        ];

        // Frame 1: closed.
        let _ = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
            underlay_clicked.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Frame 2: open, capture overlay layer id.
        let _ = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
            underlay_clicked.clone(),
        );
        let overlay_id =
            fret_ui_kit::OverlayController::stack_snapshot_for_window(&ui, &mut app, window)
                .topmost_popover
                .expect("expected an open combobox overlay");
        let overlay_root_name = radix_popover::popover_root_name(overlay_id);
        let overlay_root = fret_ui::elements::global_root(window, &overlay_root_name);
        let overlay_node =
            fret_ui::elements::node_for_element(&mut app, window, overlay_root).expect("overlay");
        let overlay_layer = ui.node_layer(overlay_node).expect("overlay layer");

        let info = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.id == overlay_layer)
            .expect("overlay layer info");
        assert!(info.visible);
        assert!(info.hit_testable);
        assert!(info.wants_pointer_move_events);
        assert!(info.wants_timer_events);

        // Frame 3: close (close transition should remain present but be click-through).
        let _ = app.models_mut().update(&open, |v| *v = false);
        let _ = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open,
            items,
            underlay_clicked.clone(),
        );

        let info = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.id == overlay_layer)
            .expect("overlay layer info");
        assert!(info.visible);
        assert!(!info.hit_testable);
        assert_eq!(
            info.pointer_occlusion,
            fret_ui::tree::PointerOcclusion::None
        );
        assert!(!info.wants_pointer_move_events);
        assert!(!info.wants_timer_events);

        // Pointer interactions should go through while closing.
        let underlay_pos = Point::new(Px(10.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));

        // Move events should not install timers while closing (no interactive policies).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let effects = app.flush_effects();
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, fret_runtime::Effect::SetTimer { .. })),
            "expected close transition to not arm timers; effects={effects:?}"
        );
    }

    #[test]
    fn combobox_list_respects_theme_max_height_in_tight_viewports() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(92.0)),
        );
        let mut services = FakeServices::default();

        let items: Vec<ComboboxItem> = (0..40)
            .map(|i| ComboboxItem::new(format!("v{i}"), format!("Item {i}")))
            .collect();

        // First frame: establish stable trigger bounds.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );
        let _ = app.models_mut().update(&open, |v| *v = true);

        // Second/third frame: open the popover and settle layout.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open,
            items,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let list = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("listbox node");
        let list_bounds = ui.debug_node_bounds(list.id).expect("listbox bounds");

        let theme_max_list_h = 280.0;
        assert!(
            list_bounds.size.height.0 <= theme_max_list_h + 0.01,
            "expected listbox height to respect theme max height; list={list_bounds:?}"
        );
    }
}
