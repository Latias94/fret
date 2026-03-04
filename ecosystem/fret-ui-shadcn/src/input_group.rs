use std::sync::Arc;

use crate::button::{ButtonVariant, variant_colors};
use crate::rtl;
use fret_core::{
    Axis, Color, Corners, Edges, FontId, FontWeight, MouseButton, Px, SemanticsRole, TextOverflow,
    TextWrap,
};
use fret_icons::IconId;
use fret_runtime::{CommandId, Model};
use fret_ui::action::{OnKeyDown, UiPointerActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, LayoutStyle, Length, Overflow, PointerRegionProps,
    PressableA11y, PressableProps, SemanticsDecoration, TextAreaProps, TextInputProps, TextProps,
};
use fret_ui::{ElementContext, TextAreaStyle, TextInputStyle, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::current_color;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Size as ComponentSize, Space};

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
    test_id: Option<Arc<str>>,
    control_test_id: Option<Arc<str>>,
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
            .field("test_id", &self.test_id.as_deref())
            .field("control_test_id", &self.control_test_id.as_deref())
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
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            control: InputGroupControlKind::Input,
            test_id: None,
            control_test_id: None,
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

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn control_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.control_test_id = Some(id.into());
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

    pub fn cancel_command(mut self, command: CommandId) -> Self {
        self.cancel_command = Some(command);
        self
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
                let mut root = self.layout.relative().w_full().min_w_0();
                if !is_block_layout && self.control == InputGroupControlKind::Input {
                    root = root.h_px(resolved.min_height);
                }
                root
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
        let control = self.control;
        let a11y_label = self.a11y_label;
        let submit_command = self.submit_command;
        let cancel_command = self.cancel_command;
        let model = self.model;
        let placeholder = self.placeholder;
        let textarea_min_height = self.textarea_min_height;
        let textarea_max_height = self.textarea_max_height;
        let test_id = self.test_id;
        let control_test_id = self.control_test_id;
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

        let root = cx.container(
            fret_ui::element::ContainerProps {
                layout: root_layout,
                background: None,
                shadow: Some(root_shadow),
                border: root_border,
                border_color: Some(border_color),
                focus_ring,
                focus_border_color,
                focus_within: true,
                corner_radii: root_corner_radii,
                ..Default::default()
            },
            |cx| {
                let dir = crate::use_direction(cx, None);
                let build_inline_addon = |cx: &mut ElementContext<'_, H>,
                                          children: Vec<AnyElement>,
                                          is_start: bool,
                                          has_button: bool,
                                          has_kbd: bool,
                                          control_focus_target: Option<
                    fret_ui::elements::GlobalElementId,
                >| {
                    let (order_inline_start, order_inline_end) =
                        rtl::inline_start_end_pair(dir, -1, 1);
                    let order = if is_start {
                        order_inline_start
                    } else {
                        order_inline_end
                    };
                    let mut layout = LayoutRefinement::default().flex_none().order(order);
                    if has_button {
                        layout = if is_start {
                            rtl::layout_refinement_apply_margin_inline_start_neg(
                                layout,
                                dir,
                                Space::N2,
                            )
                        } else {
                            rtl::layout_refinement_apply_margin_inline_end_neg(
                                layout,
                                dir,
                                Space::N2,
                            )
                        };
                    } else if has_kbd {
                        layout = if is_start {
                            rtl::layout_refinement_apply_margin_inline_start_neg(
                                layout,
                                dir,
                                Space::N1p5,
                            )
                        } else {
                            rtl::layout_refinement_apply_margin_inline_end_neg(
                                layout,
                                dir,
                                Space::N1p5,
                            )
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

                    let should_click_to_focus = control_focus_target.is_some() && !has_button;

                    if should_click_to_focus {
                        let control_focus_target =
                            control_focus_target.expect("control_focus_target");

                        let on_down = Arc::new(
                                move |host: &mut dyn UiPointerActionHost,
                                      _cx: fret_ui::action::ActionCx,
                                      down: fret_ui::action::PointerDownCx| {
                                    if down.button == MouseButton::Left {
                                        host.request_focus(control_focus_target);
                                    }
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
                                capture_phase_pointer_moves: false,
                            },
                            move |cx| {
                                cx.pointer_region_on_pointer_down(on_down);
                                vec![flex]
                            },
                        )
                    } else {
                        cx.flex(
                            FlexProps {
                                layout,
                                direction: Axis::Horizontal,
                                gap: gap.into(),
                                padding: padding.into(),
                                justify: fret_ui::element::MainAlign::Center,
                                align: fret_ui::element::CrossAlign::Center,
                                wrap: false,
                            },
                            |_cx| children,
                        )
                    }
                };

                if is_block_layout {
                    let control_el = match control {
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
                    };

                    let control_id = control_el.id;
                    let control_focus_target = (!disabled).then_some(control_id);
                    if let Some(handler) = control_on_key_down {
                        // Run before the control's internal key handling so callers can
                        // consume keys like Enter/Backspace and prevent default behavior.
                        cx.key_prepend_on_key_down_for(control_id, handler);
                    }

                    let inline_start = (control == InputGroupControlKind::Input
                        && !leading.is_empty())
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

                    let inline_end = (control == InputGroupControlKind::Input
                        && !trailing.is_empty())
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
                        decl_style::layout_style(
                            theme,
                            LayoutRefinement::default().w_full().min_w_0(),
                        )
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
                                vec![
                                    cx.flex(
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
                                    ),
                                ]
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
                                vec![
                                    cx.flex(
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
                                    ),
                                ]
                            },
                        )
                    });

                    vec![cx.flex(
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
                    )]
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

                    let control_el = cx.text_input(input);
                    let control_focus_target = (!disabled).then_some(control_el.id);

                    if let Some(handler) = control_on_key_down {
                        // Run before the control's internal key handling so callers can
                        // consume keys like Enter/Backspace and prevent default behavior.
                        cx.key_prepend_on_key_down_for(control_el.id, handler);
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

                    vec![cx.flex(
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
                    )]
                }
            },
        );

        let Some(test_id) = test_id else {
            return root;
        };

        root.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

