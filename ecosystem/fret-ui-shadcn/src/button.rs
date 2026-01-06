use std::sync::Arc;

use fret_core::{Color, FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_runtime::CommandId;
use fret_ui::element::{AnyElement, LayoutStyle, PressableA11y, PressableProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Size as ComponentSize, Space,
};

fn contains_svg_icon_like(el: &AnyElement) -> bool {
    match &el.kind {
        fret_ui::element::ElementKind::SvgIcon(_) | fret_ui::element::ElementKind::Spinner(_) => {
            return true;
        }
        _ => {}
    }

    el.children.iter().any(contains_svg_icon_like)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
    Icon,
    IconSm,
    IconLg,
}

impl ButtonSize {
    fn component_size(self) -> ComponentSize {
        match self {
            Self::Default => ComponentSize::Medium,
            Self::Sm => ComponentSize::Small,
            Self::Lg => ComponentSize::Large,
            Self::Icon => ComponentSize::Medium,
            Self::IconSm => ComponentSize::Small,
            Self::IconLg => ComponentSize::Large,
        }
    }
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn variant_colors(theme: &Theme, variant: ButtonVariant) -> (Color, Color, Color, Color, Color) {
    let transparent = Color::TRANSPARENT;

    let bg_primary = theme.color_by_key("primary").unwrap_or(theme.colors.accent);
    let fg_primary = theme
        .color_by_key("primary-foreground")
        .or_else(|| theme.color_by_key("primary.foreground"))
        .unwrap_or(theme.colors.text_primary);

    let bg_secondary = theme
        .color_by_key("secondary")
        .unwrap_or(theme.colors.panel_background);
    let fg_secondary = theme
        .color_by_key("secondary-foreground")
        .or_else(|| theme.color_by_key("secondary.foreground"))
        .unwrap_or(theme.colors.text_primary);

    let bg_destructive = theme
        .color_by_key("destructive")
        .unwrap_or(theme.colors.selection_background);
    let fg_destructive = theme
        .color_by_key("destructive-foreground")
        .or_else(|| theme.color_by_key("destructive.foreground"))
        .unwrap_or(theme.colors.text_primary);

    let fg_default = theme
        .color_by_key("foreground")
        .unwrap_or(theme.colors.text_primary);

    let bg_accent = theme
        .color_by_key("accent")
        .or_else(|| theme.color_by_key("accent.background"))
        .unwrap_or(theme.colors.hover_background);

    let bg_background = theme
        .color_by_key("background")
        .unwrap_or(theme.colors.surface_background);

    let border = theme
        .color_by_key("border")
        .unwrap_or(theme.colors.panel_border);

    match variant {
        ButtonVariant::Default => (
            bg_primary,
            alpha_mul(bg_primary, 0.9),
            alpha_mul(bg_primary, 0.8),
            transparent,
            fg_primary,
        ),
        ButtonVariant::Destructive => (
            bg_destructive,
            alpha_mul(bg_destructive, 0.9),
            alpha_mul(bg_destructive, 0.8),
            transparent,
            fg_destructive,
        ),
        ButtonVariant::Secondary => (
            bg_secondary,
            alpha_mul(bg_secondary, 0.8),
            alpha_mul(bg_secondary, 0.7),
            transparent,
            fg_secondary,
        ),
        ButtonVariant::Outline => (
            bg_background,
            bg_accent,
            alpha_mul(bg_accent, 0.8),
            border,
            fg_default,
        ),
        ButtonVariant::Ghost => (
            transparent,
            bg_accent,
            alpha_mul(bg_accent, 0.8),
            transparent,
            fg_default,
        ),
        ButtonVariant::Link => (
            transparent,
            transparent,
            transparent,
            transparent,
            bg_primary,
        ),
    }
}

fn button_text_style(theme: &Theme, size: ButtonSize) -> TextStyle {
    let px = size.component_size().control_text_px(theme);
    let line_height = theme
        .metric_by_key("font.line_height")
        .unwrap_or(theme.metrics.font_line_height);

    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::MEDIUM,
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

#[derive(Clone)]
pub struct Button {
    label: Arc<str>,
    children: Vec<AnyElement>,
    command: Option<CommandId>,
    toggle_model: Option<fret_runtime::Model<bool>>,
    disabled: bool,
    variant: ButtonVariant,
    size: ButtonSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("label", &self.label)
            .field("children_len", &self.children.len())
            .field("command", &self.command)
            .field("toggle_model", &self.toggle_model.is_some())
            .field("disabled", &self.disabled)
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Button {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            label,
            children: Vec::new(),
            command: None,
            toggle_model: None,
            disabled: false,
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            chrome: ChromeRefinement::default(),
            layout: fret_ui_kit::LayoutRefinement::default(),
        }
    }

    pub fn children(mut self, children: Vec<AnyElement>) -> Self {
        self.children = children;
        self
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn toggle_model(mut self, model: fret_runtime::Model<bool>) -> Self {
        self.toggle_model = Some(model);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: fret_ui_kit::LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let (bg, bg_hover, bg_active, border_color, fg) = variant_colors(&theme, self.variant);
            let shadow_radius = self.size.component_size().control_radius(&theme);
            let shadow = (self.variant == ButtonVariant::Outline)
                .then(|| decl_style::shadow_xs(&theme, shadow_radius));

            let size = self.size.component_size();
            let radius = size.control_radius(&theme);
            let border_w = if self.variant == ButtonVariant::Outline {
                Px(1.0)
            } else {
                Px(0.0)
            };

            let mut base_layout = self.layout;
            let is_icon_button = matches!(
                self.size,
                ButtonSize::Icon | ButtonSize::IconSm | ButtonSize::IconLg
            );
            if is_icon_button {
                let icon = size.icon_button_size(&theme);
                base_layout = base_layout
                    .min_w(MetricRef::Px(icon))
                    .min_h(MetricRef::Px(icon));
            } else {
                let min_h = size.button_h(&theme);
                base_layout = base_layout.min_h(MetricRef::Px(min_h));
            }

            let pressable_layout = decl_style::layout_style(&theme, base_layout);

            let command = self.command;
            let toggle_model = self.toggle_model;
            let a11y_label = self.label.clone();
            let disabled = self.disabled;
            let user_chrome = self.chrome;
            let user_bg_override = user_chrome.background.is_some();
            let user_border_override = user_chrome.border_color.is_some();
            let variant = self.variant;
            let text_style = button_text_style(&theme, self.size);
            let is_icon = is_icon_button;
            let has_svg_icon_like_children =
                !is_icon_button && self.children.iter().any(contains_svg_icon_like);
            let children = self.children;

            let pressable = control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
                cx.pressable_dispatch_command_opt(command);
                if let Some(model) = toggle_model {
                    cx.pressable_toggle_bool(&model);
                }

                let hovered = st.hovered && !disabled;
                let pressed = st.pressed && !disabled;
                let focused = st.focused && !disabled;

                let (bg, mut border_color, fg) = if pressed {
                    (bg_active, border_color, fg)
                } else if hovered {
                    (bg_hover, border_color, fg)
                } else {
                    (bg, border_color, fg)
                };

                if focused && variant == ButtonVariant::Outline && !user_border_override {
                    border_color = theme
                        .color_by_key("ring")
                        .unwrap_or(theme.colors.focus_ring);
                }

                let padding = if variant == ButtonVariant::Link || is_icon {
                    ChromeRefinement::default()
                } else {
                    // shadcn/ui: `has-[>svg]:px-*` uses tighter horizontal padding when an icon is present.
                    let shrink_px = has_svg_icon_like_children;
                    match size {
                        ComponentSize::Small => ChromeRefinement::default()
                            .px(if shrink_px { Space::N2p5 } else { Space::N3 })
                            .py(Space::N1),
                        ComponentSize::Medium => ChromeRefinement::default()
                            .px(if shrink_px { Space::N3 } else { Space::N4 })
                            .py(Space::N2),
                        ComponentSize::Large => ChromeRefinement::default()
                            .px(if shrink_px { Space::N4 } else { Space::N6 })
                            .py(Space::N2),
                        ComponentSize::XSmall => {
                            ChromeRefinement::default().px(Space::N2).py(Space::N1)
                        }
                    }
                };

                let mut chrome = padding.merge(ChromeRefinement {
                    radius: Some(MetricRef::Px(radius)),
                    border_width: Some(MetricRef::Px(border_w)),
                    ..Default::default()
                });

                if !user_bg_override {
                    chrome.background = Some(ColorRef::Color(bg));
                }
                if !user_border_override {
                    chrome.border_color = Some(ColorRef::Color(border_color));
                }
                chrome = chrome.merge(user_chrome.clone());

                let mut chrome_props =
                    decl_style::container_props(&theme, chrome, LayoutRefinement::default());
                chrome_props.shadow = shadow;
                chrome_props.layout.size = pressable_layout.size;

                let pressable_props = PressableProps {
                    layout: pressable_layout,
                    enabled: !disabled,
                    focusable: true,
                    focus_ring: Some(decl_style::focus_ring(&theme, radius)),
                    a11y: PressableA11y {
                        label: Some(a11y_label.clone()),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let content_children = move |cx: &mut ElementContext<'_, H>| {
                    let gap = if is_icon {
                        Space::N0
                    } else {
                        match size {
                            ComponentSize::Small | ComponentSize::XSmall => Space::N1p5,
                            ComponentSize::Medium | ComponentSize::Large => Space::N2,
                        }
                    };

                    let content = if children.is_empty() {
                        vec![cx.text_props(TextProps {
                            layout: LayoutStyle::default(),
                            text: a11y_label.clone(),
                            style: Some(text_style),
                            color: Some(fg),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        })]
                    } else {
                        children.clone()
                    };

                    vec![fret_ui_kit::declarative::stack::hstack(
                        cx,
                        fret_ui_kit::declarative::stack::HStackProps::default()
                            .justify_center()
                            .items_center()
                            .gap_x(gap),
                        |_cx| content,
                    )]
                };

                (pressable_props, chrome_props, content_children)
            });

            if disabled {
                cx.opacity(0.5, |_cx| vec![pressable])
            } else {
                pressable
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, Px, Rect, Scene, SceneOp, Size as CoreSize, SvgId, SvgService, TextBlobId,
        TextConstraints, TextMetrics, TextService, TextStyle as CoreTextStyle,
    };
    use fret_ui::SvgSource;
    use fret_ui::Theme;
    use fret_ui::element::{ContainerProps, ElementKind, LayoutStyle, Length, SizeStyle};
    use fret_ui::elements;
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
    fn disabled_button_emits_opacity_stack_ops() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(200.0), Px(80.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "disabled-button-emits-opacity-stack-ops",
            |cx| vec![Button::new("Hello").disabled(true).into_element(cx)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        assert!(matches!(
            scene.ops().first(),
            Some(SceneOp::PushOpacity { opacity }) if (*opacity - 0.5).abs() < 1e-6
        ));
        assert!(matches!(scene.ops().last(), Some(SceneOp::PopOpacity)));
    }

    #[test]
    fn icon_button_sizes_apply_min_dimensions() {
        let mut app = App::new();
        let window = AppWindowId::default();

        let theme = Theme::global(&app).clone();
        let expected_sm = fret_ui_kit::Size::Small.icon_button_size(&theme);
        let expected_md = fret_ui_kit::Size::Medium.icon_button_size(&theme);
        let expected_lg = fret_ui_kit::Size::Large.icon_button_size(&theme);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(400.0), Px(200.0)),
        );

        let icon_stub = |cx: &mut fret_ui::ElementContext<'_, App>| {
            cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(Px(1.0)),
                            height: Length::Px(Px(1.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )
        };

        for (size, expected) in [
            (ButtonSize::IconSm, expected_sm),
            (ButtonSize::Icon, expected_md),
            (ButtonSize::IconLg, expected_lg),
        ] {
            let element =
                elements::with_element_cx(&mut app, window, bounds, "icon-button-size", |cx| {
                    Button::new("Icon button")
                        .size(size)
                        .children(vec![icon_stub(cx)])
                        .into_element(cx)
                });

            let ElementKind::Pressable(props) = &element.kind else {
                panic!("expected icon button to render as a Pressable");
            };

            assert_eq!(props.layout.size.min_width, Some(expected));
            assert_eq!(props.layout.size.min_height, Some(expected));
        }
    }

    #[test]
    fn button_padding_x_compacts_when_icon_present() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let theme = Theme::global(&app).clone();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(400.0), Px(200.0)),
        );

        let icon_bytes: &'static [u8] =
            br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"></svg>"#;

        for (size, expected_px) in [
            (
                ButtonSize::Sm,
                fret_ui_kit::MetricRef::space(fret_ui_kit::Space::N2p5).resolve(&theme),
            ),
            (
                ButtonSize::Default,
                fret_ui_kit::MetricRef::space(fret_ui_kit::Space::N3).resolve(&theme),
            ),
            (
                ButtonSize::Lg,
                fret_ui_kit::MetricRef::space(fret_ui_kit::Space::N4).resolve(&theme),
            ),
        ] {
            let element =
                elements::with_element_cx(&mut app, window, bounds, "button-padding", |cx| {
                    let icon = cx.svg_icon_props(fret_ui::element::SvgIconProps::new(
                        SvgSource::Static(icon_bytes),
                    ));
                    let text = cx.text("Label");

                    Button::new("Icon label")
                        .size(size)
                        .children(vec![icon, text])
                        .into_element(cx)
                });

            let ElementKind::Pressable(_props) = &element.kind else {
                panic!("expected button to render as a Pressable");
            };
            let Some(chrome) = element.children.first() else {
                panic!("expected button pressable to contain chrome container");
            };
            let ElementKind::Container(props) = &chrome.kind else {
                panic!("expected chrome container");
            };

            assert_eq!(props.padding.left, expected_px);
            assert_eq!(props.padding.right, expected_px);
        }
    }

    #[test]
    fn outline_button_border_uses_ring_color_when_focused() {
        use std::cell::{Cell, RefCell};
        use std::rc::Rc;

        use fret_runtime::FrameId;
        use fret_ui::elements::GlobalElementId;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(400.0), Px(200.0)),
        );
        let mut services = FakeServices;

        let theme = Theme::global(&app).clone();
        let ring = theme
            .color_by_key("ring")
            .unwrap_or(theme.colors.focus_ring);

        let id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let rendered_out: Rc<RefCell<Option<AnyElement>>> = Rc::new(RefCell::new(None));

        // First frame: render once to obtain the element id and map to a node.
        app.set_frame_id(FrameId(1));
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "outline-button-focus-border",
            {
                let id_out = id_out.clone();
                |cx| {
                    let el = Button::new("Outline")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx);
                    id_out.set(Some(el.id));
                    vec![el]
                }
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let id = id_out.get().expect("button element id");
        let node =
            elements::node_for_element(&mut app, window, id).expect("button node id resolved");
        ui.set_focus(Some(node));

        // Second frame: re-render with focus applied and capture the element tree.
        app.set_frame_id(FrameId(2));
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "outline-button-focus-border",
            {
                let rendered_out = rendered_out.clone();
                |cx| {
                    let el = Button::new("Outline")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx);
                    rendered_out.borrow_mut().replace(el.clone());
                    vec![el]
                }
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let el = rendered_out
            .borrow_mut()
            .take()
            .expect("rendered element captured");
        let ElementKind::Pressable(_pressable) = &el.kind else {
            panic!("expected button root element to be Pressable");
        };

        let chrome = el
            .children
            .first()
            .expect("expected pressable to contain chrome container");
        let ElementKind::Container(props) = &chrome.kind else {
            panic!("expected chrome container element");
        };
        assert_eq!(props.border_color, Some(ring));
    }
}
