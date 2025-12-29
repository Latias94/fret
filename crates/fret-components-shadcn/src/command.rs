use std::sync::Arc;

use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::headless::roving_focus;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space};
use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::CommandId;
use fret_ui::Invalidation;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps, RowProps, TextProps,
};
use fret_ui::{ElementCx, Theme, UiHost};

use crate::{Input, ScrollArea};

fn border(theme: &Theme) -> Color {
    theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("input"))
        .unwrap_or(theme.colors.panel_border)
}

fn bg(theme: &Theme) -> Color {
    theme
        .color_by_key("popover")
        .or_else(|| theme.color_by_key("background"))
        .unwrap_or(theme.colors.surface_background)
}

fn item_bg_hover(theme: &Theme) -> Color {
    theme
        .color_by_key("accent")
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or(theme.colors.hover_background)
}

fn item_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.command.item.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("component.command.item.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or(theme.metrics.font_line_height);
    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::NORMAL,
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

#[derive(Clone)]
pub struct Command {
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl Command {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            children,
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let base = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .merge(ChromeRefinement {
                border_width: Some(MetricRef::Px(Px(1.0))),
                border_color: Some(ColorRef::Color(border(&theme))),
                background: Some(ColorRef::Color(bg(&theme))),
                ..Default::default()
            })
            .merge(self.chrome);

        let props = decl_style::container_props(&theme, base, self.layout);
        let children = self.children;
        cx.container(props, move |_cx| children)
    }
}

#[derive(Clone)]
pub struct CommandInput {
    model: fret_runtime::Model<String>,
    a11y_label: Option<Arc<str>>,
    disabled: bool,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for CommandInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandInput")
            .field("model", &"<model>")
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("disabled", &self.disabled)
            .field("layout", &self.layout)
            .finish()
    }
}

impl CommandInput {
    pub fn new(model: fret_runtime::Model<String>) -> Self {
        Self {
            model,
            a11y_label: None,
            disabled: false,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            cx.observe_model(self.model, Invalidation::Paint);

            let border = border(&theme);
            let disabled = self.disabled;
            let mut wrapper = decl_style::container_props(
                &theme,
                ChromeRefinement::default(),
                self.layout.merge(LayoutRefinement::default().w_full()),
            );
            wrapper.border = Edges {
                top: Px(0.0),
                right: Px(0.0),
                bottom: Px(1.0),
                left: Px(0.0),
            };
            wrapper.border_color = Some(border);

            let input = Input::new(self.model).a11y_label(
                self.a11y_label
                    .unwrap_or_else(|| Arc::from("Command input")),
            );

            cx.container(wrapper, move |cx| {
                let mut input = input.into_element(cx);
                if disabled {
                    input = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Generic,
                            disabled: true,
                            ..Default::default()
                        },
                        move |_cx| vec![input],
                    );
                }
                vec![input]
            })
        })
    }
}

#[derive(Clone)]
pub struct CommandItem {
    label: Arc<str>,
    disabled: bool,
    command: Option<CommandId>,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for CommandItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandItem")
            .field("label", &self.label.as_ref())
            .field("disabled", &self.disabled)
            .field("command", &self.command)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl CommandItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            label: label.clone(),
            disabled: false,
            command: None,
            children: Vec::new(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn children(mut self, children: Vec<AnyElement>) -> Self {
        self.children = children;
        self
    }
}

#[derive(Clone)]
pub struct CommandList {
    items: Vec<CommandItem>,
    disabled: bool,
    empty_text: Arc<str>,
    scroll: LayoutRefinement,
}

impl std::fmt::Debug for CommandList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandList")
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("empty_text", &self.empty_text.as_ref())
            .field("scroll", &self.scroll)
            .finish()
    }
}

