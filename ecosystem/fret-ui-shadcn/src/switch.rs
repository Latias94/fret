use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, PositionStyle, PressableProps,
    SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::controllable_state;
use fret_ui_kit::primitives::switch::{
    switch_a11y, switch_checked_from_optional_bool, switch_use_checked_model, toggle_optional_bool,
};
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius};

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
        .unwrap_or(Px(2.0))
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

#[derive(Clone)]
pub struct Switch {
    model: SwitchModel,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    on_click: Option<CommandId>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
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
            on_click: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
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
            on_click: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
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

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.on_click = Some(command.into());
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
            let model = self.model;

            let theme = Theme::global(&*cx.app).clone();

            let w = switch_track_w(&theme);
            let h = switch_track_h(&theme);
            let thumb = switch_thumb(&theme);
            let pad_x = switch_padding(&theme);

            let radius = Px((h.0 * 0.5).max(0.0));
            let mut ring = decl_style::focus_ring(&theme, radius);
            ring.color = alpha_mul(switch_ring_color(&theme), 0.5);

            let layout = LayoutRefinement::default()
                .w_px(MetricRef::Px(w))
                .h_px(MetricRef::Px(h))
                .merge(self.layout);
            let pressable_layout = decl_style::layout_style(&theme, layout);

            let a11y_label = self.a11y_label.clone();
            let disabled = self.disabled;
            let on_click = self.on_click.clone();
            let chrome = self.chrome.clone();

            let pressable = control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
                cx.pressable_dispatch_command_opt(on_click);
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

                let mut bg = if on {
                    switch_bg_on(&theme)
                } else {
                    switch_bg_off(&theme)
                };
                let hovered = st.hovered && !disabled;
                if hovered {
                    bg = alpha_mul(bg, if on { 0.9 } else { 0.7 });
                }

                let border_color = if st.focused {
                    switch_ring_color(&theme)
                } else {
                    Color::TRANSPARENT
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

                let pressable_props = PressableProps {
                    layout: pressable_layout,
                    enabled: !disabled,
                    focusable: true,
                    focus_ring: Some(ring),
                    a11y: switch_a11y(a11y_label.clone(), on),
                    ..Default::default()
                };

                let children = move |cx: &mut ElementContext<'_, H>| {
                    let pad_y = Px(((h.0 - thumb.0) * 0.5).max(0.0));
                    let x = if on {
                        Px((w.0 - pad_x.0 - thumb.0).max(0.0))
                    } else {
                        pad_x
                    };

                    let thumb_layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            top: Some(pad_y),
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

                    let thumb_bg = switch_thumb_bg(&theme);
                    let thumb_props = ContainerProps {
                        layout: thumb_layout,
                        padding: Edges::all(Px(0.0)),
                        background: Some(thumb_bg),
                        shadow: None,
                        border: Edges::all(Px(0.0)),
                        border_color: None,
                        corner_radii: Corners::all(Px((thumb.0 * 0.5).max(0.0))),
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
        TextConstraints, TextMetrics, TextService, TextStyle as CoreTextStyle,
    };
    use fret_ui::tree::UiTree;

    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &CoreTextStyle,
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
                position,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
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
                position,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
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
}
