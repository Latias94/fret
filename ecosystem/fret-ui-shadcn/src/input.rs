use std::sync::Arc;

use fret_core::{Corners, FontId, KeyCode, NodeId, Px, SemanticsRole};
use fret_runtime::{CommandId, Model};
use fret_ui::action::{ActionCx, KeyDownCx, UiFocusActionHost};
use fret_ui::element::{AnyElement, Length, Overflow, SizeStyle, TextInputProps};
use fret_ui::{ElementContext, TextInputStyle, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Size};

#[derive(Debug, Clone, Copy, Default)]
struct BorderWidthOverride {
    top: Option<Px>,
    right: Option<Px>,
    bottom: Option<Px>,
    left: Option<Px>,
}

#[derive(Debug, Clone, Default)]
pub struct InputStyle {
    pub background: Option<ColorRef>,
    pub border_color: Option<ColorRef>,
    pub border_color_focused: Option<ColorRef>,
    pub focus_ring_color: Option<ColorRef>,
}

impl InputStyle {
    pub fn background(mut self, background: ColorRef) -> Self {
        self.background = Some(background);
        self
    }

    pub fn border_color(mut self, border_color: ColorRef) -> Self {
        self.border_color = Some(border_color);
        self
    }

    pub fn border_color_focused(mut self, border_color_focused: ColorRef) -> Self {
        self.border_color_focused = Some(border_color_focused);
        self
    }

    pub fn focus_ring_color(mut self, focus_ring_color: ColorRef) -> Self {
        self.focus_ring_color = Some(focus_ring_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.background.is_some() {
            self.background = other.background;
        }
        if other.border_color.is_some() {
            self.border_color = other.border_color;
        }
        if other.border_color_focused.is_some() {
            self.border_color_focused = other.border_color_focused;
        }
        if other.focus_ring_color.is_some() {
            self.focus_ring_color = other.focus_ring_color;
        }
        self
    }
}

#[derive(Clone)]
pub struct Input {
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    a11y_role: Option<SemanticsRole>,
    test_id: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    aria_invalid: bool,
    aria_required: bool,
    disabled: bool,
    active_descendant: Option<NodeId>,
    expanded: Option<bool>,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    on_submit: Option<OnInputSubmit>,
    size: Size,
    style: InputStyle,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    border_width_override: BorderWidthOverride,
    corner_radii_override: Option<Corners>,
}

impl Input {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            a11y_label: None,
            a11y_role: None,
            test_id: None,
            placeholder: None,
            aria_invalid: false,
            aria_required: false,
            disabled: false,
            active_descendant: None,
            expanded: None,
            submit_command: None,
            cancel_command: None,
            on_submit: None,
            size: Size::default(),
            style: InputStyle::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            border_width_override: BorderWidthOverride::default(),
            corner_radii_override: None,
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn a11y_role(mut self, role: SemanticsRole) -> Self {
        self.a11y_role = Some(role);
        self
    }

    /// Sets a stable `test_id` on the underlying `TextInput` element (not the outer chrome wrapper).
    ///
    /// This is preferred over calling `.test_id(...)` on the returned `AnyElement`, because
    /// diagnostics scripts (`type_text_into`, focus assertions) need to target the focusable
    /// text input node directly.
    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Apply the upstream `aria-invalid` error state chrome (border + focus ring color).
    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
        self
    }

    pub fn aria_required(mut self, aria_required: bool) -> Self {
        self.aria_required = aria_required;
        self
    }

    /// Apply the upstream `disabled` interaction + chrome outcome.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn active_descendant(mut self, node: NodeId) -> Self {
        self.active_descendant = Some(node);
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    pub fn submit_command(mut self, command: CommandId) -> Self {
        self.submit_command = Some(command);
        self
    }

    pub fn cancel_command(mut self, command: CommandId) -> Self {
        self.cancel_command = Some(command);
        self
    }

