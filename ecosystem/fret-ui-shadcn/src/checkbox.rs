use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_icons::ids;
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, PressableProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::checkbox::{
    CheckedState, checkbox_a11y, checkbox_use_checked_model, checked_state_from_optional_bool,
    toggle_optional_bool,
};
use fret_ui_kit::primitives::controllable_state;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverrideSlot, Radius, WidgetState,
    WidgetStateProperty, WidgetStates, resolve_override_slot, resolve_override_slot_opt,
};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn checkbox_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.checkbox.size")
        .unwrap_or(Px(16.0))
}

fn checkbox_radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.checkbox.radius")
        .unwrap_or_else(|| MetricRef::radius(Radius::Sm).resolve(theme))
}

fn checkbox_border(theme: &Theme) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .expect("missing theme token: input/border")
}

fn checkbox_bg_checked(theme: &Theme) -> Color {
    theme.color_required("primary")
}

fn checkbox_fg_checked(theme: &Theme) -> Color {
    theme.color_required("primary-foreground")
}

fn checkbox_ring_color(theme: &Theme) -> Color {
    theme.color_required("ring")
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
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_click: Option<CommandId>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: CheckboxStyle,
}

#[derive(Debug, Clone)]
enum CheckboxCheckedModel {
    Bool(Model<bool>),
    OptionalBool(Model<Option<bool>>),
    TriState(Model<CheckedState>),
}

impl Checkbox {
    pub fn new(model: Model<bool>) -> Self {
        Self {
            checked: CheckboxCheckedModel::Bool(model),
            disabled: false,
            a11y_label: None,
            test_id: None,
            on_click: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: CheckboxStyle::default(),
        }
    }

    pub fn new_tristate(model: Model<CheckedState>) -> Self {
        Self {
            checked: CheckboxCheckedModel::TriState(model),
            disabled: false,
            a11y_label: None,
            test_id: None,
            on_click: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: CheckboxStyle::default(),
        }
    }

    /// Creates a checkbox bound to an optional boolean model.
    ///
    /// This maps `None` to the indeterminate state, matching Radix's `"indeterminate"` outcome.
    pub fn new_optional(model: Model<Option<bool>>) -> Self {
        Self {
            checked: CheckboxCheckedModel::OptionalBool(model),
            disabled: false,
            a11y_label: None,
            test_id: None,
            on_click: None,
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

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
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

            let theme = Theme::global(&*cx.app).clone();

            let size = checkbox_size(&theme);
            let radius = checkbox_radius(&theme);
            let border = checkbox_border(&theme);
            let bg_on = checkbox_bg_checked(&theme);
            let fg_on = checkbox_fg_checked(&theme);

            let ring_border = checkbox_ring_color(&theme);
            let mut ring = decl_style::focus_ring(&theme, radius);
            ring.color = alpha_mul(ring_border, 0.5);

            let default_background = WidgetStateProperty::new(None)
                .when(WidgetStates::SELECTED, Some(ColorRef::Color(bg_on)));
            let default_border_color = WidgetStateProperty::new(ColorRef::Color(border))
                .when(WidgetStates::SELECTED, ColorRef::Color(bg_on))
                .when(WidgetStates::FOCUS_VISIBLE, ColorRef::Color(ring_border));
            let default_foreground = WidgetStateProperty::new(ColorRef::Color(Color::TRANSPARENT))
                .when(WidgetStates::SELECTED, ColorRef::Color(fg_on));

            let layout = LayoutRefinement::default()
                .w_px(size)
                .h_px(size)
                .merge(self.layout);
            let pressable_layout = decl_style::layout_style(&theme, layout);

            let a11y_label = self.a11y_label.clone();
            let test_id = self.test_id.clone();
            let disabled_explicit = self.disabled;
            let on_click = self.on_click.clone();
            let disabled = disabled_explicit
                || on_click
                    .as_ref()
                    .is_some_and(|cmd| !cx.command_is_enabled(cmd));
            let chrome = self.chrome.clone();
            let style_override = self.style.clone();

            let pressable = control_chrome_pressable_with_id_props(cx, move |cx, st, id| {
                cx.key_add_on_key_down_for(
                    id,
                    fret_ui_kit::primitives::keyboard::consume_enter_key_handler(),
                );
                cx.pressable_dispatch_command_if_enabled_opt(on_click);
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
                }

                let theme = Theme::global(&*cx.app).clone();
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
                };
                let is_on = state.is_on();
                let is_checked = state.is_checked();
                let is_indeterminate = state.is_indeterminate();

                let mut states = WidgetStates::from_pressable(cx, st, !disabled);
                states.set(WidgetState::Selected, is_on);

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
                chrome_props.padding = Edges::all(Px(0.0));
                chrome_props.shadow = Some(decl_style::shadow_xs(&theme, radius));
                chrome_props.layout.size = pressable_layout.size;

                let mut a11y = checkbox_a11y(a11y_label.clone(), state);
                a11y.test_id = test_id.clone();
                let pressable_props = PressableProps {
                    layout: pressable_layout,
                    enabled: !disabled,
                    focusable: true,
                    focus_ring: Some(ring),
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
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
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

pub fn checkbox<H: UiHost>(cx: &mut ElementContext<'_, H>, model: Model<bool>) -> AnyElement {
    Checkbox::new(model).into_element(cx)
}

pub fn checkbox_opt<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<bool>>,
) -> AnyElement {
    Checkbox::new_optional(model).into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, MouseButton, PathCommand, PathConstraints, PathId, PathMetrics,
        PathService, PathStyle, Point, Px, Rect, Scene, Size as CoreSize, SvgId, SvgService,
        TextBlobId, TextConstraints, TextMetrics, TextService,
    };
    use fret_runtime::{
        CommandMeta, CommandScope, WindowCommandActionAvailabilityService,
        WindowCommandEnabledService, WindowCommandGatingService, WindowCommandGatingSnapshot,
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
}
