use std::sync::Arc;

use fret_core::{Color, Edges, FontWeight, Px, TextStyle};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign, PressableProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
pub use fret_ui_kit::primitives::toggle::ToggleRoot;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverrideSlot, Radius,
    Size as ComponentSize, Space, WidgetState, WidgetStateProperty, WidgetStates,
    resolve_override_slot, resolve_override_slot_opt, ui,
};

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

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn toggle_bg_hover(theme: &Theme) -> Color {
    theme.color_required("muted")
}

fn toggle_fg_muted(theme: &Theme) -> Color {
    theme.color_required("muted-foreground")
}

fn toggle_ring_color(theme: &Theme) -> Color {
    theme.color_required("ring")
}

fn toggle_bg_on(theme: &Theme) -> Color {
    theme.color_required("accent")
}

fn toggle_fg_on(theme: &Theme) -> Color {
    theme.color_required("accent-foreground")
}

fn toggle_border(theme: &Theme) -> Color {
    theme.color_required("input")
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
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.toggle.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));
    TextStyle {
        size: px,
        weight: FontWeight::MEDIUM,
        line_height: Some(line_height),
        ..Default::default()
    }
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

#[derive(Clone)]
pub struct Toggle {
    model: Option<Model<bool>>,
    default_pressed: bool,
    label: Option<Arc<str>>,
    children: Vec<AnyElement>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    on_click: Option<CommandId>,
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
            default_pressed: false,
            label: None,
            children: Vec::new(),
            disabled: false,
            a11y_label: None,
            on_click: None,
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
            default_pressed,
            label: None,
            children: Vec::new(),
            disabled: false,
            a11y_label: None,
            on_click: None,
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

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
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
        let model =
            fret_ui_kit::primitives::toggle::toggle_use_model(cx, self.model.clone(), || {
                self.default_pressed
            })
            .model();
        let label = self.label;
        let children = self.children;
        let disabled_explicit = self.disabled;
        let a11y_label = self.a11y_label.clone();
        let on_click = self.on_click;
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
                LayoutRefinement::default().min_h(h).min_w(h).merge(layout),
            );

            let fg_default = theme.color_required("foreground");
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

        control_chrome_pressable_with_id_props(cx, move |cx, state, _id| {
            cx.pressable_dispatch_command_if_enabled_opt(on_click);
            cx.pressable_toggle_bool(&model);

            let on = cx.watch_model(&model).copied().unwrap_or(false);
            let mut states = WidgetStates::from_pressable(cx, state, !disabled);
            states.set(WidgetState::Selected, on);

            let fg = resolve_override_slot(
                style_override.foreground.as_ref(),
                &default_foreground,
                states,
            );
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

            let theme = Theme::global(&*cx.app);
            let mut chrome_props = decl_style::container_props(
                theme,
                base_chrome.clone(),
                LayoutRefinement::default(),
            );
            chrome_props.padding = Edges {
                top: pad_y,
                right: pad_x,
                bottom: pad_y,
                left: pad_x,
            };
            if matches!(variant, ToggleVariant::Outline) {
                chrome_props.shadow = Some(decl_style::shadow_xs(theme, radius));
            }
            if !user_bg_override {
                if let Some(bg) = bg {
                    chrome_props.background = Some(bg.resolve(theme));
                }
            }
            if let Some(border_color) = border_color {
                chrome_props.border_color = Some(border_color.resolve(theme));
            }
            chrome_props.layout.size = pressable_layout.size;

            let pressable_props = PressableProps {
                layout: pressable_layout,
                enabled: !disabled,
                focusable: true,
                focus_ring: Some(ring),
                a11y: fret_ui_kit::primitives::toggle::toggle_a11y(a11y_label, on),
                ..Default::default()
            };

            let content_children = move |cx: &mut ElementContext<'_, H>| {
                vec![cx.flex(
                    FlexProps {
                        direction: fret_core::Axis::Horizontal,
                        gap: {
                            let theme = Theme::global(&*cx.app);
                            MetricRef::space(Space::N2).resolve(theme)
                        },
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Center,
                        align: CrossAlign::Center,
                        wrap: false,
                        ..Default::default()
                    },
                    move |cx: &mut ElementContext<'_, H>| {
                        let mut out = Vec::new();
                        out.extend(children);
                        if let Some(label) = label {
                            let mut text = ui::label(cx, label)
                                .text_size_px(text_style.size)
                                .font_weight(text_style.weight)
                                .text_color(fg.clone())
                                .nowrap();
                            if let Some(line_height) = text_style.line_height {
                                text = text.line_height_px(line_height);
                            }
                            if let Some(letter_spacing_em) = text_style.letter_spacing_em {
                                text = text.letter_spacing_em(letter_spacing_em);
                            }
                            out.push(text.into_element(cx));
                        }
                        out
                    },
                )]
            };

            (pressable_props, chrome_props, content_children)
        })
    }
}

pub fn toggle<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    model: Model<bool>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    Toggle::new(model).children(f(cx)).into_element(cx)
}

pub fn toggle_uncontrolled<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    default_pressed: bool,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    Toggle::uncontrolled(default_pressed)
        .children(f(cx))
        .into_element(cx)
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
        CommandMeta, CommandScope, FrameId, TickId, WindowCommandActionAvailabilityService,
        WindowCommandEnabledService, WindowCommandGatingService, WindowCommandGatingSnapshot,
    };
    use fret_ui::UiTree;
    use std::collections::HashMap;

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
                    size: Size::new(Px(0.0), Px(0.0)),
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
    fn toggle_uncontrolled_applies_default_pressed_once_and_does_not_reset() {
        fn is_selected(ui: &UiTree<App>, label: &str) -> bool {
            ui.semantics_snapshot()
                .expect("semantics snapshot")
                .nodes
                .iter()
                .find(|n| n.label.as_deref() == Some(label))
                .is_some_and(|n| n.flags.selected)
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
        assert!(is_selected(&ui, "Toggle"));

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
        assert!(!is_selected(&ui, "Toggle"));

        // The internal model should not be reset by repeatedly passing the same default value.
        let _ = render_uncontrolled_frame(&mut ui, &mut app, &mut services, window, bounds, true);
        assert!(!is_selected(&ui, "Toggle"));
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
}
