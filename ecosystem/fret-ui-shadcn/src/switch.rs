use std::{any::Any, sync::Arc};

use fret_core::window::ColorScheme;
use fret_core::{Color, Corners, Edges, Px};
use fret_runtime::{ActionId, CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, PositionStyle, PressableProps,
    SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::motion as decl_motion;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::control_registry::{
    ControlAction, ControlEntry, ControlId, control_registry_model,
};
use fret_ui_kit::primitives::controllable_state;
use fret_ui_kit::primitives::switch::{
    switch_a11y, switch_checked_from_optional_bool, switch_use_checked_model, toggle_optional_bool,
};
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, OverrideSlot, Radius, WidgetState,
    WidgetStateProperty, WidgetStates, resolve_override_slot,
};

use crate::bool_model::IntoBoolModel;
use crate::optional_bool_model::IntoOptionalBoolModel;
use crate::overlay_motion;

const SWITCH_THUMB_TRANSITION_EASE: fret_ui_kit::headless::easing::CubicBezier =
    fret_ui_kit::headless::easing::CubicBezier::new(0.4, 0.0, 0.2, 1.0);

fn switch_thumb_transition_ease(t: f32) -> f32 {
    SWITCH_THUMB_TRANSITION_EASE.sample(t)
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SwitchSize {
    Sm,
    #[default]
    Default,
}

fn switch_track_w(theme: &Theme, size: SwitchSize) -> Px {
    match size {
        SwitchSize::Sm => theme
            .metric_by_key("component.switch.sm.track_w")
            .unwrap_or(Px(24.0)),
        SwitchSize::Default => theme
            .metric_by_key("component.switch.track_w")
            .unwrap_or(Px(32.0)),
    }
}

fn switch_track_h(theme: &Theme, size: SwitchSize) -> Px {
    match size {
        SwitchSize::Sm => theme
            .metric_by_key("component.switch.sm.track_h")
            .unwrap_or(Px(14.0)),
        SwitchSize::Default => theme
            .metric_by_key("component.switch.track_h")
            .unwrap_or(Px(18.4)),
    }
}

fn switch_thumb(theme: &Theme, size: SwitchSize) -> Px {
    match size {
        SwitchSize::Sm => theme
            .metric_by_key("component.switch.sm.thumb")
            .unwrap_or(Px(12.0)),
        SwitchSize::Default => theme
            .metric_by_key("component.switch.thumb")
            .unwrap_or(Px(16.0)),
    }
}

fn switch_padding(theme: &Theme, size: SwitchSize) -> Px {
    match size {
        SwitchSize::Sm => theme
            .metric_by_key("component.switch.sm.thumb_pad")
            .unwrap_or(Px(0.0)),
        SwitchSize::Default => theme
            .metric_by_key("component.switch.thumb_pad")
            // shadcn-web positions the thumb flush to the content edge and relies on the track border
            // (1px) for the visible inset. Fret's border is paint-only, so we treat this as an extra
            // inset on top of the border/padding compensation in the layout below.
            .unwrap_or(Px(0.0)),
    }
}

fn switch_bg_on(theme: &Theme) -> Color {
    theme
        .color_by_key("primary")
        .unwrap_or_else(|| theme.color_token("primary"))
}

fn switch_bg_off(theme: &Theme) -> Color {
    theme
        .color_by_key("component.switch.track.bg_off")
        .unwrap_or_else(|| {
            theme
                .color_by_key("input")
                .or_else(|| theme.color_by_key("muted"))
                .unwrap_or_else(|| theme.color_token("input"))
        })
}

fn switch_thumb_bg_off(theme: &Theme) -> Color {
    match theme.color_scheme {
        Some(ColorScheme::Dark) => theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground")),
        _ => theme
            .color_by_key("background")
            .unwrap_or_else(|| theme.color_token("background")),
    }
}

fn switch_thumb_bg_on(theme: &Theme) -> Color {
    match theme.color_scheme {
        Some(ColorScheme::Dark) => theme
            .color_by_key("primary-foreground")
            .unwrap_or_else(|| theme.color_token("primary-foreground")),
        _ => switch_thumb_bg_off(theme),
    }
}

fn switch_ring_color(theme: &Theme) -> Color {
    theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_token("ring"))
}

#[derive(Debug, Clone, Default)]
pub struct SwitchStyle {
    pub track_background: OverrideSlot<ColorRef>,
    pub thumb_background: OverrideSlot<ColorRef>,
    pub border_color: OverrideSlot<ColorRef>,
}

impl SwitchStyle {
    pub fn track_background(
        mut self,
        track_background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.track_background = Some(track_background);
        self
    }

    pub fn thumb_background(
        mut self,
        thumb_background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.thumb_background = Some(thumb_background);
        self
    }

    pub fn border_color(mut self, border_color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.border_color = Some(border_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.track_background.is_some() {
            self.track_background = other.track_background;
        }
        if other.thumb_background.is_some() {
            self.thumb_background = other.thumb_background;
        }
        if other.border_color.is_some() {
            self.border_color = other.border_color;
        }
        self
    }
}

#[derive(Clone)]
pub struct Switch {
    model: SwitchModel,
    size: SwitchSize,
    disabled: bool,
    aria_invalid: bool,
    control_id: Option<ControlId>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_click: Option<CommandId>,
    action_payload: Option<Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: SwitchStyle,
}

#[derive(Clone)]
enum SwitchModel {
    Determinate(Model<bool>),
    Optional(Model<Option<bool>>),
    Value(bool),
}

impl Switch {
    pub fn new(model: impl IntoBoolModel) -> Self {
        Self {
            model: SwitchModel::Determinate(model.into_bool_model()),
            size: SwitchSize::Default,
            disabled: false,
            aria_invalid: false,
            control_id: None,
            a11y_label: None,
            test_id: None,
            on_click: None,
            action_payload: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: SwitchStyle::default(),
        }
    }

