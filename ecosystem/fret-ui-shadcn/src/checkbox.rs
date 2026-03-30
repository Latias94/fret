use std::{any::Any, sync::Arc};

use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_icons::ids;
use fret_runtime::{ActionId, CommandId, Model};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, PressableProps,
};
use fret_ui::{ElementContext, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::motion::drive_tween_f32_for_element;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::checkbox::{
    CheckedState, checkbox_a11y, checkbox_use_checked_model, checked_state_from_optional_bool,
    toggle_optional_bool,
};
use fret_ui_kit::primitives::control_registry::{
    ControlAction, ControlEntry, ControlId, control_registry_model,
};
use fret_ui_kit::primitives::controllable_state;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverrideSlot, Radius, WidgetState,
    WidgetStateProperty, WidgetStates, resolve_override_slot, resolve_override_slot_opt,
};

use crate::bool_model::IntoBoolModel;
use crate::checked_state_model::IntoCheckedStateModel;
use crate::optional_bool_model::IntoOptionalBoolModel;
use crate::overlay_motion;

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn checkbox_size(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.checkbox.size")
        .unwrap_or(Px(16.0))
}

fn checkbox_radius(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.checkbox.radius")
        .unwrap_or_else(|| MetricRef::radius(Radius::Sm).resolve(theme))
}

fn checkbox_border(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .expect("missing theme token: input/border")
}

fn checkbox_bg_checked(theme: &ThemeSnapshot) -> Color {
    theme.color_token("primary")
}

fn checkbox_fg_checked(theme: &ThemeSnapshot) -> Color {
    theme.color_token("primary-foreground")
}

fn checkbox_ring_color(theme: &ThemeSnapshot) -> Color {
    theme.color_token("ring")
}

#[derive(Debug, Clone, Default)]
pub struct CheckboxStyle {
    pub background: OverrideSlot<ColorRef>,
    pub border_color: OverrideSlot<ColorRef>,
    pub foreground: OverrideSlot<ColorRef>,
}

impl CheckboxStyle {
    pub fn background(mut self, background: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.background = Some(background);
        self
    }

    pub fn border_color(mut self, border_color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.border_color = Some(border_color);
        self
    }

    pub fn foreground(mut self, foreground: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.foreground = Some(foreground);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.background.is_some() {
            self.background = other.background;
        }
        if other.border_color.is_some() {
            self.border_color = other.border_color;
        }
        if other.foreground.is_some() {
            self.foreground = other.foreground;
        }
        self
    }
}

#[derive(Clone)]
pub struct Checkbox {
    checked: CheckboxCheckedModel,
    aria_invalid: bool,
    required: bool,
    disabled: bool,
    control_id: Option<ControlId>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_click: Option<CommandId>,
    action_payload: Option<Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: CheckboxStyle,
}

#[derive(Debug, Clone)]
enum CheckboxCheckedModel {
    Bool(Model<bool>),
    OptionalBool(Model<Option<bool>>),
    TriState(Model<CheckedState>),
    Value(CheckedState),
}

impl Checkbox {
    pub fn new(model: impl IntoBoolModel) -> Self {
        Self {
            checked: CheckboxCheckedModel::Bool(model.into_bool_model()),
            aria_invalid: false,
            required: false,
            disabled: false,
            control_id: None,
            a11y_label: None,
            test_id: None,
            on_click: None,
            action_payload: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: CheckboxStyle::default(),
        }
    }

    pub fn new_tristate(model: impl IntoCheckedStateModel) -> Self {
        Self {
            checked: CheckboxCheckedModel::TriState(model.into_checked_state_model()),
            aria_invalid: false,
            required: false,
            disabled: false,
            control_id: None,
            a11y_label: None,
            test_id: None,
            on_click: None,
            action_payload: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: CheckboxStyle::default(),
        }
    }

