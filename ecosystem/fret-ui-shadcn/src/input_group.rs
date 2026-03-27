use std::sync::Arc;

use crate::button::{ButtonVariant, variant_colors};
use crate::rtl;
use crate::text_value_model::IntoTextValueModel;
use fret_core::{
    Axis, Color, Corners, CursorIcon, Edges, FontId, FontWeight, MouseButton, Px, SemanticsRole,
    TextOverflow, TextWrap,
};
use fret_icons::IconId;
use fret_runtime::{CommandId, Model};
use fret_ui::action::{OnKeyDown, UiPointerActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, ElementKind, FlexProps, LayoutStyle, Length, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, SemanticsDecoration, TextAreaProps,
    TextInputProps, TextProps,
};
use fret_ui::{ElementContext, TextAreaStyle, TextInputStyle, Theme, UiHost, focus_visible};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::motion::{
    drive_tween_color_for_element, drive_tween_f32_for_element,
};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::control_registry::{
    ControlAction, ControlEntry, ControlId, control_registry_model,
};
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, IntoUiElement, LayoutRefinement, Size as ComponentSize, Space,
};

#[derive(Debug, Clone, Copy, Default)]
struct BorderWidthOverride {
    top: Option<Px>,
    right: Option<Px>,
    bottom: Option<Px>,
    left: Option<Px>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputGroupControlKind {
    #[default]
    Input,
    Textarea,
}

pub struct InputGroup {
    model: Model<String>,
    control: InputGroupControlKind,
    custom_control: Option<AnyElement>,
    test_id: Option<Arc<str>>,
    control_test_id: Option<Arc<str>>,
    control_id: Option<ControlId>,
    control_on_key_down: Option<OnKeyDown>,
    placeholder: Option<Arc<str>>,
    disabled: bool,
    leading: Vec<AnyElement>,
    trailing: Vec<AnyElement>,
    block_start: Vec<AnyElement>,
    block_end: Vec<AnyElement>,
    block_start_border_bottom: bool,
    block_end_border_top: bool,
    leading_has_button: bool,
    trailing_has_button: bool,
    leading_has_kbd: bool,
    trailing_has_kbd: bool,
    aria_invalid: bool,
    a11y_label: Option<Arc<str>>,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    textarea_min_height: Px,
    textarea_max_height: Option<Px>,
    size: ComponentSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    border_width_override: BorderWidthOverride,
    corner_radii_override: Option<Corners>,
}

impl std::fmt::Debug for InputGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputGroup")
            .field("model", &"<model>")
            .field("control", &self.control)
            .field("custom_control", &self.custom_control.is_some())
            .field("test_id", &self.test_id.as_deref())
            .field("control_test_id", &self.control_test_id.as_deref())
            .field(
                "control_id",
                &self.control_id.as_ref().map(|id| id.as_str()),
            )
            .field("control_on_key_down", &self.control_on_key_down.is_some())
            .field("leading_len", &self.leading.len())
            .field("trailing_len", &self.trailing.len())
            .field("block_start_len", &self.block_start.len())
            .field("block_end_len", &self.block_end.len())
            .field("block_start_border_bottom", &self.block_start_border_bottom)
            .field("block_end_border_top", &self.block_end_border_top)
            .field("leading_has_button", &self.leading_has_button)
            .field("trailing_has_button", &self.trailing_has_button)
            .field("leading_has_kbd", &self.leading_has_kbd)
            .field("trailing_has_kbd", &self.trailing_has_kbd)
            .field("aria_invalid", &self.aria_invalid)
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("submit_command", &self.submit_command)
            .field("cancel_command", &self.cancel_command)
            .field("textarea_min_height", &self.textarea_min_height)
            .field("textarea_max_height", &self.textarea_max_height)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl InputGroup {
    pub fn new(model: impl IntoTextValueModel) -> Self {
        Self {
            model: model.into_text_value_model(),
            control: InputGroupControlKind::Input,
            custom_control: None,
            test_id: None,
            control_test_id: None,
            control_id: None,
            control_on_key_down: None,
            placeholder: None,
            disabled: false,
            leading: Vec::new(),
            trailing: Vec::new(),
            block_start: Vec::new(),
            block_end: Vec::new(),
            block_start_border_bottom: false,
            block_end_border_top: false,
            leading_has_button: false,
            trailing_has_button: false,
            leading_has_kbd: false,
            trailing_has_kbd: false,
            aria_invalid: false,
            a11y_label: None,
            submit_command: None,
            cancel_command: None,
            textarea_min_height: Px(64.0),
            textarea_max_height: None,
            size: ComponentSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            border_width_override: BorderWidthOverride::default(),
            corner_radii_override: None,
        }
    }

    pub fn control(mut self, control: InputGroupControlKind) -> Self {
        self.control = control;
        self
    }

    pub fn textarea(self) -> Self {
        self.control(InputGroupControlKind::Textarea)
    }

    /// Replaces the built-in input with a caller-owned control while preserving InputGroup chrome,
    /// addon routing, focus-ring ownership, and control-id wiring.
    ///
    /// The caller owns the inner control's chrome, placeholder, and input behavior.
    pub fn custom_input(mut self, control: AnyElement) -> Self {
        self.control = InputGroupControlKind::Input;
        self.custom_control = Some(control);
        self
    }

    /// Replaces the built-in textarea with a caller-owned multiline control while preserving
    /// InputGroup chrome, block addon routing, focus-ring ownership, and control-id wiring.
    ///
    /// The caller owns the inner control's chrome, placeholder, and input behavior.
    pub fn custom_textarea(mut self, control: AnyElement) -> Self {
        self.control = InputGroupControlKind::Textarea;
        self.custom_control = Some(control);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn control_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.control_test_id = Some(id.into());
        self
    }

    /// Associates this group with a logical form control id so related elements (e.g. labels,
    /// helper text) can forward activation and attach `labelled-by` / `described-by` semantics.
    pub fn control_id(mut self, id: impl Into<ControlId>) -> Self {
        self.control_id = Some(id.into());
        self
    }

    pub fn control_on_key_down(mut self, handler: OnKeyDown) -> Self {
        self.control_on_key_down = Some(handler);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn leading<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.leading = children.into_iter().collect();
        self
    }

    pub fn trailing<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.trailing = children.into_iter().collect();
        self
    }