    /// Registers a component-owned submit handler for Enter key presses.
    ///
    /// This is useful when the consumer wants to keep the effect localized (e.g. committing a
    /// derived draft model) without relying on an app-level `on_command` handler.
    pub fn on_submit(mut self, on_submit: OnInputSubmit) -> Self {
        self.on_submit = Some(on_submit);
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn style(mut self, style: InputStyle) -> Self {
        self.style = self.style.merged(style);
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

    /// Overrides per-edge border widths (in px) for this input's chrome.
    ///
    /// This is primarily used by shadcn recipe compositions that merge borders (e.g. input groups).
    pub fn border_left_width_override(mut self, border: Px) -> Self {
        self.border_width_override.left = Some(border);
        self
    }

    pub fn border_right_width_override(mut self, border: Px) -> Self {
        self.border_width_override.right = Some(border);
        self
    }

    pub fn border_top_width_override(mut self, border: Px) -> Self {
        self.border_width_override.top = Some(border);
        self
    }

    pub fn border_bottom_width_override(mut self, border: Px) -> Self {
        self.border_width_override.bottom = Some(border);
        self
    }

    /// Overrides per-corner radii (in px) for this input's chrome.
    ///
    /// This is primarily used by shadcn recipe compositions that merge corner radii
    /// (`rounded-l-none`, `rounded-r-none`).
    pub fn corner_radii_override(mut self, corners: Corners) -> Self {
        self.corner_radii_override = Some(corners);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        input_with_style_and_submit(
            cx,
            self.model,
            self.a11y_label,
            self.a11y_role,
            self.test_id,
            self.placeholder,
            self.aria_invalid,
            self.aria_required,
            self.disabled,
            self.active_descendant,
            self.expanded,
            self.submit_command,
            self.cancel_command,
            self.on_submit,
            self.size,
            self.style,
            self.chrome,
            self.layout,
            self.border_width_override,
            self.corner_radii_override,
        )
    }
}

pub fn input<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    a11y_role: Option<SemanticsRole>,
    placeholder: Option<Arc<str>>,
    active_descendant: Option<NodeId>,
    expanded: Option<bool>,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
) -> AnyElement {
    input_with_style_and_submit(
        cx,
        model,
        a11y_label,
        a11y_role,
        None,
        placeholder,
        false,
        false,
        false,
        active_descendant,
        expanded,
        submit_command,
        cancel_command,
        None,
        Size::default(),
        InputStyle::default(),
        ChromeRefinement::default(),
        LayoutRefinement::default(),
        BorderWidthOverride::default(),
        None,
    )
}

pub type OnInputSubmit = Arc<dyn Fn(&mut dyn UiFocusActionHost, ActionCx) + 'static>;

fn input_with_style_and_submit<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    a11y_role: Option<SemanticsRole>,
    test_id: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    aria_invalid: bool,
    aria_required: bool,
    disabled: bool,
    active_descendant: Option<NodeId>,
    expanded: Option<bool>,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    on_submit: Option<OnInputSubmit>,
    size: Size,
    style_override: InputStyle,
    chrome_override: ChromeRefinement,
    layout_override: LayoutRefinement,
    border_width_override: BorderWidthOverride,
    corner_radii_override: Option<Corners>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let theme_snapshot = theme.snapshot();
    let submit_command = submit_command.filter(|cmd| cx.command_is_enabled(cmd));
    let cancel_command = cancel_command.filter(|cmd| cx.command_is_enabled(cmd));

    let resolved = resolve_input_chrome(
        theme,
        size,
        &chrome_override,
        InputTokenKeys {
            bg: Some("component.input.bg"),
            border: Some("input"),
            border_focus: Some("ring"),
            fg: Some("foreground"),
            // shadcn/ui v4 `Input` uses `selection:bg-primary selection:text-primary-foreground`.
            // We currently model the background color only.
            selection: Some("primary"),
            ..InputTokenKeys::none()
        },
    );

    let mut chrome = TextInputStyle::from_theme(theme_snapshot.clone());
    chrome.padding = resolved.padding;
    chrome.corner_radii = Corners::all(resolved.radius);
    chrome.border = fret_core::Edges::all(resolved.border_width);
    chrome.background = resolved.background;
    chrome.border_color = resolved.border_color;
    chrome.border_color_focused = resolved.border_color_focused;
    chrome.focus_ring = Some(decl_style::focus_ring(theme, resolved.radius));
    chrome.text_color = resolved.text_color;
    chrome.placeholder_color = theme
        .color_by_key("muted-foreground")
        .unwrap_or(chrome.placeholder_color);
    chrome.caret_color = resolved.text_color;
    chrome.selection_color = resolved.selection_color;
    chrome.preedit_color = chrome.text_color;
    chrome.preedit_underline_color = chrome.text_color;

    if let Some(bg) = style_override.background {
        chrome.background = bg.resolve(theme);
    }
    if let Some(border) = style_override.border_color {
        chrome.border_color = border.resolve(theme);
    }
    if let Some(border) = style_override.border_color_focused {
        chrome.border_color_focused = border.resolve(theme);
    }
    if let Some(ring_color) = style_override.focus_ring_color
        && let Some(mut ring) = chrome.focus_ring.take()
    {
        ring.color = ring_color.resolve(theme);
        chrome.focus_ring = Some(ring);
    }

    if aria_invalid {
        let border_color = theme.color_token("destructive");
        chrome.border_color = border_color;
        chrome.border_color_focused = border_color;
        if let Some(mut ring) = chrome.focus_ring.take() {
            ring.color =
                crate::theme_variants::invalid_control_ring_color(&theme_snapshot, border_color);
            chrome.focus_ring = Some(ring);
        }
    }

    if let Some(corners) = corner_radii_override {
        chrome.corner_radii = corners;
    }
    if let Some(border) = border_width_override.top {
        chrome.border.top = border;
    }
    if let Some(border) = border_width_override.right {
        chrome.border.right = border;
    }
    if let Some(border) = border_width_override.bottom {
        chrome.border.bottom = border;
    }
    if let Some(border) = border_width_override.left {
        chrome.border.left = border;
    }

