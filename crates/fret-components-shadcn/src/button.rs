use std::sync::Arc;

use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Size as ComponentSize, Space,
};
use fret_core::{Color, FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_runtime::CommandId;
use fret_ui::element::{AnyElement, LayoutStyle, PressableA11y, PressableProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};

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
}

impl ButtonSize {
    fn component_size(self) -> ComponentSize {
        match self {
            Self::Default => ComponentSize::Medium,
            Self::Sm => ComponentSize::Small,
            Self::Lg => ComponentSize::Large,
            Self::Icon => ComponentSize::Medium,
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
            alpha_mul(bg_secondary, 0.9),
            alpha_mul(bg_secondary, 0.8),
            transparent,
            fg_secondary,
        ),
        ButtonVariant::Outline => (
            transparent,
            bg_accent,
            theme.colors.selection_background,
            border,
            fg_default,
        ),
        ButtonVariant::Ghost => (
            transparent,
            bg_accent,
            theme.colors.selection_background,
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
            let shadow = matches!(
                self.variant,
                ButtonVariant::Default | ButtonVariant::Secondary | ButtonVariant::Outline
            )
            .then(|| decl_style::shadow_sm(&theme, shadow_radius));

            let size = self.size.component_size();
            let radius = size.control_radius(&theme);
            let border_w = if self.variant == ButtonVariant::Outline {
                Px(1.0)
            } else {
                Px(0.0)
            };

            let mut base_layout = self.layout;
            if self.size == ButtonSize::Icon {
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
            let is_icon = self.size == ButtonSize::Icon;
            let children = self.children;

            let pressable = control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
                cx.pressable_dispatch_command_opt(command);
                if let Some(model) = toggle_model {
                    cx.pressable_toggle_bool(&model);
                }

                let hovered = st.hovered && !disabled;
                let pressed = st.pressed && !disabled;

                let (bg, border_color, fg) = if pressed {
                    (bg_active, border_color, fg)
                } else if hovered {
                    (bg_hover, border_color, fg)
                } else {
                    (bg, border_color, fg)
                };

                let padding = if variant == ButtonVariant::Link || is_icon {
                    ChromeRefinement::default()
                } else {
                    match size {
                        ComponentSize::Small => {
                            ChromeRefinement::default().px(Space::N3).py(Space::N1)
                        }
                        ComponentSize::Medium => {
                            ChromeRefinement::default().px(Space::N4).py(Space::N2)
                        }
                        ComponentSize::Large => {
                            ChromeRefinement::default().px(Space::N6).py(Space::N2)
                        }
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
                            .gap_x(Space::N2),
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
}