    /// Creates a switch from a plain bool value, mirroring the upstream controlled `checked`
    /// prop without forcing a `Model<bool>` at the call site.
    ///
    /// This is intended for views that already own the state elsewhere (for example a
    /// `LocalState<Vec<Row>>` collection) and only need the switch to render the current value
    /// while dispatching an external action on click.
    pub fn from_checked(checked: bool) -> Self {
        Self {
            model: SwitchModel::Value(checked),
            size: SwitchSize::Default,
            disabled: false,
            aria_invalid: false,
            control_id: None,
            a11y_label: None,
            test_id: None,
            on_click: None,
            action_payload: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: SwitchStyle::default(),
        }
    }

    /// Creates a switch bound to an optional boolean model.
    ///
    /// When the value is `None`, the switch renders as unchecked (matching shadcn/ui's common
    /// `value || false` usage). Clicking will set the model to `Some(true/false)` thereafter.
    pub fn new_opt(model: impl IntoOptionalBoolModel) -> Self {
        Self {
            model: SwitchModel::Optional(model.into_optional_bool_model()),
            size: SwitchSize::Default,
            disabled: false,
            aria_invalid: false,
            control_id: None,
            a11y_label: None,
            test_id: None,
            on_click: None,
            action_payload: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: SwitchStyle::default(),
        }
    }

    /// Creates a switch with a controlled/uncontrolled checked model (Radix `checked` /
    /// `defaultChecked`).
    ///
    /// Note: If `checked` is `None`, the internal model is stored in element state at the call site.
    /// Call this from a stable subtree (key the parent node if needed).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        checked: Option<Model<bool>>,
        default_checked: bool,
    ) -> Self {
        let model = switch_use_checked_model(cx, checked, || default_checked).model();
        Self::new(model)
    }