    pub fn block_start<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.block_start = children.into_iter().collect();
        self
    }

    pub fn block_end<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.block_end = children.into_iter().collect();
        self
    }

    pub fn block_start_border_bottom(mut self, value: bool) -> Self {
        self.block_start_border_bottom = value;
        self
    }

    pub fn block_end_border_top(mut self, value: bool) -> Self {
        self.block_end_border_top = value;
        self
    }

    /// Upstream uses `has-[>button]:ml-[-0.45rem]` on inline-start addons (v4).
    ///
    /// Fret does not currently have a selector mechanism, so this is an explicit hint for the
    /// `InputGroup` recipe to apply the same layout outcome.
    pub fn leading_has_button(mut self, value: bool) -> Self {
        self.leading_has_button = value;
        self
    }

    /// Upstream uses `has-[>button]:mr-[-0.45rem]` on inline-end addons (v4).
    ///
    /// Fret does not currently have a selector mechanism, so this is an explicit hint for the
    /// `InputGroup` recipe to apply the same layout outcome.
    pub fn trailing_has_button(mut self, value: bool) -> Self {
        self.trailing_has_button = value;
        self
    }

    /// Upstream uses `has-[>kbd]:ml-[-0.35rem]` on inline-start addons (v4).
    ///
    /// Fret does not currently have a selector mechanism, so this is an explicit hint for the
    /// `InputGroup` recipe to apply the same layout outcome.
    pub fn leading_has_kbd(mut self, value: bool) -> Self {
        self.leading_has_kbd = value;
        self
    }

    /// Upstream uses `has-[>kbd]:mr-[-0.35rem]` on inline-end addons (v4).
    ///
    /// Fret does not currently have a selector mechanism, so this is an explicit hint for the
    /// `InputGroup` recipe to apply the same layout outcome.
    pub fn trailing_has_kbd(mut self, value: bool) -> Self {
        self.trailing_has_kbd = value;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn submit_command(mut self, command: CommandId) -> Self {
        self.submit_command = Some(command);
        self
    }

    /// Preferred action-first spelling for Enter submit dispatch.
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this lowers through
    /// the existing text-input command pipeline.
    pub fn submit_action(self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.submit_command(action.into())
    }

    pub fn cancel_command(mut self, command: CommandId) -> Self {
        self.cancel_command = Some(command);
        self
    }

    /// Preferred action-first spelling for Escape cancel dispatch.
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this lowers through
    /// the existing text-input command pipeline.
    pub fn cancel_action(self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.cancel_command(action.into())
    }

    pub fn textarea_min_height(mut self, min_height: Px) -> Self {
        self.textarea_min_height = min_height;
        self
    }

    pub fn textarea_max_height(mut self, max_height: Px) -> Self {
        self.textarea_max_height = Some(max_height);
        self
    }

    pub fn size(mut self, size: ComponentSize) -> Self {
        self.size = size;
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

    /// Overrides per-edge border widths (in px) for this input group's outer chrome.
    ///
    /// This is primarily used to match upstream `ButtonGroup` border-merge outcomes.
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

    /// Overrides per-corner radii (in px) for this input group's outer chrome.
    ///
    /// This is primarily used to match upstream `ButtonGroup` corner-merge outcomes.
    pub fn corner_radii_override(mut self, corners: Corners) -> Self {
        self.corner_radii_override = Some(corners);
        self
    }

    /// Apply the upstream `aria-invalid` error state chrome (group border + focus ring color).
    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        // Treat textarea-driven groups as block layouts by default, matching the upstream shadcn
        // contract (`has-[>textarea]:h-auto`) and avoiding an "inline textarea" configuration that
        // would ignore block addon alignment.
        let is_block_layout = self.control == InputGroupControlKind::Textarea
            || !self.block_start.is_empty()
            || !self.block_end.is_empty();

        let (
            resolved,
            addon_pl,
            addon_py,
            compact_px,
            textarea_py,
            input_text_style,
            textarea_text_style,
            root_layout,
            root_shadow,
            border_color,
            focus_border_color,
            focus_ring,
        ) = {
            let theme = Theme::global(&*cx.app);
            let theme_snapshot = theme.snapshot();

            let resolved =
                resolve_input_chrome(theme, self.size, &self.chrome, InputTokenKeys::none());

            let addon_pl = fret_ui_kit::MetricRef::space(Space::N3).resolve(theme);
            let addon_py = fret_ui_kit::MetricRef::space(Space::N1p5).resolve(theme);
            let compact_px = fret_ui_kit::MetricRef::space(Space::N2).resolve(theme);
            let textarea_py = fret_ui_kit::MetricRef::space(Space::N3).resolve(theme);

            let input_text_style =
                typography::control_text_style_scaled(theme, FontId::ui(), resolved.text_px);
            let textarea_text_style = typography::text_area_control_text_style_scaled(
                theme,
                FontId::ui(),
                resolved.text_px,
            );

            let root_layout = decl_style::layout_style(theme, {
                let mut root = LayoutRefinement::default().relative().w_full().min_w_0();
                if !is_block_layout && self.control == InputGroupControlKind::Input {
                    root = root.h_px(resolved.min_height);
                }
                // Match shadcn's `className` override path: recipe defaults stay baked into the
                // root, but caller refinements should still win when they explicitly override
                // width/height/constraints.
                root.merge(self.layout)
            });

            let root_shadow = {
                let mut shadow = decl_style::shadow_xs(theme, resolved.radius);
                if let Some(corners) = self.corner_radii_override {
                    shadow.corner_radii = corners;
                }
                shadow
            };

            let (border_color, focus_border_color, focus_ring) = {
                let mut ring = decl_style::focus_ring(theme, resolved.radius);
                let focus_border = Some(resolved.border_color_focused);

                if self.aria_invalid {
                    let border_color = theme.color_token("destructive");
                    ring.color = crate::theme_variants::invalid_control_ring_color(
                        &theme_snapshot,
                        border_color,
                    );
                    (border_color, None, Some(ring))
                } else {
                    (resolved.border_color, focus_border, Some(ring))
                }
            };

            (
                resolved,
                addon_pl,
                addon_py,
                compact_px,
                textarea_py,
                input_text_style,
                textarea_text_style,
                root_layout,
                root_shadow,
                border_color,
                focus_border_color,
                focus_ring,
            )
        };

        let leading = self.leading;
        let trailing = self.trailing;
        let block_start = self.block_start;
        let block_end = self.block_end;
        let block_start_border_bottom = self.block_start_border_bottom;
        let block_end_border_top = self.block_end_border_top;
        let leading_has_button = self.leading_has_button;
        let trailing_has_button = self.trailing_has_button;
        let leading_has_kbd = self.leading_has_kbd;
        let trailing_has_kbd = self.trailing_has_kbd;
        let aria_invalid = self.aria_invalid;
        let control = self.control;
        let custom_control = self.custom_control;
        let a11y_label = self.a11y_label;
        let submit_command = self.submit_command;
        let cancel_command = self.cancel_command;
        let model = self.model;
        let placeholder = self.placeholder;
        let textarea_min_height = self.textarea_min_height;
        let textarea_max_height = self.textarea_max_height;
        let test_id = self.test_id;
        let control_test_id = self.control_test_id;
        let control_id = self.control_id;
        let control_on_key_down = self.control_on_key_down;
        let disabled = self.disabled;
        let border_width_override = self.border_width_override;
        let corner_radii_override = self.corner_radii_override;

        let mut root_border = Edges::all(resolved.border_width);
        if let Some(border) = border_width_override.top {
            root_border.top = border;
        }
        if let Some(border) = border_width_override.right {
            root_border.right = border;
        }
        if let Some(border) = border_width_override.bottom {
            root_border.bottom = border;
        }
        if let Some(border) = border_width_override.left {
            root_border.left = border;
        }
        let root_corner_radii =
            corner_radii_override.unwrap_or_else(|| Corners::all(resolved.radius));

        // Web: in block layouts (`data-align=block-*`), the input ends up 2px shorter than `h-9`
        // because the outer border consumes 1px at both block edges. In inline layouts, the input
        // keeps `h-9` and simply overflows into the border area.
        let block_control_min_height =
            Px((resolved.min_height.0 - root_border.top.0 - root_border.bottom.0).max(0.0));

        let mut animated_control_id = None;
        let mut root = cx.container(ContainerProps::default(), |cx| {
            let has_a11y_label = a11y_label.is_some();
            let control_registry = control_id.as_ref().map(|_| control_registry_model(cx));
            let dir = crate::direction::use_direction(cx, None);
            let build_inline_addon = |cx: &mut ElementContext<'_, H>,
                                      children: Vec<AnyElement>,
                                      is_start: bool,
                                      has_button: bool,
                                      has_kbd: bool,
                                      control_focus_target: Option<
                fret_ui::elements::GlobalElementId,
            >| {
                let (order_inline_start, order_inline_end) = rtl::inline_start_end_pair(dir, -1, 1);
                let order = if is_start {
                    order_inline_start
                } else {
                    order_inline_end
                };
                let mut layout = LayoutRefinement::default().flex_none().order(order);
                if has_button {
                    layout = if is_start {
                        rtl::layout_refinement_apply_margin_inline_start_neg(layout, dir, Space::N2)
                    } else {
                        rtl::layout_refinement_apply_margin_inline_end_neg(layout, dir, Space::N2)
                    };
                } else if has_kbd {
                    layout = if is_start {
                        rtl::layout_refinement_apply_margin_inline_start_neg(
                            layout,
                            dir,
                            Space::N1p5,
                        )
                    } else {
                        rtl::layout_refinement_apply_margin_inline_end_neg(layout, dir, Space::N1p5)
                    };
                }

                let (layout, gap) = {
                    let theme = Theme::global(&*cx.app);
                    (
                        decl_style::layout_style(theme, layout),
                        fret_ui_kit::MetricRef::space(Space::N2).resolve(theme),
                    )
                };

                let padding = if is_start {
                    rtl::padding_edges_with_inline_start_end(
                        dir,
                        addon_py,
                        addon_py,
                        addon_pl,
                        Px(0.0),
                    )
                } else {
                    rtl::padding_edges_with_inline_start_end(
                        dir,
                        addon_py,
                        addon_py,
                        Px(0.0),
                        addon_pl,
                    )
                };

                let muted_foreground = {
                    let theme = Theme::global(&*cx.app);
                    theme.color_token("muted-foreground")
                };

                let should_click_to_focus = control_focus_target.is_some();

                let on_down = should_click_to_focus.then(|| {
                    let control_focus_target = control_focus_target.expect("control_focus_target");

                    Arc::new(
                        move |host: &mut dyn UiPointerActionHost,
                              _cx: fret_ui::action::ActionCx,
                              down: fret_ui::action::PointerDownCx| {
                            if down.button == MouseButton::Left
                                && down.hit_pressable_target.is_none()
                            {
                                host.request_focus(control_focus_target);
                            }
                            false
                        },
                    )
                });

                let on_move = Arc::new(
                    move |host: &mut dyn UiPointerActionHost,
                          _cx: fret_ui::action::ActionCx,
                          _mv: fret_ui::action::PointerMoveCx| {
                        // Upstream uses `cursor-text` for addons, even though they can be
                        // clickable for "focus the underlying input" behavior.
                        host.set_cursor_icon(CursorIcon::Text);
                        false
                    },
                );

                let flex = cx.flex(
                    FlexProps {
                        layout: LayoutStyle::default(),
                        direction: Axis::Horizontal,
                        gap: gap.into(),
                        padding: padding.into(),
                        justify: fret_ui::element::MainAlign::Center,
                        align: fret_ui::element::CrossAlign::Center,
                        wrap: false,
                    },
                    |_cx| children,
                );

                cx.pointer_region(
                    PointerRegionProps {
                        layout,
                        enabled: true,
                        // Prefer capture-phase move handlers so descendant widgets (buttons)
                        // can still win cursor arbitration in bubble.
                        capture_phase_pointer_moves: true,
                    },
                    move |cx| {
                        if let Some(on_down) = on_down.clone() {
                            cx.pointer_region_on_pointer_down(on_down);
                        }
                        cx.pointer_region_on_pointer_move(on_move.clone());
                        let content = if disabled {
                            cx.opacity(0.5, move |_cx| vec![flex])
                        } else {
                            flex
                        };
                        vec![content.inherit_foreground(muted_foreground)]
                    },
                )
            };

            if is_block_layout {
                let mut control_el = if let Some(mut custom_control) = custom_control {
                    if let Some(test_id) = control_test_id.clone() {
                        custom_control = custom_control.test_id(test_id);
                    }
                    if let Some(label) = a11y_label.clone() {
                        custom_control = custom_control.a11y_label(label);
                    }
                    custom_control
                } else {
                    match control {
                        InputGroupControlKind::Input => {
                            let (resolved_pad_inline_start, resolved_pad_inline_end) =
                                rtl::inline_start_end_pair(
                                    dir,
                                    resolved.padding.left,
                                    resolved.padding.right,
                                );
                            let pad_inline_start = if leading.is_empty() {
                                resolved_pad_inline_start
                            } else {
                                compact_px
                            };
                            let pad_inline_end = if trailing.is_empty() {
                                resolved_pad_inline_end
                            } else {
                                compact_px
                            };

                            let mut chrome =
                                TextInputStyle::from_theme(Theme::global(&*cx.app).snapshot());
                            chrome.padding = rtl::padding_edges_with_inline_start_end(
                                dir,
                                resolved.padding.top,
                                resolved.padding.bottom,
                                pad_inline_start,
                                pad_inline_end,
                            );
                            chrome.corner_radii = Corners::all(Px(0.0));
                            chrome.border = Edges::all(Px(0.0));
                            chrome.background = Color::TRANSPARENT;
                            chrome.border_color = resolved.border_color;
                            chrome.border_color_focused = resolved.border_color_focused;
                            chrome.focus_ring = None;
                            chrome.text_color = resolved.text_color;
                            chrome.caret_color = resolved.text_color;
                            chrome.selection_color = resolved.selection_color;
                            chrome.preedit_color = chrome.text_color;
                            chrome.preedit_underline_color = chrome.text_color;

                            let mut input = TextInputProps::new(model.clone());
                            input.a11y_label = a11y_label.clone();
                            input.test_id = control_test_id.clone();
                            input.placeholder = placeholder.clone();
                            input.submit_command = submit_command;
                            input.cancel_command = cancel_command;
                            input.enabled = !disabled;
                            input.focusable = !disabled;
                            input.chrome = chrome;
                            input.text_style = input_text_style.clone();
                            input.layout = {
                                let theme = Theme::global(&*cx.app);
                                decl_style::layout_style(
                                    theme,
                                    LayoutRefinement::default()
                                        .w_full()
                                        .min_w_0()
                                        .min_h(block_control_min_height),
                                )
                            };
                            cx.text_input(input)
                        }
                        InputGroupControlKind::Textarea => {
                            let mut chrome = TextAreaStyle::default();
                            chrome.padding_x = rtl::padding_x_from_physical_edges_max(
                                resolved.padding.left,
                                resolved.padding.right,
                            );
                            chrome.padding_y = textarea_py;
                            chrome.background = Color::TRANSPARENT;
                            chrome.border = Edges::all(Px(0.0));
                            chrome.border_color = resolved.border_color;
                            chrome.border_color_focused = resolved.border_color_focused;
                            chrome.corner_radii = Corners::all(Px(0.0));
                            chrome.text_color = resolved.text_color;
                            chrome.selection_color = resolved.selection_color;
                            chrome.caret_color = resolved.text_color;
                            chrome.preedit_bg_color = resolved.selection_color;
                            chrome.preedit_underline_color = resolved.selection_color;
                            chrome.focus_ring = None;

                            let mut props = TextAreaProps::new(model.clone());
                            props.a11y_label = a11y_label.clone();
                            props.test_id = control_test_id.clone();
                            props.placeholder = placeholder.clone();
                            props.enabled = !disabled;
                            props.focusable = !disabled;
                            props.chrome = chrome;
                            props.text_style = textarea_text_style.clone();
                            props.min_height = textarea_min_height;
                            props.layout = {
                                let theme = Theme::global(&*cx.app);
                                let mut layout = LayoutRefinement::default().w_full().min_w_0();
                                if let Some(max_h) = textarea_max_height {
                                    layout = layout.max_h(max_h);
                                }
                                decl_style::layout_style(theme, layout)
                            };
                            cx.text_area(props)
                        }
                    }
                };

                let control_element_id = control_el.id;
                let control_focus_target = (!disabled).then_some(control_element_id);

                if let (Some(logical_control_id), Some(control_registry)) =
                    (control_id.clone(), control_registry.clone())
                {
                    let entry = ControlEntry {
                        element: control_element_id,
                        enabled: !disabled,
                        action: ControlAction::FocusOnly,
                    };
                    let _ = cx.app.models_mut().update(&control_registry, |reg| {
                        reg.register_control(
                            cx.window,
                            cx.frame_id,
                            logical_control_id.clone(),
                            entry,
                        );
                    });

                    let labelled_by_element = if has_a11y_label {
                        None
                    } else {
                        cx.app
                            .models()
                            .read(&control_registry, |reg| {
                                reg.label_for(cx.window, &logical_control_id)
                                    .map(|l| l.element)
                            })
                            .ok()
                            .flatten()
                    };

                    let described_by_element = cx
                        .app
                        .models()
                        .read(&control_registry, |reg| {
                            reg.described_by_for(cx.window, &logical_control_id)
                        })
                        .ok()
                        .flatten();

                    if labelled_by_element.is_some() || described_by_element.is_some() {
                        let mut decoration = SemanticsDecoration::default();
                        if let Some(label) = labelled_by_element {
                            decoration = decoration.labelled_by_element(label.0);
                        }
                        if let Some(desc) = described_by_element {
                            decoration = decoration.described_by_element(desc.0);
                        }
                        control_el = control_el.attach_semantics(decoration);
                    }
                }
                if let Some(handler) = control_on_key_down {
                    // Run before the control's internal key handling so callers can
                    // consume keys like Enter/Backspace and prevent default behavior.
                    cx.key_prepend_on_key_down_for(control_element_id, handler);
                }

                let inline_start = (control == InputGroupControlKind::Input && !leading.is_empty())
                    .then(|| {
                        build_inline_addon(
                            cx,
                            leading,
                            true,
                            leading_has_button,
                            leading_has_kbd,
                            control_focus_target,
                        )
                    });

                let inline_end = (control == InputGroupControlKind::Input && !trailing.is_empty())
                    .then(|| {
                        build_inline_addon(
                            cx,
                            trailing,
                            false,
                            trailing_has_button,
                            trailing_has_kbd,
                            control_focus_target,
                        )
                    });

                let control_row_layout = {
                    let theme = Theme::global(&*cx.app);
                    decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0())
                };

                let control_row = cx.flex(
                    FlexProps {
                        layout: control_row_layout,
                        direction: Axis::Horizontal,
                        gap: Px(0.0).into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: fret_ui::element::MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Center,
                        wrap: false,
                    },
                    move |_cx| {
                        let mut children = Vec::new();
                        children.push(control_el);
                        if let Some(inline_start) = inline_start {
                            children.push(inline_start);
                        }
                        if let Some(inline_end) = inline_end {
                            children.push(inline_end);
                        }
                        children
                    },
                );

                let block_start = (!block_start.is_empty()).then(|| {
                    let px_3 = addon_pl;
                    let py_3 = addon_pl;
                    let (gap, input_pt, layout) = {
                        let theme = Theme::global(&*cx.app);
                        (
                            fret_ui_kit::MetricRef::space(Space::N2).resolve(theme),
                            fret_ui_kit::MetricRef::space(Space::N2p5).resolve(theme),
                            decl_style::layout_style(
                                theme,
                                LayoutRefinement::default().w_full().min_w_0().order(-1),
                            ),
                        )
                    };
                    let pt = if control == InputGroupControlKind::Input {
                        input_pt
                    } else {
                        py_3
                    };
                    let pb = if block_start_border_bottom {
                        py_3
                    } else {
                        addon_py
                    };

                    cx.container(
                        fret_ui::element::ContainerProps {
                            layout,
                            border: Edges {
                                top: Px(0.0),
                                right: Px(0.0),
                                bottom: if block_start_border_bottom {
                                    resolved.border_width
                                } else {
                                    Px(0.0)
                                },
                                left: Px(0.0),
                            },
                            border_color: Some(resolved.border_color),
                            ..Default::default()
                        },
                        move |cx| {
                            let muted_foreground =
                                Theme::global(&*cx.app).color_token("muted-foreground");
                            let row = cx.flex(
                                FlexProps {
                                    layout: LayoutStyle::default(),
                                    direction: Axis::Horizontal,
                                    gap: gap.into(),
                                    padding: Edges {
                                        top: pt,
                                        right: px_3,
                                        bottom: pb,
                                        left: px_3,
                                    }
                                    .into(),
                                    justify: fret_ui::element::MainAlign::Start,
                                    align: fret_ui::element::CrossAlign::Center,
                                    wrap: false,
                                },
                                move |_cx| block_start,
                            );

                            let row = if disabled {
                                cx.opacity(0.5, move |_cx| vec![row])
                            } else {
                                row
                            };
                            vec![row.inherit_foreground(muted_foreground)]
                        },
                    )
                });

                let block_end = (!block_end.is_empty()).then(|| {
                    let px_3 = addon_pl;
                    let py_3 = addon_pl;
                    let (gap, input_pb, layout) = {
                        let theme = Theme::global(&*cx.app);
                        (
                            fret_ui_kit::MetricRef::space(Space::N2).resolve(theme),
                            fret_ui_kit::MetricRef::space(Space::N2p5).resolve(theme),
                            decl_style::layout_style(
                                theme,
                                LayoutRefinement::default().w_full().min_w_0().order(1),
                            ),
                        )
                    };
                    let pt = if block_end_border_top { py_3 } else { addon_py };
                    let pb = if control == InputGroupControlKind::Input {
                        input_pb
                    } else {
                        py_3
                    };

                    cx.container(
                        fret_ui::element::ContainerProps {
                            layout,
                            border: Edges {
                                top: if block_end_border_top {
                                    resolved.border_width
                                } else {
                                    Px(0.0)
                                },
                                right: Px(0.0),
                                bottom: Px(0.0),
                                left: Px(0.0),
                            },
                            border_color: Some(resolved.border_color),
                            ..Default::default()
                        },
                        move |cx| {
                            let muted_foreground =
                                Theme::global(&*cx.app).color_token("muted-foreground");
                            let row = cx.flex(
                                FlexProps {
                                    layout: LayoutStyle::default(),
                                    direction: Axis::Horizontal,
                                    gap: gap.into(),
                                    padding: Edges {
                                        top: pt,
                                        right: px_3,
                                        bottom: pb,
                                        left: px_3,
                                    }
                                    .into(),
                                    justify: fret_ui::element::MainAlign::Start,
                                    align: fret_ui::element::CrossAlign::Center,
                                    wrap: false,
                                },
                                move |_cx| block_end,
                            );

                            let row = if disabled {
                                cx.opacity(0.5, move |_cx| vec![row])
                            } else {
                                row
                            };
                            vec![row.inherit_foreground(muted_foreground)]
                        },
                    )
                });

                let layout = cx.flex(
                    FlexProps {
                        layout: {
                            let theme = Theme::global(&*cx.app);
                            decl_style::layout_style(
                                theme,
                                LayoutRefinement::default().w_full().min_w_0(),
                            )
                        },
                        direction: Axis::Vertical,
                        gap: Px(0.0).into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: fret_ui::element::MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Stretch,
                        wrap: false,
                    },
                    move |_cx| {
                        let mut children = Vec::new();
                        children.push(control_row);
                        if let Some(block_start) = block_start {
                            children.push(block_start);
                        }
                        if let Some(block_end) = block_end {
                            children.push(block_end);
                        }
                        children
                    },
                );
                animated_control_id = Some(control_element_id);
                vec![layout]
            } else {
                let mut control_el = if let Some(mut custom_control) = custom_control {
                    if let Some(test_id) = control_test_id.clone() {
                        custom_control = custom_control.test_id(test_id);
                    }
                    if let Some(label) = a11y_label.clone() {
                        custom_control = custom_control.a11y_label(label);
                    }
                    custom_control
                } else {
                    let (resolved_pad_inline_start, resolved_pad_inline_end) =
                        rtl::inline_start_end_pair(
                            dir,
                            resolved.padding.left,
                            resolved.padding.right,
                        );
                    let pad_inline_start = if leading.is_empty() {
                        resolved_pad_inline_start
                    } else {
                        compact_px
                    };
                    let pad_inline_end = if trailing.is_empty() {
                        resolved_pad_inline_end
                    } else {
                        compact_px
                    };

                    let mut chrome = TextInputStyle::from_theme(Theme::global(&*cx.app).snapshot());
                    chrome.padding = rtl::padding_edges_with_inline_start_end(
                        dir,
                        resolved.padding.top,
                        resolved.padding.bottom,
                        pad_inline_start,
                        pad_inline_end,
                    );
                    chrome.corner_radii = Corners::all(Px(0.0));
                    chrome.border = Edges::all(Px(0.0));
                    chrome.background = Color::TRANSPARENT;
                    chrome.border_color = resolved.border_color;
                    chrome.border_color_focused = resolved.border_color_focused;
                    chrome.focus_ring = None;
                    chrome.text_color = resolved.text_color;
                    chrome.caret_color = resolved.text_color;
                    chrome.selection_color = resolved.selection_color;
                    chrome.preedit_color = chrome.text_color;
                    chrome.preedit_underline_color = chrome.text_color;

                    let mut input = TextInputProps::new(model.clone());
                    input.a11y_label = a11y_label.clone();
                    input.test_id = control_test_id.clone();
                    input.placeholder = placeholder.clone();
                    input.submit_command = submit_command;
                    input.cancel_command = cancel_command;
                    input.enabled = !disabled;
                    input.focusable = !disabled;
                    input.chrome = chrome;
                    input.text_style = input_text_style.clone();
                    input.layout = {
                        let theme = Theme::global(&*cx.app);
                        decl_style::layout_style(
                            theme,
                            LayoutRefinement::default()
                                .flex_1()
                                .h_full()
                                .min_w_0()
                                .min_h(resolved.min_height),
                        )
                    };

                    cx.text_input(input)
                };
                let control_element_id = control_el.id;
                let control_focus_target = (!disabled).then_some(control_element_id);

                if let (Some(logical_control_id), Some(control_registry)) =
                    (control_id.clone(), control_registry.clone())
                {
                    let entry = ControlEntry {
                        element: control_element_id,
                        enabled: !disabled,
                        action: ControlAction::FocusOnly,
                    };
                    let _ = cx.app.models_mut().update(&control_registry, |reg| {
                        reg.register_control(
                            cx.window,
                            cx.frame_id,
                            logical_control_id.clone(),
                            entry,
                        );
                    });

                    let labelled_by_element = if has_a11y_label {
                        None
                    } else {
                        cx.app
                            .models()
                            .read(&control_registry, |reg| {
                                reg.label_for(cx.window, &logical_control_id)
                                    .map(|l| l.element)
                            })
                            .ok()
                            .flatten()
                    };

                    let described_by_element = cx
                        .app
                        .models()
                        .read(&control_registry, |reg| {
                            reg.described_by_for(cx.window, &logical_control_id)
                        })
                        .ok()
                        .flatten();

                    if labelled_by_element.is_some() || described_by_element.is_some() {
                        let mut decoration = SemanticsDecoration::default();
                        if let Some(label) = labelled_by_element {
                            decoration = decoration.labelled_by_element(label.0);
                        }
                        if let Some(desc) = described_by_element {
                            decoration = decoration.described_by_element(desc.0);
                        }
                        control_el = control_el.attach_semantics(decoration);
                    }
                }

                if let Some(handler) = control_on_key_down {
                    // Run before the control's internal key handling so callers can
                    // consume keys like Enter/Backspace and prevent default behavior.
                    cx.key_prepend_on_key_down_for(control_element_id, handler);
                }

                let leading = (!leading.is_empty()).then(|| {
                    build_inline_addon(
                        cx,
                        leading,
                        true,
                        leading_has_button,
                        leading_has_kbd,
                        control_focus_target,
                    )
                });

                let trailing = (!trailing.is_empty()).then(|| {
                    build_inline_addon(
                        cx,
                        trailing,
                        false,
                        trailing_has_button,
                        trailing_has_kbd,
                        control_focus_target,
                    )
                });

                let flex_layout = {
                    let theme = Theme::global(&*cx.app);
                    decl_style::layout_style(theme, LayoutRefinement::default().size_full())
                };

                let layout = cx.flex(
                    FlexProps {
                        layout: flex_layout,
                        direction: Axis::Horizontal,
                        gap: Px(0.0).into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: fret_ui::element::MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Center,
                        wrap: false,
                    },
                    move |_cx| {
                        let mut children = Vec::new();
                        children.push(control_el);
                        if let Some(leading) = leading {
                            children.push(leading);
                        }
                        if let Some(trailing) = trailing {
                            children.push(trailing);
                        }
                        children
                    },
                );
                animated_control_id = Some(control_element_id);
                vec![layout]
            }
        });

        let animated_control_id = animated_control_id.expect("input_group control element id");
        let focus_visible_for_control = cx.is_focused_element(animated_control_id)
            && focus_visible::is_focus_visible(cx.app, Some(cx.window));

        let duration = crate::overlay_motion::shadcn_motion_duration_150(cx);
        let ease = crate::overlay_motion::shadcn_ease;
        let target_border_color = if aria_invalid {
            border_color
        } else if focus_visible_for_control {
            focus_border_color.unwrap_or(border_color)
        } else {
            border_color
        };

        let border_motion = drive_tween_color_for_element(
            cx,
            animated_control_id,
            "input-group-border-color",
            target_border_color,
            duration,
            ease,
        );

        let ring_alpha = drive_tween_f32_for_element(
            cx,
            animated_control_id,
            "input-group-ring-alpha",
            if focus_visible_for_control { 1.0 } else { 0.0 },
            duration,
            ease,
        );

        let ring = focus_ring.map(|mut ring| {
            ring.color.a = (ring.color.a * ring_alpha.value).clamp(0.0, 1.0);
            if let Some(offset) = ring.offset_color {
                ring.offset_color = Some(Color {
                    a: (offset.a * ring_alpha.value).clamp(0.0, 1.0),
                    ..offset
                });
            }
            ring
        });

        root.kind = ElementKind::Container(ContainerProps {
            layout: root_layout,
            background: Some(resolved.background),
            shadow: Some(root_shadow),
            border: root_border,
            border_color: Some(border_motion.value),
            focus_ring: ring,
            focus_border_color: None,
            focus_within: false,
            focus_ring_always_paint: ring_alpha.animating || ring_alpha.value > 1e-4,
            corner_radii: root_corner_radii,
            ..Default::default()
        });

        let mut semantics = SemanticsDecoration::default().role(SemanticsRole::Group);
        if let Some(test_id) = test_id {
            semantics = semantics.test_id(test_id);
        }
        root.attach_semantics(semantics)
    }
}