pub fn input_group<H: UiHost>(cx: &mut ElementContext<'_, H>, group: InputGroup) -> AnyElement {
    group.into_element(cx)
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

    pub fn cancel_command(mut self, command: CommandId) -> Self {
        self.cancel_command = Some(command);
        self
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

    pub fn cancel_command(mut self, command: CommandId) -> Self {
        self.cancel_command = Some(command);
        self
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
}

impl InputGroup {
    /// Part-based authoring adapter aligned with shadcn/ui v4.
    ///
    /// This routes `InputGroupAddon` alignments into the recipe's slot-based surface and applies
    /// `InputGroupInput` / `InputGroupTextarea` control configuration.
    ///
    /// Note: In the upstream DOM implementation, addons can click-to-focus the inner input.
    /// In Fret we approximate this by requesting focus for the control on left-button pointer
    /// down for non-button addons (suppressed when the addon hints `has_button=true`).
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
        style.weight = FontWeight::NORMAL;

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

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

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
                    current_color::scope_children(cx, fret_ui_kit::ColorRef::Color(fg), |cx| {
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

                        vec![cx.flex(
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
                        )]
                    })
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
    fn input_group_inline_addon_with_button_hint_does_not_steal_focus() {
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
            "shadcn-input-group-addon-click-to-focus-suppressed-by-hint",
            |cx| {
                vec![
                    InputGroup::new(model.clone())
                        .leading([cx.text("lead")])
                        .leading_has_button(true)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let group_node = ui.children(root)[0];
        let row_node = ui.children(group_node)[0];
        let row_children = ui.children(row_node);
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

        assert_eq!(ui.focus(), None);
    }
}
