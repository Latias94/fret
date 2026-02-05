use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, PositionStyle, PressableProps,
    SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::controllable_state;
use fret_ui_kit::primitives::switch::{
    switch_a11y, switch_checked_from_optional_bool, switch_use_checked_model, toggle_optional_bool,
};
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, OverrideSlot, Radius, WidgetState,
    WidgetStateProperty, WidgetStates, resolve_override_slot,
};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn switch_track_w(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.switch.track_w")
        .unwrap_or(Px(32.0))
}

fn switch_track_h(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.switch.track_h")
        .unwrap_or(Px(18.4))
}

fn switch_thumb(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.switch.thumb")
        .unwrap_or(Px(16.0))
}

fn switch_padding(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.switch.thumb_pad")
        // shadcn-web positions the thumb flush to the content edge and relies on the track border
        // (1px) for the visible inset. Fret's border is paint-only, so we treat this as an extra
        // inset on top of the border/padding compensation in the layout below.
        .unwrap_or(Px(0.0))
}

fn switch_bg_on(theme: &Theme) -> Color {
    theme
        .color_by_key("primary")
        .unwrap_or_else(|| theme.color_required("primary"))
}

fn switch_bg_off(theme: &Theme) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or_else(|| theme.color_required("input"))
}

fn switch_thumb_bg(theme: &Theme) -> Color {
    theme
        .color_by_key("background")
        .unwrap_or_else(|| theme.color_required("background"))
}

fn switch_ring_color(theme: &Theme) -> Color {
    theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_required("ring"))
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
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_click: Option<CommandId>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: SwitchStyle,
}

#[derive(Clone)]
enum SwitchModel {
    Determinate(Model<bool>),
    Optional(Model<Option<bool>>),
}

impl Switch {
    pub fn new(model: Model<bool>) -> Self {
        Self {
            model: SwitchModel::Determinate(model),
            disabled: false,
            a11y_label: None,
            test_id: None,
            on_click: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: SwitchStyle::default(),
        }
    }

    /// Creates a switch bound to an optional boolean model.
    ///
    /// When the value is `None`, the switch renders as unchecked (matching shadcn/ui's common
    /// `value || false` usage). Clicking will set the model to `Some(true/false)` thereafter.
    pub fn new_opt(model: Model<Option<bool>>) -> Self {
        Self {
            model: SwitchModel::Optional(model),
            disabled: false,
            a11y_label: None,
            test_id: None,
            on_click: None,
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

    pub fn style(mut self, style: SwitchStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let model = self.model;

            let theme = Theme::global(&*cx.app).clone();

            let w = switch_track_w(&theme);
            let h = switch_track_h(&theme);
            let thumb = switch_thumb(&theme);
            let pad_x = switch_padding(&theme);

            let radius = Px((h.0 * 0.5).max(0.0));
            let ring_border = switch_ring_color(&theme);
            let mut ring = decl_style::focus_ring(&theme, radius);
            ring.color = alpha_mul(ring_border, 0.5);

            let bg_off = switch_bg_off(&theme);
            let bg_on = switch_bg_on(&theme);
            let thumb_bg = switch_thumb_bg(&theme);

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

            let default_thumb_background = WidgetStateProperty::new(ColorRef::Color(thumb_bg));

            let default_border_color =
                WidgetStateProperty::new(ColorRef::Color(Color::TRANSPARENT))
                    .when(WidgetStates::FOCUS_VISIBLE, ColorRef::Color(ring_border));

            let layout = LayoutRefinement::default()
                .w_px(w)
                .h_px(h)
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

            let pressable = control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
                cx.pressable_dispatch_command_if_enabled_opt(on_click);
                match &model {
                    SwitchModel::Determinate(model) => cx.pressable_toggle_bool(model),
                    SwitchModel::Optional(model) => {
                        cx.pressable_update_model(model, |v| {
                            *v = toggle_optional_bool(*v);
                        });
                    }
                }

                let theme = Theme::global(&*cx.app).clone();
                let on = match &model {
                    SwitchModel::Determinate(model) => {
                        cx.watch_model(model).copied().unwrap_or(false)
                    }
                    SwitchModel::Optional(model) => {
                        switch_checked_from_optional_bool(cx.watch_model(model).copied().flatten())
                    }
                };

                let mut states = WidgetStates::from_pressable(cx, st, !disabled);
                states.set(WidgetState::Selected, on);

                let bg = resolve_override_slot(
                    style_override.track_background.as_ref(),
                    &default_track_background,
                    states,
                )
                .resolve(&theme);
                let border_color = resolve_override_slot(
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
                let chrome_inset_y = Px(chrome_props.border.top.0.max(0.0)
                    + chrome_props.border.bottom.0.max(0.0)
                    + chrome_props.padding.top.0.max(0.0)
                    + chrome_props.padding.bottom.0.max(0.0));

                let mut a11y = switch_a11y(a11y_label.clone(), on);
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
                    // Align with shadcn-web:
                    // - Outer track size is border-box (`h-[1.15rem] w-8 border ...`).
                    // - Thumb is laid out at the content edge, so its outer offset equals the
                    //   track border (1px) plus any explicit padding.
                    let chrome_inset_x = Px(chrome_props.border.left.0.max(0.0)
                        + chrome_props.border.right.0.max(0.0)
                        + chrome_props.padding.left.0.max(0.0)
                        + chrome_props.padding.right.0.max(0.0));

                    let inner_w = Px((w.0 - chrome_inset_x.0).max(0.0));
                    let inner_h = Px((h.0 - chrome_inset_y.0).max(0.0));

                    let y = Px(((inner_h.0 - thumb.0) * 0.5).max(0.0));

                    // Additional inset beyond the border/padding insets (e.g. shadcn `p-[2px]`-like
                    // outcomes). This is relative to the inner content area.
                    let extra_x = Px(pad_x.0.max(0.0));

                    let x = if on {
                        Px((inner_w.0 - extra_x.0 - thumb.0).max(extra_x.0))
                    } else {
                        extra_x
                    };

                    let thumb_layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            top: Some(y),
                            left: Some(x),
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
                        padding: Edges::all(Px(0.0)),
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

pub fn switch<H: UiHost>(cx: &mut ElementContext<'_, H>, model: Model<bool>) -> AnyElement {
    Switch::new(model).into_element(cx)
}

pub fn switch_opt<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<bool>>,
) -> AnyElement {
    Switch::new_opt(model).into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, MouseButton, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
        PathStyle, Point, Px, Rect, Scene, Size as CoreSize, SvgId, SvgService, TextBlobId,
        TextConstraints, TextMetrics, TextService,
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

        let theme = Theme::global(&app).clone();
        let track_bg = switch_bg_off(&theme);
        let thumb_size = switch_thumb(&theme);
        let thumb_bg = switch_thumb_bg(&theme);

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
                && *background == thumb_bg;
            if is_thumb {
                thumb_rect = Some(*rect);
            }

            if *background == track_bg {
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
}
