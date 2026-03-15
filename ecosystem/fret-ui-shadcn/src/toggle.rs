use std::{any::Any, sync::Arc};

use fret_core::{Color, Edges, FontId, FontWeight, Px, TextStyle};
use fret_icons::IconId;
use fret_runtime::{ActionId, CommandId, Model};
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, Length, MainAlign, PressableProps};
use fret_ui::{ElementContext, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::current_color;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::motion::drive_tween_color_for_element;
use fret_ui_kit::declarative::motion::drive_tween_f32_for_element;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::control_registry::{
    ControlAction, ControlEntry, ControlId, control_registry_model,
};
pub use fret_ui_kit::primitives::toggle::ToggleRoot;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverrideSlot, Radius,
    Size as ComponentSize, Space, WidgetState, WidgetStateProperty, WidgetStates,
    resolve_override_slot, resolve_override_slot_opt, ui,
};

use crate::overlay_motion;

fn tailwind_transition_ease_in_out(t: f32) -> f32 {
    // Tailwind default `ease-in-out`: cubic-bezier(0.4, 0, 0.2, 1)
    fret_ui_kit::headless::easing::CubicBezier::new(0.4, 0.0, 0.2, 1.0).sample(t)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleVariant {
    #[default]
    Default,
    Outline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleSize {
    #[default]
    Default,
    Sm,
    Lg,
}

impl ToggleSize {
    pub fn component_size(self) -> ComponentSize {
        match self {
            Self::Default => ComponentSize::Medium,
            Self::Sm => ComponentSize::Small,
            Self::Lg => ComponentSize::Large,
        }
    }
}

/// Upstream shadcn/ui `toggleVariants(...)` compat surface.
///
/// Upstream returns a Tailwind/CVA class string. In Fret we expose the closest equivalent as
/// mergeable refinements.
#[derive(Debug, Clone)]
pub struct ToggleVariants {
    pub chrome: ChromeRefinement,
    pub layout: LayoutRefinement,
}

fn toggle_h_snapshot(theme: &ThemeSnapshot, size: ToggleSize) -> Px {
    let (key, fallback) = match size {
        ToggleSize::Default => ("component.toggle.h", Px(36.0)),
        ToggleSize::Sm => ("component.toggle.h_sm", Px(32.0)),
        ToggleSize::Lg => ("component.toggle.h_lg", Px(40.0)),
    };
    theme.metric_by_key(key).unwrap_or(fallback)
}

pub fn toggle_variants(
    theme: &ThemeSnapshot,
    variant: ToggleVariant,
    size: ToggleSize,
) -> ToggleVariants {
    let radius = MetricRef::radius(Radius::Md).resolve(theme);
    let border = theme.color_token("input");
    let fg = theme.color_token("foreground");

    let chrome = match variant {
        ToggleVariant::Default => ChromeRefinement::default()
            .radius(radius)
            .border_width(Px(1.0))
            .border_color(ColorRef::Color(Color::TRANSPARENT)),
        ToggleVariant::Outline => ChromeRefinement::default()
            .radius(radius)
            .border_width(Px(1.0))
            .border_color(ColorRef::Color(border)),
    }
    .text_color(ColorRef::Color(fg));

    let h = toggle_h_snapshot(theme, size);
    let layout = LayoutRefinement::default().h_px(h).min_w(h);

    ToggleVariants { chrome, layout }
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn collect_toggle_children<H: UiHost, I, T>(
    cx: &mut ElementContext<'_, H>,
    children: I,
) -> Vec<AnyElement>
where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    children
        .into_iter()
        .map(|child| child.into_element(cx))
        .collect()
}

fn toggle_bg_hover(theme: &Theme) -> Color {
    theme.color_token("muted")
}

fn toggle_fg_muted(theme: &Theme) -> Color {
    theme.color_token("muted-foreground")
}

fn toggle_ring_color(theme: &Theme) -> Color {
    theme.color_token("ring")
}

fn toggle_bg_on(theme: &Theme) -> Color {
    theme.color_token("accent")
}

fn toggle_fg_on(theme: &Theme) -> Color {
    theme.color_token("accent-foreground")
}

fn toggle_border(theme: &Theme) -> Color {
    theme.color_token("input")
}

fn toggle_h(theme: &Theme, size: ToggleSize) -> Px {
    let (key, fallback) = match size {
        ToggleSize::Default => ("component.toggle.h", Px(36.0)),
        ToggleSize::Sm => ("component.toggle.h_sm", Px(32.0)),
        ToggleSize::Lg => ("component.toggle.h_lg", Px(40.0)),
    };
    theme.metric_by_key(key).unwrap_or(fallback)
}

fn toggle_pad_x(theme: &Theme, size: ToggleSize) -> Px {
    let (key, fallback) = match size {
        ToggleSize::Default => ("component.toggle.px", Px(8.0)),
        ToggleSize::Sm => ("component.toggle.px_sm", Px(6.0)),
        ToggleSize::Lg => ("component.toggle.px_lg", Px(10.0)),
    };
    theme.metric_by_key(key).unwrap_or(fallback)
}

fn toggle_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.toggle.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.toggle.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    let mut style = typography::fixed_line_box_style(FontId::ui(), px, line_height);
    style.weight = FontWeight::MEDIUM;
    style
}

fn apply_toggle_inherited_style(
    mut element: AnyElement,
    fg: Color,
    default_icon_color: Color,
) -> AnyElement {
    match &mut element.kind {
        fret_ui::element::ElementKind::Text(props) => {
            props.color.get_or_insert(fg);
        }
        fret_ui::element::ElementKind::SvgIcon(fret_ui::element::SvgIconProps {
            color, ..
        }) => {
            // Heuristic:
            // - Older callsites may build an `SvgIcon` with the default white color.
            // - `declarative::icon::icon(...)` built outside a `currentColor` provider resolves
            //   `muted-foreground` eagerly.
            //
            // In a Toggle, both shapes should track the toggle foreground by default.
            let is_default_white = *color
                == Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                };
            let is_default_muted_fg = *color == default_icon_color;
            if is_default_white || is_default_muted_fg {
                *color = fg;
            }
        }
        fret_ui::element::ElementKind::Spinner(fret_ui::element::SpinnerProps {
            color, ..
        }) => {
            color.get_or_insert(fg);
        }
        _ => {}
    }

    element.children = element
        .children
        .into_iter()
        .map(|child| apply_toggle_inherited_style(child, fg, default_icon_color))
        .collect();
    element
}