    /// Creates a checkbox from a plain bool value, mirroring the upstream controlled `checked`
    /// prop without forcing a `Model<bool>` at the call site.
    ///
    /// This is intended for views that already own the state elsewhere (for example a
    /// `LocalState<Vec<Row>>` collection) and only need the checkbox to render the current value
    /// while dispatching an external action on click.
    pub fn from_checked(checked: bool) -> Self {
        Self::from_checked_state(CheckedState::from(checked))
    }

    /// Creates a checkbox from an explicit tri-state snapshot.
    pub fn from_checked_state(checked: CheckedState) -> Self {
        Self {
            checked: CheckboxCheckedModel::Value(checked),
            aria_invalid: false,
            required: false,
            disabled: false,
            control_id: None,
            a11y_label: None,
            test_id: None,
            on_click: None,
            action_payload: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: CheckboxStyle::default(),
        }
    }

    /// Creates a checkbox bound to an optional boolean model.
    ///
    /// This maps `None` to the indeterminate state, matching Radix's `"indeterminate"` outcome.
    pub fn new_optional(model: impl IntoOptionalBoolModel) -> Self {
        Self {
            checked: CheckboxCheckedModel::OptionalBool(model.into_optional_bool_model()),
            aria_invalid: false,
            required: false,
            disabled: false,
            control_id: None,
            a11y_label: None,
            test_id: None,
            on_click: None,
            action_payload: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: CheckboxStyle::default(),
        }
    }

