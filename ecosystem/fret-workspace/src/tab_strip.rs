use std::cell::{Cell, RefCell};
use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Modifiers, MouseButton, Point, PointerId, Px, Rect,
    SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{CommandId, Model, TickId};
use fret_ui::action::{
    OnActivate, OnPressablePointerDown, OnPressablePointerMove, OnPressablePointerUp,
    PressablePointerDownResult, PressablePointerUpResult,
};
use fret_ui::element::ElementKind;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, ScrollAxis, ScrollProps, SemanticsProps, TextProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::dnd as ui_dnd;

use crate::commands::{
    tab_activate_command, tab_close_command, tab_move_active_after_command,
    tab_move_active_before_command,
};
use crate::tab_drag::{
    DRAG_KIND_WORKSPACE_TAB, WorkspaceTabDragState, WorkspaceTabDropZone, WorkspaceTabHitRect,
    WorkspaceTabInsertionSide, compute_tab_drop_target,
};

#[cfg(feature = "shadcn-context-menu")]
use fret_ui_shadcn::{ContextMenu, ContextMenuEntry, ContextMenuItem};

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn tab_list_semantics_layout() -> LayoutStyle {
    // The tab strip is commonly placed into "center" regions of editor-like top bars where it
    // must be allowed to shrink; otherwise, long tab titles can push right-side controls out of
    // the window.
    //
    // This mirrors Tailwind's `w-full min-w-0 flex-shrink` rule of thumb.
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.min_width = Some(Px(0.0));
    layout.flex.shrink = 1.0;
    layout
}

fn row_layout(height: Px) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Px(height);
    layout.size.min_width = Some(Px(0.0));
    layout.flex.shrink = 1.0;
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

fn fixed_square_layout(size: Px) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(size);
    layout.size.height = Length::Px(size);
    layout.flex.shrink = 0.0;
    layout
}

fn fill_grow_layout() -> LayoutStyle {
    let mut layout = fill_layout();
    layout.size.min_width = Some(Px(0.0));
    layout.flex.grow = 1.0;
    layout.flex.shrink = 1.0;
    layout
}

fn centered_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    style: TextStyle,
    color: Option<Color>,
) -> AnyElement {
    cx.flex(
        FlexProps {
            layout: fill_layout(),
            direction: fret_core::Axis::Horizontal,
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            ..Default::default()
        },
        |cx| {
            vec![cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text,
                style: Some(style),
                color,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            })]
        },
    )
}

#[derive(Debug, Default, Clone)]
struct WorkspaceTabStripDragState {
    pointer: Option<PointerId>,
    start_tick: TickId,
    start_position: Point,
    dragged_tab: Option<Arc<str>>,
    dragging: bool,
    drop_target: Option<(Arc<str>, WorkspaceTabInsertionSide)>,
    tab_rects: Vec<WorkspaceTabHitRect>,
}

#[derive(Debug, Default)]
struct WorkspaceTabStripDragStateModel {
    model: Option<Model<WorkspaceTabStripDragState>>,
}

fn get_drag_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<WorkspaceTabStripDragState> {
    let existing = cx.with_state(WorkspaceTabStripDragStateModel::default, |st| {
        st.model.clone()
    });
    if let Some(m) = existing {
        return m;
    }

    let model = cx
        .app
        .models_mut()
        .insert(WorkspaceTabStripDragState::default());
    cx.with_state(WorkspaceTabStripDragStateModel::default, |st| {
        st.model = Some(model.clone());
    });
    model
}

fn fnv1a64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn dnd_scope_for_pane(pane_id: Option<&Arc<str>>) -> ui_dnd::DndScopeId {
    pane_id
        .map(|id| ui_dnd::DndScopeId(fnv1a64(id.as_ref())))
        .unwrap_or(ui_dnd::DND_SCOPE_DEFAULT)
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
    active: Option<Arc<str>>,
    tabs: Vec<WorkspaceTab>,
    height: Px,
    pane_id: Option<Arc<str>>,
    tab_drag: Option<Model<WorkspaceTabDragState>>,
    root_test_id: Option<Arc<str>>,
    tab_test_id_prefix: Option<Arc<str>>,
}

#[derive(Default)]
struct WorkspaceTabStripState {
    scroll: ScrollHandle,
    last_active: Option<Arc<str>>,
}

#[cfg(feature = "shadcn-context-menu")]
#[derive(Debug, Default)]
struct WorkspaceTabStripContextMenuState {
    open: Option<Model<bool>>,
}

#[cfg(feature = "shadcn-context-menu")]
fn get_context_menu_open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let existing = cx.with_state(WorkspaceTabStripContextMenuState::default, |st| {
        st.open.clone()
    });
    if let Some(m) = existing {
        return m;
    }

    let model = cx.app.models_mut().insert(false);
    cx.with_state(WorkspaceTabStripContextMenuState::default, |st| {
        st.open = Some(model.clone());
    });
    model
}

impl WorkspaceTabStrip {
    pub fn new(active: impl Into<Arc<str>>) -> Self {
        Self {
            active: Some(active.into()),
            tabs: Vec::new(),
            height: Px(28.0),
            pane_id: None,
            tab_drag: None,
            root_test_id: None,
            tab_test_id_prefix: None,
        }
    }

    pub fn new_optional(active: Option<Arc<str>>) -> Self {
        Self {
            active,
            tabs: Vec::new(),
            height: Px(28.0),
            pane_id: None,
            tab_drag: None,
            root_test_id: None,
            tab_test_id_prefix: None,
        }
    }

