use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

use fret_components_ui::declarative::action_hooks::ActionHooksExt as _;
use fret_components_ui::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::headless::cmdk_selection;
use fret_components_ui::headless::roving_focus;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space};
use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, KeyCode, Px, SemanticsRole, TextOverflow, TextStyle,
    TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::Invalidation;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps, RowProps, TextProps,
};
use fret_ui::{ElementCx, Theme, UiHost};

use crate::{Dialog, DialogContent, Input, ScrollArea};

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

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
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
            cx.observe_model(&self.model, Invalidation::Paint);

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

        // Note: `CommandList` is a simple list rendering helper (legacy roving-style semantics).
        // `CommandPalette` is the cmdk-style implementation that keeps focus in the input and
        // drives highlight via `active_descendant` (ADR 0073).
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
                                let text_style = text_style.clone();

                                out.push(cx.pressable(
                                    PressableProps {
                                        layout: item_layout,
                                        enabled,
                                        focusable,
                                        focus_ring: Some(ring),
                                        a11y: PressableA11y {
                                            role: Some(SemanticsRole::ListItem),
                                            label: Some(label.clone()),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    move |cx, st| {
                                        cx.pressable_dispatch_command_opt(command);
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
                                                            style: Some(text_style.clone()),
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

#[derive(Clone)]
pub struct CommandPalette {
    model: Model<String>,
    items: Vec<CommandItem>,
    disabled: bool,
    wrap: bool,
    empty_text: Arc<str>,
    a11y_label: Arc<str>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    scroll: LayoutRefinement,
}

impl std::fmt::Debug for CommandPalette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandPalette")
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("wrap", &self.wrap)
            .field("empty_text", &self.empty_text.as_ref())
            .field("a11y_label", &self.a11y_label.as_ref())
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("scroll", &self.scroll)
            .finish()
    }
}

impl CommandPalette {
    pub fn new(model: Model<String>, items: Vec<CommandItem>) -> Self {
        Self {
            model,
            items,
            disabled: false,
            wrap: true,
            empty_text: Arc::from("No results."),
            a11y_label: Arc::from("Command input"),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
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

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn empty_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.empty_text = text.into();
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = label.into();
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

    pub fn refine_scroll_layout(mut self, layout: LayoutRefinement) -> Self {
        self.scroll = self.scroll.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        #[derive(Clone)]
        struct PaletteEntry {
            command: Option<CommandId>,
            disabled: bool,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        enum RowKey {
            Command(CommandId),
            Label(Arc<str>),
        }

        struct KeyHandlerState {
            disabled: Rc<Cell<bool>>,
            wrap: Rc<Cell<bool>>,
            entries: Rc<RefCell<Arc<[PaletteEntry]>>>,
            handler: fret_ui::action::OnKeyDown,
        }

        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let disabled = self.disabled;
            let wrap = self.wrap;
            let items = self.items;

            let items_fingerprint = {
                let mut hasher = DefaultHasher::new();
                items.len().hash(&mut hasher);
                for item in &items {
                    item.label.as_ref().hash(&mut hasher);
                    item.disabled.hash(&mut hasher);
                    item.command
                        .as_ref()
                        .map(|c| c.as_str())
                        .unwrap_or("")
                        .hash(&mut hasher);
                }
                hasher.finish()
            };

            let (entries, disabled_flags): (Vec<PaletteEntry>, Vec<bool>) = items
                .iter()
                .map(|i| {
                    let disabled = disabled || i.disabled || i.command.is_none();
                    (
                        PaletteEntry {
                            command: i.command.clone(),
                            disabled,
                        },
                        disabled,
                    )
                })
                .unzip();
            let entries_arc: Arc<[PaletteEntry]> = Arc::from(entries.into_boxed_slice());

            let active = cx.with_state(CommandPaletteState::default, |st| st.active.clone());
            let active = if let Some(active) = active {
                active
            } else {
                let init = cmdk_selection::clamp_active_index(&disabled_flags, None);
                let active = cx.app.models_mut().insert(init);
                cx.with_state(CommandPaletteState::default, |st| {
                    st.active = Some(active.clone())
                });
                active
            };

            cx.observe_model(&active, Invalidation::Paint);

            let items_changed = cx.with_state(CommandPaletteState::default, |st| {
                if st.items_fingerprint != items_fingerprint {
                    st.items_fingerprint = items_fingerprint;
                    true
                } else {
                    false
                }
            });

            let cur_active = cx.app.models().get_copied(&active).unwrap_or(None);
            let next_active = if items_changed {
                cmdk_selection::clamp_active_index(&disabled_flags, None)
            } else {
                cmdk_selection::clamp_active_index(&disabled_flags, cur_active)
            };
            if next_active != cur_active {
                let _ = cx.app.models_mut().update(&active, |v| *v = next_active);
            }

            let mut row_ids: Vec<fret_ui::elements::GlobalElementId> =
                Vec::with_capacity(items.len());

            let row_h = MetricRef::space(Space::N8).resolve(&theme);
            let row_gap = MetricRef::space(Space::N2).resolve(&theme);
            let pad_x = MetricRef::space(Space::N2).resolve(&theme);
            let pad_y = MetricRef::space(Space::N1).resolve(&theme);
            let radius = MetricRef::radius(Radius::Sm).resolve(&theme);

            let bg_hover = item_bg_hover(&theme);
            let bg_selected = alpha_mul(bg_hover, 0.85);
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

            let mut key_counts: HashMap<RowKey, u32> = HashMap::new();

            let active_idx = cx.app.models().get_copied(&active).unwrap_or(None);
            let item_count = items.len();
            let rows: Vec<AnyElement> = items
                .into_iter()
                .enumerate()
                .map(|(idx, item)| {
                    let base = item
                        .command
                        .clone()
                        .map(RowKey::Command)
                        .unwrap_or_else(|| RowKey::Label(item.label.clone()));
                    let count = key_counts.entry(base.clone()).or_insert(0);
                    let occ = *count;
                    *count = count.saturating_add(1);

                    let active_for_row = active.clone();
                    cx.keyed((base, occ), |cx| {
                        let enabled = disabled_flags.get(idx).copied() == Some(false);
                        let selected = active_idx.is_some_and(|i| i == idx);

                        let label = item.label.clone();
                        let command = item.command;
                        let children = item.children;
                        let text_style = text_style.clone();

                        let row = cx.pressable(
                            PressableProps {
                                layout: item_layout,
                                enabled,
                                focusable: false,
                                focus_ring: None,
                                a11y: PressableA11y {
                                    role: Some(SemanticsRole::ListItem),
                                    label: Some(label.clone()),
                                    selected,
                                    ..Default::default()
                                }
                                .with_collection_position(idx, item_count),
                                ..Default::default()
                            },
                            move |cx, st| {
                                cx.pressable_dispatch_command_opt(command);
                                if enabled {
                                    let active = active_for_row.clone();
                                    cx.pressable_on_hover_change(Arc::new(
                                        move |host, action_cx, hovered| {
                                            if !hovered {
                                                return;
                                            }
                                            let current = host
                                                .models_mut()
                                                .get_copied(&active)
                                                .unwrap_or(None);
                                            let next = Some(idx);
                                            if current != next {
                                                let _ = host
                                                    .models_mut()
                                                    .update(&active, |v| *v = next);
                                                host.request_redraw(action_cx.window);
                                            }
                                        },
                                    ));
                                }

                                let hovered = st.hovered && !st.pressed;
                                let pressed = st.pressed;
                                let bg = if selected {
                                    Some(bg_selected)
                                } else if hovered || pressed {
                                    Some(bg_hover)
                                } else {
                                    None
                                };

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
                                                    style: Some(text_style.clone()),
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
                        );

                        row_ids.push(row.id);
                        row
                    })
                })
                .collect();

            let active_descendant = active_idx
                .and_then(|idx| row_ids.get(idx).copied())
                .and_then(|row| cx.node_for_element(row));

            let border = border(&theme);
            let mut wrapper = decl_style::container_props(
                &theme,
                ChromeRefinement::default(),
                LayoutRefinement::default().w_full(),
            );
            wrapper.border = Edges {
                top: Px(0.0),
                right: Px(0.0),
                bottom: Px(1.0),
                left: Px(0.0),
            };
            wrapper.border_color = Some(border);

            let mut input = Input::new(self.model).a11y_label(self.a11y_label);
            if let Some(active_descendant) = active_descendant {
                input = input.active_descendant(active_descendant);
            }

            let mut input = input.into_element(cx);

            let key_handler = cx.with_state(
                || {
                    let entries_cell: Rc<RefCell<Arc<[PaletteEntry]>>> =
                        Rc::new(RefCell::new(Arc::from([])));
                    let entries_read = entries_cell.clone();
                    let disabled_cell = Rc::new(Cell::new(false));
                    let wrap_cell = Rc::new(Cell::new(true));

                    let disabled_read = disabled_cell.clone();
                    let wrap_read = wrap_cell.clone();

                    let handler: fret_ui::action::OnKeyDown =
                        Arc::new(move |host, action_cx, down| {
                            if disabled_read.get() {
                                return false;
                            }

                            let entries = entries_read.borrow();
                            let disabled_flags: Vec<bool> =
                                entries.iter().map(|e| e.disabled).collect();

                            match down.key {
                                KeyCode::ArrowDown | KeyCode::ArrowUp => {
                                    let current =
                                        host.models_mut().get_copied(&active).unwrap_or(None);
                                    let forward = down.key == KeyCode::ArrowDown;
                                    let next = cmdk_selection::next_active_index(
                                        &disabled_flags,
                                        current,
                                        forward,
                                        wrap_read.get(),
                                    );

                                    if next != current {
                                        let _ = host.models_mut().update(&active, |v| *v = next);
                                        host.request_redraw(action_cx.window);
                                    }
                                    true
                                }
                                KeyCode::Home => {
                                    let current =
                                        host.models_mut().get_copied(&active).unwrap_or(None);
                                    let next = cmdk_selection::first_enabled(&disabled_flags);
                                    if next != current {
                                        let _ = host.models_mut().update(&active, |v| *v = next);
                                        host.request_redraw(action_cx.window);
                                    }
                                    true
                                }
                                KeyCode::End => {
                                    let current =
                                        host.models_mut().get_copied(&active).unwrap_or(None);
                                    let next = cmdk_selection::last_enabled(&disabled_flags);
                                    if next != current {
                                        let _ = host.models_mut().update(&active, |v| *v = next);
                                        host.request_redraw(action_cx.window);
                                    }
                                    true
                                }
                                KeyCode::PageDown | KeyCode::PageUp => {
                                    let current =
                                        host.models_mut().get_copied(&active).unwrap_or(None);
                                    let forward = down.key == KeyCode::PageDown;
                                    let next = cmdk_selection::advance_active_index(
                                        &disabled_flags,
                                        current,
                                        forward,
                                        wrap_read.get(),
                                        10,
                                    );
                                    if next != current {
                                        let _ = host.models_mut().update(&active, |v| *v = next);
                                        host.request_redraw(action_cx.window);
                                    }
                                    true
                                }
                                KeyCode::Enter | KeyCode::NumpadEnter => {
                                    let current =
                                        host.models_mut().get_copied(&active).unwrap_or(None);
                                    let Some(idx) = cmdk_selection::clamp_active_index(
                                        &disabled_flags,
                                        current,
                                    ) else {
                                        return false;
                                    };

                                    let Some(entry) = entries.get(idx) else {
                                        return false;
                                    };
                                    let Some(command) = entry.command.clone() else {
                                        return false;
                                    };
                                    host.dispatch_command(Some(action_cx.window), command);
                                    true
                                }
                                _ => false,
                            }
                        });

                    KeyHandlerState {
                        disabled: disabled_cell,
                        wrap: wrap_cell,
                        entries: entries_cell,
                        handler,
                    }
                },
                |state: &mut KeyHandlerState| {
                    state.disabled.set(disabled);
                    state.wrap.set(wrap);
                    *state.entries.borrow_mut() = entries_arc.clone();
                    state.handler.clone()
                },
            );

            cx.key_on_key_down_for(input.id, key_handler);

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

            let list = if row_ids.is_empty() {
                let fg = theme.colors.text_muted;
                let text_style = item_text_style(&theme);
                let empty = self.empty_text;
                cx.container(ContainerProps::default(), move |cx| {
                    vec![cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: empty,
                        style: Some(text_style),
                        color: Some(fg),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    })]
                })
            } else {
                let scroll = self.scroll;
                cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::List,
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            ScrollArea::new(vec![cx.flex(
                                FlexProps {
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
                                move |_cx| rows,
                            )])
                            .refine_layout(scroll)
                            .into_element(cx),
                        ]
                    },
                )
            };

            Command::new(vec![cx.container(wrapper, move |_cx| vec![input]), list])
                .refine_style(self.chrome)
                .refine_layout(self.layout)
                .into_element(cx)
        })
    }
}

#[derive(Clone)]
pub struct CommandDialog {
    open: Model<bool>,
    query: Model<String>,
    items: Vec<CommandItem>,
    a11y_label: Option<Arc<str>>,
    disabled: bool,
    wrap: bool,
    empty_text: Arc<str>,
}

impl std::fmt::Debug for CommandDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandDialog")
            .field("open", &"<model>")
            .field("query", &"<model>")
            .field("items_len", &self.items.len())
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("disabled", &self.disabled)
            .field("wrap", &self.wrap)
            .field("empty_text", &self.empty_text.as_ref())
            .finish()
    }
}

