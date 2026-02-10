use std::sync::Arc;

use crate::button::{ButtonVariant, variant_colors};
use fret_core::{
    Axis, Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle,
    TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, LayoutStyle, Length, Overflow, PressableA11y,
    PressableProps, SemanticsProps, TextAreaProps, TextInputProps, TextProps,
};
use fret_ui::{ElementContext, TextAreaStyle, TextInputStyle, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
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

#[derive(Clone)]
pub struct InputGroup {
    model: Model<String>,
    control: InputGroupControlKind,
    test_id: Option<Arc<str>>,
    control_test_id: Option<Arc<str>>,
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let is_block_layout = !self.block_start.is_empty() || !self.block_end.is_empty();

        let (
            resolved,
            addon_pl,
            addon_py,
            compact_px,
            textarea_py,
            text_style,
            root_layout,
            root_shadow,
            border_color,
            focus_border_color,
            focus_ring,
        ) = {
            let theme = Theme::global(&*cx.app);

            let resolved =
                resolve_input_chrome(theme, self.size, &self.chrome, InputTokenKeys::none());

            let addon_pl = fret_ui_kit::MetricRef::space(Space::N3).resolve(theme);
            let addon_py = fret_ui_kit::MetricRef::space(Space::N1p5).resolve(theme);
            let compact_px = fret_ui_kit::MetricRef::space(Space::N2).resolve(theme);
            let textarea_py = fret_ui_kit::MetricRef::space(Space::N3).resolve(theme);

            let font_line_height = theme.metric_required("font.line_height");
            let text_style = TextStyle {
                font: FontId::default(),
                size: resolved.text_px,
                line_height: Some(font_line_height),
                ..Default::default()
            };

            let root_layout = decl_style::layout_style(theme, {
                let mut root = self.layout.relative().w_full().min_w_0();
                if !is_block_layout && self.control == InputGroupControlKind::Input {
                    root = root.h_px(resolved.min_height);
                }
                root
            });

            let root_shadow = decl_style::shadow_xs(theme, resolved.radius);

            let (border_color, focus_border_color, focus_ring) = {
                let mut ring = decl_style::focus_ring(theme, resolved.radius);
                let focus_border = Some(resolved.border_color_focused);

                if self.aria_invalid {
                    let border_color = theme.color_required("destructive");
                    let ring_key = if theme.name.contains("/dark") {
                        "destructive/40"
                    } else {
                        "destructive/20"
                    };
                    ring.color = theme
                        .color_by_key(ring_key)
                        .or_else(|| theme.color_by_key("destructive/20"))
                        .unwrap_or(border_color);
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
                text_style,
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
        let textarea_min_height = self.textarea_min_height;
        let test_id = self.test_id;
        let control_test_id = self.control_test_id;
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
                if is_block_layout {
                    let control_el = match control {
                        InputGroupControlKind::Input => {
                            let left_pad = if leading.is_empty() {
                                resolved.padding.left
                            } else {
                                compact_px
                            };
                            let right_pad = if trailing.is_empty() {
                                resolved.padding.right
                            } else {
                                compact_px
                            };

                            let mut chrome =
                                TextInputStyle::from_theme(Theme::global(&*cx.app).snapshot());
                            chrome.padding = Edges {
                                top: resolved.padding.top,
                                right: right_pad,
                                bottom: resolved.padding.bottom,
                                left: left_pad,
                            };
                            chrome.corner_radii = Corners::all(Px(0.0));
                            chrome.border = Edges::all(Px(0.0));
                            chrome.background = Color::TRANSPARENT;
                            chrome.border_color = resolved.border_color;
                            chrome.border_color_focused = resolved.border_color_focused;
                            chrome.focus_ring = None;
                            chrome.text_color = resolved.text_color;
                            chrome.caret_color = resolved.text_color;
                            chrome.selection_color = resolved.selection_color;

                            let mut input = TextInputProps::new(model.clone());
                            input.a11y_label = a11y_label.clone();
                            input.test_id = control_test_id.clone();
                            input.submit_command = submit_command;
                            input.cancel_command = cancel_command;
                            input.chrome = chrome;
                            input.text_style = text_style.clone();
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
                            chrome.padding_x = resolved.padding.left;
                            chrome.padding_y = textarea_py;
                            chrome.background = Color::TRANSPARENT;
                            chrome.border = Edges::all(Px(0.0));
                            chrome.border_color = resolved.border_color;
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
                            props.chrome = chrome;
                            props.text_style = text_style.clone();
                            props.min_height = textarea_min_height;
                            props.layout = {
                                let theme = Theme::global(&*cx.app);
                                decl_style::layout_style(
                                    theme,
                                    LayoutRefinement::default().w_full().min_w_0(),
                                )
                            };
                            cx.text_area(props)
                        }
                    };

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
                                    LayoutRefinement::default().w_full().min_w_0(),
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
                                vec![cx.flex(
                                    FlexProps {
                                        layout: LayoutStyle::default(),
                                        direction: Axis::Horizontal,
                                        gap,
                                        padding: Edges {
                                            top: pt,
                                            right: px_3,
                                            bottom: pb,
                                            left: px_3,
                                        },
                                        justify: fret_ui::element::MainAlign::Start,
                                        align: fret_ui::element::CrossAlign::Center,
                                        wrap: false,
                                    },
                                    move |_cx| block_start,
                                )]
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
                                    LayoutRefinement::default().w_full().min_w_0(),
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
                                vec![cx.flex(
                                    FlexProps {
                                        layout: LayoutStyle::default(),
                                        direction: Axis::Horizontal,
                                        gap,
                                        padding: Edges {
                                            top: pt,
                                            right: px_3,
                                            bottom: pb,
                                            left: px_3,
                                        },
                                        justify: fret_ui::element::MainAlign::Start,
                                        align: fret_ui::element::CrossAlign::Center,
                                        wrap: false,
                                    },
                                    move |_cx| block_end,
                                )]
                            },
                        )
                    });

                    vec![cx.flex(
                        FlexProps {
                            layout: {
                                let theme = Theme::global(&*cx.app);
                                decl_style::layout_style(
                                    theme,
                                    LayoutRefinement::default().size_full(),
                                )
                            },
                            direction: Axis::Vertical,
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
                            justify: fret_ui::element::MainAlign::Start,
                            align: fret_ui::element::CrossAlign::Stretch,
                            wrap: false,
                        },
                        move |_cx| {
                            let mut children = Vec::new();
                            if let Some(block_start) = block_start {
                                children.push(block_start);
                            }
                            children.push(control_el);
                            if let Some(block_end) = block_end {
                                children.push(block_end);
                            }
                            children
                        },
                    )]
                } else {
                    let left_pad = if leading.is_empty() {
                        resolved.padding.left
                    } else {
                        compact_px
                    };
                    let right_pad = if trailing.is_empty() {
                        resolved.padding.right
                    } else {
                        compact_px
                    };

                    let mut chrome = TextInputStyle::from_theme(Theme::global(&*cx.app).snapshot());
                    chrome.padding = Edges {
                        top: resolved.padding.top,
                        right: right_pad,
                        bottom: resolved.padding.bottom,
                        left: left_pad,
                    };
                    chrome.corner_radii = Corners::all(Px(0.0));
                    chrome.border = Edges::all(Px(0.0));
                    chrome.background = Color::TRANSPARENT;
                    chrome.border_color = resolved.border_color;
                    chrome.border_color_focused = resolved.border_color_focused;
                    chrome.focus_ring = None;
                    chrome.text_color = resolved.text_color;
                    chrome.caret_color = resolved.text_color;
                    chrome.selection_color = resolved.selection_color;

                    let mut input = TextInputProps::new(model);
                    input.a11y_label = a11y_label;
                    input.test_id = control_test_id.clone();
                    input.submit_command = submit_command;
                    input.cancel_command = cancel_command;
                    input.chrome = chrome;
                    input.text_style = text_style;
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

                    let flex_layout = {
                        let theme = Theme::global(&*cx.app);
                        decl_style::layout_style(theme, LayoutRefinement::default().size_full())
                    };

                    let leading = (!leading.is_empty()).then(|| {
                        let layout = if leading_has_button {
                            LayoutRefinement::default().flex_none().ml_neg(Space::N2)
                        } else if leading_has_kbd {
                            LayoutRefinement::default().flex_none().ml_neg(Space::N1p5)
                        } else {
                            LayoutRefinement::default().flex_none()
                        };
                        let (layout, gap) = {
                            let theme = Theme::global(&*cx.app);
                            (
                                decl_style::layout_style(theme, layout),
                                fret_ui_kit::MetricRef::space(Space::N2).resolve(theme),
                            )
                        };
                        cx.flex(
                            FlexProps {
                                layout,
                                direction: Axis::Horizontal,
                                gap,
                                padding: Edges {
                                    top: addon_py,
                                    right: Px(0.0),
                                    bottom: addon_py,
                                    left: addon_pl,
                                },
                                justify: fret_ui::element::MainAlign::Center,
                                align: fret_ui::element::CrossAlign::Center,
                                wrap: false,
                            },
                            |_cx| leading,
                        )
                    });

                    let trailing = (!trailing.is_empty()).then(|| {
                        let layout = if trailing_has_button {
                            LayoutRefinement::default().flex_none().mr_neg(Space::N2)
                        } else if trailing_has_kbd {
                            LayoutRefinement::default().flex_none().mr_neg(Space::N1p5)
                        } else {
                            LayoutRefinement::default().flex_none()
                        };
                        let (layout, gap) = {
                            let theme = Theme::global(&*cx.app);
                            (
                                decl_style::layout_style(theme, layout),
                                fret_ui_kit::MetricRef::space(Space::N2).resolve(theme),
                            )
                        };
                        cx.flex(
                            FlexProps {
                                layout,
                                direction: Axis::Horizontal,
                                gap,
                                padding: Edges {
                                    top: addon_py,
                                    right: addon_pl,
                                    bottom: addon_py,
                                    left: Px(0.0),
                                },
                                justify: fret_ui::element::MainAlign::Center,
                                align: fret_ui::element::CrossAlign::Center,
                                wrap: false,
                            },
                            |_cx| trailing,
                        )
                    });

                    let input = cx.text_input(input);

                    vec![cx.flex(
                        FlexProps {
                            layout: flex_layout,
                            direction: Axis::Horizontal,
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
                            justify: fret_ui::element::MainAlign::Start,
                            align: fret_ui::element::CrossAlign::Center,
                            wrap: false,
                        },
                        move |_cx| {
                            let mut children = Vec::new();
                            if let Some(leading) = leading {
                                children.push(leading);
                            }
                            children.push(input);
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

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![root],
        )
    }
}

pub fn input_group<H: UiHost>(cx: &mut ElementContext<'_, H>, group: InputGroup) -> AnyElement {
    group.into_element(cx)
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let color = theme.color_required("muted-foreground");

        let (px, line_height) = match self.size {
            InputGroupTextSize::Sm => (
                theme.metric_required("metric.font.size"),
                theme.metric_required("metric.font.line_height"),
            ),
            // Tailwind: `text-xs leading-4`.
            InputGroupTextSize::Xs => (Px(12.0), Px(16.0)),
        };

        cx.text_props(TextProps {
            layout: decl_style::layout_style(theme, self.layout.h_px(line_height)),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(color),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
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

#[derive(Debug, Clone)]
pub struct InputGroupButton {
    label: Arc<str>,
    children: Vec<AnyElement>,
    command: Option<CommandId>,
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
            command: None,
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

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
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
                        Px((theme.metric_required("metric.radius.md").0 - 5.0).max(0.0)),
                    ),
                    InputGroupButtonSize::Sm => (
                        Px(32.0),
                        fret_ui_kit::MetricRef::space(Space::N2p5).resolve(theme),
                        fret_ui_kit::MetricRef::space(Space::N1p5).resolve(theme),
                        theme.metric_required("metric.radius.md"),
                    ),
                    InputGroupButtonSize::IconXs => (
                        Px(24.0),
                        Px(0.0),
                        Px(0.0),
                        Px((theme.metric_required("metric.radius.md").0 - 5.0).max(0.0)),
                    ),
                    InputGroupButtonSize::IconSm => (
                        Px(32.0),
                        Px(0.0),
                        Px(0.0),
                        theme.metric_required("metric.radius.md"),
                    ),
                };

                let text_px = theme.metric_required("metric.font.size");
                let line_height = theme.metric_required("metric.font.line_height");

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
            let disabled = self.disabled
                || command
                    .as_ref()
                    .is_some_and(|cmd| !cx.command_is_enabled(cmd));
            let _chrome = self.chrome;
            let label = self.label;
            let a11y_label = self.a11y_label;
            let children = self.children;
            let fill_content_width = matches!(
                self.size,
                InputGroupButtonSize::IconXs | InputGroupButtonSize::IconSm
            );

            control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
                cx.pressable_dispatch_command_if_enabled_opt(command);

                let hovered = st.hovered && !disabled;
                let pressed = st.pressed && !disabled;

                let bg = if pressed {
                    bg_active
                } else if hovered {
                    bg_hover
                } else {
                    bg
                };

                let mut pressable_props = PressableProps {
                    layout: pressable_layout,
                    enabled: !disabled,
                    focusable: !disabled,
                    focus_ring: None,
                    focus_ring_bounds: None,
                    a11y: PressableA11y {
                        role: Some(fret_core::SemanticsRole::Button),
                        label: Some(a11y_label.clone().unwrap_or_else(|| label.clone())),
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
                    },
                    ..Default::default()
                };

                let content = move |cx: &mut ElementContext<'_, H>| {
                    let mut row = Vec::new();
                    if !label.is_empty() {
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
                            style: Some(TextStyle {
                                font: FontId::default(),
                                size: text_px,
                                weight: FontWeight::MEDIUM,
                                slant: Default::default(),
                                line_height: Some(line_height),
                                letter_spacing_em: None,
                            }),
                            color: Some(fg),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        }));
                    }
                    row.extend(children);

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
                            gap,
                            padding: Edges::all(Px(0.0)),
                            justify: fret_ui::element::MainAlign::Center,
                            align: fret_ui::element::CrossAlign::Center,
                            wrap: false,
                        },
                        move |_cx| row,
                    )]
                };

                (pressable_props, chrome_props, content)
            })
        })
    }
}
