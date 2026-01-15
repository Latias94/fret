use std::cell::Cell;
use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Point, Px, Rect, SemanticsRole, TextOverflow,
    TextStyle, TextWrap,
};
use fret_runtime::CommandId;
use fret_ui::action::OnActivate;
use fret_ui::element::ElementKind;
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

fn scroll_content_row_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Auto;
    layout.size.height = Length::Fill;
    layout.flex.shrink = 0.0;
    layout
}

fn tab_strip_scroll_content_layout() -> LayoutStyle {
    if std::env::var_os("FRET_DEBUG_TABSTRIP_FILL").is_some() {
        fill_layout()
    } else {
        scroll_content_row_layout()
    }
}

fn tab_text_style(theme: &Theme) -> TextStyle {
    let px = theme.metric_by_key("font.size").unwrap_or(Px(13.0));
    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::MEDIUM,
        slant: Default::default(),
        line_height: None,
        letter_spacing_em: None,
    }
}

fn scroll_rect_into_view_x(handle: &ScrollHandle, viewport: Rect, child: Rect) {
    let margin = Px(12.0);

    let current = handle.offset();
    let view_left = viewport.origin.x;
    let view_right = Px(viewport.origin.x.0 + viewport.size.width.0);
    let child_left = child.origin.x;
    let child_right = Px(child.origin.x.0 + child.size.width.0);

    let next_x = if child_left.0 < (view_left.0 + margin.0) {
        Px(current.x.0 + (child_left.0 - (view_left.0 + margin.0)))
    } else if child_right.0 > (view_right.0 - margin.0) {
        Px(current.x.0 + (child_right.0 - (view_right.0 - margin.0)))
    } else {
        current.x
    };

    if next_x != current.x {
        handle.set_offset(Point::new(next_x, current.y));
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
                let (scroll_handle, last_active) = cx.with_state(
                    WorkspaceTabStripState::default,
                    |state| (state.scroll.clone(), state.last_active.clone()),
                );
                let scroll_element = Cell::<Option<GlobalElementId>>::new(None);
                let active_tab_element = Cell::<Option<GlobalElementId>>::new(None);

                let root = cx.container(
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
                            let scroll = cx.scope(|cx| {
                                let id = cx.root_id();
                                scroll_element.set(Some(id));

                                let children = vec![cx.flex(
                                    FlexProps {
                                        layout: tab_strip_scroll_content_layout(),
                                        direction: fret_core::Axis::Horizontal,
                                        gap: Px(2.0),
                                        padding: Edges::all(Px(0.0)),
                                        justify: MainAlign::Start,
                                        align: CrossAlign::Center,
                                        wrap: false,
                                    },
                                    |cx| {
                                        let mut out: Vec<AnyElement> = Vec::new();

                                        for (index, tab) in tabs.iter().enumerate() {
                                            let tab_id = tab.id.clone();
                                            let tab_title = tab.title.clone();
                                            let tab_command = tab.command.clone();
                                            let tab_close_command = tab.close_command.clone();
                                            let tab_dirty = tab.dirty;
                                            let is_active =
                                                tab_id.as_ref() == active.as_ref();
                                            let pos_in_set = (index as u32) + 1;

                                            let element = cx.keyed(tab_id.as_ref(), |cx| {
                                                cx.pressable_with_id(
                                                    PressableProps {
                                                        layout: {
                                                            let mut layout =
                                                                LayoutStyle::default();
                                                            layout.size.height = Length::Fill;
                                                            layout.size.width = Length::Auto;
                                                            layout
                                                        },
                                                        a11y: PressableA11y {
                                                            role: Some(SemanticsRole::Tab),
                                                            label: Some(tab_title.clone()),
                                                            selected: is_active,
                                                            pos_in_set: Some(pos_in_set),
                                                            set_size: Some(set_size),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                    |cx, press_state, element_id| {
                                                        if is_active {
                                                            active_tab_element.set(Some(element_id));
                                                        }

                                                        let handler: OnActivate = Arc::new(
                                                            move |host, acx, _reason| {
                                                                host.dispatch_command(
                                                                    Some(acx.window),
                                                                    tab_command.clone(),
                                                                );
                                                            },
                                                        );
                                                        cx.pressable_add_on_activate(handler);

                                                        let bg = if is_active {
                                                            active_bg
                                                        } else if press_state.hovered
                                                            || press_state.pressed
                                                        {
                                                            Some(hover_bg)
                                                        } else {
                                                            None
                                                        };

                                                        let mut label = tab_title.clone();
                                                        if tab_dirty {
                                                            label = Arc::from(format!(
                                                                "{} \u{2022}",
                                                                tab_title.as_ref()
                                                            ));
                                                        }

                                                        vec![cx.container(
                                                            ContainerProps {
                                                                layout: {
                                                                    let mut layout =
                                                                        LayoutStyle::default();
                                                                    layout.size.height =
                                                                        Length::Fill;
                                                                    layout.size.width =
                                                                        Length::Auto;
                                                                    layout
                                                                },
                                                                padding: Edges {
                                                                    left: Px(10.0),
                                                                    right: Px(6.0),
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
                                                                vec![cx.flex(
                                                                    FlexProps {
                                                                        layout: {
                                                                            let mut layout =
                                                                                LayoutStyle::default();
                                                                            layout.size.height =
                                                                                Length::Fill;
                                                                            layout.size.width =
                                                                                Length::Auto;
                                                                            layout
                                                                        },
                                                                        direction:
                                                                            fret_core::Axis::Horizontal,
                                                                        gap: Px(6.0),
                                                                        justify: MainAlign::Start,
                                                                        align: CrossAlign::Center,
                                                                        ..Default::default()
                                                                    },
                                                                    |cx| {
                                                                        let mut children = vec![
                                                                            cx.text_props(TextProps {
                                                                                layout: LayoutStyle::default(),
                                                                                text: label,
                                                                                style: Some(text_style.clone()),
                                                                                color: Some(inactive_fg),
                                                                                wrap: TextWrap::None,
                                                                                overflow: TextOverflow::Ellipsis,
                                                                            }),
                                                                        ];

                                                                        if let Some(
                                                                            close_command,
                                                                        ) = tab_close_command
                                                                            .clone()
                                                                        {
                                                                            children.push(cx.pressable(
                                                                                PressableProps {
                                                                                    layout: {
                                                                                        let mut layout = LayoutStyle::default();
                                                                                        layout.size.width = Length::Px(Px(18.0));
                                                                                        layout.size.height = Length::Px(Px(18.0));
                                                                                        layout
                                                                                    },
                                                                                    focusable: false,
                                                                                    a11y: PressableA11y {
                                                                                        role: Some(SemanticsRole::Button),
                                                                                        label: Some(Arc::from("Close tab")),
                                                                                        ..Default::default()
                                                                                    },
                                                                                    ..Default::default()
                                                                                },
                                                                                move |cx, close_state| {
                                                                                    let close_handler: OnActivate = Arc::new(
                                                                                        move |host, acx, _reason| {
                                                                                            host.dispatch_command(
                                                                                                Some(acx.window),
                                                                                                close_command.clone(),
                                                                                            );
                                                                                        },
                                                                                    );
                                                                                    cx.pressable_add_on_activate(close_handler);

                                                                                    let bg = if close_state.hovered || close_state.pressed {
                                                                                        Some(hover_bg)
                                                                                    } else {
                                                                                        None
                                                                                    };

                                                                                    vec![cx.container(
                                                                                        ContainerProps {
                                                                                            layout: fill_layout(),
                                                                                            background: bg,
                                                                                            corner_radii: Corners::all(Px(4.0)),
                                                                                            ..Default::default()
                                                                                        },
                                                                                        |cx| vec![cx.text("×")],
                                                                                    )]
                                                                                },
                                                                            ));
                                                                        }

                                                                        children
                                                                    },
                                                                )]
                                                            },
                                                        )]
                                                    },
                                                )
                                            });
                                            out.push(element);
                                        }

                                        out
                                    },
                                )];

                                AnyElement::new(
                                    id,
                                    ElementKind::Scroll(ScrollProps {
                                        layout: fill_layout(),
                                        axis: ScrollAxis::X,
                                        scroll_handle: Some(scroll_handle.clone()),
                                        // Important: keep the scroll child width `Auto` (see
                                        // `scroll_content_row_layout`) to avoid recursive
                                        // "fill-to-max" probing that can blow the stack in layout.
                                        probe_unbounded: true,
                                    }),
                                    children,
                                )
                            });

                            vec![scroll]
                        },
                    );

                    let active_changed = last_active.as_deref() != Some(active.as_ref());
                    if active_changed {
                        if let (Some(scroll_id), Some(tab_id)) =
                            (scroll_element.get(), active_tab_element.get())
                        {
                            if let (Some(viewport), Some(tab_rect)) = (
                                cx.last_bounds_for_element(scroll_id),
                                cx.last_bounds_for_element(tab_id),
                            ) {
                                scroll_rect_into_view_x(&scroll_handle, viewport, tab_rect);
                            }
                        }
                    }

                    cx.with_state(WorkspaceTabStripState::default, |state| {
                        state.last_active = Some(active.clone());
                    });

                    vec![root]
            },
        )
    }
}