    /// Creates a switch with a controlled/uncontrolled optional-bool model.
    ///
    /// This is shadcn-friendly ergonomics (treats `None` as unchecked).
    pub fn new_opt_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        checked: Option<Model<Option<bool>>>,
        default_checked: Option<bool>,
    ) -> Self {
        let model =
            controllable_state::use_controllable_model(cx, checked, || default_checked).model();
        Self::new_opt(model)
    }

    /// Upstream shadcn/ui supports `size: "sm" | "default"`.
    pub fn size(mut self, size: SwitchSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // Apply the upstream `aria-invalid` error state chrome (border + focus ring color).
    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
        self
    }

    /// Associates this switch with a logical form control id so related elements (e.g. labels)
    /// can forward pointer activation and focus.
    pub fn control_id(mut self, id: impl Into<ControlId>) -> Self {
        self.control_id = Some(id.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    /// Bind a stable action ID to this switch (action-first authoring).
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

    /// Like [`Switch::action_payload`], but computes the payload lazily on activation.
    pub fn action_payload_factory(
        mut self,
        payload: Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>,
    ) -> Self {
        self.action_payload = Some(payload);
        self
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.on_click = Some(command.into());
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn style(mut self, style: SwitchStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let motion_key = match &self.model {
            SwitchModel::Determinate(model) => Some(("determinate", model.id())),
            SwitchModel::Optional(model) => Some(("optional", model.id())),
            SwitchModel::Value(_) => None,
        };

        match motion_key {
            Some(motion_key) => cx.keyed(("shadcn-switch", motion_key), |cx| {
                self.into_element_scoped(cx)
            }),
            None => self.into_element_scoped(cx),
        }
    }

    fn into_element_scoped<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let model = self.model;
            let size = self.size;
            let aria_invalid = self.aria_invalid;

            let (
                w,
                h,
                thumb,
                pad_x,
                radius,
                ring_border,
                bg_off,
                bg_on,
                thumb_bg_off,
                thumb_bg_on,
                pressable_layout,
            ) = {
                let theme = Theme::global(&*cx.app);

                let w = switch_track_w(theme, size);
                let h = switch_track_h(theme, size);
                let thumb = switch_thumb(theme, size);
                let pad_x = switch_padding(theme, size);

                let radius = Px((h.0 * 0.5).max(0.0));
                let ring_border = if aria_invalid {
                    theme.color_token("destructive")
                } else {
                    switch_ring_color(theme)
                };

                let bg_off = switch_bg_off(theme);
                let bg_on = switch_bg_on(theme);
                let thumb_bg_off = switch_thumb_bg_off(theme);
                let thumb_bg_on = switch_thumb_bg_on(theme);

                let layout = LayoutRefinement::default()
                    .w_px(w)
                    .h_px(h)
                    .merge(self.layout);
                let pressable_layout = decl_style::layout_style(theme, layout);

                (
                    w,
                    h,
                    thumb,
                    pad_x,
                    radius,
                    ring_border,
                    bg_off,
                    bg_on,
                    thumb_bg_off,
                    thumb_bg_on,
                    pressable_layout,
                )
            };

            let theme = Theme::global(&*cx.app).snapshot();

            let default_track_background = WidgetStateProperty::new(ColorRef::Color(bg_off))
                .when(WidgetStates::SELECTED, ColorRef::Color(bg_on))
                .when(
                    WidgetStates::HOVERED,
                    ColorRef::Color(alpha_mul(bg_off, 0.7)),
                )
                .when(
                    WidgetStates::HOVERED | WidgetStates::SELECTED,
                    ColorRef::Color(alpha_mul(bg_on, 0.9)),
                )
                .when(
                    WidgetStates::ACTIVE,
                    ColorRef::Color(alpha_mul(bg_off, 0.6)),
                )
                .when(
                    WidgetStates::ACTIVE | WidgetStates::SELECTED,
                    ColorRef::Color(alpha_mul(bg_on, 0.8)),
                );

            let default_thumb_background = WidgetStateProperty::new(ColorRef::Color(thumb_bg_off))
                .when(WidgetStates::SELECTED, ColorRef::Color(thumb_bg_on));

            let default_border_color = WidgetStateProperty::new(ColorRef::Color(if aria_invalid {
                theme.color_token("destructive")
            } else {
                Color::TRANSPARENT
            }))
            .when(WidgetStates::FOCUS_VISIBLE, ColorRef::Color(ring_border));

            let a11y_label = self.a11y_label.clone();
            let test_id = self.test_id.clone();
            let disabled_explicit = self.disabled;
            let on_click = self.on_click.clone();
            let action_payload = self.action_payload.clone();
            let disabled = disabled_explicit
                || on_click
                    .as_ref()
                    .is_some_and(|cmd| !cx.command_is_enabled(cmd));
            let chrome = self.chrome.clone();
            let style_override = self.style.clone();
            let control_id = self.control_id.clone();
            let control_registry = control_id.as_ref().map(|_| control_registry_model(cx));

            let pressable = control_chrome_pressable_with_id_props(cx, move |cx, st, id| {
                if let Some(payload) = action_payload.clone() {
                    cx.pressable_dispatch_command_with_payload_factory_if_enabled_opt(
                        on_click.clone(),
                        payload,
                    );
                } else {
                    cx.pressable_dispatch_command_if_enabled_opt(on_click.clone());
                }
                match &model {
                    SwitchModel::Determinate(model) => cx.pressable_toggle_bool(model),
                    SwitchModel::Optional(model) => {
                        cx.pressable_update_model(model, |v| {
                            *v = toggle_optional_bool(*v);
                        });
                    }
                    SwitchModel::Value(_) => {}
                }

                let on = match &model {
                    SwitchModel::Determinate(model) => {
                        cx.watch_model(model).copied().unwrap_or(false)
                    }
                    SwitchModel::Optional(model) => {
                        switch_checked_from_optional_bool(cx.watch_model(model).copied().flatten())
                    }
                    SwitchModel::Value(checked) => *checked,
                };

                let mut states = WidgetStates::from_pressable(cx, st, !disabled);
                states.set(WidgetState::Selected, on);

                let theme = Theme::global(&*cx.app).snapshot();
                let bg_target = resolve_override_slot(
                    style_override.track_background.as_ref(),
                    &default_track_background,
                    states,
                )
                .resolve(&theme);
                let border_color_target = resolve_override_slot(
                    style_override.border_color.as_ref(),
                    &default_border_color,
                    states,
                )
                .resolve(&theme);
                let thumb_color = resolve_override_slot(
                    style_override.thumb_background.as_ref(),
                    &default_thumb_background,
                    states,
                )
                .resolve(&theme);

                // shadcn/ui v4 uses `transition-all` on the switch track, so hover/active/checked
                // background and focus-visible border/ring should ease instead of snapping.
                let track_duration = overlay_motion::shadcn_motion_duration_150(cx);
                let bg = decl_motion::drive_tween_color_for_element(
                    cx,
                    id,
                    "track-bg",
                    bg_target,
                    track_duration,
                    overlay_motion::shadcn_ease,
                )
                .value;
                let border_color = decl_motion::drive_tween_color_for_element(
                    cx,
                    id,
                    "track-border-color",
                    border_color_target,
                    track_duration,
                    overlay_motion::shadcn_ease,
                )
                .value;

                let mut ring = decl_style::focus_ring(&theme, radius);
                ring.color = if aria_invalid {
                    crate::theme_variants::invalid_control_ring_color(&theme, ring_border)
                } else {
                    alpha_mul(ring_border, 0.5)
                };

                let mut chrome_props = decl_style::container_props(
                    &theme,
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(bg))
                        .rounded(Radius::Full)
                        .border_1()
                        .border_color(ColorRef::Color(border_color))
                        .merge(chrome.clone()),
                    LayoutRefinement::default(),
                );
                chrome_props.corner_radii = Corners::all(radius);
                chrome_props.shadow = Some(decl_style::shadow_xs(&theme, radius));
                chrome_props.layout.size = pressable_layout.size;

                // NOTE: Container layout already treats border as part of layout insets
                // (Tailwind-like border-box behavior). Child positioning is relative to the inner
                // content area, so we should not double-count border/padding when computing the
                // thumb's absolute insets.
                let pad_px = |v: fret_ui::element::SpacingLength| match v {
                    fret_ui::element::SpacingLength::Px(px) => px.0.max(0.0),
                    fret_ui::element::SpacingLength::Fill
                    | fret_ui::element::SpacingLength::Fraction(_) => 0.0,
                };
                let chrome_inset_y = Px(chrome_props.border.top.0.max(0.0)
                    + chrome_props.border.bottom.0.max(0.0)
                    + pad_px(chrome_props.padding.top)
                    + pad_px(chrome_props.padding.bottom));

                if let (Some(control_id), Some(control_registry)) =
                    (control_id.clone(), control_registry.clone())
                {
                    let toggle_action = match &model {
                        SwitchModel::Determinate(model) => {
                            Some(ControlAction::ToggleBool(model.clone()))
                        }
                        SwitchModel::Optional(model) => {
                            Some(ControlAction::ToggleOptionalBool(model.clone()))
                        }
                        SwitchModel::Value(_) => None,
                    };
                    let action = match (on_click.clone(), action_payload.clone(), toggle_action) {
                        (Some(command), payload, Some(toggle_action)) => {
                            Some(ControlAction::Sequence(
                                vec![
                                    ControlAction::DispatchCommand { command, payload },
                                    toggle_action,
                                ]
                                .into(),
                            ))
                        }
                        (Some(command), payload, None) => {
                            Some(ControlAction::DispatchCommand { command, payload })
                        }
                        (None, _, Some(toggle_action)) => Some(toggle_action),
                        (None, _, None) => None,
                    };
                    if let Some(action) = action {
                        let entry = ControlEntry {
                            element: id,
                            enabled: !disabled,
                            action,
                        };
                        let _ = cx.app.models_mut().update(&control_registry, |reg| {
                            reg.register_control(cx.window, cx.frame_id, control_id, entry);
                        });
                    }
                }

                let labelled_by_element = if let (Some(control_id), Some(control_registry)) =
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

                // Prefer explicit `a11y_label`, but fall back to `labelled-by` when available.
                let a11y_label = if a11y_label.is_some() || labelled_by_element.is_none() {
                    a11y_label.clone()
                } else {
                    None
                };

                let mut a11y = switch_a11y(a11y_label, on);
                if let Some(label) = labelled_by_element {
                    a11y.labelled_by_element = Some(label.0);
                }
                a11y.test_id = test_id.clone();
                let pressable_props = PressableProps {
                    layout: pressable_layout,
                    enabled: !disabled,
                    focusable: true,
                    focus_ring: Some(ring),
                    focus_ring_always_paint: false,
                    a11y,
                    ..Default::default()
                };

                let children = move |cx: &mut ElementContext<'_, H>| {
                    // Align with shadcn-web:
                    // - Outer track size is border-box (`h-[1.15rem] w-8 border ...`).
                    // - Thumb is laid out at the content edge, so its outer offset equals the
                    //   track border (1px) plus any explicit padding.
                    let chrome_inset_x = Px(chrome_props.border.left.0.max(0.0)
                        + chrome_props.border.right.0.max(0.0)
                        + pad_px(chrome_props.padding.left)
                        + pad_px(chrome_props.padding.right));

                    let inner_w = Px((w.0 - chrome_inset_x.0).max(0.0));
                    let inner_h = Px((h.0 - chrome_inset_y.0).max(0.0));

                    let y = Px(((inner_h.0 - thumb.0) * 0.5).max(0.0));

                    // Additional inset beyond the border/padding insets (e.g. shadcn `p-[2px]`-like
                    // outcomes). This is relative to the inner content area.
                    let extra_x = Px(pad_x.0.max(0.0));

                    let off_x = extra_x;
                    let on_x = Px((inner_w.0 - extra_x.0 - thumb.0).max(extra_x.0));

                    let duration = overlay_motion::shadcn_motion_duration_150(cx);
                    let x_target = if on { on_x } else { off_x };

                    // shadcn/ui v4 uses `transition-transform` for the thumb translation (Tailwind
                    // default duration) and avoids animating on initial mount.
                    let x = Px(decl_motion::drive_tween_f32_for_element(
                        cx,
                        id,
                        "thumb-x",
                        x_target.0,
                        duration,
                        switch_thumb_transition_ease,
                    )
                    .value);

                    let thumb_layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            top: Some(y).into(),
                            left: Some(x).into(),
                            ..Default::default()
                        },
                        size: SizeStyle {
                            width: Length::Px(thumb),
                            height: Length::Px(thumb),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let thumb_props = ContainerProps {
                        layout: thumb_layout,
                        padding: Edges::all(Px(0.0)).into(),
                        background: Some(thumb_color),
                        shadow: None,
                        border: Edges::all(Px(0.0)),
                        border_color: None,
                        corner_radii: Corners::all(Px((thumb.0 * 0.5).max(0.0))),
                        ..Default::default()
                    };

                    vec![cx.container(thumb_props, |_cx| Vec::new())]
                };

                (pressable_props, chrome_props, children)
            });

            if disabled {
                cx.opacity(0.5, |_cx| vec![pressable])
            } else {
                pressable
            }
        })
    }
}