    pub fn active(mut self, active: Option<Arc<str>>) -> Self {
        self.active = active;
        self
    }

    pub fn height(mut self, height: Px) -> Self {
        self.height = height;
        self
    }

    pub fn pane_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.pane_id = Some(id.into());
        self
    }

    pub fn tab_drag_model(mut self, model: Model<WorkspaceTabDragState>) -> Self {
        self.tab_drag = Some(model);
        self
    }

    /// Attach a deterministic test id for the tab strip root semantics node.
    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.root_test_id = Some(id.into());
        self
    }

    /// Attach deterministic test ids for individual tabs.
    ///
    /// Shape: `{prefix}-{tab_id}`.
    pub fn tab_test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.tab_test_id_prefix = Some(prefix.into());
        self
    }

    pub fn tabs(mut self, tabs: impl IntoIterator<Item = WorkspaceTab>) -> Self {
        self.tabs.extend(tabs);
        self
    }

    pub fn from_workspace_tabs(
        state: &crate::tabs::WorkspaceTabs,
        title: impl Fn(&str) -> Arc<str>,
    ) -> Self {
        let active = state.active().cloned();
        let mut out = WorkspaceTabStrip::new_optional(active);
        out.tabs = state
            .tabs()
            .iter()
            .filter_map(|id| {
                let activate = tab_activate_command(id.as_ref())?;
                let mut tab = WorkspaceTab::new(id.clone(), title(id.as_ref()), activate);
                if let Some(close) = tab_close_command(id.as_ref()) {
                    tab = tab.close_command(close);
                }
                tab.dirty = state.is_dirty(id.as_ref());
                Some(tab)
            })
            .collect();

        out
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let tabs = self.tabs;
        let set_size = tabs.len() as u32;
        let active = self.active;
        let pane_id = self.pane_id;
        let pane_activate_cmd = pane_id
            .as_deref()
            .and_then(crate::commands::pane_activate_command);
        let tab_drag_model = self.tab_drag;
        let root_test_id = self.root_test_id;
        let tab_test_id_prefix = self.tab_test_id_prefix;

        let drag_model = get_drag_model(cx);
        cx.observe_model(&drag_model, Invalidation::Paint);

        let dnd = ui_dnd::dnd_service_model(cx);
        let dnd_scope = dnd_scope_for_pane(pane_id.as_ref());

        let drag_snapshot = cx
            .get_model_cloned(&drag_model, Invalidation::Paint)
            .unwrap_or_default();
        let dragging = drag_snapshot.dragging;
        let dragged_tab = drag_snapshot.dragged_tab.clone();
        let drop_target = drag_snapshot.drop_target.clone();

        let (
            bar_bg,
            bar_border,
            active_bg,
            active_fg,
            inactive_fg,
            dirty_fg,
            hover_bg,
            indicator_color,
            text_style,
            tab_radius,
            scroll_button_fg,
            tab_max_width,
        ) = {
            let theme = Theme::global(cx.app);

            let bar_bg = theme
                .color_by_key("workspace.tab_strip.bg")
                .or_else(|| theme.color_by_key("muted"))
                .or_else(|| theme.color_by_key("background"));
            let bar_border = theme.color_by_key("border");

            let active_bg = theme
                .color_by_key("workspace.tab.active_bg")
                .or_else(|| theme.color_by_key("background"));
            let active_fg = theme.color_required("foreground");
            let inactive_fg = theme.color_by_key("muted-foreground").unwrap_or(active_fg);
            let dirty_fg = theme
                .color_by_key("workspace.tab.dirty_fg")
                .or_else(|| theme.color_by_key("ring"))
                .or_else(|| theme.color_by_key("primary"))
                .unwrap_or(active_fg);
            let hover_bg = theme
                .color_by_key("accent")
                .or_else(|| theme.color_by_key("workspace.tab.hover_bg"))
                .unwrap_or(Color::TRANSPARENT);

            let indicator_color = theme
                .color_by_key("workspace.tab.drop_indicator")
                .or_else(|| theme.color_by_key("ring"))
                .or_else(|| theme.color_by_key("accent"));

            let text_style = tab_text_style(theme);
            let tab_radius = theme.metric_by_key("radius").unwrap_or(Px(6.0));
            let scroll_button_fg = theme
                .color_by_key("workspace.tab_strip.scroll_fg")
                .or_else(|| theme.color_by_key("muted-foreground"))
                .unwrap_or(active_fg);
            let tab_max_width = theme
                .metric_by_key("workspace.tab.max_width")
                .unwrap_or(Px(240.0));

            (
                bar_bg,
                bar_border,
                active_bg,
                active_fg,
                inactive_fg,
                dirty_fg,
                hover_bg,
                indicator_color,
                text_style,
                tab_radius,
                scroll_button_fg,
                tab_max_width,
            )
        };

        cx.semantics(
            SemanticsProps {
                layout: tab_list_semantics_layout(),
                role: SemanticsRole::TabList,
                test_id: root_test_id.clone(),
                ..Default::default()
            },
            |cx| {
                let (scroll_handle, last_active) = cx.with_state(
                    WorkspaceTabStripState::default,
                    |state| (state.scroll.clone(), state.last_active.clone()),
                );
                let scroll_element = Cell::<Option<GlobalElementId>>::new(None);
                let active_tab_element = Cell::<Option<GlobalElementId>>::new(None);
                let tab_elements: RefCell<Vec<(Arc<str>, GlobalElementId)>> = RefCell::new(Vec::new());

                let cross_drop_target: Option<(Arc<str>, WorkspaceTabInsertionSide)> = match (
                    tab_drag_model.as_ref(),
                    pane_id.as_ref(),
                ) {
                    (Some(model), Some(pane_id)) => {
                        cx.observe_model(model, Invalidation::Paint);
                        let snapshot = cx
                            .get_model_cloned(model, Invalidation::Paint)
                            .unwrap_or_default();

                        if let Some(pointer_id) = snapshot.pointer
                            && let Some(session) = cx.app.drag(pointer_id)
                            && session.kind == DRAG_KIND_WORKSPACE_TAB
                            && session.dragging
                            && session.current_window == cx.window
                            && snapshot.source_pane.as_deref() != Some(pane_id.as_ref())
                            && snapshot.hovered_pane.as_deref() == Some(pane_id.as_ref())
                            && snapshot.hovered_zone == Some(WorkspaceTabDropZone::Center)
                            && !snapshot.hovered_pane_tab_rects.is_empty()
                            && let (Some(tab), Some(side)) =
                                (snapshot.hovered_tab, snapshot.hovered_tab_side)
                        {
                            Some((tab, side))
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

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
                                            let tab_activate_command = tab_command.clone();
                                            let tab_drag_command = tab_command.clone();
                                            let text_style = text_style.clone();
                                            let pane_id_for_drag = pane_id.clone();
                                            let tab_drag_model_for_drag = tab_drag_model.clone();
                                            let pane_activate_cmd_for_activate = pane_activate_cmd.clone();
                                            let pane_activate_cmd_for_close = pane_activate_cmd.clone();
                                            let pane_activate_cmd_for_drag = pane_activate_cmd.clone();
                                            let tab_close_command = tab.close_command.clone();
                                            let tab_dirty = tab.dirty;
                                            let tab_test_id_prefix_for_tab = tab_test_id_prefix.clone();
                                            #[cfg(feature = "shadcn-context-menu")]
                                            let has_left = index > 0;
                                            #[cfg(feature = "shadcn-context-menu")]
                                            let has_right = index + 1 < tabs.len();
                                            #[cfg(feature = "shadcn-context-menu")]
                                            let has_others = tabs.len() > 1;
                                            let is_active = active
                                                .as_deref()
                                                .is_some_and(|a| tab_id.as_ref() == a);
                                            let pos_in_set = (index as u32) + 1;
                                            let cross_drop_target = cross_drop_target.clone();

                                            let element = cx.keyed(tab_id.as_ref(), |cx| {
                                                let tab_element = cx.pressable_with_id(
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
                                                            test_id: tab_test_id_prefix_for_tab
                                                                .as_ref()
                                                                .map(|prefix| {
                                                                    Arc::<str>::from(format!(
                                                                        "{prefix}-{}",
                                                                        tab_id.as_ref()
                                                                    ))
                                                                }),
                                                            selected: is_active,
                                                            pos_in_set: Some(pos_in_set),
                                                            set_size: Some(set_size),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                    |cx, press_state, element_id| {
                                                        tab_elements.borrow_mut().push((tab_id.clone(), element_id));
                                                        if is_active {
                                                            active_tab_element.set(Some(element_id));
                                                        }

                                                        let tab_activate_cmd_for_activate =
                                                            tab_activate_command.clone();
                                                        let pane_activate_cmd_for_activate_handler =
                                                            pane_activate_cmd_for_activate.clone();
                                                        let handler: OnActivate = Arc::new(
                                                            move |host, acx, _reason| {
                                                                if let Some(cmd) =
                                                                    pane_activate_cmd_for_activate_handler
                                                                        .clone()
                                                                {
                                                                    host.dispatch_command(
                                                                        Some(acx.window),
                                                                        cmd,
                                                                    );
                                                                }
                                                                host.dispatch_command(
                                                                    Some(acx.window),
                                                                    tab_activate_cmd_for_activate
                                                                        .clone(),
                                                                );
                                                            },
                                                        );
                                                        cx.pressable_on_activate(handler);

                                                        let dnd_on_down: OnPressablePointerDown = {
                                                            let drag_model = drag_model.clone();
                                                            let tab_id = tab_id.clone();
                                                            let tab_activate_command =
                                                                tab_activate_command.clone();
                                                            let pane_activate_cmd_for_pointer =
                                                                pane_activate_cmd_for_activate
                                                                    .clone();
                                                            let tab_close_command =
                                                                tab_close_command.clone();
                                                            let dnd = dnd.clone();
                                                            Arc::new(move |host, acx, down| {
                                                                match down.button {
                                                                    MouseButton::Middle => {
                                                                        if let Some(cmd) =
                                                                            pane_activate_cmd_for_pointer
                                                                                .clone()
                                                                        {
                                                                            host.dispatch_command(
                                                                                Some(acx.window),
                                                                                cmd,
                                                                            );
                                                                        }
                                                                        if let Some(cmd) =
                                                                            tab_close_command.clone()
                                                                        {
                                                                            host.dispatch_command(
                                                                                Some(acx.window),
                                                                                cmd,
                                                                            );
                                                                            host.request_redraw(
                                                                                acx.window,
                                                                            );
                                                                        }
                                                                        host.prevent_default(
                                                                            fret_runtime::DefaultAction::FocusOnPointerDown,
                                                                        );
                                                                        return PressablePointerDownResult::SkipDefaultAndStopPropagation;
                                                                    }
                                                                    MouseButton::Right => {
                                                                        if let Some(cmd) =
                                                                            pane_activate_cmd_for_pointer
                                                                                .clone()
                                                                        {
                                                                            host.dispatch_command(
                                                                                Some(acx.window),
                                                                                cmd,
                                                                            );
                                                                        }
                                                                        host.dispatch_command(
                                                                            Some(acx.window),
                                                                            tab_activate_command
                                                                                .clone(),
                                                                        );
                                                                        host.request_redraw(
                                                                            acx.window,
                                                                        );
                                                                        host.prevent_default(
                                                                            fret_runtime::DefaultAction::FocusOnPointerDown,
                                                                        );
                                                                        return PressablePointerDownResult::SkipDefault;
                                                                    }
                                                                    _ => {}
                                                                }

                                                                if down.button != MouseButton::Left
                                                                {
                                                                    return PressablePointerDownResult::Continue;
                                                                }
                                                                if down.modifiers != Modifiers::default() {
                                                                    return PressablePointerDownResult::Continue;
                                                                }

                                                                let _ = host.models_mut().update(&drag_model, |st| {
                                                                    st.pointer = Some(down.pointer_id);
                                                                    st.start_tick = down.tick_id;
                                                                    st.start_position = down.position;
                                                                    st.dragged_tab = Some(tab_id.clone());
                                                                    st.dragging = false;
                                                                    st.drop_target = None;
                                                                });
                                                                ui_dnd::clear_pointer_in_scope(
                                                                    host.models_mut(),
                                                                    &dnd,
                                                                    acx.window,
                                                                    DRAG_KIND_WORKSPACE_TAB,
                                                                    dnd_scope,
                                                                    down.pointer_id,
                                                                );
                                                                PressablePointerDownResult::Continue
                                                            })
                                                        };
                                                        cx.pressable_on_pointer_down(dnd_on_down);

                                                        let dnd_on_move: OnPressablePointerMove = {
                                                            let drag_model = drag_model.clone();
                                                            let tab_command = tab_drag_command.clone();
                                                            let pane_activate_cmd = pane_activate_cmd_for_drag.clone();
                                                            let tab_drag_model = tab_drag_model_for_drag.clone();
                                                            let source_pane = pane_id_for_drag.clone();
                                                            let dragged_tab_id = tab_id.clone();
                                                            let dnd = dnd.clone();
                                                            Arc::new(move |host, acx, mv| {
                                                                let mut start_tick = TickId::default();
                                                                let mut start_position = Point::default();
                                                                let mut dragging = false;
                                                                let mut dragged_tab: Option<Arc<str>> = None;
                                                                let mut tab_rects: Vec<WorkspaceTabHitRect> = Vec::new();

                                                                let _ = host.models_mut().read(&drag_model, |st| {
                                                                    if st.pointer != Some(mv.pointer_id) {
                                                                        return;
                                                                    }
                                                                    start_tick = st.start_tick;
                                                                    start_position = st.start_position;
                                                                    dragging = st.dragging;
                                                                    dragged_tab = st.dragged_tab.clone();
                                                                    tab_rects = st.tab_rects.clone();
                                                                });

                                                                if dragged_tab.is_none() {
                                                                    return false;
                                                                }

                                                                if !mv.buttons.left {
                                                                    ui_dnd::clear_pointer_in_scope(
                                                                        host.models_mut(),
                                                                        &dnd,
                                                                        acx.window,
                                                                        DRAG_KIND_WORKSPACE_TAB,
                                                                        dnd_scope,
                                                                        mv.pointer_id,
                                                                    );
                                                                    let _ = host.models_mut().update(&drag_model, |st| {
                                                                        if st.pointer != Some(mv.pointer_id) {
                                                                            return;
                                                                        }
                                                                        *st = WorkspaceTabStripDragState::default();
                                                                    });
                                                                    host.request_redraw(acx.window);
                                                                    return false;
                                                                }

                                                                let mut activate_on_drag_start = false;
                                                                let mut drag_start_position: Option<Point> = None;

                                                                if !dragging {
                                                                    let sensor =
                                                                        ui_dnd::handle_sensor_move_or_init_in_scope(
                                                                            host.models_mut(),
                                                                            &dnd,
                                                                            acx.window,
                                                                            DRAG_KIND_WORKSPACE_TAB,
                                                                            dnd_scope,
                                                                            mv.pointer_id,
                                                                            start_tick,
                                                                            start_position,
                                                                            mv.position,
                                                                            mv.tick_id,
                                                                            ui_dnd::ActivationConstraint::Distance {
                                                                                px: 6.0,
                                                                            },
                                                                        );
                                                                    if !matches!(
                                                                        sensor,
                                                                        ui_dnd::SensorOutput::DragStart { .. }
                                                                    ) {
                                                                        return false;
                                                                    }
                                                                    activate_on_drag_start = true;
                                                                    drag_start_position = Some(start_position);
                                                                    ui_dnd::clear_pointer_in_scope(
                                                                        host.models_mut(),
                                                                        &dnd,
                                                                        acx.window,
                                                                        DRAG_KIND_WORKSPACE_TAB,
                                                                        dnd_scope,
                                                                        mv.pointer_id,
                                                                    );
                                                                }

                                                                let dragged = dragged_tab.expect("checked above");
                                                                let drop_target = compute_tab_drop_target(
                                                                    mv.position,
                                                                    dragged.as_ref(),
                                                                    &tab_rects,
                                                                );

                                                                let _ = host.models_mut().update(&drag_model, |st| {
                                                                    if st.pointer != Some(mv.pointer_id) {
                                                                        return;
                                                                    }
                                                                    if activate_on_drag_start {
                                                                        st.dragging = true;
                                                                    }
                                                                    st.drop_target = drop_target;
                                                                });

                                                                if activate_on_drag_start {
                                                                    if let Some(cmd) = pane_activate_cmd.clone() {
                                                                        host.dispatch_command(Some(acx.window), cmd);
                                                                    }
                                                                    host.dispatch_command(Some(acx.window), tab_command.clone());

                                                                    if let (Some(model), Some(source)) =
                                                                        (tab_drag_model.clone(), source_pane.clone())
                                                                    {
                                                                        let dragged_tab_id = dragged_tab_id.clone();
                                                                        let _ = host.models_mut().update(&model, |st| {
                                                                            st.pointer = Some(mv.pointer_id);
                                                                            st.source_window =
                                                                                Some(acx.window);
                                                                            st.source_pane = Some(source.clone());
                                                                            st.dragged_tab = Some(dragged_tab_id);
                                                                            st.hovered_pane = Some(source);
                                                                            st.hovered_zone = Some(
                                                                                WorkspaceTabDropZone::Center,
                                                                            );
                                                                            st.hovered_tab = None;
                                                                            st.hovered_tab_side = None;
                                                                            st.hovered_pane_tab_rects = Vec::new();
                                                                        });
                                                                    }

                                                                    host.begin_cross_window_drag_with_kind(
                                                                        mv.pointer_id,
                                                                        DRAG_KIND_WORKSPACE_TAB,
                                                                        acx.window,
                                                                        drag_start_position.unwrap_or(mv.position),
                                                                    );
                                                                    if let Some(drag) = host.drag_mut(mv.pointer_id) {
                                                                        drag.position = mv.position;
                                                                        drag.dragging = true;
                                                                    }
                                                                }
                                                                host.request_redraw(acx.window);
                                                                true
                                                            })
                                                        };
                                                        cx.pressable_on_pointer_move(dnd_on_move);

                                                        let dnd_on_up: OnPressablePointerUp = {
                                                            let drag_model = drag_model.clone();
                                                            let tab_command = tab_drag_command.clone();
                                                            let tab_drag_model = tab_drag_model_for_drag.clone();
                                                            let dnd = dnd.clone();
                                                            Arc::new(move |host, acx, up| {
                                                                if up.button != MouseButton::Left {
                                                                    return PressablePointerUpResult::Continue;
                                                                }

                                                                ui_dnd::clear_pointer_in_scope(
                                                                    host.models_mut(),
                                                                    &dnd,
                                                                    acx.window,
                                                                    DRAG_KIND_WORKSPACE_TAB,
                                                                    dnd_scope,
                                                                    up.pointer_id,
                                                                );

                                                                let cross_pane_drop = tab_drag_model
                                                                    .as_ref()
                                                                    .and_then(|m| {
                                                                        host.models_mut()
                                                                            .read(m, |st| {
                                                                                if st.pointer != Some(up.pointer_id)
                                                                                    || st.source_window
                                                                                        != Some(acx.window)
                                                                                {
                                                                                    return false;
                                                                                }

                                                                                let Some(source) =
                                                                                    st.source_pane.as_deref()
                                                                                else {
                                                                                    return false;
                                                                                };
                                                                                let Some(hovered) =
                                                                                    st.hovered_pane.as_deref()
                                                                                else {
                                                                                    return false;
                                                                                };

                                                                                let zone = st
                                                                                    .hovered_zone
                                                                                    .unwrap_or(
                                                                                        WorkspaceTabDropZone::Center,
                                                                                    );

                                                                                !(hovered == source
                                                                                    && zone
                                                                                        == WorkspaceTabDropZone::Center)
                                                                            })
                                                                            .ok()
                                                                    })
                                                                    .unwrap_or(false);

                                                                let mut outcome = PressablePointerUpResult::Continue;
                                                                let mut maybe_drop: Option<(
                                                                    Arc<str>,
                                                                    WorkspaceTabInsertionSide,
                                                                )> = None;
                                                                let _ = host.models_mut().update(&drag_model, |st| {
                                                                    if st.pointer != Some(up.pointer_id) {
                                                                        return;
                                                                    }
                                                                    if st.dragging {
                                                                        outcome = PressablePointerUpResult::SkipActivate;
                                                                        maybe_drop = st.drop_target.clone();
                                                                    }
                                                                    *st = WorkspaceTabStripDragState::default();
                                                                });

                                                                if outcome == PressablePointerUpResult::SkipActivate && cross_pane_drop {
                                                                    host.request_redraw(acx.window);
                                                                    return outcome;
                                                                }

                                                                if let Some((target, side)) = maybe_drop {
                                                                    host.dispatch_command(Some(acx.window), tab_command.clone());
                                                                    let cmd = match side {
                                                                        WorkspaceTabInsertionSide::Before => {
                                                                            tab_move_active_before_command(target.as_ref())
                                                                        }
                                                                        WorkspaceTabInsertionSide::After => {
                                                                            tab_move_active_after_command(target.as_ref())
                                                                        }
                                                                    };
                                                                    if let Some(cmd) = cmd {
                                                                        host.dispatch_command(Some(acx.window), cmd);
                                                                    }
                                                                    host.request_redraw(acx.window);
                                                                }

                                                                outcome
                                                            })
                                                        };
                                                        cx.pressable_on_pointer_up(dnd_on_up);

                                                        let bg = if is_active {
                                                            active_bg
                                                        } else if press_state.hovered
                                                            || press_state.pressed
                                                        {
                                                            Some(hover_bg)
                                                        } else {
                                                            None
                                                        };

                                                        let label = tab_title.clone();

                                                        let (indicator_border, indicator_border_color) = {
                                                            let local_hit = (dragging
                                                                && dragged_tab
                                                                    .as_deref()
                                                                    .is_some_and(|dragged| dragged != tab_id.as_ref()))
                                                            .then_some(())
                                                            .and_then(|_| {
                                                                drop_target.as_ref().and_then(|(target, side)| {
                                                                    if target.as_ref() == tab_id.as_ref() {
                                                                        Some(*side)
                                                                    } else {
                                                                        None
                                                                    }
                                                                })
                                                            });

                                                            let cross_hit = cross_drop_target
                                                                .as_ref()
                                                                .and_then(|(target, side)| {
                                                                    if target.as_ref() == tab_id.as_ref() {
                                                                        Some(*side)
                                                                    } else {
                                                                        None
                                                                    }
                                                                });

                                                            let side = local_hit.or(cross_hit);
                                                            match side {
                                                                Some(WorkspaceTabInsertionSide::Before) => (
                                                                    Edges {
                                                                        left: Px(2.0),
                                                                        ..Edges::all(Px(0.0))
                                                                    },
                                                                    indicator_color,
                                                                ),
                                                                Some(WorkspaceTabInsertionSide::After) => (
                                                                    Edges {
                                                                        right: Px(2.0),
                                                                        ..Edges::all(Px(0.0))
                                                                    },
                                                                    indicator_color,
                                                                ),
                                                                None => (Edges::all(Px(0.0)), None),
                                                            }
                                                        };

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
                                                                border: indicator_border,
                                                                border_color: indicator_border_color,
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
                                                                        let tab_fg = if is_active {
                                                                            active_fg
                                                                        } else {
                                                                            inactive_fg
                                                                        };

                                                                        let show_close = tab_close_command
                                                                            .is_some()
                                                                            && (is_active
                                                                                || press_state.hovered
                                                                                || press_state.pressed);
                                                                        let has_trailing_slot =
                                                                            tab_close_command.is_some()
                                                                                || tab_dirty;

                                                                        let mut children = vec![
                                                                            cx.text_props(TextProps {
                                                                                layout: {
                                                                                    let mut layout =
                                                                                        LayoutStyle::default();
                                                                                    layout.size.max_width =
                                                                                        Some(tab_max_width);
                                                                                    layout
                                                                                },
                                                                                text: label,
                                                                                style: Some(text_style.clone()),
                                                                                color: Some(tab_fg),
                                                                                wrap: TextWrap::None,
                                                                                overflow: TextOverflow::Ellipsis,
                                                                            }),
                                                                        ];

                                                                        if has_trailing_slot {
                                                                            if show_close {
                                                                                if let Some(close_command) =
                                                                                    tab_close_command.clone()
                                                                                {
                                                                                    children.push(cx.pressable(
                                                                                        PressableProps {
                                                                                            layout: fixed_square_layout(Px(18.0)),
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
                                                                                                    if let Some(cmd) =
                                                                                                        pane_activate_cmd_for_close.clone()
                                                                                                    {
                                                                                                        host.dispatch_command(Some(acx.window), cmd);
                                                                                                    }
                                                                                                    host.dispatch_command(
                                                                                                        Some(acx.window),
                                                                                                        close_command.clone(),
                                                                                                    );
                                                                                                },
                                                                                            );
                                                                                            cx.pressable_on_activate(close_handler);

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
                                                                                                |cx| {
                                                                                                    vec![cx.text_props(TextProps {
                                                                                                        layout: LayoutStyle::default(),
                                                                                                        text: Arc::from("×"),
                                                                                                        style: Some(text_style.clone()),
                                                                                                        color: Some(tab_fg),
                                                                                                        wrap: TextWrap::None,
                                                                                                        overflow: TextOverflow::Clip,
                                                                                                    })]
                                                                                                },
                                                                                            )]
                                                                                        },
                                                                                    ));
                                                                                }
                                                                            } else if tab_dirty {
                                                                                let mut dot_style = text_style.clone();
                                                                                dot_style.size =
                                                                                    Px((dot_style.size.0 - 1.0).max(10.0));
                                                                                children.push(cx.container(
                                                                                    ContainerProps {
                                                                                        layout: fixed_square_layout(Px(18.0)),
                                                                                        ..Default::default()
                                                                                    },
                                                                                    |cx| {
                                                                                        vec![cx.flex(
                                                                                            FlexProps {
                                                                                                layout: fill_layout(),
                                                                                                direction:
                                                                                                    fret_core::Axis::Horizontal,
                                                                                                justify:
                                                                                                    MainAlign::Center,
                                                                                                align: CrossAlign::Center,
                                                                                                ..Default::default()
                                                                                            },
                                                                                            |cx| {
                                                                                                vec![cx.text_props(TextProps {
                                                                                                    layout: LayoutStyle::default(),
                                                                                                    text: Arc::from("•"),
                                                                                                    style: Some(dot_style),
                                                                                                    color: Some(dirty_fg),
                                                                                                    wrap: TextWrap::None,
                                                                                                    overflow: TextOverflow::Clip,
                                                                                                })]
                                                                                            },
                                                                                        )]
                                                                                    },
                                                                                ));
                                                                            } else {
                                                                                children.push(cx.container(
                                                                                    ContainerProps {
                                                                                        layout: fixed_square_layout(Px(18.0)),
                                                                                        ..Default::default()
                                                                                    },
                                                                                    |_cx| Vec::new(),
                                                                                ));
                                                                            }
                                                                        }

                                                                        children
                                                                    },
                                                                )]
                                                            },
                                                        )]
                                                    },
                                                );

                                                #[cfg(feature = "shadcn-context-menu")]
                                                {
                                                    let open = get_context_menu_open_model(cx);
                                                    let close_cmd = tab_close_command.clone();
                                                    let has_left = has_left;
                                                    let has_right = has_right;
                                                    let has_others = has_others;
                                                    ContextMenu::new(open).into_element(
                                                        cx,
                                                        |_cx| tab_element,
                                                        move |_cx| {
                                                            let mut entries = Vec::new();
                                                            if let Some(cmd) = close_cmd {
                                                                entries.push(ContextMenuEntry::Item(
                                                                    ContextMenuItem::new("Close Tab").on_select(cmd),
                                                                ));
                                                            }
                                                            entries.push(ContextMenuEntry::Item(
                                                                ContextMenuItem::new("Close Other Tabs")
                                                                    .disabled(!has_others)
                                                                    .on_select(CommandId::new(
                                                                        crate::commands::CMD_WORKSPACE_TAB_CLOSE_OTHERS,
                                                                    )),
                                                            ));
                                                            entries.push(ContextMenuEntry::Item(
                                                                ContextMenuItem::new("Close Tabs to the Left")
                                                                    .disabled(!has_left)
                                                                    .on_select(CommandId::new(
                                                                        crate::commands::CMD_WORKSPACE_TAB_CLOSE_LEFT,
                                                                    )),
                                                            ));
                                                            entries.push(ContextMenuEntry::Item(
                                                                ContextMenuItem::new("Close Tabs to the Right")
                                                                    .disabled(!has_right)
                                                                    .on_select(CommandId::new(
                                                                        crate::commands::CMD_WORKSPACE_TAB_CLOSE_RIGHT,
                                                                    )),
                                                            ));
                                                            entries.push(ContextMenuEntry::Separator);
                                                            entries.push(ContextMenuEntry::Item(
                                                                ContextMenuItem::new("Split Right").on_select(
                                                                    CommandId::new(crate::commands::CMD_WORKSPACE_PANE_SPLIT_RIGHT),
                                                                ),
                                                            ));
                                                            entries.push(ContextMenuEntry::Item(
                                                                ContextMenuItem::new("Split Left").on_select(
                                                                    CommandId::new(crate::commands::CMD_WORKSPACE_PANE_SPLIT_LEFT),
                                                                ),
                                                            ));
                                                            entries.push(ContextMenuEntry::Item(
                                                                ContextMenuItem::new("Split Up").on_select(
                                                                    CommandId::new(crate::commands::CMD_WORKSPACE_PANE_SPLIT_UP),
                                                                ),
                                                            ));
                                                            entries.push(ContextMenuEntry::Item(
                                                                ContextMenuItem::new("Split Down").on_select(
                                                                    CommandId::new(crate::commands::CMD_WORKSPACE_PANE_SPLIT_DOWN),
                                                                ),
                                                            ));
                                                            entries
                                                        },
                                                    )
                                                }

                                                #[cfg(not(feature = "shadcn-context-menu"))]
                                                {
                                                    tab_element
                                                }
                                            });
                                            out.push(element);
                                        }

                                        out
                                    },
                                )];

                                AnyElement::new(
                                    id,
                                    ElementKind::Scroll(ScrollProps {
                                        layout: fill_grow_layout(),
                                        axis: ScrollAxis::X,
                                        scroll_handle: Some(scroll_handle.clone()),
                                        // Important: keep the scroll child width `Auto` (see
                                        // `scroll_content_row_layout`) to avoid recursive
                                        // "fill-to-max" probing that can blow the stack in layout.
                                        probe_unbounded: true,
                                        ..Default::default()
                                    }),
                                    children,
                                )
                            });

                            let scroll_step = Px(120.0);
                            let scroll_x = scroll_handle.offset().x;
                            let scroll_max_x = scroll_handle.max_offset().x;
                            let can_scroll_left = scroll_x.0 > 0.5;
                            let can_scroll_right = scroll_x.0 + 0.5 < scroll_max_x.0;

                            let scroll_button = |cx: &mut ElementContext<'_, H>,
                                                 enabled: bool,
                                                 glyph: &'static str,
                                                 a11y_label: &'static str,
                                                 delta_x: f32| {
                                let scroll_handle = scroll_handle.clone();
                                let button_text_style = text_style.clone();
                                let glyph = Arc::<str>::from(glyph);
                                cx.pressable(
                                    PressableProps {
                                        layout: fixed_square_layout(Px(20.0)),
                                        enabled,
                                        focusable: false,
                                        a11y: PressableA11y {
                                            role: Some(SemanticsRole::Button),
                                            label: Some(Arc::from(a11y_label)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    move |cx, state| {
                                        let handler: OnActivate = Arc::new(move |_host, _acx, _r| {
                                            let current = scroll_handle.offset();
                                            scroll_handle.set_offset(Point::new(
                                                Px(current.x.0 + delta_x * scroll_step.0),
                                                current.y,
                                            ));
                                        });
                                        cx.pressable_on_activate(handler);

                                        let alpha = if enabled { 1.0 } else { 0.35 };
                                        let fg = Some(Color {
                                            a: scroll_button_fg.a * alpha,
                                            ..scroll_button_fg
                                        });
                                        let bg = if enabled && (state.hovered || state.pressed) {
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
                                            |cx| {
                                                vec![centered_row(
                                                    cx,
                                                    glyph.clone(),
                                                    button_text_style.clone(),
                                                    fg,
                                                )]
                                            },
                                        )]
                                    },
                                )
                            };

                            let mut rects: Vec<WorkspaceTabHitRect> = Vec::new();
                            for (id, el) in tab_elements.borrow().iter() {
                                if let Some(rect) = cx.last_bounds_for_element(*el) {
                                    rects.push(WorkspaceTabHitRect {
                                        id: id.clone(),
                                        rect,
                                    });
                                }
                            }

                            let should_sync_rects =
                                drag_snapshot.pointer.is_some() || drag_snapshot.dragged_tab.is_some();
                            let should_clear = drag_snapshot.dragged_tab.as_ref().is_some_and(|dragged| {
                                !rects.iter().any(|r| r.id.as_ref() == dragged.as_ref())
                            });
                            let rects_changed = rects != drag_snapshot.tab_rects;

                            if should_clear || (should_sync_rects && rects_changed) {
                                let rects_for_model = rects.clone();
                                let _ = cx.app.models_mut().update(&drag_model, move |st| {
                                    if should_clear {
                                        *st = WorkspaceTabStripDragState::default();
                                        return;
                                    }

                                    st.tab_rects = rects_for_model;
                                    if let Some((target, _)) = st.drop_target.clone() {
                                        if !st.tab_rects.iter().any(|r| r.id.as_ref() == target.as_ref()) {
                                            st.drop_target = None;
                                        }
                                    }
                                });
                            }

                            if let (Some(tab_drag_model), Some(pane_id)) =
                                (tab_drag_model.clone(), pane_id.clone())
                            {
                                cx.observe_model(&tab_drag_model, Invalidation::Paint);
                                let tab_drag_snapshot = cx
                                    .get_model_cloned(&tab_drag_model, Invalidation::Paint)
                                    .unwrap_or_default();

                                let should_compute = tab_drag_snapshot.pointer.is_some()
                                    && tab_drag_snapshot.hovered_pane.as_deref() == Some(pane_id.as_ref())
                                    && tab_drag_snapshot.hovered_zone == Some(WorkspaceTabDropZone::Center);

                                if let (true, Some(pointer_id)) = (should_compute, tab_drag_snapshot.pointer) {
                                    let session = cx.app.drag(pointer_id);
                                    let inside_tab_strip = session.is_some_and(|s| {
                                        if s.kind != DRAG_KIND_WORKSPACE_TAB || !s.dragging {
                                            return false;
                                        }
                                        if s.current_window != cx.window {
                                            return false;
                                        }
                                        let Some(scroll_el) = scroll_element.get() else {
                                            return false;
                                        };
                                        cx.last_bounds_for_element(scroll_el)
                                            .is_some_and(|b| b.contains(s.position))
                                    });

                                    let (next_tab, next_side, next_rects) = if inside_tab_strip {
                                        let session = session.expect("checked above");
                                        let drop_target = tab_drag_snapshot
                                            .dragged_tab
                                            .as_deref()
                                            .and_then(|dragged| {
                                                compute_tab_drop_target(session.position, dragged, &rects)
                                            });
                                        (
                                            drop_target.as_ref().map(|(id, _)| id.clone()),
                                            drop_target.as_ref().map(|(_, side)| *side),
                                            rects.clone(),
                                        )
                                    } else {
                                        (None, None, Vec::new())
                                    };

                                    if tab_drag_snapshot.hovered_tab != next_tab
                                        || tab_drag_snapshot.hovered_tab_side != next_side
                                        || tab_drag_snapshot.hovered_pane_tab_rects != next_rects
                                    {
                                        let _ = cx.app.models_mut().update(&tab_drag_model, |st| {
                                            if st.pointer != Some(pointer_id)
                                                || st.hovered_pane.as_deref() != Some(pane_id.as_ref())
                                                || st.hovered_zone != Some(WorkspaceTabDropZone::Center)
                                            {
                                                return;
                                            }
                                            st.hovered_tab = next_tab.clone();
                                            st.hovered_tab_side = next_side;
                                            st.hovered_pane_tab_rects = next_rects.clone();
                                        });
                                    }
                                }
                            }

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
                                    vec![
                                        scroll_button(cx, can_scroll_left, "<", "Scroll left", -1.0),
                                        scroll,
                                        scroll_button(cx, can_scroll_right, ">", "Scroll right", 1.0),
                                    ]
                                },
                            )]
                        },
                    );

                    let active_changed = last_active.as_deref() != active.as_deref();
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
                        state.last_active = active.clone();
                    });

                    vec![root]
            },
        )
    }
}