impl CommandDialog {
    pub fn new(open: Model<bool>, query: Model<String>, items: Vec<CommandItem>) -> Self {
        Self {
            open,
            query,
            items,
            a11y_label: None,
            disabled: false,
            wrap: true,
            empty_text: Arc::from("No results."),
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

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn empty_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.empty_text = text.into();
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementCx<'_, H>,
        trigger: impl FnOnce(&mut ElementCx<'_, H>) -> AnyElement,
    ) -> AnyElement {
        let open = self.open;
        let query = self.query;
        let items = self.items;
        let a11y_label = self.a11y_label;
        let disabled = self.disabled;
        let wrap = self.wrap;
        let empty_text = self.empty_text;

        Dialog::new(open).into_element(cx, trigger, move |cx| {
            let palette = CommandPalette::new(query, items)
                .a11y_label(a11y_label.unwrap_or_else(|| Arc::from("Command palette")))
                .disabled(disabled)
                .wrap(wrap)
                .empty_text(empty_text)
                .into_element(cx);

            DialogContent::new(vec![palette])
                .refine_style(ChromeRefinement::default().p(Space::N0))
                .into_element(cx)
        })
    }
}

#[derive(Default)]
struct CommandPaletteState {
    active: Option<Model<Option<usize>>>,
    items_fingerprint: u64,
}

pub fn command<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    Command::new(f(cx)).into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, MouseButtons, Point, Px, Rect, SemanticsRole, Size, SvgId,
        SvgService,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle};
    use fret_ui::tree::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &TextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
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

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<String>,
        items: Vec<CommandItem>,
    ) -> fret_core::NodeId {
        let next_frame = fret_core::FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_components_ui::window_overlays::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "cmdk", |cx| {
                vec![CommandPalette::new(model, items).into_element(cx)]
            });
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn cmdk_arrow_moves_highlight_while_focus_stays_in_input() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(String::new());

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            CommandItem::new("Alpha").on_select(CommandId::new("alpha")),
            CommandItem::new("Beta").on_select(CommandId::new("beta")),
        ];

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            items.clone(),
        );

        let input = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable text input");
        ui.set_focus(Some(input));

        // Move highlight down.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            items,
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");

        let focus = snap.focus.expect("focus");
        assert_eq!(focus, input, "focus should remain on the input node");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::TextField && n.id == focus)
            .expect("focused text field node");

        let active = input
            .active_descendant
            .expect("active_descendant should be set");
        let active_node = snap
            .nodes
            .iter()
            .find(|n| n.id == active)
            .expect("active_descendant should reference a node in the snapshot");

        assert_eq!(active_node.role, SemanticsRole::ListItem);
        assert_eq!(active_node.label.as_deref(), Some("Beta"));
        assert_eq!(active_node.pos_in_set, Some(2));
        assert_eq!(active_node.set_size, Some(2));
        assert!(
            active_node.flags.selected,
            "highlighted row should be selected"
        );
    }

    #[test]
    fn cmdk_hover_moves_highlight_while_focus_stays_in_input() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(String::new());

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            CommandItem::new("Alpha").on_select(CommandId::new("alpha")),
            CommandItem::new("Beta").on_select(CommandId::new("beta")),
            CommandItem::new("Gamma").on_select(CommandId::new("gamma")),
        ];

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            items.clone(),
        );

        let input = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable text input");
        ui.set_focus(Some(input));

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta_bounds = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListItem && n.label.as_deref() == Some("Beta"))
            .map(|n| n.bounds)
            .expect("Beta row bounds");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(
                    Px(beta_bounds.origin.x.0 + 1.0),
                    Px(beta_bounds.origin.y.0 + 1.0),
                ),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
            }),
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            items,
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");

        let focus = snap.focus.expect("focus");
        assert_eq!(focus, input, "focus should remain on the input node");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::TextField && n.id == focus)
            .expect("focused text field node");

        let active = input
            .active_descendant
            .expect("active_descendant should be set");
        let active_node = snap
            .nodes
            .iter()
            .find(|n| n.id == active)
            .expect("active_descendant should reference a node in the snapshot");

        assert_eq!(active_node.role, SemanticsRole::ListItem);
        assert_eq!(active_node.label.as_deref(), Some("Beta"));
        assert_eq!(active_node.pos_in_set, Some(2));
        assert_eq!(active_node.set_size, Some(3));
    }
}