pub fn switch<H: UiHost, M: IntoBoolModel>(model: M) -> impl IntoUiElement<H> + use<H, M> {
    Switch::new(model)
}

pub fn switch_opt<H: UiHost, M: IntoOptionalBoolModel>(
    model: M,
) -> impl IntoUiElement<H> + use<H, M> {
    Switch::new_opt(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, MouseButton, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
        PathStyle, Point, Px, Rect, Scene, Size as CoreSize, SvgId, SvgService, TextBlobId,
        TextConstraints, TextMetrics, TextService, WindowFrameClockService,
    };
    use fret_runtime::{
        CommandId, CommandMeta, CommandScope, Effect, FrameId, TickId,
        WindowCommandActionAvailabilityService, WindowCommandEnabledService,
        WindowCommandGatingService, WindowCommandGatingSnapshot,
    };
    use fret_ui::tree::UiTree;
    use std::collections::HashMap;

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
                    size: CoreSize::new(Px(10.0), Px(10.0)),
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

    #[test]
    fn switch_thumb_is_vertically_centered_in_track() {
        fn overlap_area(a: Rect, b: Rect) -> f32 {
            let ax0 = a.origin.x.0;
            let ay0 = a.origin.y.0;
            let ax1 = ax0 + a.size.width.0;
            let ay1 = ay0 + a.size.height.0;

            let bx0 = b.origin.x.0;
            let by0 = b.origin.y.0;
            let bx1 = bx0 + b.size.width.0;
            let by1 = by0 + b.size.height.0;

            let x0 = ax0.max(bx0);
            let y0 = ay0.max(by0);
            let x1 = ax1.min(bx1);
            let y1 = ay1.min(by1);

            let w = (x1 - x0).max(0.0);
            let h = (y1 - y0).max(0.0);
            w * h
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(160.0), Px(80.0)),
        );
        let mut services = FakeServices;

        let model = app.models_mut().insert(false);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-switch-thumb-centered",
            |cx| {
                vec![
                    Switch::new(model.clone())
                        .a11y_label("Switch")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let switch = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::Switch && n.label.as_deref() == Some("Switch")
            })
            .or_else(|| {
                snap.nodes
                    .iter()
                    .find(|n| n.role == fret_core::SemanticsRole::Switch)
            })
            .expect("missing semantics for switch");
        let switch_bounds = switch.bounds;

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let theme = Theme::global(&app);
        let track_bg = fret_core::Paint::Solid(switch_bg_off(theme)).into();
        let thumb_size = switch_thumb(theme, SwitchSize::Default);
        let thumb_bg = fret_core::Paint::Solid(switch_thumb_bg_off(theme)).into();

        let mut track_rect: Option<Rect> = None;
        let mut thumb_rect: Option<Rect> = None;
        for op in scene.ops() {
            let fret_core::SceneOp::Quad {
                rect, background, ..
            } = op
            else {
                continue;
            };

            let is_thumb = (rect.size.width.0 - thumb_size.0).abs() <= 0.1
                && (rect.size.height.0 - thumb_size.0).abs() <= 0.1
                && background.paint == thumb_bg;
            if is_thumb {
                thumb_rect = Some(*rect);
            }

            if background.paint == track_bg {
                let score = overlap_area(*rect, switch_bounds);
                if score <= 0.0 {
                    continue;
                }
                let replace =
                    track_rect.is_none_or(|best| overlap_area(best, switch_bounds) < score);
                if replace {
                    track_rect = Some(*rect);
                }
            }
        }

        let track = track_rect.expect("missing switch track quad");
        let thumb = thumb_rect.expect("missing switch thumb quad");

        let track_cy = track.origin.y.0 + track.size.height.0 * 0.5;
        let thumb_cy = thumb.origin.y.0 + thumb.size.height.0 * 0.5;
        assert!(
            (thumb_cy - track_cy).abs() <= 0.2,
            "expected thumb center_y {thumb_cy} close to track center_y {track_cy}"
        );
    }

    #[test]
    fn switch_toggles_model_on_click_and_exposes_checked_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(160.0), Px(80.0)),
        );
        let mut services = FakeServices;

        let model = app.models_mut().insert(false);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-switch-toggles-model-on-click",
            |cx| vec![Switch::new(model.clone()).into_element(cx)],
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.role == fret_core::SemanticsRole::Switch)
            .expect("switch semantics node");
        assert_eq!(node.flags.checked, Some(false));

        let switch_node = ui.children(root)[0];
        let switch_bounds = ui.debug_node_bounds(switch_node).expect("switch bounds");
        let position = Point::new(
            Px(switch_bounds.origin.x.0 + switch_bounds.size.width.0 * 0.5),
            Px(switch_bounds.origin.y.0 + switch_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
                button: MouseButton::Left,
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
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&model), Some(true));

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert!(!scene.ops().is_empty());
    }

    #[test]
    fn switch_checked_value_exposes_semantics_without_model() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(160.0), Px(80.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-switch-checked-value-semantics",
            |cx| {
                vec![
                    Switch::from_checked(true)
                        .a11y_label("Airplane mode")
                        .test_id("checked-switch")
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
            .find(|n| n.test_id.as_deref() == Some("checked-switch"))
            .expect("switch semantics node");
        assert_eq!(node.role, fret_core::SemanticsRole::Switch);
        assert_eq!(node.flags.checked, Some(true));
        assert_eq!(node.label.as_deref(), Some("Airplane mode"));
    }

    #[test]
    fn field_label_click_dispatches_action_for_snapshot_switch_control() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(240.0), Px(80.0)),
        );
        let mut services = FakeServices;

        let cmd = CommandId::from("test.switch.label-action");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Switch Label Action").with_scope(CommandScope::App),
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
            "shadcn-field-label-dispatches-switch-action",
            |cx| {
                let mut row_layout = LayoutStyle::default();
                row_layout.size.width = Length::Fill;

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
                            Switch::from_checked(true)
                                .control_id("test.switch")
                                .a11y_label("Test switch")
                                .action(cmd.clone())
                                .test_id("test.switch")
                                .into_element(cx),
                            crate::field::FieldLabel::new("Toggle via label")
                                .for_control("test.switch")
                                .into_element(cx)
                                .test_id("test.switch.label"),
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
            .find(|n| n.test_id.as_deref() == Some("test.switch.label"))
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
                button: MouseButton::Left,
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
                button: MouseButton::Left,
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
    fn switch_optional_none_toggles_to_some_true() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(160.0), Px(80.0)),
        );
        let mut services = FakeServices;

        let model = app.models_mut().insert(None::<bool>);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-switch-opt-toggles-model-on-click",
            |cx| vec![Switch::new_opt(model.clone()).into_element(cx)],
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.role == fret_core::SemanticsRole::Switch)
            .expect("switch semantics node");
        assert_eq!(node.flags.checked, Some(false));
        assert_eq!(app.models().get_copied(&model), Some(None));

        let switch_node = ui.children(root)[0];
        let switch_bounds = ui.debug_node_bounds(switch_node).expect("switch bounds");
        let position = Point::new(
            Px(switch_bounds.origin.x.0 + switch_bounds.size.width.0 * 0.5),
            Px(switch_bounds.origin.y.0 + switch_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
                button: MouseButton::Left,
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
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&model), Some(Some(true)));

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-switch-opt-toggles-model-on-click",
            |cx| vec![Switch::new_opt(model.clone()).into_element(cx)],
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.role == fret_core::SemanticsRole::Switch)
            .expect("switch semantics node");
        assert_eq!(node.flags.checked, Some(true));
    }

    #[test]
    fn switch_thumb_slides_between_states_over_time() {
        let window = AppWindowId::default();
        let mut app = App::new();

        // Stabilize transition duration scaling so the first frame doesn't collapse to a single
        // tick under host-reported deltas.
        app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
            svc.set_fixed_delta(window, Some(std::time::Duration::from_millis(16)));
        });
        for fid in [FrameId(1), FrameId(2)] {
            app.set_frame_id(fid);
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });
        }

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(2));
        app.with_global_mut(WindowFrameClockService::default, |svc, app| {
            svc.record_frame(window, app.frame_id());
        });

        let theme = Theme::global(&app);
        let thumb_size = switch_thumb(theme, SwitchSize::Default);
        let thumb_bg_off = switch_thumb_bg_off(theme);
        let thumb_bg_on = switch_thumb_bg_on(theme);
        let thumb_bgs = [thumb_bg_off, thumb_bg_on];

        fn find_thumb_left(el: &AnyElement, thumb_bgs: &[Color], thumb_size: Px) -> Option<Px> {
            match &el.kind {
                fret_ui::element::ElementKind::Container(props) => {
                    if props.layout.position == PositionStyle::Absolute
                        && props
                            .background
                            .is_some_and(|bg| thumb_bgs.iter().any(|c| *c == bg))
                        && props.layout.size.width == Length::Px(thumb_size)
                        && props.layout.size.height == Length::Px(thumb_size)
                    {
                        if let fret_ui::element::InsetEdge::Px(left) = props.layout.inset.left {
                            return Some(left);
                        }
                    }
                    el.children
                        .iter()
                        .find_map(|c| find_thumb_left(c, thumb_bgs, thumb_size))
                }
                _ => el
                    .children
                    .iter()
                    .find_map(|c| find_thumb_left(c, thumb_bgs, thumb_size)),
            }
        }

        fn render_switch(app: &mut App, window: AppWindowId, model: Model<bool>) -> AnyElement {
            fret_ui::elements::with_element_cx(
                app,
                window,
                Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    CoreSize::new(Px(200.0), Px(120.0)),
                ),
                "switch_thumb_slide",
                |cx| Switch::new(model).into_element(cx),
            )
        }

        let model = app.models_mut().insert(false);
        let off_el = render_switch(&mut app, window, model.clone());
        let off_x = find_thumb_left(&off_el, &thumb_bgs, thumb_size)
            .expect("thumb left inset (off)")
            .0;

        let _ = app.models_mut().update(&model, |v| *v = true);

        let mut xs = Vec::new();
        for i in 0..24u64 {
            app.set_tick_id(TickId(2 + i));
            app.set_frame_id(FrameId(3 + i));
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });
            let el = render_switch(&mut app, window, model.clone());
            let left = find_thumb_left(&el, &thumb_bgs, thumb_size).expect("thumb left inset");
            xs.push(left.0);
        }

        let on_x = *xs.last().expect("thumb samples");
        assert!(
            on_x > off_x + 0.5,
            "expected thumb to move right, off_x={off_x} on_x={on_x}"
        );

        let first = xs[0];
        assert!(
            first < on_x - 0.1,
            "expected thumb not to jump to final position on first frame; first={first} on_x={on_x}"
        );

        for pair in xs.windows(2) {
            let a = pair[0];
            let b = pair[1];
            assert!(
                b + 0.2 >= a,
                "expected thumb x to be mostly monotonic; a={a} b={b}"
            );
        }
    }

    #[test]
    fn switch_track_background_tweens_between_states_over_time() {
        let window = AppWindowId::default();
        let mut app = App::new();

        // Stabilize transition duration scaling so the first frame doesn't collapse to a single
        // tick under host-reported deltas.
        app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
            svc.set_fixed_delta(window, Some(std::time::Duration::from_millis(16)));
        });
        for fid in [FrameId(1), FrameId(2)] {
            app.set_frame_id(fid);
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });
        }

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(2));
        app.with_global_mut(WindowFrameClockService::default, |svc, app| {
            svc.record_frame(window, app.frame_id());
        });

        let theme = Theme::global(&app);
        let track_w = switch_track_w(theme, SwitchSize::Default);
        let track_h = switch_track_h(theme, SwitchSize::Default);
        let bg_on = switch_bg_on(theme);
        let thumb_bg_off = switch_thumb_bg_off(theme);
        let thumb_bg_on = switch_thumb_bg_on(theme);
        let thumb_bgs = [thumb_bg_off, thumb_bg_on];

        fn find_track_bg(el: &AnyElement, w: Px, h: Px, thumb_bgs: &[Color]) -> Option<Color> {
            match &el.kind {
                fret_ui::element::ElementKind::Container(props) => {
                    let looks_like_track = props.shadow.is_some()
                        && props.border.top.0 > 0.0
                        && props.border.left.0 > 0.0
                        && props.corner_radii.top_left.0 > 0.0
                        && props.background.is_some()
                        && !props
                            .background
                            .is_some_and(|bg| thumb_bgs.iter().any(|c| *c == bg))
                        && (props.layout.size.width == Length::Px(w)
                            || props.layout.size.width == Length::Fill)
                        && (props.layout.size.height == Length::Px(h)
                            || props.layout.size.height == Length::Fill);
                    if looks_like_track {
                        return props.background;
                    }
                    el.children
                        .iter()
                        .find_map(|c| find_track_bg(c, w, h, thumb_bgs))
                }
                _ => el
                    .children
                    .iter()
                    .find_map(|c| find_track_bg(c, w, h, thumb_bgs)),
            }
        }

        fn render_switch(app: &mut App, window: AppWindowId, model: Model<bool>) -> AnyElement {
            fret_ui::elements::with_element_cx(
                app,
                window,
                Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    CoreSize::new(Px(200.0), Px(120.0)),
                ),
                "switch_track_bg_tween",
                |cx| Switch::new(model).into_element(cx),
            )
        }

        fn channel_abs_sum(a: Color, b: Color) -> f32 {
            (a.r - b.r).abs() + (a.g - b.g).abs() + (a.b - b.b).abs() + (a.a - b.a).abs()
        }

        let model = app.models_mut().insert(false);
        let off_el = render_switch(&mut app, window, model.clone());
        let bg_off =
            find_track_bg(&off_el, track_w, track_h, &thumb_bgs).expect("track background");

        let _ = app.models_mut().update(&model, |v| *v = true);

        let mut bgs = Vec::new();
        for i in 0..24u64 {
            app.set_tick_id(TickId(2 + i));
            app.set_frame_id(FrameId(3 + i));
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });

            let el = render_switch(&mut app, window, model.clone());
            let bg = find_track_bg(&el, track_w, track_h, &thumb_bgs).expect("track background");
            bgs.push(bg);
        }

        let first = bgs[0];
        let last = *bgs.last().expect("track background samples");

        assert!(
            channel_abs_sum(first, bg_on) > 0.0001,
            "expected track background not to snap to final on color on first frame; first={first:?} on={bg_on:?}"
        );
        assert!(
            channel_abs_sum(first, bg_off) > 0.0001,
            "expected track background to start transitioning away from off color on first frame; off={bg_off:?} first={first:?}"
        );
        assert!(
            channel_abs_sum(last, bg_on) < 0.02,
            "expected track background to settle near on color; last={last:?} on={bg_on:?}"
        );
        assert!(
            channel_abs_sum(last, bg_on) < channel_abs_sum(first, bg_on),
            "expected track background to move closer to on color over time; first={first:?} last={last:?} on={bg_on:?}"
        );
    }

    #[test]
    fn command_gating_switch_is_disabled_by_window_command_enabled_service() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let checked = app.models_mut().insert(false);
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
            CoreSize::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "command-gating-switch-enabled-service",
            |cx| {
                vec![
                    Switch::new(checked.clone())
                        .a11y_label("Switch")
                        .on_click(cmd.clone())
                        .test_id("disabled-switch")
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
            .find(|n| n.test_id.as_deref() == Some("disabled-switch"))
            .expect("expected a semantics node for the switch test_id");
        assert!(node.flags.disabled);
    }

    #[test]
    fn command_gating_switch_is_disabled_when_widget_action_is_unavailable() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let checked = app.models_mut().insert(false);
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
            CoreSize::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "command-gating-switch-action-availability",
            |cx| {
                vec![
                    Switch::new(checked.clone())
                        .a11y_label("Switch")
                        .on_click(cmd.clone())
                        .test_id("disabled-switch")
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
            .find(|n| n.test_id.as_deref() == Some("disabled-switch"))
            .expect("expected a semantics node for the switch test_id");
        assert!(node.flags.disabled);
    }

    #[test]
    fn command_gating_switch_prefers_window_command_gating_snapshot_when_present() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let checked = app.models_mut().insert(false);
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
            CoreSize::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "command-gating-switch-gating-snapshot",
            |cx| {
                vec![
                    Switch::new(checked.clone())
                        .a11y_label("Switch")
                        .on_click(cmd.clone())
                        .test_id("disabled-switch")
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
            .find(|n| n.test_id.as_deref() == Some("disabled-switch"))
            .expect("expected a semantics node for the switch test_id");
        assert!(node.flags.disabled);
    }

    #[test]
    fn switch_size_sm_keeps_thumb_centered() {
        fn overlap_area(a: Rect, b: Rect) -> f32 {
            let ax0 = a.origin.x.0;
            let ay0 = a.origin.y.0;
            let ax1 = ax0 + a.size.width.0;
            let ay1 = ay0 + a.size.height.0;

            let bx0 = b.origin.x.0;
            let by0 = b.origin.y.0;
            let bx1 = bx0 + b.size.width.0;
            let by1 = by0 + b.size.height.0;

            let x0 = ax0.max(bx0);
            let y0 = ay0.max(by0);
            let x1 = ax1.min(bx1);
            let y1 = ay1.min(by1);

            let w = (x1 - x0).max(0.0);
            let h = (y1 - y0).max(0.0);
            w * h
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(160.0), Px(80.0)),
        );
        let mut services = FakeServices;

        let model = app.models_mut().insert(false);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-switch-size-sm-thumb-centered",
            |cx| {
                vec![
                    Switch::new(model.clone())
                        .size(SwitchSize::Sm)
                        .a11y_label("Switch")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let switch = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::Switch && n.label.as_deref() == Some("Switch")
            })
            .or_else(|| {
                snap.nodes
                    .iter()
                    .find(|n| n.role == fret_core::SemanticsRole::Switch)
            })
            .expect("missing semantics for switch");
        let switch_bounds = switch.bounds;

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let theme = Theme::global(&app);
        let track_bg = fret_core::Paint::Solid(switch_bg_off(theme)).into();
        let thumb_size = switch_thumb(theme, SwitchSize::Sm);
        let thumb_bg = fret_core::Paint::Solid(switch_thumb_bg_off(theme)).into();

        let mut track_rect: Option<Rect> = None;
        let mut thumb_rect: Option<Rect> = None;
        for op in scene.ops() {
            let fret_core::SceneOp::Quad {
                rect, background, ..
            } = op
            else {
                continue;
            };

            let is_thumb = (rect.size.width.0 - thumb_size.0).abs() <= 0.1
                && (rect.size.height.0 - thumb_size.0).abs() <= 0.1
                && background.paint == thumb_bg;
            if is_thumb {
                thumb_rect = Some(*rect);
            }

            if background.paint == track_bg {
                let score = overlap_area(*rect, switch_bounds);
                if score <= 0.0 {
                    continue;
                }
                let replace =
                    track_rect.is_none_or(|best| overlap_area(best, switch_bounds) < score);
                if replace {
                    track_rect = Some(*rect);
                }
            }
        }

        let track = track_rect.expect("missing switch track quad");
        let thumb = thumb_rect.expect("missing switch thumb quad");

        let track_cy = track.origin.y.0 + track.size.height.0 * 0.5;
        let thumb_cy = thumb.origin.y.0 + thumb.size.height.0 * 0.5;
        assert!(
            (thumb_cy - track_cy).abs() <= 0.2,
            "expected thumb center_y {thumb_cy} close to track center_y {track_cy}"
        );
    }

    #[test]
    fn switch_thumb_color_matches_shadcn_dark_scheme() {
        use crate::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york};

        fn find_thumb_bg(scene: &Scene, expected_size: Px) -> Option<fret_core::Paint> {
            for op in scene.ops() {
                let fret_core::SceneOp::Quad {
                    rect, background, ..
                } = op
                else {
                    continue;
                };
                let is_thumb = (rect.size.width.0 - expected_size.0).abs() <= 0.1
                    && (rect.size.height.0 - expected_size.0).abs() <= 0.1;
                if is_thumb {
                    return Some(background.paint);
                }
            }
            None
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Dark);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(160.0), Px(80.0)),
        );
        let mut services = FakeServices;

        let off_model = app.models_mut().insert(false);
        let on_model = app.models_mut().insert(true);

        for (label, model, expected_key) in [
            ("off", off_model, "foreground"),
            ("on", on_model, "primary-foreground"),
        ] {
            let mut ui: UiTree<App> = UiTree::new();
            ui.set_window(window);

            let root = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                &format!("shadcn-switch-thumb-color-dark-{label}"),
                |cx| vec![Switch::new(model.clone()).into_element(cx)],
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

            let theme = Theme::global(&app);
            assert_eq!(theme.color_scheme, Some(ColorScheme::Dark));

            let thumb_size = switch_thumb(theme, SwitchSize::Default);
            let got = find_thumb_bg(&scene, thumb_size).expect("missing thumb quad");
            let expected = fret_core::Paint::Solid(theme.color_token(expected_key));
            assert_eq!(got, expected, "thumb paint mismatch for state {label}");
        }
    }
}