impl CommandList {
    pub fn new(items: Vec<CommandItem>) -> Self {
        Self {
            items,
            disabled: false,
            empty_text: Arc::from("No results."),
            scroll: LayoutRefinement::default()
                .max_h(MetricRef::Px(Px(300.0)))
                .w_full()
                .min_w_0(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn empty_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.empty_text = text.into();
        self
    }

    pub fn refine_scroll_layout(mut self, layout: LayoutRefinement) -> Self {
        self.scroll = self.scroll.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let disabled = self.disabled;
        let items = self.items;

        // TODO(a11y): Switch to cmdk-style behavior (focus stays in the input) and drive highlight
        // changes via `TextInputProps::active_descendant` (ADR 0073). The current implementation
        // uses roving focus (moves focus between rows), which is a reasonable fallback but not the
        // desired long-term semantics for command palettes.
        if items.is_empty() {
            let empty = self.empty_text;
            let fg = theme.colors.text_muted;
            let text_style = item_text_style(&theme);
            return cx.container(ContainerProps::default(), move |cx| {
                vec![cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: empty,
                    style: Some(text_style),
                    color: Some(fg),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                })]
            });
        }

        let disabled_flags: Vec<bool> = items.iter().map(|i| disabled || i.disabled).collect();
        let tab_stop = roving_focus::first_enabled(&disabled_flags);

        let roving = RovingFocusProps {
            enabled: !disabled,
            wrap: true,
            disabled: Arc::from(disabled_flags.clone().into_boxed_slice()),
            ..Default::default()
        };

        let row_h = MetricRef::space(Space::N8).resolve(&theme);
        let row_gap = MetricRef::space(Space::N2).resolve(&theme);
        let pad_x = MetricRef::space(Space::N2).resolve(&theme);
        let pad_y = MetricRef::space(Space::N1).resolve(&theme);
        let radius = MetricRef::radius(Radius::Sm).resolve(&theme);
        let ring = decl_style::focus_ring(&theme, radius);
        let bg_hover = item_bg_hover(&theme);
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);
        let text_style = item_text_style(&theme);
        let item_layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .w_full()
                .min_h(MetricRef::Px(row_h))
                .min_w_0(),
        );

        let scroll = self.scroll;

        cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::List,
                ..Default::default()
            },
            move |cx| {
                vec![
                    ScrollArea::new(vec![cx.roving_flex(
                        RovingFlexProps {
                            flex: FlexProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.min_height = Some(Px(0.0));
                                    layout
                                },
                                direction: fret_core::Axis::Vertical,
                                gap: Px(0.0),
                                padding: Edges::all(Px(0.0)),
                                justify: MainAlign::Start,
                                align: CrossAlign::Stretch,
                                wrap: false,
                                ..Default::default()
                            },
                            roving,
                        },
                        move |cx| {
                            let mut out = Vec::with_capacity(items.len());

                            for (idx, item) in items.into_iter().enumerate() {
                                let enabled = !disabled_flags.get(idx).copied().unwrap_or(true);
                                let focusable = tab_stop.is_some_and(|i| i == idx);

                                let label = item.label.clone();
                                let command = item.command;
                                let children = item.children;

                                out.push(cx.pressable(
                                    PressableProps {
                                        layout: item_layout,
                                        enabled,
                                        focusable,
                                        on_click: command,
                                        focus_ring: Some(ring),
                                        a11y: PressableA11y {
                                            role: Some(SemanticsRole::ListItem),
                                            label: Some(label.clone()),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    move |cx, st| {
                                        let hovered = st.hovered && !st.pressed;
                                        let pressed = st.pressed;

                                        let bg = (hovered || pressed).then_some(bg_hover);
                                        let props = ContainerProps {
                                            layout: LayoutStyle::default(),
                                            padding: Edges {
                                                top: pad_y,
                                                right: pad_x,
                                                bottom: pad_y,
                                                left: pad_x,
                                            },
                                            background: bg,
                                            shadow: None,
                                            border: Edges::all(Px(0.0)),
                                            border_color: None,
                                            corner_radii: Corners::all(radius),
                                        };

                                        vec![cx.container(props, move |cx| {
                                            vec![cx.row(
                                                RowProps {
                                                    layout: LayoutStyle::default(),
                                                    gap: row_gap,
                                                    padding: Edges::all(Px(0.0)),
                                                    justify: MainAlign::SpaceBetween,
                                                    align: CrossAlign::Center,
                                                },
                                                move |cx| {
                                                    if children.is_empty() {
                                                        vec![cx.text_props(TextProps {
                                                            layout: LayoutStyle::default(),
                                                            text: label.clone(),
                                                            style: Some(text_style),
                                                            color: Some(fg),
                                                            wrap: TextWrap::None,
                                                            overflow: TextOverflow::Clip,
                                                        })]
                                                    } else {
                                                        children
                                                    }
                                                },
                                            )]
                                        })]
                                    },
                                ));
                            }

                            out
                        },
                    )])
                    .refine_layout(scroll)
                    .into_element(cx),
                ]
            },
        )
    }
}

pub fn command<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    Command::new(f(cx)).into_element(cx)
}