#[derive(Debug, Clone, Default)]
pub struct ToggleStyle {
    pub background: OverrideSlot<ColorRef>,
    pub foreground: OverrideSlot<ColorRef>,
    pub border_color: OverrideSlot<ColorRef>,
}

impl ToggleStyle {
    pub fn background(mut self, background: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.background = Some(background);
        self
    }

    pub fn foreground(mut self, foreground: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.foreground = Some(foreground);
        self
    }

    pub fn border_color(mut self, border_color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.border_color = Some(border_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.background.is_some() {
            self.background = other.background;
        }
        if other.foreground.is_some() {
            self.foreground = other.foreground;
        }
        if other.border_color.is_some() {
            self.border_color = other.border_color;
        }
        self
    }
}

pub struct Toggle {
    model: Option<Model<bool>>,
    pressed_snapshot: Option<bool>,
    default_pressed: bool,
    label: Option<Arc<str>>,
    children: Vec<AnyElement>,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    disabled: bool,
    control_id: Option<ControlId>,
    a11y_label: Option<Arc<str>>,
    on_click: Option<CommandId>,
    action_payload: Option<Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>>,
    variant: ToggleVariant,
    size: ToggleSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: ToggleStyle,
}

impl std::fmt::Debug for Toggle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toggle")
            .field("model", &"<model>")
            .field("pressed_snapshot", &self.pressed_snapshot)
            .field("label", &self.label.as_ref().map(|s| s.as_ref()))
            .field("children_len", &self.children.len())
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("on_click", &self.on_click)
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("style", &self.style)
            .finish()
    }
}

impl Toggle {
    pub fn new(model: Model<bool>) -> Self {
        Self {
            model: Some(model),
            pressed_snapshot: None,
            default_pressed: false,
            label: None,
            children: Vec::new(),
            leading_icon: None,
            trailing_icon: None,
            disabled: false,
            control_id: None,
            a11y_label: None,
            on_click: None,
            action_payload: None,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: ToggleStyle::default(),
        }
    }

    /// Creates a toggle from a plain pressed snapshot, mirroring the upstream controlled
    /// `pressed` prop without forcing a `Model<bool>` at the call site.
    ///
    /// This is intended for views that already own the state elsewhere and only need the toggle to
    /// render the current value while dispatching an external action on activation.
    pub fn from_pressed(pressed: bool) -> Self {
        Self {
            model: None,
            pressed_snapshot: Some(pressed),
            default_pressed: false,
            label: None,
            children: Vec::new(),
            leading_icon: None,
            trailing_icon: None,
            disabled: false,
            control_id: None,
            a11y_label: None,
            on_click: None,
            action_payload: None,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: ToggleStyle::default(),
        }
    }