pub fn input_group<H: UiHost>(group: InputGroup) -> impl IntoUiElement<H> + use<H> {
    group
}

/// Upstream `InputGroupAddon` alignment variants (shadcn/ui v4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputGroupAddonAlign {
    #[default]
    InlineStart,
    InlineEnd,
    BlockStart,
    BlockEnd,
}

/// shadcn/ui `InputGroupAddon` (v4).
///
/// In the upstream DOM implementation, addons can be aligned to any edge of the group
/// (`inline-start`, `inline-end`, `block-start`, `block-end`).
///
/// Fret's `InputGroup` recipe is slot-based (`leading/trailing/block_start/block_end`), so this
/// type acts as a part adapter that can be routed into the appropriate slot via
/// [`InputGroup::into_element_parts`].
#[derive(Debug)]
pub struct InputGroupAddon {
    align: InputGroupAddonAlign,
    children: Vec<AnyElement>,
    has_button: bool,
    has_kbd: bool,
}

impl InputGroupAddon {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            align: InputGroupAddonAlign::default(),
            children: children.into_iter().collect(),
            has_button: false,
            has_kbd: false,
        }
    }

    pub fn align(mut self, align: InputGroupAddonAlign) -> Self {
        self.align = align;
        self
    }

    /// Hint: addon subtree contains a `Button`.
    ///
    /// Upstream uses selectors like `has-[>button]:ml-[-0.45rem]`. Fret does not have selectors,
    /// so callers (or recipes) can provide this hint to match the same geometry.
    pub fn has_button(mut self, has_button: bool) -> Self {
        self.has_button = has_button;
        self
    }

    /// Hint: addon subtree contains a `Kbd`.
    ///
    /// Upstream uses selectors like `has-[>kbd]:ml-[-0.35rem]`. Fret does not have selectors, so
    /// callers (or recipes) can provide this hint to match the same geometry.
    pub fn has_kbd(mut self, has_kbd: bool) -> Self {
        self.has_kbd = has_kbd;
        self
    }

    pub(crate) fn into_parts(self) -> (InputGroupAddonAlign, Vec<AnyElement>, bool, bool) {
        (self.align, self.children, self.has_button, self.has_kbd)
    }
}