    let text_style = typography::control_text_style_scaled(theme, FontId::ui(), resolved.text_px);

    let mut props = TextInputProps::new(model);
    props.enabled = !disabled;
    props.focusable = !disabled;
    props.a11y_label = a11y_label;
    props.a11y_role = a11y_role;
    props.test_id = test_id;
    props.placeholder = placeholder;
    props.a11y_required = aria_required;
    props.a11y_invalid = aria_invalid.then_some(fret_core::SemanticsInvalid::True);
    props.active_descendant = active_descendant;
    props.expanded = expanded;
    props.submit_command = submit_command.clone();
    props.cancel_command = cancel_command;
    props.chrome = chrome;
    props.text_style = text_style;
    props.layout.size = SizeStyle {
        width: Length::Fill,
        height: Length::Fill,
        min_width: Some(Length::Px(fret_core::Px(0.0))),
        ..Default::default()
    };
    props.layout.overflow = Overflow::Clip;

    // shadcn/ui `Input` uses `shadow-xs`. The text input widget does not expose a shadow field in
    // its chrome style, so we model the outcome by wrapping it in a shadow-only container.
    let mut root_layout = decl_style::layout_style(
        theme,
        LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .h_px(resolved.min_height)
            .merge(layout_override),
    );
    root_layout.overflow = fret_ui::element::Overflow::Visible;
    let root_shadow = {
        let mut shadow = decl_style::shadow_xs(theme, resolved.radius);
        if let Some(corners) = corner_radii_override {
            shadow.corner_radii = corners;
        }
        shadow
    };

    let model_for_hook = props.model.clone();
    let on_submit_hook = on_submit.clone();
    let submit_command_for_hook = submit_command.clone();
    let input = cx.text_input_with_id_props(|cx, id| {
        if let Some(on_submit_hook) = on_submit_hook.clone() {
            cx.key_add_on_key_down_for(
                id,
                Arc::new(
                    move |host: &mut dyn UiFocusActionHost,
                          action_cx: ActionCx,
                          down: KeyDownCx| {
                        if down.key != KeyCode::Enter {
                            return false;
                        }
                        on_submit_hook(host, action_cx);
                        if let Some(command) = submit_command_for_hook.clone() {
                            host.dispatch_command(Some(action_cx.window), command);
                        }
                        host.request_redraw(action_cx.window);
                        true
                    },
                ),
            );
        }
        let mut props = props.clone();
        // Ensure the key hook reads the latest text from the model on the dispatch cycle.
        props.model = model_for_hook.clone();
        props
    });

    let input = cx.container(
        fret_ui::element::ContainerProps {
            layout: root_layout,
            background: None,
            shadow: Some(root_shadow),
            corner_radii: Corners::all(resolved.radius),
            ..Default::default()
        },
        |_cx| vec![input],
    );

    if disabled {
        cx.opacity(0.5, move |_cx| vec![input])
    } else {
        input
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size as CoreSize};
    use fret_ui::element::{ElementKind, Length};
    use fret_ui::elements;

    #[test]
    fn input_selection_color_uses_primary_in_shadcn_light_theme() {
        let mut app = fret_app::App::new();
        crate::shadcn_themes::apply_shadcn_new_york_v4(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Slate,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let theme = Theme::global(&app);
        let expected = theme.color_token("primary");

        let resolved = resolve_input_chrome(
            theme,
            Size::default(),
            &ChromeRefinement::default(),
            InputTokenKeys {
                bg: Some("component.input.bg"),
                border: Some("input"),
                border_focus: Some("ring"),
                fg: Some("foreground"),
                selection: Some("primary"),
                ..InputTokenKeys::none()
            },
        );

        assert_eq!(resolved.selection_color, expected);
    }

    #[test]
    fn input_wraps_in_shadow_container_like_shadcn() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york_v4(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Slate,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(320.0), Px(120.0)),
        );

        let model = app.models_mut().insert(String::new());
        let el =
            elements::with_element_cx(&mut app, window, bounds, "input-shadow-wrapper", |cx| {
                Input::new(model.clone())
                    .a11y_label("Input")
                    .into_element(cx)
            });

        let ElementKind::Container(root) = &el.kind else {
            panic!(
                "expected Input root to be a shadow container, got {:?}",
                el.kind
            );
        };
        assert!(
            root.shadow.is_some(),
            "expected Input to have shadow-xs wrapper"
        );
        assert_eq!(root.layout.size.width, Length::Fill);

        let child = el.children.first().expect("shadow wrapper child");
        let ElementKind::TextInput(props) = &child.kind else {
            panic!(
                "expected shadow wrapper child to be TextInput, got {:?}",
                child.kind
            );
        };
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Fill);
    }
}