    /// Creates an uncontrolled toggle with an initial pressed value (Radix `defaultPressed`).
    pub fn uncontrolled(default_pressed: bool) -> Self {
        Self {
            model: None,
            pressed_snapshot: None,
            default_pressed,
            label: None,
            children: Vec::new(),
            leading_icon: None,
            trailing_icon: None,
            disabled: false,
            control_id: None,
            a11y_label: None,
            on_click: None,
            action_payload: None,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: ToggleStyle::default(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    /// Icon-only toggle content (common in shadcn toolbar patterns).
    ///
    /// The icon is stored as an `IconId` and built by the toggle host so it can inherit the
    /// resolved foreground via `currentColor`.
    pub fn icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self.trailing_icon = None;
        self.label = None;
        self.children.clear();
        self
    }

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn trailing_icon(mut self, icon: IconId) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Binds this Toggle to a logical form control id (similar to HTML `id`).
    ///
    /// When set, `Label::for_control(ControlId)` forwards focus to the toggle pressable and
    /// mirrors the same press activation path (typed action dispatch and/or pressed-state toggle).
    pub fn control_id(mut self, id: impl Into<ControlId>) -> Self {
        self.control_id = Some(id.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    /// Bind a stable action ID to this toggle (action-first authoring).
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this still dispatches
    /// through the existing command pipeline.
    pub fn action(mut self, action: impl Into<ActionId>) -> Self {
        self.on_click = Some(action.into());
        self
    }

    /// Attach a payload for parameterized actions (ADR 0312).
    pub fn action_payload<T>(mut self, payload: T) -> Self
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        let payload = Arc::new(payload);
        self.action_payload = Some(Arc::new(move || Box::new(payload.as_ref().clone())));
        self
    }

    /// Like [`Toggle::action_payload`], but computes the payload lazily on activation.
    pub fn action_payload_factory(
        mut self,
        payload: Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>,
    ) -> Self {
        self.action_payload = Some(payload);
        self
    }

    /// Sets the uncontrolled initial pressed value (Radix `defaultPressed`).
    ///
    /// Note: If a controlled `model` is provided, this value is ignored.
    pub fn default_pressed(mut self, default_pressed: bool) -> Self {
        self.default_pressed = default_pressed;
        self
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.on_click = Some(command.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn style(mut self, style: ToggleStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let pressed_snapshot = self.pressed_snapshot;
        let model = match (self.model.clone(), pressed_snapshot) {
            (Some(model), _) => Some(model),
            (None, Some(_)) => None,
            (None, None) => Some(
                fret_ui_kit::primitives::toggle::toggle_use_model(cx, None, || {
                    self.default_pressed
                })
                .model(),
            ),
        };
        let label = self.label;
        let children = self.children;
        let leading_icon = self.leading_icon;
        let trailing_icon = self.trailing_icon;
        let disabled_explicit = self.disabled;
        let a11y_label = self.a11y_label.clone();
        let control_id = self.control_id;
        let on_click = self.on_click;
        let action_payload = self.action_payload;
        let disabled = disabled_explicit
            || on_click
                .as_ref()
                .is_some_and(|cmd| !cx.command_is_enabled(cmd));
        let variant = self.variant;
        let size_token = self.size;
        let chrome = self.chrome;
        let layout = self.layout;
        let style_override = self.style;

        let (
            radius,
            ring_border,
            ring,
            text_style,
            pad_x,
            pressable_layout,
            fg_default,
            fg_disabled,
            fg_muted,
            bg_hover,
            bg_on,
            fg_on,
            border,
        ) = {
            let theme = Theme::global(&*cx.app);

            let radius = MetricRef::radius(Radius::Md).resolve(theme);
            let ring_border = toggle_ring_color(theme);
            let mut ring = decl_style::focus_ring(theme, radius);
            ring.color = alpha_mul(ring_border, 0.5);
            let text_style = toggle_text_style(theme);

            let h = toggle_h(theme, size_token);
            let pad_x = toggle_pad_x(theme, size_token);
            let pressable_layout = decl_style::layout_style(
                theme,
                LayoutRefinement::default().h_px(h).min_w(h).merge(layout),
            );

            let fg_default = theme.color_token("foreground");
            let fg_disabled = alpha_mul(fg_default, 0.5);
            let fg_muted = toggle_fg_muted(theme);
            let bg_hover = toggle_bg_hover(theme);
            let bg_on = toggle_bg_on(theme);
            let fg_on = toggle_fg_on(theme);
            let border = toggle_border(theme);

            (
                radius,
                ring_border,
                ring,
                text_style,
                pad_x,
                pressable_layout,
                fg_default,
                fg_disabled,
                fg_muted,
                bg_hover,
                bg_on,
                fg_on,
                border,
            )
        };

        let pad_y = Px(0.0);

        let (hover_bg, hover_fg) = match variant {
            ToggleVariant::Default => (bg_hover, fg_muted),
            ToggleVariant::Outline => (bg_on, fg_on),
        };

        let default_background = WidgetStateProperty::new(None)
            .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_bg)))
            .when(WidgetStates::SELECTED, Some(ColorRef::Color(bg_on)))
            .when(WidgetStates::ACTIVE, Some(ColorRef::Color(hover_bg)))
            .when(WidgetStates::DISABLED, None);

        let default_foreground = WidgetStateProperty::new(ColorRef::Color(fg_default))
            .when(WidgetStates::HOVERED, ColorRef::Color(hover_fg))
            .when(WidgetStates::SELECTED, ColorRef::Color(fg_on))
            .when(WidgetStates::ACTIVE, ColorRef::Color(hover_fg))
            .when(WidgetStates::DISABLED, ColorRef::Color(fg_disabled));

        let default_border_color = WidgetStateProperty::new(None)
            .when(
                WidgetStates::FOCUS_VISIBLE,
                Some(ColorRef::Color(ring_border)),
            )
            .when(WidgetStates::DISABLED, None);

        let user_bg_override = chrome.background.is_some();

        let base_chrome = match variant {
            ToggleVariant::Default => ChromeRefinement::default()
                .radius(radius)
                .border_width(Px(1.0))
                .border_color(ColorRef::Color(Color::TRANSPARENT)),
            ToggleVariant::Outline => ChromeRefinement::default()
                .radius(radius)
                .border_width(Px(1.0))
                .border_color(ColorRef::Color(border)),
        }
        .merge(chrome);

        let control_id = control_id.clone();
        let control_registry = control_id.as_ref().map(|_| control_registry_model(cx));
        let labelled_by_element = if a11y_label.is_some() {
            None
        } else if let (Some(control_id), Some(control_registry)) =
            (control_id.as_ref(), control_registry.as_ref())
        {
            cx.app
                .models()
                .read(control_registry, |reg| {
                    reg.label_for(cx.window, control_id).map(|l| l.element)
                })
                .ok()
                .flatten()
        } else {
            None
        };
        let described_by_element = if let (Some(control_id), Some(control_registry)) =
            (control_id.as_ref(), control_registry.as_ref())
        {
            cx.app
                .models()
                .read(control_registry, |reg| {
                    reg.described_by_for(cx.window, control_id)
                })
                .ok()
                .flatten()
        } else {
            None
        };

        let control_id_for_register = control_id.clone();
        let control_registry_for_register = control_registry.clone();
        let pressed_model_for_toggle = model.clone();
        let labelled_by_element_for_toggle = labelled_by_element;
        let described_by_element_for_toggle = described_by_element;
        let has_a11y_label_for_toggle = a11y_label.is_some();

        control_chrome_pressable_with_id_props(cx, move |cx, state, _id| {
            if let (Some(control_id), Some(control_registry)) = (
                control_id_for_register.clone(),
                control_registry_for_register.clone(),
            ) {
                let toggle_action = pressed_model_for_toggle
                    .clone()
                    .map(ControlAction::ToggleBool);
                let action = match (on_click.clone(), action_payload.clone(), toggle_action) {
                    (Some(command), payload, Some(toggle_action)) => Some(ControlAction::Sequence(
                        vec![
                            ControlAction::DispatchCommand { command, payload },
                            toggle_action,
                        ]
                        .into(),
                    )),
                    (Some(command), payload, None) => {
                        Some(ControlAction::DispatchCommand { command, payload })
                    }
                    (None, _, Some(toggle_action)) => Some(toggle_action),
                    (None, _, None) => None,
                };
                if let Some(action) = action {
                    let entry = ControlEntry {
                        element: _id,
                        enabled: !disabled,
                        action,
                    };
                    let _ = cx.app.models_mut().update(&control_registry, |reg| {
                        reg.register_control(cx.window, cx.frame_id, control_id, entry);
                    });
                }
            }

            if let Some(payload) = action_payload.clone() {
                cx.pressable_dispatch_command_with_payload_factory_if_enabled_opt(
                    on_click.clone(),
                    payload,
                );
            } else {
                cx.pressable_dispatch_command_if_enabled_opt(on_click.clone());
            }
            if let Some(model) = pressed_model_for_toggle.as_ref() {
                cx.pressable_toggle_bool(model);
            }

            let on = if let Some(model) = pressed_model_for_toggle.as_ref() {
                cx.watch_model(model).copied().unwrap_or(false)
            } else {
                pressed_snapshot.unwrap_or(false)
            };
            let mut states = WidgetStates::from_pressable(cx, state, !disabled);
            states.set(WidgetState::Selected, on);

            let fg = resolve_override_slot(
                style_override.foreground.as_ref(),
                &default_foreground,
                states,
            );
            let default_icon_color = fg_muted;
            let bg = resolve_override_slot_opt(
                style_override.background.as_ref(),
                &default_background,
                states,
            );
            let border_color = resolve_override_slot_opt(
                style_override.border_color.as_ref(),
                &default_border_color,
                states,
            );

            let theme = Theme::global(&*cx.app).snapshot();
            let duration = overlay_motion::shadcn_motion_duration_150(cx);

            let fg_motion = drive_tween_color_for_element(
                cx,
                _id,
                "toggle.content.fg",
                fg.resolve(&theme),
                duration,
                tailwind_transition_ease_in_out,
            );
            let fg_color = fg_motion.value;
            let fg = ColorRef::Color(fg_color);

            let ring_alpha = drive_tween_f32_for_element(
                cx,
                _id,
                "toggle.chrome.ring.alpha",
                if states.contains(WidgetStates::FOCUS_VISIBLE) {
                    1.0
                } else {
                    0.0
                },
                duration,
                tailwind_transition_ease_in_out,
            );
            let mut ring = ring;
            ring.color.a = (ring.color.a * ring_alpha.value).clamp(0.0, 1.0);
            if let Some(offset_color) = ring.offset_color {
                ring.offset_color = Some(Color {
                    a: (offset_color.a * ring_alpha.value).clamp(0.0, 1.0),
                    ..offset_color
                });
            }

            let mut chrome_props = decl_style::container_props(
                &theme,
                base_chrome.clone(),
                LayoutRefinement::default(),
            );
            chrome_props.padding = Edges {
                top: pad_y,
                right: pad_x,
                bottom: pad_y,
                left: pad_x,
            }
            .into();
            if matches!(variant, ToggleVariant::Outline) {
                chrome_props.shadow = Some(decl_style::shadow_xs(&theme, radius));
            }

            let bg_present = bg.is_some();
            let target_bg = bg
                .map(|bg| bg.resolve(&theme))
                .unwrap_or(Color::TRANSPARENT);
            let bg_motion = drive_tween_color_for_element(
                cx,
                _id,
                "toggle.chrome.bg",
                target_bg,
                duration,
                tailwind_transition_ease_in_out,
            );
            if !user_bg_override {
                let wants_bg = bg_present || bg_motion.animating || bg_motion.value.a > 0.0;
                chrome_props.background = wants_bg.then_some(bg_motion.value);
            }

            let base_border = chrome_props.border_color.unwrap_or(Color::TRANSPARENT);
            let target_border = border_color
                .map(|border_color| border_color.resolve(&theme))
                .unwrap_or(base_border);
            let border_motion = drive_tween_color_for_element(
                cx,
                _id,
                "toggle.chrome.border",
                target_border,
                duration,
                tailwind_transition_ease_in_out,
            );
            chrome_props.border_color = Some(border_motion.value);

            chrome_props.layout.size = pressable_layout.size;

            let mut a11y = fret_ui_kit::primitives::toggle::toggle_a11y(a11y_label, on);
            if !has_a11y_label_for_toggle {
                a11y.labelled_by_element = labelled_by_element_for_toggle.map(|id| id.0);
            }
            a11y.described_by_element = described_by_element_for_toggle.map(|id| id.0);

            let pressable_props = PressableProps {
                layout: pressable_layout,
                enabled: !disabled,
                focusable: true,
                focus_ring: Some(ring),
                focus_ring_always_paint: ring_alpha.animating,
                a11y,
                ..Default::default()
            };

            let content_children = move |cx: &mut ElementContext<'_, H>| {
                current_color::scope_children(cx, fg.clone(), |cx| {
                    let styled_children: Vec<AnyElement> = children
                        .into_iter()
                        .map(|child| {
                            apply_toggle_inherited_style(child, fg_color, default_icon_color)
                        })
                        .collect();

                    vec![cx.flex(
                        FlexProps {
                            direction: fret_core::Axis::Horizontal,
                            layout: {
                                let mut layout = fret_ui::element::LayoutStyle::default();
                                layout.size.height = Length::Fill;
                                layout
                            },
                            gap: {
                                let theme = Theme::global(&*cx.app);
                                MetricRef::space(Space::N2).resolve(theme).into()
                            },
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                            ..Default::default()
                        },
                        move |cx: &mut ElementContext<'_, H>| {
                            let mut out = Vec::new();
                            if let Some(icon) = leading_icon.clone() {
                                out.push(decl_icon::icon(cx, icon));
                            }
                            out.extend(styled_children);
                            if let Some(label) = label.clone() {
                                let mut text = ui::label(label)
                                    .text_size_px(text_style.size)
                                    .font_weight(text_style.weight)
                                    .nowrap();
                                if let Some(line_height) = text_style.line_height {
                                    text = text.line_height_px(line_height).line_height_policy(
                                        fret_core::TextLineHeightPolicy::FixedFromStyle,
                                    );
                                }
                                if let Some(letter_spacing_em) = text_style.letter_spacing_em {
                                    text = text.letter_spacing_em(letter_spacing_em);
                                }
                                out.push(text.into_element(cx));
                            }
                            if let Some(icon) = trailing_icon.clone() {
                                out.push(decl_icon::icon(cx, icon));
                            }
                            out
                        },
                    )]
                })
            };

            (pressable_props, chrome_props, content_children)
        })
    }
}