/// shadcn/ui `InputGroupInput` (v4).
///
/// Upstream wraps the `Input` component and applies `flex-1` and related classes.
/// In Fret the control is owned by the `InputGroup` recipe, so this type is a configuration part
/// that can be applied via [`InputGroup::into_element_parts`].
#[derive(Default, Clone)]
pub struct InputGroupInput {
    placeholder: Option<Arc<str>>,
    disabled: Option<bool>,
    aria_invalid: Option<bool>,
    a11y_label: Option<Arc<str>>,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    test_id: Option<Arc<str>>,
    on_key_down: Option<OnKeyDown>,
}

impl std::fmt::Debug for InputGroupInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputGroupInput")
            .field("placeholder", &self.placeholder.as_deref())
            .field("disabled", &self.disabled)
            .field("aria_invalid", &self.aria_invalid)
            .field("a11y_label", &self.a11y_label.as_deref())
            .field("submit_command", &self.submit_command)
            .field("cancel_command", &self.cancel_command)
            .field("test_id", &self.test_id.as_deref())
            .field("on_key_down", &self.on_key_down.is_some())
            .finish()
    }
}

impl InputGroupInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = Some(aria_invalid);
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn submit_command(mut self, command: CommandId) -> Self {
        self.submit_command = Some(command);
        self
    }

    /// Preferred action-first spelling for Enter submit dispatch.
    pub fn submit_action(self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.submit_command(action.into())
    }

    pub fn cancel_command(mut self, command: CommandId) -> Self {
        self.cancel_command = Some(command);
        self
    }

    /// Preferred action-first spelling for Escape cancel dispatch.
    pub fn cancel_action(self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.cancel_command(action.into())
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn on_key_down(mut self, handler: OnKeyDown) -> Self {
        self.on_key_down = Some(handler);
        self
    }
}