    /// Creates a checkbox with a controlled/uncontrolled bool model (Radix `checked` /
    /// `defaultChecked` as a boolean).
    ///
    /// Note: If `checked` is `None`, the internal model is stored in element state at the call site.
    /// Call this from a stable subtree (key the parent node if needed).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        checked: Option<Model<bool>>,
        default_checked: bool,
    ) -> Self {
        let model =
            controllable_state::use_controllable_model(cx, checked, || default_checked).model();
        Self::new(model)
    }

    /// Creates a checkbox with a controlled/uncontrolled optional-bool model.
    ///
    /// This is shadcn-friendly ergonomics for representing an "unset/indeterminate" value.
    pub fn new_optional_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        checked: Option<Model<Option<bool>>>,
        default_checked: Option<bool>,
    ) -> Self {
        let model =
            controllable_state::use_controllable_model(cx, checked, || default_checked).model();
        Self::new_optional(model)
    }

    /// Creates a checkbox with a controlled/uncontrolled tri-state model (Radix
    /// `boolean | "indeterminate"` via [`CheckedState`]).
    ///
    /// Note: If `checked` is `None`, the internal model is stored in element state at the call site.
    /// Call this from a stable subtree (key the parent node if needed).
    pub fn new_tristate_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        checked: Option<Model<CheckedState>>,
        default_checked: CheckedState,
    ) -> Self {
        let model = checkbox_use_checked_model(cx, checked, || default_checked).model();
        Self::new_tristate(model)
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Apply the upstream `aria-invalid` error state chrome (border + focus ring color).
    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Associates this checkbox with a logical form control id so related elements (e.g. labels)
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

    /// Bind a stable action ID to this checkbox (action-first authoring).
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

    /// Like [`Checkbox::action_payload`], but computes the payload lazily on activation.
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

    pub fn style(mut self, style: CheckboxStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let checked = self.checked;
            let aria_invalid = self.aria_invalid;
            let required = self.required;

            let theme = Theme::global(&*cx.app).snapshot();

            let size = checkbox_size(&theme);
            let radius = checkbox_radius(&theme);
            let border = checkbox_border(&theme);
            let bg_on = checkbox_bg_checked(&theme);
            let fg_on = checkbox_fg_checked(&theme);

            let ring_border = if aria_invalid {
                theme.color_token("destructive")
            } else {
                checkbox_ring_color(&theme)
            };
            let mut ring = decl_style::focus_ring(&theme, radius);
            ring.color = if aria_invalid {
                crate::theme_variants::invalid_control_ring_color(&theme, ring_border)
            } else {
                alpha_mul(ring_border, 0.5)
            };

            // Upstream shadcn checkbox uses `dark:bg-input/30` for the unchecked state.
            let default_background = WidgetStateProperty::new(Some(ColorRef::Token {
                key: "component.input.bg",
                fallback: fret_ui_kit::ColorFallback::Color(Color::TRANSPARENT),
            }))
            .when(WidgetStates::SELECTED, Some(ColorRef::Color(bg_on)));
            let default_border_color_off = if aria_invalid {
                theme.color_token("destructive")
            } else {
                border
            };
            let default_border_color =
                WidgetStateProperty::new(ColorRef::Color(default_border_color_off))
                    .when(
                        WidgetStates::SELECTED,
                        ColorRef::Color(if aria_invalid {
                            theme.color_token("destructive")
                        } else {
                            bg_on
                        }),
                    )
                    .when(WidgetStates::FOCUS_VISIBLE, ColorRef::Color(ring_border));
            let default_foreground = WidgetStateProperty::new(ColorRef::Color(Color::TRANSPARENT))
                .when(WidgetStates::SELECTED, ColorRef::Color(fg_on));

            let layout = LayoutRefinement::default()
                .w_px(size)
                .h_px(size)
                .min_w(size)
                .min_h(size)
                // shadcn/ui sets `shrink-0` so the checkbox keeps its 16px affordance when
                // horizontally constrained (e.g. in tables or dense control rows).
                .flex_shrink_0()
                .merge(self.layout);
            let pressable_layout = decl_style::layout_style(&theme, layout);

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
                cx.key_add_on_key_down_for(
                    id,
                    fret_ui_kit::primitives::keyboard::consume_enter_key_handler(),
                );
                if let Some(payload) = action_payload.clone() {
                    cx.pressable_dispatch_command_with_payload_factory_if_enabled_opt(
                        on_click.clone(),
                        payload,
                    );
                } else {
                    cx.pressable_dispatch_command_if_enabled_opt(on_click.clone());
                }
                match &checked {
                    CheckboxCheckedModel::Bool(model) => cx.pressable_toggle_bool(model),
                    CheckboxCheckedModel::OptionalBool(model) => {
                        cx.pressable_update_model(model, |v| {
                            *v = toggle_optional_bool(*v);
                        });
                    }
                    CheckboxCheckedModel::TriState(model) => {
                        cx.pressable_update_model(model, |v| *v = v.toggle());
                    }
                    CheckboxCheckedModel::Value(_) => {}
                }

                let theme = Theme::global(&*cx.app).snapshot();
                let state = match &checked {
                    CheckboxCheckedModel::Bool(model) => {
                        CheckedState::from(cx.watch_model(model).copied().unwrap_or(false))
                    }
                    CheckboxCheckedModel::OptionalBool(model) => {
                        checked_state_from_optional_bool(cx.watch_model(model).copied().flatten())
                    }
                    CheckboxCheckedModel::TriState(model) => {
                        cx.watch_model(model).copied().unwrap_or_default()
                    }
                    CheckboxCheckedModel::Value(state) => *state,
                };
                let is_on = state.is_on();
                let is_checked = state.is_checked();
                let is_indeterminate = state.is_indeterminate();

                let mut states = WidgetStates::from_pressable(cx, st, !disabled);
                states.set(WidgetState::Selected, is_on);
                let focus_visible = states.contains(WidgetStates::FOCUS_VISIBLE);
                let duration = overlay_motion::shadcn_motion_duration_150(cx);
                let ring_alpha = drive_tween_f32_for_element(
                    cx,
                    id,
                    "checkbox.ring.alpha",
                    if focus_visible { 1.0 } else { 0.0 },
                    duration,
                    overlay_motion::shadcn_ease,
                );

                let bg = resolve_override_slot_opt(
                    style_override.background.as_ref(),
                    &default_background,
                    states,
                )
                .map(|bg| bg.resolve(&theme))
                .unwrap_or(Color::TRANSPARENT);

                let border_color = resolve_override_slot(
                    style_override.border_color.as_ref(),
                    &default_border_color,
                    states,
                )
                .resolve(&theme);
                let fg = resolve_override_slot(
                    style_override.foreground.as_ref(),
                    &default_foreground,
                    states,
                )
                .resolve(&theme);

                let mut chrome_props = decl_style::container_props(
                    &theme,
                    ChromeRefinement::default()
                        .rounded(Radius::Sm)
                        .border_1()
                        .bg(ColorRef::Color(bg))
                        .border_color(ColorRef::Color(border_color))
                        .merge(chrome.clone()),
                    LayoutRefinement::default(),
                );
                chrome_props.corner_radii = Corners::all(radius);
                chrome_props.padding = Edges::all(Px(0.0)).into();
                chrome_props.shadow = Some(decl_style::shadow_xs(&theme, radius));
                chrome_props.layout.size = pressable_layout.size;

                if let (Some(control_id), Some(control_registry)) =
                    (control_id.clone(), control_registry.clone())
                {
                    let toggle_action = match &checked {
                        CheckboxCheckedModel::Bool(model) => {
                            Some(ControlAction::ToggleBool(model.clone()))
                        }
                        CheckboxCheckedModel::OptionalBool(model) => {
                            Some(ControlAction::ToggleOptionalBool(model.clone()))
                        }
                        CheckboxCheckedModel::TriState(model) => {
                            Some(ControlAction::ToggleCheckedState(model.clone()))
                        }
                        CheckboxCheckedModel::Value(_) => None,
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

                let mut a11y = checkbox_a11y(a11y_label, state);
                a11y.required = required;
                if let Some(label) = labelled_by_element {
                    a11y.labelled_by_element = Some(label.0);
                }
                a11y.test_id = test_id.clone();
                let mut ring = ring;
                ring.color.a = (ring.color.a * ring_alpha.value).clamp(0.0, 1.0);
                if let Some(offset_color) = ring.offset_color {
                    ring.offset_color = Some(Color {
                        a: (offset_color.a * ring_alpha.value).clamp(0.0, 1.0),
                        ..offset_color
                    });
                }
                let pressable_props = PressableProps {
                    layout: pressable_layout,
                    enabled: !disabled,
                    focusable: true,
                    focus_ring: Some(ring),
                    focus_ring_always_paint: ring_alpha.animating,
                    a11y,
                    ..Default::default()
                };

                let children = move |cx: &mut ElementContext<'_, H>| {
                    if !is_on {
                        return Vec::new();
                    }

                    let icon_size = theme
                        .metric_by_key("component.checkbox.icon_size")
                        .unwrap_or(Px(14.0));
                    let icon_id = if is_checked {
                        ids::ui::CHECK
                    } else if is_indeterminate {
                        ids::ui::MINUS
                    } else {
                        ids::ui::CHECK
                    };
                    let icon = decl_icon::icon_with(
                        &mut *cx,
                        icon_id,
                        Some(icon_size),
                        Some(ColorRef::Color(fg)),
                    );

                    let mut inner_layout = LayoutStyle::default();
                    inner_layout.size.width = Length::Fill;
                    inner_layout.size.height = Length::Fill;

                    vec![cx.flex(
                        FlexProps {
                            layout: inner_layout,
                            direction: Axis::Horizontal,
                            gap: Px(0.0).into(),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |_cx| vec![icon],
                    )]
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

pub fn checkbox<H: UiHost, M: IntoBoolModel>(model: M) -> impl IntoUiElement<H> + use<H, M> {
    Checkbox::new(model)
}

pub fn checkbox_opt<H: UiHost, M: IntoOptionalBoolModel>(
    model: M,
) -> impl IntoUiElement<H> + use<H, M> {
    Checkbox::new_optional(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;
    use std::rc::Rc;
    use std::time::Duration;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, MouseButton, PathCommand, PathConstraints, PathId, PathMetrics,
        PathService, PathStyle, Point, Px, Rect, Scene, Size as CoreSize, SvgId, SvgService,
        TextBlobId, TextConstraints, TextMetrics, TextService,
    };
    use fret_runtime::{
        CommandId, CommandMeta, CommandScope, Effect, FrameId,
        WindowCommandActionAvailabilityService, WindowCommandEnabledService,
        WindowCommandGatingService, WindowCommandGatingSnapshot, WindowPendingActionPayloadService,
    };
    use fret_ui::element::{ElementKind, PressableProps};
    use fret_ui::tree::UiTree;
    use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;
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
    fn checkbox_toggles_model_on_click_and_exposes_checked_semantics() {
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
            "shadcn-checkbox-toggles-model-on-click",
            |cx| vec![Checkbox::new(model.clone()).into_element(cx)],
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.role == fret_core::SemanticsRole::Checkbox)
            .expect("checkbox semantics node");
        assert_eq!(
            node.flags.checked_state,
            Some(fret_core::SemanticsCheckedState::False)
        );
        assert_eq!(node.flags.checked, Some(false));

        let checkbox_node = ui.children(root)[0];
        let checkbox_bounds = ui
            .debug_node_bounds(checkbox_node)
            .expect("checkbox bounds");
        let position = Point::new(
            Px(checkbox_bounds.origin.x.0 + checkbox_bounds.size.width.0 * 0.5),
            Px(checkbox_bounds.origin.y.0 + checkbox_bounds.size.height.0 * 0.5),
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
    fn checkbox_checked_value_exposes_semantics_without_model() {
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
            "shadcn-checkbox-checked-value-semantics",
            |cx| {
                vec![
                    Checkbox::from_checked(true)
                        .test_id("checked-value-checkbox")
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
            .find(|n| n.test_id.as_deref() == Some("checked-value-checkbox"))
            .expect("checkbox semantics node");
        assert_eq!(node.role, fret_core::SemanticsRole::Checkbox);
        assert_eq!(
            node.flags.checked_state,
            Some(fret_core::SemanticsCheckedState::True)
        );
        assert_eq!(node.flags.checked, Some(true));

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert!(!scene.ops().is_empty());
    }

    #[test]
    fn checkbox_required_exposes_required_semantics() {
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
            "shadcn-checkbox-required-semantics",
            |cx| {
                vec![
                    Checkbox::from_checked(true)
                        .required(true)
                        .a11y_label("Terms")
                        .test_id("required-checkbox")
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
            .find(|n| n.test_id.as_deref() == Some("required-checkbox"))
            .expect("checkbox semantics node");
        assert_eq!(node.role, fret_core::SemanticsRole::Checkbox);
        assert!(node.flags.required);
    }

    #[test]
    fn checkbox_tristate_indeterminate_toggles_to_checked() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(160.0), Px(80.0)),
        );
        let mut services = FakeServices;

        let model = app.models_mut().insert(CheckedState::Indeterminate);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-checkbox-tristate-indeterminate-toggles",
            |cx| vec![Checkbox::new_tristate(model.clone()).into_element(cx)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let checkbox_node = ui.children(root)[0];
        let checkbox_bounds = ui
            .debug_node_bounds(checkbox_node)
            .expect("checkbox bounds");
        let position = Point::new(
            Px(checkbox_bounds.origin.x.0 + checkbox_bounds.size.width.0 * 0.5),
            Px(checkbox_bounds.origin.y.0 + checkbox_bounds.size.height.0 * 0.5),
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

        assert_eq!(app.models().get_copied(&model), Some(CheckedState::Checked));
    }

    #[test]
    fn checkbox_optional_none_is_indeterminate_and_toggles_to_checked() {
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
            "shadcn-checkbox-optional-none-indeterminate",
            |cx| vec![Checkbox::new_optional(model.clone()).into_element(cx)],
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.role == fret_core::SemanticsRole::Checkbox)
            .expect("checkbox semantics node");
        assert_eq!(
            node.flags.checked_state,
            Some(fret_core::SemanticsCheckedState::Mixed)
        );
        assert_eq!(node.flags.checked, None);

        let checkbox_node = ui.children(root)[0];
        let checkbox_bounds = ui
            .debug_node_bounds(checkbox_node)
            .expect("checkbox bounds");
        let position = Point::new(
            Px(checkbox_bounds.origin.x.0 + checkbox_bounds.size.width.0 * 0.5),
            Px(checkbox_bounds.origin.y.0 + checkbox_bounds.size.height.0 * 0.5),
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
            "shadcn-checkbox-optional-none-indeterminate",
            |cx| vec![Checkbox::new_optional(model.clone()).into_element(cx)],
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.role == fret_core::SemanticsRole::Checkbox)
            .expect("checkbox semantics node");
        assert_eq!(node.flags.checked, Some(true));
        assert_eq!(
            node.flags.checked_state,
            Some(fret_core::SemanticsCheckedState::True)
        );
    }

    #[test]
    fn checkbox_does_not_toggle_on_enter_key() {
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
            "shadcn-checkbox-enter-does-not-toggle",
            |cx| vec![Checkbox::new(model.clone()).into_element(cx)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable checkbox");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyUp {
                key: fret_core::KeyCode::Enter,
                modifiers: Modifiers::default(),
            },
        );

        assert_eq!(app.models().get_copied(&model), Some(false));
    }

    #[test]
    fn checkbox_toggles_on_space_key() {
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
            "shadcn-checkbox-space-toggles",
            |cx| vec![Checkbox::new(model.clone()).into_element(cx)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable checkbox");
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

        assert_eq!(app.models().get_copied(&model), Some(true));
    }

    #[test]
    fn field_label_click_toggles_registered_control() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(240.0), Px(80.0)),
        );
        let mut services = FakeServices;

        let model = app.models_mut().insert(true);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-field-label-click-toggles-control",
            |cx| {
                let mut row_layout = LayoutStyle::default();
                row_layout.size.width = Length::Fill;

                vec![cx.flex(
                    FlexProps {
                        layout: row_layout,
                        direction: Axis::Horizontal,
                        gap: Px(8.0).into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    |cx| {
                        vec![
                            Checkbox::new(model.clone())
                                .control_id("test.checkbox")
                                .a11y_label("Test checkbox")
                                .test_id("test.checkbox")
                                .into_element(cx),
                            crate::field::FieldLabel::new("Toggle via label")
                                .for_control("test.checkbox")
                                .into_element(cx)
                                .test_id("test.checkbox.label"),
                        ]
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let label = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("test.checkbox.label"))
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

        assert_eq!(app.models().get_copied(&model), Some(false));
    }

    #[test]
    fn field_label_click_mirrors_checkbox_action_sequence() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(240.0), Px(80.0)),
        );
        let mut services = FakeServices;

        let model = app.models_mut().insert(true);
        let cmd = CommandId::from("test.checkbox.label-action");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Checkbox Label Action").with_scope(CommandScope::App),
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
            "shadcn-field-label-mirrors-checkbox-action-sequence",
            |cx| {
                let mut row_layout = LayoutStyle::default();
                row_layout.size.width = Length::Fill;

                vec![cx.flex(
                    FlexProps {
                        layout: row_layout,
                        direction: Axis::Horizontal,
                        gap: Px(8.0).into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    |cx| {
                        vec![
                            Checkbox::new(model.clone())
                                .control_id("test.checkbox")
                                .a11y_label("Test checkbox")
                                .action(cmd.clone())
                                .action_payload(41u32)
                                .test_id("test.checkbox")
                                .into_element(cx),
                            crate::field::FieldLabel::new("Toggle via label")
                                .for_control("test.checkbox")
                                .into_element(cx)
                                .test_id("test.checkbox.label"),
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
            .find(|n| n.test_id.as_deref() == Some("test.checkbox.label"))
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

        assert_eq!(app.models().get_copied(&model), Some(false));
        let effects = app.flush_effects();
        assert!(
            effects
                .iter()
                .any(|effect| matches!(effect, Effect::Command { command, .. } if command.as_str() == cmd.as_str())),
            "expected label click to dispatch {cmd:?}, got {effects:?}"
        );
        let payload = app
            .with_global_mut(WindowPendingActionPayloadService::default, |svc, app| {
                svc.consume(window, app.tick_id(), &cmd)
            })
            .expect("expected pending payload for checkbox label action");
        let payload = payload
            .downcast::<u32>()
            .ok()
            .expect("payload type must match");
        assert_eq!(*payload, 41);
    }

    #[test]
    fn checkbox_does_not_shrink_when_horizontally_constrained() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        // Constrain the row so flex shrink would otherwise squeeze fixed-size affordances.
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(30.0), Px(40.0)),
        );
        let mut services = FakeServices;

        // Use a checked checkbox so the indicator subtree is present (matches the reported regression).
        let model = app.models_mut().insert(true);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-checkbox-does-not-shrink",
            |cx| {
                let mut row_layout = LayoutStyle::default();
                row_layout.size.width = Length::Fill;

                let row = cx.flex(
                    FlexProps {
                        layout: row_layout,
                        direction: Axis::Horizontal,
                        gap: Px(0.0).into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Start,
                        wrap: false,
                    },
                    |cx| {
                        let checkbox = Checkbox::new(model.clone())
                            .test_id("shadcn-checkbox-shrink-0")
                            .into_element(cx);

                        let mut filler_layout = LayoutStyle::default();
                        filler_layout.size.width = Length::Px(Px(24.0));
                        filler_layout.size.height = Length::Px(Px(16.0));
                        let filler = cx.container(
                            fret_ui::element::ContainerProps {
                                layout: filler_layout,
                                ..Default::default()
                            },
                            |_cx| Vec::new(),
                        );

                        vec![checkbox, filler]
                    },
                );

                vec![row]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let row_node = ui.children(root)[0];
        let checkbox_node = ui.children(row_node)[0];
        let checkbox_bounds = ui
            .debug_node_bounds(checkbox_node)
            .expect("checkbox bounds");

        assert!((checkbox_bounds.size.width.0 - 16.0).abs() <= 0.5);
        assert!((checkbox_bounds.size.height.0 - 16.0).abs() <= 0.5);
    }

    #[test]
    fn command_gating_checkbox_is_disabled_by_window_command_enabled_service() {
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
            "command-gating-checkbox-enabled-service",
            |cx| {
                vec![
                    Checkbox::new(checked.clone())
                        .a11y_label("Checkbox")
                        .on_click(cmd.clone())
                        .test_id("disabled-checkbox")
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
            .find(|n| n.test_id.as_deref() == Some("disabled-checkbox"))
            .expect("expected a semantics node for the checkbox test_id");
        assert!(node.flags.disabled);
    }

    #[test]
    fn command_gating_checkbox_is_disabled_when_widget_action_is_unavailable() {
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
            "command-gating-checkbox-action-availability",
            |cx| {
                vec![
                    Checkbox::new(checked.clone())
                        .a11y_label("Checkbox")
                        .on_click(cmd.clone())
                        .test_id("disabled-checkbox")
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
            .find(|n| n.test_id.as_deref() == Some("disabled-checkbox"))
            .expect("expected a semantics node for the checkbox test_id");
        assert!(node.flags.disabled);
    }

    #[test]
    fn command_gating_checkbox_prefers_window_command_gating_snapshot_when_present() {
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
            "command-gating-checkbox-gating-snapshot",
            |cx| {
                vec![
                    Checkbox::new(checked.clone())
                        .a11y_label("Checkbox")
                        .on_click(cmd.clone())
                        .test_id("disabled-checkbox")
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
            .find(|n| n.test_id.as_deref() == Some("disabled-checkbox"))
            .expect("expected a semantics node for the checkbox test_id");
        assert!(node.flags.disabled);
    }

    fn find_pressable_by_test_id<'a>(
        el: &'a AnyElement,
        test_id: &str,
    ) -> Option<&'a PressableProps> {
        match &el.kind {
            ElementKind::Pressable(props) => {
                if props.a11y.test_id.as_deref() == Some(test_id) {
                    return Some(props);
                }
            }
            _ => {}
        }

        for child in &el.children {
            if let Some(found) = find_pressable_by_test_id(child, test_id) {
                return Some(found);
            }
        }

        None
    }

    #[test]
    fn checkbox_focus_ring_tweens_in_and_out_like_a_transition() {
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
            CoreSize::new(Px(200.0), Px(120.0)),
        );
        let mut services = FakeServices;

        let model = app.models_mut().insert(false);

        let ring_alpha_out: Rc<Cell<Option<f32>>> = Rc::new(Cell::new(None));
        let always_paint_out: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            model: Model<bool>,
            ring_alpha_out: Rc<Cell<Option<f32>>>,
            always_paint_out: Rc<Cell<Option<bool>>>,
        ) -> fret_core::NodeId {
            let next_frame = FrameId(app.frame_id().0.saturating_add(1));
            app.set_frame_id(next_frame);

            let model_for_render = model.clone();
            let ring_alpha_out_for_render = ring_alpha_out.clone();
            let always_paint_out_for_render = always_paint_out.clone();
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "shadcn-checkbox-focus-ring-transition",
                move |cx| {
                    let el = Checkbox::new(model_for_render.clone())
                        .test_id("checkbox")
                        .into_element(cx);
                    let pressable =
                        find_pressable_by_test_id(&el, "checkbox").expect("checkbox pressable");
                    let ring = pressable.focus_ring.expect("focus ring");
                    ring_alpha_out_for_render.set(Some(ring.color.a));
                    always_paint_out_for_render.set(Some(pressable.focus_ring_always_paint));
                    vec![el]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
            root
        }

        app.set_frame_id(FrameId(0));
        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );

        let a0 = ring_alpha_out.get().expect("a0");
        assert!(a0.abs() <= 1e-6, "expected ring alpha=0, got {a0}");

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable checkbox");
        ui.set_focus(Some(focusable));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Tab,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a1 = ring_alpha_out.get().expect("a1");
        assert!(a1 > 0.0, "expected ring alpha to animate in, got {a1}");

        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for _ in 0..settle {
            let _ = render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
        }

        let a_focused = ring_alpha_out.get().expect("a_focused");
        assert!(
            a_focused > a1 + 1e-4,
            "expected ring alpha to increase over time, got a1={a1} a_focused={a_focused}"
        );

        ui.set_focus(None);
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a_blur = ring_alpha_out.get().expect("a_blur");
        let always_paint = always_paint_out.get().expect("always_paint");
        assert!(
            a_blur > 0.0 && a_blur < a_focused,
            "expected ring alpha to be intermediate after blur, got a_blur={a_blur} a_focused={a_focused}"
        );
        assert!(always_paint, "expected always_paint while animating out");

        for _ in 0..settle {
            let _ = render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
        }

        let a_final = ring_alpha_out.get().expect("a_final");
        let always_paint_final = always_paint_out.get().expect("always_paint_final");
        assert!(
            a_final.abs() <= 1e-4,
            "expected ring alpha=0, got {a_final}"
        );
        assert!(
            !always_paint_final,
            "expected always_paint to stop after settling"
        );
    }
}