/// Builder-preserving controlled helper for the common toggle authoring path.
pub fn toggle<H: UiHost, I, T>(
    cx: &mut ElementContext<'_, H>,
    model: Model<bool>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> Toggle
where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    let children = f(cx);
    Toggle::new(model).children(collect_toggle_children(cx, children))
}

/// Builder-preserving uncontrolled helper for the common `defaultPressed` authoring path.
pub fn toggle_uncontrolled<H: UiHost, I, T>(
    cx: &mut ElementContext<'_, H>,
    default_pressed: bool,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> Toggle
where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    let children = f(cx);
    Toggle::uncontrolled(default_pressed).children(collect_toggle_children(cx, children))
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, PathCommand, Point, Px, Rect, Size, SvgId, SvgService,
    };
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::{
        CommandId, CommandMeta, CommandScope, Effect, FrameId, TickId,
        WindowCommandActionAvailabilityService, WindowCommandEnabledService,
        WindowCommandGatingService, WindowCommandGatingSnapshot,
    };
    use fret_ui::UiTree;
    use std::collections::HashMap;
    use std::time::Duration;

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

    fn render_uncontrolled_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        default_pressed: bool,
    ) -> fret_core::NodeId {
        app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "toggle", |cx| {
                vec![
                    Toggle::uncontrolled(default_pressed)
                        .a11y_label("Toggle")
                        .label("Hello")
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn toggle_pressed_value_exposes_semantics_without_model() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "toggle-pressed-value-semantics",
            |cx| {
                vec![
                    Toggle::from_pressed(true)
                        .a11y_label("Toggle bookmark")
                        .label("Bookmark")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("Toggle bookmark"))
            .expect("toggle semantics node");
        assert_eq!(node.role, fret_core::SemanticsRole::Button);
        assert_eq!(
            node.flags.pressed_state,
            Some(fret_core::SemanticsPressedState::True)
        );
    }

    #[test]
    fn field_label_click_dispatches_action_for_snapshot_toggle_control() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(120.0)),
        );
        let mut services = FakeServices::default();

        let cmd = CommandId::from("test.toggle.label-action");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Toggle Label Action").with_scope(CommandScope::App),
        );
        app.set_global(WindowCommandActionAvailabilityService::default());
        app.with_global_mut(
            WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                let mut snapshot: HashMap<CommandId, bool> = HashMap::new();
                snapshot.insert(cmd.clone(), true);
                svc.set_snapshot(window, snapshot);
            },
        );

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "toggle-label-dispatches-action-for-snapshot-control",
            |cx| {
                let mut row_layout = fret_ui::element::LayoutStyle::default();
                row_layout.size.width = fret_ui::element::Length::Fill;

                vec![cx.flex(
                    fret_ui::element::FlexProps {
                        layout: row_layout,
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(8.0).into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: fret_ui::element::MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Center,
                        wrap: false,
                    },
                    |cx| {
                        vec![
                            Toggle::from_pressed(true)
                                .control_id("test.toggle")
                                .a11y_label("Test toggle")
                                .label("Bookmark")
                                .action(cmd.clone())
                                .into_element(cx),
                            crate::field::FieldLabel::new("Toggle via label")
                                .for_control("test.toggle")
                                .into_element(cx)
                                .test_id("test.toggle.label"),
                        ]
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let _ = app.flush_effects();

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let label = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("test.toggle.label"))
            .expect("label semantics node");

        let position = Point::new(
            Px(label.bounds.origin.x.0 + label.bounds.size.width.0 * 0.5),
            Px(label.bounds.origin.y.0 + label.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let effects = app.flush_effects();
        assert!(
            effects
                .iter()
                .any(|effect| matches!(effect, Effect::Command { command, .. } if command.as_str() == cmd.as_str())),
            "expected label click to dispatch {cmd:?}, got {effects:?}"
        );
    }

    #[test]
    fn toggle_uncontrolled_applies_default_pressed_once_and_does_not_reset() {
        fn is_pressed(ui: &UiTree<App>, label: &str) -> bool {
            ui.semantics_snapshot()
                .expect("semantics snapshot")
                .nodes
                .iter()
                .find(|n| n.label.as_deref() == Some(label))
                .is_some_and(|n| {
                    n.flags.pressed_state == Some(fret_core::SemanticsPressedState::True)
                })
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices::default();

        let root =
            render_uncontrolled_frame(&mut ui, &mut app, &mut services, window, bounds, true);
        assert!(is_pressed(&ui, "Toggle"));

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable toggle");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyUp {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
            },
        );

        let _ = render_uncontrolled_frame(&mut ui, &mut app, &mut services, window, bounds, true);
        assert!(!is_pressed(&ui, "Toggle"));

        // The internal model should not be reset by repeatedly passing the same default value.
        let _ = render_uncontrolled_frame(&mut ui, &mut app, &mut services, window, bounds, true);
        assert!(!is_pressed(&ui, "Toggle"));
    }

    #[test]
    fn command_gating_toggle_is_disabled_by_window_command_enabled_service() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let cmd = CommandId::from("test.disabled-command");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Disabled Command").with_scope(CommandScope::Widget),
        );

        app.set_global(WindowCommandEnabledService::default());
        app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
            svc.set_enabled(window, cmd.clone(), false);
        });

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "toggle",
            |cx| {
                vec![
                    Toggle::uncontrolled(false)
                        .a11y_label("Disabled Toggle")
                        .label("Hello")
                        .on_click(cmd.clone())
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("Disabled Toggle"))
            .expect("toggle semantics node");
        assert!(node.flags.disabled);
    }

    #[test]
    fn command_gating_toggle_is_disabled_when_widget_action_is_unavailable() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let cmd = CommandId::from("test.widget-action");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Widget Action").with_scope(CommandScope::Widget),
        );

        app.set_global(WindowCommandActionAvailabilityService::default());
        app.with_global_mut(
            WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                let mut snapshot: HashMap<CommandId, bool> = HashMap::new();
                snapshot.insert(cmd.clone(), false);
                svc.set_snapshot(window, snapshot);
            },
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "toggle",
            |cx| {
                vec![
                    Toggle::uncontrolled(false)
                        .a11y_label("Disabled Toggle")
                        .label("Hello")
                        .on_click(cmd.clone())
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("Disabled Toggle"))
            .expect("toggle semantics node");
        assert!(node.flags.disabled);
    }

    #[test]
    fn command_gating_toggle_prefers_window_command_gating_snapshot_when_present() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let cmd = CommandId::from("test.widget-action");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Widget Action").with_scope(CommandScope::Widget),
        );

        app.set_global(WindowCommandActionAvailabilityService::default());
        app.with_global_mut(
            WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                let mut snapshot: HashMap<CommandId, bool> = HashMap::new();
                snapshot.insert(cmd.clone(), true);
                svc.set_snapshot(window, snapshot);
            },
        );

        app.set_global(WindowCommandGatingService::default());
        app.with_global_mut(WindowCommandGatingService::default, |svc, app| {
            let input_ctx = crate::command_gating::default_input_context(app);
            let enabled_overrides: HashMap<CommandId, bool> = HashMap::new();
            let mut availability: HashMap<CommandId, bool> = HashMap::new();
            availability.insert(cmd.clone(), false);
            let _token = svc.push_snapshot(
                window,
                WindowCommandGatingSnapshot::new(input_ctx, enabled_overrides)
                    .with_action_availability(Some(Arc::new(availability))),
            );
        });

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "toggle",
            |cx| {
                vec![
                    Toggle::uncontrolled(false)
                        .a11y_label("Disabled Toggle")
                        .label("Hello")
                        .on_click(cmd.clone())
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("Disabled Toggle"))
            .expect("toggle semantics node");
        assert!(node.flags.disabled);
    }

    #[test]
    fn toggle_hover_background_tweens_instead_of_snapping() {
        use std::cell::Cell;
        use std::rc::Rc;

        use fret_core::MouseButtons;
        use fret_runtime::FrameId;
        use fret_ui::elements::GlobalElementId;
        use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;

        fn color_eq_eps(a: Color, b: Color, eps: f32) -> bool {
            (a.r - b.r).abs() <= eps
                && (a.g - b.g).abs() <= eps
                && (a.b - b.b).abs() <= eps
                && (a.a - b.a).abs() <= eps
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices::default();

        let toggle_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let bg_out: Rc<Cell<Option<Color>>> = Rc::new(Cell::new(None));

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            toggle_id: Rc<Cell<Option<GlobalElementId>>>,
            bg_out: Rc<Cell<Option<Color>>>,
        ) {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "toggle-hover-bg-tween",
                move |cx| {
                    let el = Toggle::uncontrolled(false)
                        .a11y_label("Toggle")
                        .label("Hello")
                        .into_element(cx);
                    toggle_id.set(Some(el.id));

                    let chrome = el
                        .children
                        .first()
                        .expect("expected pressable to contain chrome container");
                    let fret_ui::element::ElementKind::Container(props) = &chrome.kind else {
                        panic!("expected chrome container element");
                    };
                    let bg = props.background.unwrap_or(Color::TRANSPARENT);
                    bg_out.set(Some(bg));

                    vec![el]
                },
            );
            ui.set_root(root);
        }

        let theme = Theme::global(&app);
        let base_bg = Color::TRANSPARENT;
        let hover_bg = theme.color_token("muted");

        // Frame 1: baseline render.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            toggle_id.clone(),
            bg_out.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let bg0 = bg_out.get().expect("bg0");
        assert!(
            color_eq_eps(bg0, base_bg, 1e-6),
            "expected base background to match; got bg0={bg0:?} base={base_bg:?}"
        );

        let id = toggle_id.get().expect("toggle id");
        let node = fret_ui::elements::node_for_element(&mut app, window, id).expect("toggle node");
        let b = ui.debug_node_bounds(node).expect("toggle bounds");
        let center = Point::new(
            Px(b.origin.x.0 + b.size.width.0 * 0.5),
            Px(b.origin.y.0 + b.size.height.0 * 0.5),
        );

        // Hover to retarget the transition.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: center,
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: hover is applied; the background should be in-between (not snapped).
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            toggle_id.clone(),
            bg_out.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let bg1 = bg_out.get().expect("bg1");
        assert!(
            !color_eq_eps(bg1, base_bg, 1e-6) && !color_eq_eps(bg1, hover_bg, 1e-6),
            "expected hover background to tween (intermediate), got bg1={bg1:?} base={base_bg:?} hover={hover_bg:?}"
        );

        // Advance frames until the default 150ms transition settles.
        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for i in 0..settle {
            app.set_frame_id(FrameId(3 + i));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                toggle_id.clone(),
                bg_out.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let bg_final = bg_out.get().expect("bg_final");
        assert!(
            color_eq_eps(bg_final, hover_bg, 1e-4),
            "expected hover background to settle; got bg={bg_final:?} hover={hover_bg:?}"
        );
    }

    #[test]
    fn toggle_focus_ring_tweens_in_and_out_like_a_transition() {
        use std::cell::Cell;
        use std::rc::Rc;

        use fret_core::KeyCode;
        use fret_ui::element::ElementKind;
        use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices::default();

        let ring_alpha_out: Rc<Cell<Option<f32>>> = Rc::new(Cell::new(None));
        let always_paint_out: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            ring_alpha_out: Rc<Cell<Option<f32>>>,
            always_paint_out: Rc<Cell<Option<bool>>>,
        ) -> fret_core::NodeId {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "toggle-focus-ring-tween",
                move |cx| {
                    let el = Toggle::uncontrolled(false)
                        .a11y_label("Toggle")
                        .label("Hello")
                        .into_element(cx);

                    let ElementKind::Pressable(pressable) = &el.kind else {
                        panic!("expected toggle to render as pressable");
                    };
                    let ring = pressable.focus_ring.expect("focus ring");
                    ring_alpha_out.set(Some(ring.color.a));
                    always_paint_out.set(Some(pressable.focus_ring_always_paint));

                    vec![el]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
            root
        }

        // Frame 1: baseline render (no focus-visible), ring alpha should be 0.
        app.set_frame_id(FrameId(1));
        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a0 = ring_alpha_out.get().expect("a0");
        assert!(
            a0.abs() <= 1e-6,
            "expected ring alpha to start at 0, got {a0}"
        );

        // Focus the toggle and mark focus-visible via a navigation key.
        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable toggle");
        ui.set_focus(Some(focusable));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        // Frame 2: ring should be in-between (not snapped).
        app.set_frame_id(FrameId(2));
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a1 = ring_alpha_out.get().expect("a1");
        assert!(
            a1 > 0.0,
            "expected ring alpha to start animating in, got {a1}"
        );

        // Advance frames until the default 150ms transition settles.
        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for i in 0..settle {
            app.set_frame_id(FrameId(3 + i));
            let _ = render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
        }
        let a_focused = ring_alpha_out.get().expect("a_focused");
        assert!(
            a_focused > a1 + 1e-4,
            "expected ring alpha to increase over time, got a1={a1} a_focused={a_focused}"
        );

        // Blur and ensure ring animates out while still being painted.
        ui.set_focus(None);
        app.set_frame_id(FrameId(1000));
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a_blur = ring_alpha_out.get().expect("a_blur");
        let always_paint = always_paint_out.get().expect("always_paint");
        assert!(
            a_blur > 0.0 && a_blur < a_focused,
            "expected ring alpha to be intermediate after blur, got a_blur={a_blur} a_focused={a_focused}"
        );
        assert!(
            always_paint,
            "expected focus ring to request painting while animating out"
        );

        for i in 0..settle {
            app.set_frame_id(FrameId(1001 + i));
            let _ = render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
        }
        let a_final = ring_alpha_out.get().expect("a_final");
        let always_paint_final = always_paint_out.get().expect("always_paint_final");
        assert!(
            a_final.abs() <= 1e-4,
            "expected ring alpha to settle at 0, got {a_final}"
        );
        assert!(
            !always_paint_final,
            "expected focus ring to stop requesting painting after settling"
        );
    }
}
