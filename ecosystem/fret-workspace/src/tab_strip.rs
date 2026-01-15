use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, Point, Px, Rect, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::CommandId;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, ScrollAxis, ScrollProps, SemanticsProps, TextProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn row_layout(height: Px) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Px(height);
    layout.flex.shrink = 0.0;
    layout
}

fn tab_text_style(theme: &Theme) -> TextStyle {
    let px = theme.metric_by_key("font.size").unwrap_or(Px(13.0));
    TextStyle {
        font: fret_core::FontId::default(),
        size: px,
        weight: fret_core::FontWeight::MEDIUM,
        slant: Default::default(),
        line_height: None,
        letter_spacing_em: None,
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceTab {
    pub id: Arc<str>,
    pub title: Arc<str>,
    pub command: CommandId,
    pub close_command: Option<CommandId>,
    pub dirty: bool,
}

impl WorkspaceTab {
    pub fn new(
        id: impl Into<Arc<str>>,
        title: impl Into<Arc<str>>,
        command: impl Into<CommandId>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            command: command.into(),
            close_command: None,
            dirty: false,
        }
    }

    pub fn close_command(mut self, command: impl Into<CommandId>) -> Self {
        self.close_command = Some(command.into());
        self
    }

    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }
}

/// A minimal “editor-like” tab strip meant for workspace shells.
///
/// Notes:
/// - This is intentionally lightweight and policy-oriented, so it lives in `ecosystem/`.
/// - This is not a replacement for shadcn `Tabs` (which targets in-page navigation semantics).
#[derive(Debug, Clone)]
pub struct WorkspaceTabStrip {
    active: Arc<str>,
    tabs: Vec<WorkspaceTab>,
    height: Px,
}

#[derive(Default)]
struct WorkspaceTabStripState {
    scroll: ScrollHandle,
    last_active: Option<Arc<str>>,
    scroll_element: Option<GlobalElementId>,
    tab_elements: HashMap<Arc<str>, GlobalElementId>,
}

impl WorkspaceTabStrip {
    pub fn new(active: impl Into<Arc<str>>) -> Self {
        Self {
            active: active.into(),
            tabs: Vec::new(),
            height: Px(28.0),
        }
    }

    pub fn height(mut self, height: Px) -> Self {
        self.height = height;
        self
    }

    pub fn tabs(mut self, tabs: impl IntoIterator<Item = WorkspaceTab>) -> Self {
        self.tabs.extend(tabs);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(cx.app);

        let bar_bg = theme
            .color_by_key("workspace.tab_strip.bg")
            .or_else(|| theme.color_by_key("muted"))
            .or_else(|| theme.color_by_key("background"));
        let bar_border = theme.color_by_key("border");

        let active_bg = theme
            .color_by_key("workspace.tab.active_bg")
            .or_else(|| theme.color_by_key("background"));
        let inactive_fg = theme.color_required("foreground");
        let hover_bg = theme
            .color_by_key("accent")
            .or_else(|| theme.color_by_key("workspace.tab.hover_bg"))
            .unwrap_or(Color::TRANSPARENT);

        let text_style = tab_text_style(theme);
        let tab_radius = theme.metric_by_key("radius").unwrap_or(Px(6.0));

        let tabs = self.tabs;
        let set_size = tabs.len() as u32;
        let active = self.active;

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::TabList,
                ..Default::default()
            },
            |cx| {
                cx.with_state(WorkspaceTabStripState::default, |state| {
                    state.tab_elements.clear();
                    vec![cx.container(
                        ContainerProps {
                            layout: row_layout(self.height),
                            padding: Edges::all(Px(2.0)),
                            background: bar_bg,
                            border: Edges {
                                bottom: Px(1.0),
                                ..Edges::all(Px(0.0))
                            },
                            border_color: bar_border,
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.scroll(
                                ScrollProps {
                                    layout: fill_layout(),
                                    axis: ScrollAxis::X,
                                    scroll_handle: Some(state.scroll.clone()),
                                    probe_unbounded: true,
                                },
                                |cx| {
                                    vec![cx.flex(
                                        FlexProps {
                                            layout: fill_layout(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap: Px(2.0),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            ..Default::default()
                                        },
                                        |cx| {
                                            tabs.into_iter()
                                                .enumerate()
                                                .map(|(index, tab)| {
                                                let is_active = tab.id.as_ref() == active.as_ref();
                                                let command = tab.command.clone();
                                                let title_for_a11y = tab.title.clone();
                                                let selected = is_active;
                                                let pos_in_set = (index as u32) + 1;

                                                cx.pressable(
                                                    PressableProps {
                                                        layout: {
                                                            let mut layout = LayoutStyle::default();
                                                            layout.size.height = Length::Fill;
                                                            layout
                                                        },
                                                        a11y: PressableA11y {
                                                            role: Some(SemanticsRole::Tab),
                                                            label: Some(title_for_a11y),
                                                            selected,
                                                            pos_in_set: Some(pos_in_set),
                                                            set_size: Some(set_size),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                    |cx, state| {
                                                        let handler: OnActivate =
                                                            Arc::new(move |host, acx, _reason| {
                                                                host.dispatch_command(
                                                                    Some(acx.window),
                                                                    command.clone(),
                                                                );
                                                            });
                                                        cx.pressable_add_on_activate(handler);

                                                        let bg = if is_active {
                                                            active_bg
                                                        } else if state.hovered || state.pressed {
                                                            Some(hover_bg)
                                                        } else {
                                                            None
                                                        };

                                                        let mut layout = LayoutStyle::default();
                                                        layout.size.height = Length::Fill;
                                                        layout.size.width = Length::Auto;

                                                        let mut label = tab.title.clone();
                                                        if tab.dirty {
                                                            label = Arc::from(format!(
                                                                "{} \u{2022}",
                                                                tab.title.as_ref()
                                                            ));
                                                        }

                                                        vec![cx.container(
                                                            ContainerProps {
                                                                layout,
                                                                padding: Edges {
                                                                    left: Px(10.0),
                                                                    right: Px(10.0),
                                                                    top: Px(4.0),
                                                                    bottom: Px(4.0),
                                                                },
                                                                background: bg,
                                                                corner_radii: Corners::all(Px(
                                                                    tab_radius.0.max(0.0),
                                                                )),
                                                                ..Default::default()
                                                            },
                                                            |cx| {
                                                                vec![cx.text_props(TextProps {
                                                                    layout: LayoutStyle::default(),
                                                                    text: label,
                                                                    style: Some(text_style.clone()),
                                                                    color: Some(inactive_fg),
                                                                    wrap: TextWrap::None,
                                                                    overflow:
                                                                        TextOverflow::Ellipsis,
                                                                })]
                                                            },
                                                        )]
                                                    },
                                                )
                                            })
                                            .collect()
                                        },
                                    )]
                                },
                            )]
                        },
                    )]
                })
            },
        )
    }
}