/// shadcn/ui `InputGroupTextarea` (v4).
///
/// Upstream wraps the `Textarea` component and applies `flex-1 resize-none` and related classes.
/// In Fret the control is owned by the `InputGroup` recipe, so this type is a configuration part
/// that can be applied via [`InputGroup::into_element_parts`].
#[derive(Default, Clone)]
pub struct InputGroupTextarea {
    placeholder: Option<Arc<str>>,
    disabled: Option<bool>,
    aria_invalid: Option<bool>,
    a11y_label: Option<Arc<str>>,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    test_id: Option<Arc<str>>,
    on_key_down: Option<OnKeyDown>,
    min_height: Option<Px>,
    max_height: Option<Px>,
}

impl std::fmt::Debug for InputGroupTextarea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputGroupTextarea")
            .field("placeholder", &self.placeholder.as_deref())
            .field("disabled", &self.disabled)
            .field("aria_invalid", &self.aria_invalid)
            .field("a11y_label", &self.a11y_label.as_deref())
            .field("submit_command", &self.submit_command)
            .field("cancel_command", &self.cancel_command)
            .field("test_id", &self.test_id.as_deref())
            .field("on_key_down", &self.on_key_down.is_some())
            .field("min_height", &self.min_height)
            .field("max_height", &self.max_height)
            .finish()
    }
}

impl InputGroupTextarea {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = Some(aria_invalid);
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn submit_command(mut self, command: CommandId) -> Self {
        self.submit_command = Some(command);
        self
    }

    /// Preferred action-first spelling for Enter submit dispatch.
    pub fn submit_action(self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.submit_command(action.into())
    }

    pub fn cancel_command(mut self, command: CommandId) -> Self {
        self.cancel_command = Some(command);
        self
    }

    /// Preferred action-first spelling for Escape cancel dispatch.
    pub fn cancel_action(self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.cancel_command(action.into())
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn on_key_down(mut self, handler: OnKeyDown) -> Self {
        self.on_key_down = Some(handler);
        self
    }

    pub fn min_height(mut self, min_height: Px) -> Self {
        self.min_height = Some(min_height);
        self
    }

    pub fn max_height(mut self, max_height: Px) -> Self {
        self.max_height = Some(max_height);
        self
    }
}

/// Part-based authoring surface aligned with shadcn/ui v4 exports.
#[derive(Debug)]
pub enum InputGroupPart {
    Addon(InputGroupAddon),
    Input(InputGroupInput),
    Textarea(InputGroupTextarea),
    CustomInput(AnyElement),
    CustomTextarea(AnyElement),
}

impl InputGroupPart {
    pub fn addon(addon: InputGroupAddon) -> Self {
        Self::Addon(addon)
    }

    pub fn input(input: InputGroupInput) -> Self {
        Self::Input(input)
    }

    pub fn textarea(textarea: InputGroupTextarea) -> Self {
        Self::Textarea(textarea)
    }

    pub fn custom_input(control: AnyElement) -> Self {
        Self::CustomInput(control)
    }

