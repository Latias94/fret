use std::sync::Arc;

use fret_core::{Color, Corners, FontId, KeyCode, NodeId, Px, SemanticsRole};
use fret_runtime::{CommandId, Model};
use fret_ui::action::{ActionCx, KeyDownCx, UiFocusActionHost};
use fret_ui::element::{
    AnyElement, Length, Overflow, SemanticsDecoration, SizeStyle, TextInputProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, TextInputStyle, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::motion::{
    drive_tween_color_for_element, drive_tween_f32_for_element,
};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::control_registry::{
    ControlAction, ControlEntry, ControlId, control_registry_model,
};
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Size};

use crate::overlay_motion;

fn tailwind_transition_ease_in_out(t: f32) -> f32 {
    // Tailwind default transition timing function: cubic-bezier(0.4, 0, 0.2, 1).
    // (Often described as `ease-in-out`-ish.)
    fret_ui_headless::easing::SHADCN_EASE.sample(t)
}

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
    labelled_by_element: Option<GlobalElementId>,
    control_id: Option<ControlId>,
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
            labelled_by_element: None,
            control_id: None,
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

    /// Associates this control with a label element (ARIA `aria-labelledby`-like outcome).
    ///
    /// This is the preferred way to model shadcn/Radix `Label` → `Input` association in Fret,
    /// since we do not have DOM `id`/`htmlFor`.
    pub fn labelled_by_element(mut self, label: GlobalElementId) -> Self {
        self.labelled_by_element = Some(label);
        self
    }

    /// Associates this input with a logical form control id so related elements (e.g. labels,
    /// helper text) can forward activation and attach `labelled-by` / `described-by` semantics.
    pub fn control_id(mut self, id: impl Into<ControlId>) -> Self {
        self.control_id = Some(id.into());
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
            self.labelled_by_element,
            self.control_id,
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
        None,
        None,
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
    labelled_by_element: Option<GlobalElementId>,
    control_id: Option<ControlId>,
    border_width_override: BorderWidthOverride,
    corner_radii_override: Option<Corners>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let theme_snapshot = theme.snapshot();
    let submit_command = submit_command.filter(|cmd| cx.command_is_enabled(cmd));
    let cancel_command = cancel_command.filter(|cmd| cx.command_is_enabled(cmd));
    let has_a11y_label = a11y_label.is_some();

    let resolved = resolve_input_chrome(
        theme,
        size,
        &chrome_override,
        InputTokenKeys {
            bg: Some("component.input.bg"),
            border: Some("input"),
            border_focus: Some("ring"),
            fg: Some("foreground"),
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
    let control_id = control_id.clone();
    let control_registry = control_id.as_ref().map(|_| control_registry_model(cx));
    let input = cx.text_input_with_id_props(|cx, id| {
        if let (Some(control_id), Some(control_registry)) =
            (control_id.clone(), control_registry.clone())
        {
            let entry = ControlEntry {
                element: id,
                enabled: !disabled,
                action: ControlAction::Noop,
            };
            let _ = cx.app.models_mut().update(&control_registry, |reg| {
                reg.register_control(cx.window, cx.frame_id, control_id, entry);
            });
        }

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

        // shadcn/ui v4 input uses `transition-[color,box-shadow]` with Tailwind defaults, so
        // border + ring should ease instead of snapping.
        let duration = overlay_motion::shadcn_motion_duration_150(cx);
        let focus_visible = cx.is_focused_element(id)
            && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));

        let target_border = if focus_visible {
            props.chrome.border_color_focused
        } else {
            props.chrome.border_color
        };
        let border_motion = drive_tween_color_for_element(
            cx,
            id,
            "input.chrome.border",
            target_border,
            duration,
            tailwind_transition_ease_in_out,
        );
        props.chrome.border_color = border_motion.value;
        props.chrome.border_color_focused = border_motion.value;

        let ring_alpha = drive_tween_f32_for_element(
            cx,
            id,
            "input.chrome.ring.alpha",
            if focus_visible { 1.0 } else { 0.0 },
            duration,
            tailwind_transition_ease_in_out,
        );
        if let Some(mut ring) = props.chrome.focus_ring.take() {
            ring.color.a = (ring.color.a * ring_alpha.value).clamp(0.0, 1.0);
            if let Some(offset_color) = ring.offset_color {
                ring.offset_color = Some(Color {
                    a: (offset_color.a * ring_alpha.value).clamp(0.0, 1.0),
                    ..offset_color
                });
            }
            props.chrome.focus_ring = Some(ring);
        }
        props.focus_ring_always_paint = ring_alpha.animating;
        props
    });

    let labelled_by_element = if labelled_by_element.is_some() {
        labelled_by_element
    } else if has_a11y_label {
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

    let input = if labelled_by_element.is_some() || described_by_element.is_some() {
        let mut decoration = SemanticsDecoration::default();
        if let Some(label) = labelled_by_element {
            decoration = decoration.labelled_by_element(label.0);
        }
        if let Some(desc) = described_by_element {
            decoration = decoration.described_by_element(desc.0);
        }
        input.attach_semantics(decoration)
    } else {
        input
    };

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
    use fret_core::{
        Modifiers, PathCommand, Size as UiSize, SvgId, SvgService, TextBlobId, TextConstraints,
        TextMetrics, TextService,
    };
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_runtime::FrameId;
    use fret_ui::element::{ElementKind, Length};
    use fret_ui::elements;
    use fret_ui::{UiTree, focus_visible};
    use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;
    use fret_ui_kit::primitives::control_registry::ControlId;

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
                    size: UiSize::new(Px(0.0), Px(0.0)),
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

    #[test]
    fn input_selection_color_uses_selection_background_in_shadcn_light_theme() {
        let mut app = fret_app::App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Slate,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let theme = Theme::global(&app);
        let expected = theme.color_token("selection.background");

        let resolved = resolve_input_chrome(
            theme,
            Size::default(),
            &ChromeRefinement::default(),
            InputTokenKeys {
                bg: Some("component.input.bg"),
                border: Some("input"),
                border_focus: Some("ring"),
                fg: Some("foreground"),
                ..InputTokenKeys::none()
            },
        );

        assert_eq!(resolved.selection_color, expected);
    }

    #[test]
    fn input_wraps_in_shadow_container_like_shadcn() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
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

    #[test]
    fn input_focus_ring_tweens_in_and_out_like_a_transition() {
        use std::cell::Cell;
        use std::rc::Rc;
        use std::time::Duration;

        use fret_core::{Event, KeyCode, Rect, Size};

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
            Size::new(Px(320.0), Px(160.0)),
        );
        let mut services = FakeServices::default();
        let model = app.models_mut().insert(String::new());

        let ring_alpha_out: Rc<Cell<Option<f32>>> = Rc::new(Cell::new(None));
        let always_paint_out: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            model: Model<String>,
            ring_alpha_out: Rc<Cell<Option<f32>>>,
            always_paint_out: Rc<Cell<Option<bool>>>,
        ) -> fret_core::NodeId {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "input-focus-ring-tween",
                move |cx| {
                    let el = Input::new(model).a11y_label("Input").into_element(cx);

                    let ElementKind::Container(_) = &el.kind else {
                        panic!("expected Input to wrap in a Container");
                    };
                    let child = el.children.first().expect("input inner child");
                    let ElementKind::TextInput(props) = &child.kind else {
                        panic!("expected Input inner to be TextInput");
                    };

                    let a = props
                        .chrome
                        .focus_ring
                        .map(|ring| ring.color.a)
                        .unwrap_or(0.0);
                    ring_alpha_out.set(Some(a));
                    always_paint_out.set(Some(props.focus_ring_always_paint));

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
            model.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a0 = ring_alpha_out.get().expect("a0");
        assert!(
            a0.abs() <= 1e-6,
            "expected ring alpha to start at 0, got {a0}"
        );

        // Focus the input and mark focus-visible via a navigation key.
        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable input");
        ui.set_focus(Some(focusable));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Tab,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(
            focus_visible::is_focus_visible(&mut app, Some(window)),
            "sanity: focus-visible should be enabled after navigation key"
        );

        // Frame 2: ring should be in-between (not snapped).
        app.set_frame_id(FrameId(2));
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

        // Blur and ensure ring animates out while still being painted.
        ui.set_focus(None);
        app.set_frame_id(FrameId(1000));
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
                model.clone(),
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

    #[test]
    fn input_can_reference_a_label_element_for_a11y_association() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
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

        let root = elements::with_element_cx(&mut app, window, bounds, "labelled-input", |cx| {
            let label = crate::Label::new("Email").into_element(cx);
            let label_id = label.id;

            let input = Input::new(model.clone())
                .labelled_by_element(label_id)
                .into_element(cx);

            cx.column(fret_ui::element::ColumnProps::default(), |_cx| {
                vec![label, input]
            })
        });

        let ElementKind::Column(col) = &root.kind else {
            panic!("expected test root to be a Column, got {:?}", root.kind);
        };
        assert_eq!(
            col.layout.size.width,
            Length::Auto,
            "sanity: ColumnProps default width stays Auto"
        );

        let input = root.children.get(1).expect("input child");
        let ElementKind::Container(_) = &input.kind else {
            panic!("expected Input to wrap in a Container");
        };
        let text_input = input.children.first().expect("text input");
        let ElementKind::TextInput(_) = &text_input.kind else {
            panic!("expected Input inner node to be a TextInput");
        };
        let decoration = text_input
            .semantics_decoration
            .as_ref()
            .expect("expected labelled_by decoration on TextInput");
        assert_eq!(decoration.labelled_by_element, Some(root.children[0].id.0));
    }

    #[test]
    fn input_control_id_uses_registry_labelled_by_and_described_by_elements() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Slate,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(320.0), Px(180.0)),
        );

        let model = app.models_mut().insert(String::new());

        let root = elements::with_element_cx(&mut app, window, bounds, "control-id-input", |cx| {
            let id = ControlId::from("email");

            cx.column(fret_ui::element::ColumnProps::default(), move |cx| {
                vec![
                    crate::field::FieldLabel::new("Email")
                        .for_control(id.clone())
                        .into_element(cx),
                    crate::field::FieldDescription::new("We will never share it.")
                        .for_control(id.clone())
                        .into_element(cx),
                    Input::new(model.clone()).control_id(id).into_element(cx),
                ]
            })
        });

        let label_id = root.children[0].id;
        let desc_id = root.children[1].id;

        fn find_text_input(el: &AnyElement) -> Option<&AnyElement> {
            if matches!(el.kind, ElementKind::TextInput(_)) {
                return Some(el);
            }
            for child in &el.children {
                if let Some(found) = find_text_input(child) {
                    return Some(found);
                }
            }
            None
        }

        let input = find_text_input(&root).expect("expected a TextInput node");
        let decoration = input
            .semantics_decoration
            .as_ref()
            .expect("expected semantics decoration on TextInput");
        assert_eq!(decoration.labelled_by_element, Some(label_id.0));
        assert_eq!(decoration.described_by_element, Some(desc_id.0));
    }

    #[test]
    fn input_control_id_uses_registry_label_element_from_label_primitive() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Slate,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(320.0), Px(180.0)),
        );

        let model = app.models_mut().insert(String::new());

        let root =
            elements::with_element_cx(&mut app, window, bounds, "control-id-input-label", |cx| {
                let id = ControlId::from("email");

                cx.column(fret_ui::element::ColumnProps::default(), move |cx| {
                    vec![
                        crate::label::Label::new("Email")
                            .for_control(id.clone())
                            .into_element(cx),
                        Input::new(model.clone()).control_id(id).into_element(cx),
                    ]
                })
            });

        let label_id = root.children[0].id;

        fn find_text_input(el: &AnyElement) -> Option<&AnyElement> {
            if matches!(el.kind, ElementKind::TextInput(_)) {
                return Some(el);
            }
            for child in &el.children {
                if let Some(found) = find_text_input(child) {
                    return Some(found);
                }
            }
            None
        }

        let input = find_text_input(&root).expect("expected a TextInput node");
        let decoration = input
            .semantics_decoration
            .as_ref()
            .expect("expected semantics decoration on TextInput");
        assert_eq!(decoration.labelled_by_element, Some(label_id.0));
    }
}