    pub fn custom_textarea(control: AnyElement) -> Self {
        Self::CustomTextarea(control)
    }
}

impl InputGroup {
    /// Part-based authoring adapter aligned with shadcn/ui v4.
    ///
    /// This routes `InputGroupAddon` alignments into the recipe's slot-based surface and applies
    /// `InputGroupInput` / `InputGroupTextarea` control configuration.
    ///
    /// Note: In the upstream DOM implementation, addons can click-to-focus the inner input.
    /// In Fret we approximate this by requesting focus for the control on left-button pointer
    /// down, skipping requests when the pointer-down hit-test target is (or is inside) a
    /// pressable subtree (e.g. an embedded button inside the addon).
    #[track_caller]
    pub fn into_element_parts<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        parts: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<InputGroupPart>,
    ) -> AnyElement {
        let mut group = self;
        let parts = parts(cx);

        let mut leading = std::mem::take(&mut group.leading);
        let mut trailing = std::mem::take(&mut group.trailing);
        let mut block_start = std::mem::take(&mut group.block_start);
        let mut block_end = std::mem::take(&mut group.block_end);

        let mut leading_has_button = group.leading_has_button;
        let mut trailing_has_button = group.trailing_has_button;
        let mut leading_has_kbd = group.leading_has_kbd;
        let mut trailing_has_kbd = group.trailing_has_kbd;

        for part in parts {
            match part {
                InputGroupPart::Addon(addon) => match addon.align {
                    InputGroupAddonAlign::InlineStart => {
                        leading_has_button |= addon.has_button;
                        leading_has_kbd |= addon.has_kbd;
                        leading.extend(addon.children);
                    }
                    InputGroupAddonAlign::InlineEnd => {
                        trailing_has_button |= addon.has_button;
                        trailing_has_kbd |= addon.has_kbd;
                        trailing.extend(addon.children);
                    }
                    InputGroupAddonAlign::BlockStart => {
                        block_start.extend(addon.children);
                    }
                    InputGroupAddonAlign::BlockEnd => {
                        block_end.extend(addon.children);
                    }
                },
                InputGroupPart::Input(input) => {
                    if let Some(placeholder) = input.placeholder {
                        group.placeholder = Some(placeholder);
                    }
                    if let Some(disabled) = input.disabled {
                        group.disabled = disabled;
                    }
                    if let Some(aria_invalid) = input.aria_invalid {
                        group.aria_invalid = aria_invalid;
                    }
                    if let Some(label) = input.a11y_label {
                        group.a11y_label = Some(label);
                    }
                    if let Some(cmd) = input.submit_command {
                        group.submit_command = Some(cmd);
                    }
                    if let Some(cmd) = input.cancel_command {
                        group.cancel_command = Some(cmd);
                    }
                    if let Some(test_id) = input.test_id {
                        group.control_test_id = Some(test_id);
                    }
                    if let Some(handler) = input.on_key_down {
                        group.control_on_key_down = Some(handler);
                    }
                }
                InputGroupPart::Textarea(textarea) => {
                    group.control = InputGroupControlKind::Textarea;

                    if let Some(placeholder) = textarea.placeholder {
                        group.placeholder = Some(placeholder);
                    }
                    if let Some(disabled) = textarea.disabled {
                        group.disabled = disabled;
                    }
                    if let Some(aria_invalid) = textarea.aria_invalid {
                        group.aria_invalid = aria_invalid;
                    }
                    if let Some(label) = textarea.a11y_label {
                        group.a11y_label = Some(label);
                    }
                    if let Some(cmd) = textarea.submit_command {
                        group.submit_command = Some(cmd);
                    }
                    if let Some(cmd) = textarea.cancel_command {
                        group.cancel_command = Some(cmd);
                    }
                    if let Some(test_id) = textarea.test_id {
                        group.control_test_id = Some(test_id);
                    }
                    if let Some(handler) = textarea.on_key_down {
                        group.control_on_key_down = Some(handler);
                    }
                    if let Some(min_h) = textarea.min_height {
                        group.textarea_min_height = min_h;
                    }
                    if let Some(max_h) = textarea.max_height {
                        group.textarea_max_height = Some(max_h);
                    }
                }
                InputGroupPart::CustomInput(control) => {
                    group.control = InputGroupControlKind::Input;
                    group.custom_control = Some(control);
                }
                InputGroupPart::CustomTextarea(control) => {
                    group.control = InputGroupControlKind::Textarea;
                    group.custom_control = Some(control);
                }
            }
        }

        group.leading = leading;
        group.trailing = trailing;
        group.block_start = block_start;
        group.block_end = block_end;
        group.leading_has_button = leading_has_button;
        group.trailing_has_button = trailing_has_button;
        group.leading_has_kbd = leading_has_kbd;
        group.trailing_has_kbd = trailing_has_kbd;

        group.into_element(cx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputGroupTextSize {
    #[default]
    Sm,
    Xs,
}

#[derive(Debug, Clone)]
pub struct InputGroupText {
    text: Arc<str>,
    size: InputGroupTextSize,
    layout: LayoutRefinement,
}

impl InputGroupText {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            size: InputGroupTextSize::Sm,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn size(mut self, size: InputGroupTextSize) -> Self {
        self.size = size;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let color = theme.color_token("muted-foreground");

        let (px, line_height) = match self.size {
            InputGroupTextSize::Sm => (
                theme.metric_token("metric.font.size"),
                theme.metric_token("metric.font.line_height"),
            ),
            // Tailwind: `text-xs leading-4`.
            InputGroupTextSize::Xs => (Px(12.0), Px(16.0)),
        };

        let mut style = typography::fixed_line_box_style(FontId::ui(), px, line_height);
        // Upstream `InputGroupAddon` sets `font-medium`, and `InputGroupText` inherits that
        // weight when rendered inside an addon.
        style.weight = FontWeight::MEDIUM;

        cx.text_props(TextProps {
            layout: decl_style::layout_style(theme, self.layout.h_px(line_height)),
            text: self.text,
            style: Some(style),
            color: Some(color),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputGroupButtonSize {
    #[default]
    Xs,
    Sm,
    IconXs,
    IconSm,
}

#[derive(Debug)]
pub struct InputGroupButton {
    label: Arc<str>,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    icon: Option<IconId>,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    command: Option<CommandId>,
    toggle_model: Option<Model<bool>>,
    disabled: bool,
    variant: ButtonVariant,
    size: InputGroupButtonSize,
    a11y_label: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl InputGroupButton {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
            test_id: None,
            icon: None,
            leading_icon: None,
            trailing_icon: None,
            command: None,
            toggle_model: None,
            disabled: false,
            variant: ButtonVariant::Ghost,
            size: InputGroupButtonSize::default(),
            a11y_label: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.children = children.into_iter().collect();
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    /// Sets an icon-only affordance rendered under the button's `currentColor` scope.
    ///
    /// When set, the label and `children` are ignored for the visual content (but can still be
    /// used as the accessibility label via `a11y_label`).
    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Adds a leading icon rendered under the button's `currentColor` scope.
    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// Adds a trailing icon rendered under the button's `currentColor` scope.
    pub fn trailing_icon(mut self, icon: IconId) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    /// Bind a stable action ID to this input-group button (action-first authoring).
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
    /// through the existing command pipeline.
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.command = Some(action.into());
        self
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Toggle an externally owned boolean model when the input-group button activates.
    ///
    /// This stays on the component/policy layer for trigger-style affordances and should not be
    /// read as precedent for reintroducing `toggle_model(...)` on the `imui` form-control lane.
    pub fn toggle_model(mut self, model: Model<bool>) -> Self {
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

    pub fn size(mut self, size: InputGroupButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let (
                bg,
                bg_hover,
                bg_active,
                _border_color,
                fg,
                _size_px,
                padding_x,
                gap,
                radius,
                text_px,
                line_height,
                pressable_layout,
            ) = {
                let theme = Theme::global(&*cx.app);
                let (bg, bg_hover, bg_active, border_color, fg) =
                    variant_colors(theme, self.variant);

                let (size_px, padding_x, gap, radius) = match self.size {
                    InputGroupButtonSize::Xs => (
                        Px(24.0),
                        fret_ui_kit::MetricRef::space(Space::N2).resolve(theme),
                        fret_ui_kit::MetricRef::space(Space::N1).resolve(theme),
                        Px((theme.metric_token("metric.radius.md").0 - 5.0).max(0.0)),
                    ),
                    InputGroupButtonSize::Sm => (
                        Px(32.0),
                        fret_ui_kit::MetricRef::space(Space::N2p5).resolve(theme),
                        fret_ui_kit::MetricRef::space(Space::N1p5).resolve(theme),
                        theme.metric_token("metric.radius.md"),
                    ),
                    InputGroupButtonSize::IconXs => (
                        Px(24.0),
                        Px(0.0),
                        Px(0.0),
                        Px((theme.metric_token("metric.radius.md").0 - 5.0).max(0.0)),
                    ),
                    InputGroupButtonSize::IconSm => (
                        Px(32.0),
                        Px(0.0),
                        Px(0.0),
                        theme.metric_token("metric.radius.md"),
                    ),
                };

                let text_px = theme.metric_token("metric.font.size");
                let line_height = theme.metric_token("metric.font.line_height");

                let mut layout = self.layout;
                layout = match self.size {
                    InputGroupButtonSize::IconXs | InputGroupButtonSize::IconSm => layout
                        .w_px(size_px)
                        .h_px(size_px)
                        .min_w(size_px)
                        .min_h(size_px),
                    _ => layout.min_h(size_px),
                };
                let pressable_layout = decl_style::layout_style(theme, layout);

                (
                    bg,
                    bg_hover,
                    bg_active,
                    border_color,
                    fg,
                    size_px,
                    padding_x,
                    gap,
                    radius,
                    text_px,
                    line_height,
                    pressable_layout,
                )
            };

            let command = self.command;
            let toggle_model = self.toggle_model;
            let disabled = self.disabled
                || command
                    .as_ref()
                    .is_some_and(|cmd| !cx.command_is_enabled(cmd));
            let user_chrome = self.chrome;
            let label = self.label;
            let a11y_label = self.a11y_label;
            let children = self.children;
            let test_id = self.test_id;
            let icon = self.icon;
            let leading_icon = self.leading_icon;
            let trailing_icon = self.trailing_icon;
            let fill_content_width = matches!(
                self.size,
                InputGroupButtonSize::IconXs | InputGroupButtonSize::IconSm
            );

            control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
                cx.pressable_dispatch_command_if_enabled_opt(command);
                if let Some(model) = toggle_model.clone() {
                    cx.pressable_toggle_bool(&model);
                }

                let hovered = st.hovered && !disabled;
                let pressed = st.pressed && !disabled;

                let bg = if pressed {
                    bg_active
                } else if hovered {
                    bg_hover
                } else {
                    bg
                };

                let (bg, fg) = {
                    let theme = Theme::global(&*cx.app).snapshot();

                    let bg = user_chrome
                        .background
                        .clone()
                        .map(|c| c.resolve(&theme))
                        .unwrap_or(bg);
                    let fg = user_chrome
                        .text_color
                        .clone()
                        .map(|c| c.resolve(&theme))
                        .unwrap_or(fg);
                    (bg, fg)
                };

                let mut pressable_props = PressableProps {
                    layout: pressable_layout,
                    enabled: !disabled,
                    focusable: !disabled,
                    focus_ring: None,
                    focus_ring_always_paint: false,
                    focus_ring_bounds: None,
                    key_activation: Default::default(),
                    a11y: PressableA11y {
                        role: Some(fret_core::SemanticsRole::Button),
                        label: Some(a11y_label.clone().unwrap_or_else(|| label.clone())),
                        test_id: test_id.clone(),
                        ..Default::default()
                    },
                };
                pressable_props.layout.overflow = Overflow::Visible;

                let chrome_props = ContainerProps {
                    layout: LayoutStyle {
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                    background: Some(bg),
                    corner_radii: Corners::all(radius),
                    padding: Edges {
                        top: Px(0.0),
                        right: padding_x,
                        bottom: Px(0.0),
                        left: padding_x,
                    }
                    .into(),
                    ..Default::default()
                };

                let content = move |cx: &mut ElementContext<'_, H>| {
                    let icon_px = Px(16.0);

                    let mut row = Vec::new();
                    if let Some(icon) = icon {
                        row.push(decl_icon::icon_with(cx, icon, Some(icon_px), None));
                    } else {
                        if let Some(icon) = leading_icon {
                            row.push(decl_icon::icon_with(cx, icon, Some(icon_px), None));
                        }

                        if !label.is_empty() {
                            let mut style = typography::fixed_line_box_style(
                                FontId::ui(),
                                text_px,
                                line_height,
                            );
                            style.weight = FontWeight::MEDIUM;
                            row.push(cx.text_props(TextProps {
                                layout: LayoutStyle {
                                    size: fret_ui::element::SizeStyle {
                                        width: Length::Auto,
                                        height: Length::Px(line_height),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                text: label,
                                style: Some(style),
                                color: Some(fg),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                                align: fret_core::TextAlign::Start,
                                ink_overflow: Default::default(),
                            }));
                        }
                        row.extend(children);

                        if let Some(icon) = trailing_icon {
                            row.push(decl_icon::icon_with(cx, icon, Some(icon_px), None));
                        }
                    }

                    vec![
                        cx.flex(
                            FlexProps {
                                layout: LayoutStyle {
                                    size: fret_ui::element::SizeStyle {
                                        width: if fill_content_width {
                                            Length::Fill
                                        } else {
                                            Length::Auto
                                        },
                                        height: Length::Fill,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                direction: Axis::Horizontal,
                                gap: gap.into(),
                                padding: Edges::all(Px(0.0)).into(),
                                justify: fret_ui::element::MainAlign::Center,
                                align: fret_ui::element::CrossAlign::Center,
                                wrap: false,
                            },
                            move |_cx| row,
                        )
                        .inherit_foreground(fg),
                    ]
                };

                (pressable_props, chrome_props, content)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, MaterialService, Modifiers, MouseButton, PathCommand, PathConstraints, PathId,
        PathMetrics, PathService, PathStyle, Point, Px, Rect, Size as CoreSize, SvgId, SvgService,
        TextBlobId, TextConstraints, TextInput, TextMetrics, TextService,
    };
    use fret_runtime::Model;
    use fret_ui::element::{ElementKind, TextInputProps, TextProps};
    use fret_ui::tree::UiTree;

    use crate::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(200.0)),
        )
    }

    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
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

    impl MaterialService for FakeServices {
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

    fn find_text_input<'a>(node: &'a AnyElement) -> Option<&'a TextInputProps> {
        match &node.kind {
            ElementKind::TextInput(props) => Some(props),
            _ => node.children.iter().find_map(find_text_input),
        }
    }

    fn find_text<'a>(node: &'a AnyElement, text: &str) -> Option<&'a TextProps> {
        match &node.kind {
            ElementKind::Text(props) if props.text.as_ref() == text => Some(props),
            _ => node.children.iter().find_map(|c| find_text(c, text)),
        }
    }

    fn find_flex_with_text_and_order(node: &AnyElement, text: &str, order: i32) -> bool {
        let matches = match &node.kind {
            ElementKind::Flex(FlexProps { layout, .. }) => {
                layout.flex.order == order && find_text(node, text).is_some()
            }
            ElementKind::PointerRegion(props) => {
                props.layout.flex.order == order && find_text(node, text).is_some()
            }
            _ => false,
        };
        if matches {
            return true;
        }
        node.children
            .iter()
            .any(|c| find_flex_with_text_and_order(c, text, order))
    }

    fn find_container_with_test_id<'a>(
        node: &'a AnyElement,
        test_id: &str,
    ) -> Option<&'a ContainerProps> {
        if let ElementKind::Container(props) = &node.kind {
            let matches = node
                .semantics_decoration
                .as_ref()
                .and_then(|d| d.test_id.as_deref())
                .is_some_and(|id| id == test_id);
            if matches {
                return Some(props);
            }
        }
        node.children
            .iter()
            .find_map(|c| find_container_with_test_id(c, test_id))
    }

    fn find_opacity(node: &AnyElement, opacity: f32) -> bool {
        let matches = match &node.kind {
            ElementKind::Opacity(props) => (props.opacity - opacity).abs() <= 1e-6,
            _ => false,
        };
        if matches {
            return true;
        }
        node.children.iter().any(|c| find_opacity(c, opacity))
    }

    fn find_foreground_scope_with_color(node: &AnyElement, color: Color) -> bool {
        let matches = node.inherited_foreground == Some(color)
            || match &node.kind {
                ElementKind::ForegroundScope(props) => props.foreground == Some(color),
                _ => false,
            };
        if matches {
            return true;
        }
        node.children
            .iter()
            .any(|c| find_foreground_scope_with_color(c, color))
    }

    fn find_first_inherited_foreground_node(node: &AnyElement) -> Option<&AnyElement> {
        if node.inherited_foreground.is_some() {
            return Some(node);
        }
        node.children
            .iter()
            .find_map(find_first_inherited_foreground_node)
    }

    fn contains_foreground_scope(node: &AnyElement) -> bool {
        matches!(node.kind, ElementKind::ForegroundScope(_))
            || node.children.iter().any(contains_foreground_scope)
    }

    #[test]
    fn input_group_parts_apply_placeholder_to_control() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "input_group_parts", |cx| {
            let model: Model<String> = cx.app.models_mut().insert(String::new());
            let el = InputGroup::new(model).into_element_parts(cx, |cx| {
                vec![
                    InputGroupPart::addon(
                        InputGroupAddon::new([cx.text("lead")])
                            .align(InputGroupAddonAlign::InlineStart),
                    ),
                    InputGroupPart::input(InputGroupInput::new().placeholder("placeholder")),
                ]
            });

            let props = find_text_input(&el).expect("expected text input in InputGroup");
            assert_eq!(props.placeholder.as_deref(), Some("placeholder"));
        });
    }

    #[test]
    fn input_group_custom_control_stamps_group_control_test_id_and_a11y_label() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "input_group_custom_control_test_id",
            |cx| {
                let model: Model<String> = cx.app.models_mut().insert(String::new());
                let custom = cx
                    .text_area(TextAreaProps::new(model.clone()))
                    .a11y_role(SemanticsRole::TextField);

                let el = InputGroup::new(model)
                    .custom_textarea(custom)
                    .control_test_id("input-group.custom.control")
                    .a11y_label("Custom control")
                    .into_element(cx);

                let mut found_test_id = false;
                let mut found_label = false;

                fn walk(node: &AnyElement, found_test_id: &mut bool, found_label: &mut bool) {
                    if let Some(sem) = node.semantics_decoration.as_ref() {
                        if sem.test_id.as_deref() == Some("input-group.custom.control") {
                            *found_test_id = true;
                        }
                        if sem.label.as_deref() == Some("Custom control") {
                            *found_label = true;
                        }
                    }
                    for child in &node.children {
                        walk(child, found_test_id, found_label);
                    }
                }

                walk(&el, &mut found_test_id, &mut found_label);
                assert!(
                    found_test_id,
                    "expected custom control to inherit control_test_id"
                );
                assert!(
                    found_label,
                    "expected custom control to inherit group a11y_label"
                );
            },
        );
    }

    #[test]
    fn input_group_parts_route_inline_addons_by_align_order() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "input_group_addons",
            |cx| {
                let model: Model<String> = cx.app.models_mut().insert(String::new());
                let el = InputGroup::new(model).into_element_parts(cx, |cx| {
                    vec![
                        InputGroupPart::addon(
                            InputGroupAddon::new([cx.text("lead")])
                                .align(InputGroupAddonAlign::InlineStart),
                        ),
                        InputGroupPart::addon(
                            InputGroupAddon::new([cx.text("trail")])
                                .align(InputGroupAddonAlign::InlineEnd),
                        ),
                        InputGroupPart::input(InputGroupInput::new()),
                    ]
                });

                assert!(
                    find_flex_with_text_and_order(&el, "lead", -1),
                    "expected inline-start addon flex to carry order=-1"
                );
                assert!(
                    find_flex_with_text_and_order(&el, "trail", 1),
                    "expected inline-end addon flex to carry order=1"
                );
            },
        );
    }

    #[test]
    fn input_group_stamps_group_role_even_without_test_id() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "input_group_group_role",
            |cx| {
                let model: Model<String> = cx.app.models_mut().insert(String::new());
                let el = InputGroup::new(model).into_element(cx);
                assert_eq!(
                    el.semantics_decoration.as_ref().and_then(|d| d.role),
                    Some(SemanticsRole::Group)
                );
            },
        );
    }

    #[test]
    fn input_group_root_keeps_recipe_width_default_but_allows_caller_override() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "input_group_root_width_ownership",
            |cx| {
                let intrinsic_model: Model<String> = cx.app.models_mut().insert(String::new());
                let fill = InputGroup::new(intrinsic_model)
                    .test_id("input_group.fill")
                    .into_element(cx);
                let fill_props = find_container_with_test_id(&fill, "input_group.fill")
                    .expect("expected InputGroup root container");
                assert_eq!(fill_props.layout.size.width, Length::Fill);
                assert_eq!(fill_props.layout.size.min_width, Some(Length::Px(Px(0.0))));

                let fixed_model: Model<String> = cx.app.models_mut().insert(String::new());
                let fixed = InputGroup::new(fixed_model)
                    .test_id("input_group.fixed")
                    .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                    .into_element(cx);
                let fixed_props = find_container_with_test_id(&fixed, "input_group.fixed")
                    .expect("expected InputGroup root container");
                assert_eq!(fixed_props.layout.size.width, Length::Px(Px(240.0)));
                assert_eq!(fixed_props.layout.size.min_width, Some(Length::Px(Px(0.0))));
            },
        );
    }

    #[test]
    fn input_group_inline_addon_click_to_focus_requests_focus_for_control() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = bounds();
        let model: Model<String> = app.models_mut().insert(String::new());

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-input-group-addon-click-to-focus",
            |cx| {
                vec![
                    InputGroup::new(model.clone())
                        .leading([cx.text("lead")])
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let group_node = ui.children(root)[0];
        let row_node = ui.children(group_node)[0];
        let row_children = ui.children(row_node);
        let control_node = row_children[0];
        let addon_node = row_children[1];

        assert_eq!(ui.focus(), None);

        let addon_bounds = ui.debug_node_bounds(addon_node).expect("addon bounds");
        let position = Point::new(
            Px(addon_bounds.origin.x.0 + addon_bounds.size.width.0 * 0.5),
            Px(addon_bounds.origin.y.0 + addon_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(ui.focus(), Some(control_node));
    }

    #[test]
    fn input_group_chrome_overlay_does_not_intercept_hit_test_for_control() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = bounds();
        let model: Model<String> = app.models_mut().insert(String::new());

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-input-group-chrome-hit-test",
            |cx| {
                vec![
                    InputGroup::new(model.clone())
                        .test_id("input_group")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let group_node = ui.children(root)[0];
        let row_node = ui.children(group_node)[0];
        let control_node = ui.children(row_node)[0];

        let control_bounds = ui.debug_node_bounds(control_node).expect("control bounds");
        let p = Point::new(
            Px(control_bounds.origin.x.0 + control_bounds.size.width.0 * 0.5),
            Px(control_bounds.origin.y.0 + control_bounds.size.height.0 * 0.5),
        );

        let hit = ui.debug_hit_test(p).hit;
        assert_eq!(
            hit,
            Some(control_node),
            "expected chrome overlay to be hit-test transparent (hit={hit:?} control={control_node:?} p={p:?})",
        );
    }

    #[test]
    fn input_group_disabled_addons_render_at_half_opacity_like_shadcn() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "input_group_disabled_addon_opacity",
            |cx| {
                let model: Model<String> = cx.app.models_mut().insert(String::new());
                let el = InputGroup::new(model)
                    .disabled(true)
                    .leading([cx.text("lead")])
                    .trailing([cx.text("trail")])
                    .into_element(cx);

                assert!(
                    find_opacity(&el, 0.5),
                    "expected disabled addons to include an opacity wrapper"
                );
            },
        );
    }

    #[test]
    fn input_group_addons_scope_muted_foreground_for_current_color_parity() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "input_group_addon_muted_foreground",
            |cx| {
                let theme = Theme::global(&*cx.app);
                let muted = theme.color_token("muted-foreground");

                let model: Model<String> = cx.app.models_mut().insert(String::new());
                let el = InputGroup::new(model)
                    .leading([crate::spinner::Spinner::new().into_element(cx)])
                    .into_element(cx);

                assert!(
                    find_foreground_scope_with_color(&el, muted),
                    "expected addons to stamp inherited foreground with muted-foreground"
                );
            },
        );
    }

    #[test]
    fn input_group_button_content_attaches_foreground_without_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "input_group_button_fg",
            |cx| {
                let theme = Theme::global(&*cx.app);
                let (_, _, _, _, expected_fg) = variant_colors(theme, ButtonVariant::Ghost);

                let el = InputGroupButton::new("Search")
                    .leading_icon(fret_icons::IconId::new_static("lucide.search"))
                    .trailing_icon(fret_icons::IconId::new_static("lucide.chevron-right"))
                    .into_element(cx);

                let inherited = find_first_inherited_foreground_node(&el)
                    .expect("expected input-group button subtree to carry inherited foreground");
                assert_eq!(inherited.inherited_foreground, Some(expected_fg));
                assert!(
                    !contains_foreground_scope(&el),
                    "expected input-group button content to attach inherited foreground without inserting a ForegroundScope"
                );
            },
        );
    }

    #[test]
    fn input_group_inline_addon_click_to_focus_skips_pressable_descendants() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = bounds();
        let model: Model<String> = app.models_mut().insert(String::new());
        let seen_pressable: Model<bool> = app.models_mut().insert(false);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-input-group-addon-click-to-focus-pressable-suppression",
            |cx| {
                let seen = seen_pressable.clone();
                let on_down = Arc::new(
                    move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                          _cx: fret_ui::action::ActionCx,
                          down: fret_ui::action::PointerDownCx| {
                        let _ = host.models_mut().update(&seen, |v: &mut bool| {
                            *v = down.hit_pressable_target.is_some()
                        });
                        false
                    },
                );

                let mut wrapper_props = fret_ui::element::PointerRegionProps::default();
                wrapper_props.layout.size.width = Length::Fill;
                wrapper_props.layout.size.height = Length::Fill;
                vec![cx.pointer_region(wrapper_props, move |cx| {
                    cx.pointer_region_on_pointer_down(on_down.clone());
                    vec![
                        InputGroup::new(model.clone())
                            .control_test_id("control")
                            .leading([
                                cx.text("lead"),
                                cx.pressable(
                                    fret_ui::element::PressableProps {
                                        a11y: fret_ui::element::PressableA11y {
                                            test_id: Some("addon.button".into()),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    |cx, _state| vec![cx.text("button")],
                                ),
                            ])
                            .leading_has_button(true)
                            .into_element(cx),
                    ]
                })]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let wrapper_node = ui.children(root)[0];
        let group_node = ui.children(wrapper_node)[0];
        let row_node = ui.children(group_node)[0];
        let row_children = ui.children(row_node);
        let control_node = row_children[0];
        let addon_node = row_children[1];

        let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
        let button_node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("addon.button"))
            .map(|n| n.id)
            .expect("expected addon button semantics node");

        let center_of = |bounds: fret_core::Rect| {
            Point::new(
                Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
                Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
            )
        };

        let button_bounds = ui.debug_node_bounds(button_node).expect("button bounds");
        let addon_bounds = ui.debug_node_bounds(addon_node).expect("addon bounds");
        let control_bounds = ui.debug_node_bounds(control_node).expect("control bounds");

        assert_eq!(ui.focus(), None);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: center_of(button_bounds),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: center_of(button_bounds),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
                is_click: true,
            }),
        );

        assert_eq!(
            app.models().get_copied(&seen_pressable),
            Some(true),
            "expected wrapper PointerRegion to observe hit_pressable_target=Some(..) when clicking the embedded pressable"
        );
        assert_ne!(ui.focus(), Some(control_node));

        let _ = app
            .models_mut()
            .update(&seen_pressable, |v: &mut bool| *v = false);

        let y_mid = Px(addon_bounds.origin.y.0 + addon_bounds.size.height.0 * 0.5);
        let x0 = addon_bounds.origin.x.0;
        let w = addon_bounds.size.width.0;
        let candidates = [
            Point::new(Px(x0 + 4.0), y_mid),
            Point::new(Px(x0 + w * 0.25), y_mid),
            Point::new(Px(x0 + w * 0.75), y_mid),
            Point::new(Px(x0 + w - 4.0), y_mid),
        ];
        let focus_position = candidates
            .into_iter()
            .find(|p| !button_bounds.contains(*p) && !control_bounds.contains(*p))
            .expect("expected a point in addon bounds outside the pressable bounds");

        let hit = ui
            .debug_hit_test(focus_position)
            .hit
            .expect("expected click-to-focus point to hit-test something");
        let hit_path = ui.debug_node_path(hit);
        assert!(
            hit_path.contains(&addon_node),
            "expected click-to-focus point to route through the addon subtree (hit={hit:?} path={hit_path:?} addon={addon_node:?} p={focus_position:?})"
        );
        assert!(
            !hit_path.contains(&button_node),
            "expected click-to-focus point to land outside the embedded pressable subtree (hit={hit:?} path={hit_path:?} button={button_node:?} p={focus_position:?})"
        );
        assert!(
            !hit_path.contains(&control_node),
            "expected click-to-focus point to land outside the control subtree (hit={hit:?} path={hit_path:?} control={control_node:?} p={focus_position:?})"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(1),
                position: focus_position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(
            app.models().get_copied(&seen_pressable),
            Some(false),
            "expected wrapper PointerRegion to observe hit_pressable_target=None when clicking outside the embedded pressable"
        );
        assert_eq!(ui.focus(), Some(control_node));
    }

    #[test]
    fn input_group_focus_ring_tweens_in_and_out_like_a_transition() {
        use std::cell::Cell;
        use std::rc::Rc;
        use std::time::Duration;

        use fret_core::{Event, FrameId, KeyCode, Rect, Size};
        use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;

        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(160.0)),
        );
        let mut services = FakeServices;
        let model: Model<String> = app.models_mut().insert(String::new());

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
                "input-group-focus-ring-tween",
                move |cx| {
                    let el = InputGroup::new(model)
                        .test_id("input_group")
                        .into_element(cx);

                    let root = find_container_with_test_id(&el, "input_group")
                        .expect("input group root container");

                    let a = root.focus_ring.map(|ring| ring.color.a).unwrap_or(0.0);
                    ring_alpha_out.set(Some(a));
                    always_paint_out.set(Some(root.focus_ring_always_paint));

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

        // Focus the inner control and enable focus-visible via a navigation key.
        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable input group control");
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
            fret_ui::focus_visible::is_focus_visible(&mut app, Some(window)),
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
}
