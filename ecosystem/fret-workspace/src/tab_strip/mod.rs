use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use fret_core::{
    Corners, Edges, KeyCode, MouseButton, Point, Px, SemanticsRole, TextOverflow, TextSlant,
    TextWrap,
};
use fret_runtime::{CommandId, Effect, Model};
use fret_ui::action::{
    ActivateReason, OnActivate, OnInternalDrag, OnPressablePointerMove, OnPressablePointerUp,
    OnWheel, PressablePointerUpResult,
};
use fret_ui::element::ElementKind;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, HitTestGateProps, InsetEdge,
    InternalDragRegionProps, LayoutStyle, Length, MainAlign, PointerRegionProps, PositionStyle,
    PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps, ScrollAxis, ScrollProps,
    SemanticsProps, TextInkOverflow, TextProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::dnd as ui_dnd;

use crate::focus_registry::{WorkspaceTabElementKey, workspace_tab_element_registry_model};

use crate::commands::CMD_WORKSPACE_PANE_FOCUS_CONTENT;
use crate::commands::{
    pane_activate_command, pane_move_active_tab_to_command, tab_activate_command,
    tab_close_command, tab_move_active_after_command, tab_move_active_before_command,
    tab_pin_command, tab_unpin_command,
};
use crate::tab_drag::{
    DRAG_KIND_WORKSPACE_TAB, WorkspacePaneDragGeometry, WorkspaceTabDragState,
    WorkspaceTabDropIntent, WorkspaceTabDropZone, WorkspaceTabInsertionSide,
    resolve_workspace_tab_drop_intent,
};

mod drag_state;
mod geometry;
mod intent;
mod interaction;
mod kernel;
mod layouts;
mod state;
mod surface;
mod theme;
mod utils;
mod widgets;

#[cfg(feature = "shadcn-context-menu")]
mod overflow;

#[cfg(feature = "shadcn-context-menu")]
use overflow::compute_overflow_menu_entries;

use kernel::{
    WorkspaceTabStripDropTarget, compute_tab_strip_edge_auto_scroll_delta_x,
    compute_workspace_tab_strip_drop_target,
};

use drag_state::{WorkspaceTabStripDragState, get_drag_model, read_drag_snapshot_for_pointer};
use geometry::{bounds_for_optional_element_id, collect_tab_hit_rects};
use intent::{WorkspaceTabStripIntent, dispatch_intent};
use interaction::tab_pointer_down_handler;
use layouts::{
    fill_grow_layout, fill_layout, row_layout, tab_list_semantics_layout,
    tab_strip_scroll_content_layout,
};
use state::{WorkspaceTabStripState, get_focus_restore_model, get_reveal_hint_model};
use theme::WorkspaceTabStripTheme;
use utils::{
    dnd_scope_for_pane, resolve_end_drop_target_in_canonical_order,
    scroll_rect_into_view_x_with_margin,
};
use widgets::{
    tab_close_button, tab_dirty_indicator, tab_strip_scroll_button, tab_trailing_slot_placeholder,
};

#[cfg(feature = "shadcn-context-menu")]
use widgets::tab_strip_overflow_button;

#[cfg(feature = "shadcn-context-menu")]
use state::get_context_menu_open_model;

#[cfg(feature = "shadcn-context-menu")]
use state::get_overflow_menu_open_model;

#[cfg(feature = "shadcn-context-menu")]
use fret_ui_shadcn::{
    ContextMenu, ContextMenuEntry, ContextMenuItem, DropdownMenu, DropdownMenuAlign,
    DropdownMenuSide,
};

#[derive(Debug, Clone)]
pub struct WorkspaceTab {
    pub id: Arc<str>,
    pub title: Arc<str>,
    pub command: CommandId,
    pub close_command: Option<CommandId>,
    pub dirty: bool,
    pub pinned: bool,
    pub preview: bool,
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
            pinned: false,
            preview: false,
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

    pub fn pinned(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }

    pub fn preview(mut self, preview: bool) -> Self {
        self.preview = preview;
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
    /// Most-recently-used ordering (including the active tab at index 0) when the strip is backed
    /// by `WorkspaceTabs`.
    ///
    /// This is used for editor-grade focus restore policies (e.g. close active tab while the tab
    /// strip is focused).
    mru: Option<Arc<[Arc<str>]>>,
    height: Px,
    pane_id: Option<Arc<str>>,
    tab_drag: Option<Model<WorkspaceTabDragState>>,
    root_test_id: Option<Arc<str>>,
    tab_test_id_prefix: Option<Arc<str>>,
}

impl WorkspaceTabStrip {
    pub fn new(active: impl Into<Arc<str>>) -> Self {
        Self {
            active: Some(active.into()),
            tabs: Vec::new(),
            mru: None,
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
            mru: None,
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

    /// Provide MRU ordering for editor-grade behaviors (e.g. focus restore after close).
    pub fn mru(mut self, mru: impl IntoIterator<Item = Arc<str>>) -> Self {
        self.mru = Some(mru.into_iter().collect::<Vec<_>>().into());
        self
    }

    pub fn from_workspace_tabs(
        state: &crate::tabs::WorkspaceTabs,
        title: impl Fn(&str) -> Arc<str>,
    ) -> Self {
        let active = state.active().cloned();
        let mut out = WorkspaceTabStrip::new_optional(active);
        out.mru = Some(state.mru().to_vec().into());
        out.tabs = state
            .tabs()
            .iter()
            .filter_map(|id| {
                let activate = tab_activate_command(id.as_ref())?;
                let mut tab = WorkspaceTab::new(id.clone(), title(id.as_ref()), activate)
                    .pinned(state.is_tab_pinned(id.as_ref()))
                    .preview(state.is_tab_preview(id.as_ref()));
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
        let mru = self.mru;
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

        let theme = WorkspaceTabStripTheme::resolve(cx);
        let WorkspaceTabStripTheme {
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
        } = theme;

        cx.semantics(
            SemanticsProps {
                layout: tab_list_semantics_layout(),
                role: SemanticsRole::TabList,
                test_id: root_test_id.clone(),
                ..Default::default()
            },
            |cx| {
                let pinned_by_id: Arc<std::collections::HashMap<Arc<str>, bool>> = Arc::new(
                    tabs.iter()
                        .map(|tab| (tab.id.clone(), tab.pinned))
                        .collect(),
                );
                let canonical_tab_order: Arc<[Arc<str>]> = tabs
                    .iter()
                    .map(|tab| tab.id.clone())
                    .collect::<Vec<_>>()
                    .into();
                let roving_tab_commands: Arc<[CommandId]> = tabs
                    .iter()
                    .map(|tab| tab.command.clone())
                    .collect::<Vec<_>>()
                    .into();
                let roving_disabled: Arc<[bool]> = vec![false; tabs.len()].into();

                let tab_element_registry = workspace_tab_element_registry_model(cx);
                let focus_restore_model = get_focus_restore_model(cx);
                let reveal_hint_model = get_reveal_hint_model(cx, pane_id.as_ref());
                cx.observe_model(&reveal_hint_model, Invalidation::Paint);

                let (
                    scroll_handle,
                    last_active,
                    cached_tab_rects,
                    cached_scroll_viewport,
                ) = cx.with_state(
                    WorkspaceTabStripState::default,
                    |state| {
                        (
                            state.scroll.clone(),
                            state.last_active.clone(),
                            state.last_tab_rects.clone(),
                            state.last_scroll_viewport,
                        )
                    },
                );
                let scroll_element = Cell::<Option<GlobalElementId>>::new(None);
                let active_tab_element = Cell::<Option<GlobalElementId>>::new(None);
                let tab_elements: Rc<RefCell<Vec<(Arc<str>, GlobalElementId)>>> =
                    Rc::new(RefCell::new(Vec::new()));
                let pinned_boundary_element = Cell::<Option<GlobalElementId>>::new(None);
                let end_drop_target_element = Cell::<Option<GlobalElementId>>::new(None);
                let overflow_control_element: Rc<Cell<Option<GlobalElementId>>> =
                    Rc::new(Cell::new(None));
                let scroll_left_control_element: Rc<Cell<Option<GlobalElementId>>> =
                    Rc::new(Cell::new(None));
                let scroll_right_control_element: Rc<Cell<Option<GlobalElementId>>> =
                    Rc::new(Cell::new(None));

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
                            padding: Edges::all(Px(2.0)).into(),
                            background: bar_bg,
                            border: Edges {
                                bottom: Px(1.0),
                                ..Edges::all(Px(0.0))
                            },
                            border_color: bar_border,
                            ..Default::default()
                        },
                        |cx| {
                            // Escape exits the tab strip (best-effort) by asking the workspace
                            // shell to restore focus to pane content.
                            cx.key_add_on_key_down_for(
                                cx.root_id(),
                                Arc::new(move |host, acx, down| {
                                    if down.ime_composing {
                                        return false;
                                    }
                                    if down.key != KeyCode::Escape {
                                        return false;
                                    }
                                    host.dispatch_command(
                                        Some(acx.window),
                                        CommandId::from(CMD_WORKSPACE_PANE_FOCUS_CONTENT),
                                    );
                                    true
                                }),
                            );

                            let scroll = cx.keyed("workspace-tab-strip-scroll", |cx| {
                                let id = cx.root_id();
                                scroll_element.set(Some(id));

                                let children = vec![cx.roving_flex(
                                    RovingFlexProps {
                                        flex: FlexProps {
                                            layout: tab_strip_scroll_content_layout(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap: Px(2.0).into(),
                                            padding: Edges::all(Px(0.0)).into(),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            wrap: false,
                                        },
                                        roving: RovingFocusProps {
                                            enabled: true,
                                            wrap: true,
                                            disabled: roving_disabled.clone(),
                                        },
                                    },
                                    |cx| {
                                        cx.roving_nav_apg();
                                        let pane_activate_cmd_for_roving = pane_activate_cmd.clone();
                                        let tab_commands_for_roving = roving_tab_commands.clone();
                                        let canonical_tab_order_for_roving =
                                            canonical_tab_order.clone();
                                        let reveal_hint_model_for_roving =
                                            reveal_hint_model.clone();
                                        cx.roving_on_active_change(Arc::new(
                                            move |host, acx, idx| {
                                                let tab_id = canonical_tab_order_for_roving
                                                    .get(idx)
                                                    .cloned();
                                                if let Some(tab_id) = tab_id {
                                                    let _ = host.models_mut().update(
                                                        &reveal_hint_model_for_roving,
                                                        |st| {
                                                            st.tab_id = Some(tab_id);
                                                            st.reason =
                                                                Some(ActivateReason::Keyboard);
                                                        },
                                                    );
                                                }
                                                if let Some(cmd) =
                                                    pane_activate_cmd_for_roving.clone()
                                                {
                                                    dispatch_intent(
                                                        host,
                                                        acx.window,
                                                        WorkspaceTabStripIntent::Activate(cmd),
                                                    );
                                                }
                                                let Some(cmd) =
                                                    tab_commands_for_roving.get(idx).cloned()
                                                else {
                                                    return;
                                                };
                                                dispatch_intent(
                                                    host,
                                                    acx.window,
                                                    WorkspaceTabStripIntent::Activate(cmd),
                                                );
                                                dispatch_intent(
                                                    host,
                                                    acx.window,
                                                    WorkspaceTabStripIntent::RequestRedraw,
                                                );
                                            },
                                        ));

                                        let mut out: Vec<AnyElement> = Vec::new();
                                        let pinned_count = tabs
                                            .iter()
                                            .take_while(|tab| tab.pinned)
                                            .count();

                                        for (index, tab) in tabs.iter().enumerate() {
                                            if pinned_count > 0
                                                && pinned_count < tabs.len()
                                                && index == pinned_count
                                            {
                                                let test_id = root_test_id.as_ref().map(|root| {
                                                    Arc::<str>::from(format!(
                                                        "{root}.drop_pinned_boundary"
                                                    ))
                                                });
                                                let active = matches!(
                                                    &drop_target,
                                                    WorkspaceTabStripDropTarget::PinnedBoundary
                                                );
                                                let border = if active {
                                                    Edges {
                                                        left: Px(2.0),
                                                        ..Edges::all(Px(0.0))
                                                    }
                                                } else {
                                                    Edges {
                                                        left: Px(1.0),
                                                        ..Edges::all(Px(0.0))
                                                    }
                                                };
                                                let border_color = if active {
                                                    indicator_color
                                                } else {
                                                    bar_border
                                                };

                                                let boundary = cx.keyed(
                                                    "workspace-tab-strip-pinned-boundary",
                                                    |cx| {
                                                        let mut layout = LayoutStyle::default();
                                                        layout.size.width = Length::Px(Px(8.0));
                                                        layout.size.height = Length::Fill;
                                                        layout.flex.shrink = 0.0;
                                                        let mut el = cx.container(
                                                            ContainerProps {
                                                                layout,
                                                                border,
                                                                border_color,
                                                                ..Default::default()
                                                            },
                                                            |_cx| Vec::new(),
                                                        );
                                                        if let Some(id) = test_id.clone() {
                                                            el = el.test_id(id);
                                                        }
                                                        pinned_boundary_element.set(Some(el.id));
                                                        el
                                                    },
                                                );
                                                out.push(boundary);
                                            }

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
                                            let tab_pinned = tab.pinned;
                                            let tab_preview = tab.preview;
                                            let pane_id_for_registry = pane_id.clone();
                                            let tab_element_registry_for_tab = tab_element_registry.clone();
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
                                            let is_roving_focusable =
                                                is_active || (active.is_none() && index == 0);
                                            let pos_in_set = (index as u32) + 1;
                                            let cross_drop_target = cross_drop_target.clone();

                                            let tab_key = tab_id.clone();
                                            let element = cx.keyed(tab_key, |cx| {
                                                let tab_test_id = tab_test_id_prefix_for_tab
                                                    .as_ref()
                                                    .map(|prefix| {
                                                        Arc::<str>::from(format!(
                                                            "{prefix}-{}",
                                                            tab_id.as_ref()
                                                        ))
                                                    });
                                                let tab_chrome_test_id = tab_test_id
                                                    .as_ref()
                                                    .map(|id| Arc::<str>::from(format!("{id}.chrome")));
                                                let tab_preview_test_id = tab_test_id
                                                    .as_ref()
                                                    .map(|id| Arc::<str>::from(format!("{id}.preview")));
                                                let tab_pinned_test_id = tab_test_id
                                                    .as_ref()
                                                    .map(|id| Arc::<str>::from(format!("{id}.pinned")));
                                                let tab_dirty_test_id = tab_test_id
                                                    .as_ref()
                                                    .map(|id| Arc::<str>::from(format!("{id}.dirty")));
                                                        let tab_element = cx.pressable_with_id(
                                                            PressableProps {
                                                                layout: {
                                                                    let mut layout =
                                                                        LayoutStyle::default();
                                                                    layout.size.height = Length::Fill;
                                                                    layout.size.width = Length::Auto;
                                                                    // Allow tabs to shrink so long titles don't push later tabs
                                                                    // fully off-screen (ellipsis needs min-width: 0 behavior).
                                                                    layout.size.min_width =
                                                                        Some(Length::Px(Px(0.0)));
                                                                    layout.flex.shrink = 1.0;
                                                                    layout
                                                                },
                                                                a11y: PressableA11y {
                                                                    role: Some(SemanticsRole::Tab),
                                                                    label: Some(tab_title.clone()),
                                                            test_id: tab_test_id.clone(),
                                                            selected: is_active,
                                                            pos_in_set: Some(pos_in_set),
                                                            set_size: Some(set_size),
                                                            ..Default::default()
                                                        },
                                                        focusable: is_roving_focusable,
                                                        ..Default::default()
                                                    },
                                                    |cx, press_state, element_id| {
                                                        tab_elements
                                                            .borrow_mut()
                                                            .push((tab_id.clone(), element_id));
                                                        if is_active {
                                                            active_tab_element.set(Some(element_id));
                                                        }

                                                        // Registry for cross-frame focus restore.
                                                        //
                                                        // We avoid updating the model unless the mapping actually
                                                        // changes, because `ModelStore::update` always marks dirty.
                                                        let registry_key = WorkspaceTabElementKey {
                                                            window: cx.window,
                                                            pane_id: pane_id_for_registry.clone(),
                                                            tab_id: tab_id.clone(),
                                                        };
                                                        let needs_registry_update = cx
                                                            .app
                                                            .models_mut()
                                                            .read(&tab_element_registry_for_tab, |reg| {
                                                                reg.get(&registry_key) != Some(element_id)
                                                            })
                                                            .unwrap_or(true);
                                                        if needs_registry_update {
                                                            let _ = cx.app.models_mut().update(
                                                                &tab_element_registry_for_tab,
                                                                |reg| {
                                                                    reg.set_if_changed(registry_key, element_id);
                                                                },
                                                            );
                                                        }

                                                        let tab_activate_cmd_for_activate =
                                                            tab_activate_command.clone();
                                                        let pane_activate_cmd_for_activate_handler =
                                                            pane_activate_cmd_for_activate.clone();
                                                        let reveal_hint_model_for_activate =
                                                            reveal_hint_model.clone();
                                                        let tab_id_for_activate =
                                                            tab_id.clone();
                                                        let handler: OnActivate = Arc::new(
                                                            move |host, acx, reason| {
                                                                let _ = host.models_mut().update(
                                                                    &reveal_hint_model_for_activate,
                                                                    |st| {
                                                                        st.tab_id = Some(
                                                                            tab_id_for_activate
                                                                                .clone(),
                                                                        );
                                                                        st.reason = Some(reason);
                                                                    },
                                                                );
                                                                if let Some(cmd) =
                                                                    pane_activate_cmd_for_activate_handler
                                                                        .clone()
                                                                {
                                                                    dispatch_intent(
                                                                        host,
                                                                        acx.window,
                                                                        WorkspaceTabStripIntent::Activate(cmd),
                                                                    );
                                                                }
                                                                dispatch_intent(
                                                                    host,
                                                                    acx.window,
                                                                    WorkspaceTabStripIntent::Activate(
                                                                        tab_activate_cmd_for_activate.clone(),
                                                                    ),
                                                                );
                                                            },
                                                        );
                                                        cx.pressable_on_activate(handler);

                                                        cx.pressable_on_pointer_down(
                                                            tab_pointer_down_handler(
                                                                drag_model.clone(),
                                                                tab_id.clone(),
                                                                tab_activate_command.clone(),
                                                                pane_activate_cmd_for_activate.clone(),
                                                                tab_close_command.clone(),
                                                                dnd.clone(),
                                                                dnd_scope,
                                                            ),
                                                        );

                                                        let dnd_on_move: OnPressablePointerMove = {
                                                            let drag_model = drag_model.clone();
                                                            let tab_command = tab_drag_command.clone();
                                                            let pane_activate_cmd = pane_activate_cmd_for_drag.clone();
                                                            let tab_drag_model = tab_drag_model_for_drag.clone();
                                                            let source_pane = pane_id_for_drag.clone();
                                                            let dragged_tab_id = tab_id.clone();
                                                            let dnd = dnd.clone();
                                                            let scroll_handle = scroll_handle.clone();
                                                            let pinned_by_id = pinned_by_id.clone();
                                                            let canonical_tab_order = canonical_tab_order.clone();
                                                            Arc::new(move |host, acx, mv| {
                                                                let Some(snapshot) =
                                                                    read_drag_snapshot_for_pointer(
                                                                        host.models_mut(),
                                                                        &drag_model,
                                                                        mv.pointer_id,
                                                                    )
                                                                else {
                                                                    return false;
                                                                };
                                                                let start_tick = snapshot.start_tick;
                                                                let start_position = snapshot.start_position;
                                                                let dragging = snapshot.dragging;
                                                                let dragged_tab = snapshot.dragged_tab;
                                                                let tab_rects = snapshot.tab_rects;
                                                                let pinned_boundary_rect =
                                                                    snapshot.pinned_boundary_rect;
                                                                let end_drop_target_rect =
                                                                    snapshot.end_drop_target_rect;
                                                                let scroll_viewport_rect =
                                                                    snapshot.scroll_viewport_rect;
                                                                let overflow_control_rect =
                                                                    snapshot.overflow_control_rect;
                                                                let scroll_left_control_rect =
                                                                    snapshot.scroll_left_control_rect;
                                                                let scroll_right_control_rect =
                                                                    snapshot.scroll_right_control_rect;

                                                                let Some(dragged_tab) = dragged_tab else {
                                                                    return false;
                                                                };

                                                                let mut activate_on_drag_start = false;

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
                                                                        // Defensive fallback: some synthetic/scripted pointer
                                                                        // injections can produce move events that do not advance the
                                                                        // DnD sensor in the way we expect (tick quirks, transport
                                                                        // differences). Keep the user-visible behavior stable by
                                                                        // falling back to a simple distance threshold.
                                                                        let dx = mv.position.x.0 - start_position.x.0;
                                                                        let dy = mv.position.y.0 - start_position.y.0;
                                                                        let dist2 = (dx * dx) + (dy * dy);
                                                                        if dist2 < (6.0 * 6.0) {
                                                                            return false;
                                                                        }
                                                                    }
                                                                    activate_on_drag_start = true;
                                                                    ui_dnd::clear_pointer_in_scope(
                                                                        host.models_mut(),
                                                                        &dnd,
                                                                        acx.window,
                                                                        DRAG_KIND_WORKSPACE_TAB,
                                                                        dnd_scope,
                                                                        mv.pointer_id,
                                                                    );
                                                                }

                                                                let hit_position =
                                                                    mv.position_window.unwrap_or(mv.position);

                                                                let dragged = dragged_tab;
                                                                let mut drop_target =
                                                                    compute_workspace_tab_strip_drop_target(
                                                                        hit_position,
                                                                        dragged.as_ref(),
                                                                        &tab_rects,
                                                                        pinned_boundary_rect,
                                                                        end_drop_target_rect,
                                                                        scroll_viewport_rect,
                                                                        overflow_control_rect,
                                                                        scroll_left_control_rect,
                                                                        scroll_right_control_rect,
                                                                    );
                                                                if matches!(drop_target, WorkspaceTabStripDropTarget::End) {
                                                                    drop_target = resolve_end_drop_target_in_canonical_order(
                                                                        pinned_by_id.as_ref(),
                                                                        canonical_tab_order.as_ref(),
                                                                        dragged.as_ref(),
                                                                    )
                                                                    .map(|id| {
                                                                        WorkspaceTabStripDropTarget::Tab(
                                                                            id,
                                                                            WorkspaceTabInsertionSide::After,
                                                                        )
                                                                    })
                                                                    .unwrap_or(WorkspaceTabStripDropTarget::None);
                                                                } else if let WorkspaceTabStripDropTarget::Tab(target, _) =
                                                                    &drop_target
                                                                {
                                                                    let dragged_is_pinned =
                                                                        pinned_by_id.get(dragged.as_ref()).copied().unwrap_or(false);
                                                                    let target_is_pinned = pinned_by_id
                                                                        .get(target)
                                                                        .copied()
                                                                        .unwrap_or(false);
                                                                    if dragged_is_pinned != target_is_pinned {
                                                                        drop_target =
                                                                            WorkspaceTabStripDropTarget::None;
                                                                    }
                                                                }

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
                                                                        dispatch_intent(
                                                                            host,
                                                                            acx.window,
                                                                            WorkspaceTabStripIntent::Activate(cmd),
                                                                        );
                                                                    }
                                                                    dispatch_intent(
                                                                        host,
                                                                        acx.window,
                                                                        WorkspaceTabStripIntent::Activate(tab_command.clone()),
                                                                    );

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
                                                                        mv.position_window
                                                                            .unwrap_or(mv.position),
                                                                    );
                                                                    if let Some(drag) = host.drag_mut(mv.pointer_id) {
                                                                        drag.position = mv
                                                                            .position_window
                                                                            .unwrap_or(mv.position);
                                                                        drag.dragging = true;
                                                                    }
                                                                }

                                                                // Keep the drag session position fresh on every move so other
                                                                // workspace surfaces (pane drop zones, split previews, etc.) can
                                                                // resolve hover/drop behavior from `DragSession::position`.
                                                                if dragging || activate_on_drag_start {
                                                                    let pointer_pos = mv
                                                                        .position_window
                                                                        .unwrap_or(mv.position);
                                                                    if let Some(drag) = host.drag_mut(mv.pointer_id)
                                                                        && drag.kind == DRAG_KIND_WORKSPACE_TAB
                                                                    {
                                                                        drag.position = pointer_pos;
                                                                        drag.dragging = true;
                                                                    }

                                                                    if let Some(model) = tab_drag_model.clone() {
                                                                        let _ = host.models_mut().update(&model, |st| {
                                                                            if st.pointer != Some(mv.pointer_id)
                                                                                || st.source_window
                                                                                    != Some(acx.window)
                                                                            {
                                                                                return;
                                                                            }

                                                                            let mut next_hovered: Option<Arc<str>> =
                                                                                None;
                                                                            for (pane_id, geom) in
                                                                                &st.pane_geometry
                                                                            {
                                                                                if geom.bounds
                                                                                    .contains(
                                                                                        pointer_pos,
                                                                                    )
                                                                                {
                                                                                    next_hovered =
                                                                                        Some(pane_id.clone());
                                                                                    break;
                                                                                }
                                                                            }

                                                                            if st.hovered_pane
                                                                                != next_hovered
                                                                            {
                                                                                st.hovered_tab =
                                                                                    None;
                                                                                st.hovered_tab_side =
                                                                                    None;
                                                                                st.hovered_pane_tab_rects =
                                                                                    Vec::new();
                                                                            }

                                                                            let prev_hovered =
                                                                                st.hovered_pane.clone();
                                                                            st.hovered_pane = next_hovered;
                                                                            if st.hovered_pane != prev_hovered {
                                                                                st.hovered_zone = st
                                                                                    .hovered_pane
                                                                                    .as_ref()
                                                                                    .map(|_| {
                                                                                        WorkspaceTabDropZone::Center
                                                                                    });
                                                                            }
                                                                        });
                                                                    }
                                                                }
                                                                // Best-effort: if we changed the scroll offset due to an earlier
                                                                // auto-scroll, make sure we keep requesting redraws while dragging.
                                                                if dragging && scroll_handle.max_offset().x.0 > 0.5 {
                                                                    host.request_redraw(acx.window);
                                                                }
                                                                host.request_redraw(acx.window);
                                                                true
                                                            })
                                                        };
                                                        cx.pressable_on_pointer_move(dnd_on_move);

                                                        let tab_id_for_pinned_boundary =
                                                            tab_id.clone();
                                                        let dnd_on_up: OnPressablePointerUp = {
                                                            let drag_model = drag_model.clone();
                                                            let tab_command = tab_drag_command.clone();
                                                            let tab_drag_model = tab_drag_model_for_drag.clone();
                                                            let dnd = dnd.clone();
                                                            let pinned_by_id = pinned_by_id.clone();
                                                            let canonical_tab_order = canonical_tab_order.clone();
                                                            Arc::new(move |host, acx, up| {
                                                                if up.button != MouseButton::Left {
                                                                    return PressablePointerUpResult::Continue;
                                                                }

                                                                let session_pos = host
                                                                    .drag(up.pointer_id)
                                                                    .map(|session| session.position);

                                                                ui_dnd::clear_pointer_in_scope(
                                                                    host.models_mut(),
                                                                    &dnd,
                                                                    acx.window,
                                                                    DRAG_KIND_WORKSPACE_TAB,
                                                                    dnd_scope,
                                                                    up.pointer_id,
                                                                );

                                                                let mut outcome = PressablePointerUpResult::Continue;
                                                                let mut maybe_drop: WorkspaceTabStripDropTarget =
                                                                    WorkspaceTabStripDropTarget::None;
                                                                let _ = host.models_mut().update(&drag_model, |st| {
                                                                    if st.pointer != Some(up.pointer_id) {
                                                                        return;
                                                                    }
                                                                    let dx = up.position.x.0 - st.start_position.x.0;
                                                                    let dy = up.position.y.0 - st.start_position.y.0;
                                                                    let dist2 = (dx * dx) + (dy * dy);
                                                                    let treat_as_drag = st.dragging || dist2 >= (6.0 * 6.0);

                                                                    if treat_as_drag {
                                                                        outcome = PressablePointerUpResult::SkipActivate;
                                                                        if let Some(dragged) =
                                                                            st.dragged_tab.clone()
                                                                        {
                                                                            if !matches!(
                                                                                st.drop_target,
                                                                                WorkspaceTabStripDropTarget::None
                                                                            ) {
                                                                                maybe_drop =
                                                                                    st.drop_target.clone();
                                                                                *st = WorkspaceTabStripDragState::default();
                                                                                return;
                                                                            }

                                                                            let pointer_pos_window =
                                                                                session_pos
                                                                                    .unwrap_or(
                                                                                        up.position_window
                                                                                            .unwrap_or(up.position),
                                                                                    );
                                                                            let pointer_pos_layout =
                                                                                up.position;

                                                                            let mut next_drop =
                                                                                compute_workspace_tab_strip_drop_target(
                                                                                    pointer_pos_window,
                                                                                    dragged.as_ref(),
                                                                                    &st.tab_rects,
                                                                                    st.pinned_boundary_rect,
                                                                                    st.end_drop_target_rect,
                                                                                    st.scroll_viewport_rect,
                                                                                    st.overflow_control_rect,
                                                                                    st.scroll_left_control_rect,
                                                                                    st.scroll_right_control_rect,
                                                                                );
                                                                            if matches!(
                                                                                next_drop,
                                                                                WorkspaceTabStripDropTarget::None
                                                                            ) && pointer_pos_layout
                                                                                != pointer_pos_window
                                                                            {
                                                                                next_drop =
                                                                                    compute_workspace_tab_strip_drop_target(
                                                                                        pointer_pos_layout,
                                                                                        dragged.as_ref(),
                                                                                        &st.tab_rects,
                                                                                        st.pinned_boundary_rect,
                                                                                        st.end_drop_target_rect,
                                                                                        st.scroll_viewport_rect,
                                                                                        st.overflow_control_rect,
                                                                                        st.scroll_left_control_rect,
                                                                                        st.scroll_right_control_rect,
                                                                                    );
                                                                            }

                                                                            if matches!(
                                                                                next_drop,
                                                                                WorkspaceTabStripDropTarget::End
                                                                            ) {
                                                                                next_drop = resolve_end_drop_target_in_canonical_order(
                                                                                    pinned_by_id.as_ref(),
                                                                                    canonical_tab_order.as_ref(),
                                                                                    dragged.as_ref(),
                                                                                )
                                                                                .map(|id| {
                                                                                    WorkspaceTabStripDropTarget::Tab(
                                                                                        id,
                                                                                        WorkspaceTabInsertionSide::After,
                                                                                    )
                                                                                })
                                                                                .unwrap_or(WorkspaceTabStripDropTarget::None);
                                                                            } else if let WorkspaceTabStripDropTarget::Tab(target, _) =
                                                                                &next_drop
                                                                            {
                                                                                let dragged_is_pinned = pinned_by_id
                                                                                    .get(dragged.as_ref())
                                                                                    .copied()
                                                                                    .unwrap_or(false);
                                                                                let target_is_pinned = pinned_by_id
                                                                                    .get(target)
                                                                                    .copied()
                                                                                    .unwrap_or(false);
                                                                                if dragged_is_pinned != target_is_pinned {
                                                                                    next_drop = WorkspaceTabStripDropTarget::None;
                                                                                }
                                                                            }

                                                                            maybe_drop = next_drop;
                                                                        } else {
                                                                            maybe_drop =
                                                                                st.drop_target.clone();
                                                                        }
                                                                    }
                                                                    *st = WorkspaceTabStripDragState::default();
                                                                });

                                                                if outcome == PressablePointerUpResult::SkipActivate {
                                                                    let intent = tab_drag_model
                                                                        .as_ref()
                                                                        .and_then(|m| {
                                                                            let pointer_pos_window =
                                                                                up.position_window
                                                                                    .unwrap_or(up.position);
                                                                            let pointer_pos_layout =
                                                                                up.position;
                                                                            host.models_mut()
                                                                                .update(m, |st| {
                                                                                    if st.pointer
                                                                                        != Some(up.pointer_id)
                                                                                        || st.source_window
                                                                                            != Some(
                                                                                                acx.window,
                                                                                            )
                                                                                    {
                                                                                        return WorkspaceTabDropIntent::None;
                                                                                    }

                                                                                    let mut target_pane: Option<Arc<str>> =
                                                                                        None;
                                                                                    let mut target_geom: Option<WorkspacePaneDragGeometry> =
                                                                                        None;
                                                                                    for (pane_id, geom) in
                                                                                        &st.pane_geometry
                                                                                    {
                                                                                        if geom
                                                                                            .bounds
                                                                                            .contains(pointer_pos_window)
                                                                                            || geom
                                                                                                .bounds
                                                                                                .contains(pointer_pos_layout)
                                                                                        {
                                                                                            target_pane =
                                                                                                Some(pane_id.clone());
                                                                                            target_geom = Some(*geom);
                                                                                            break;
                                                                                        }
                                                                                    }
                                                                                    let Some(target_pane) =
                                                                                        target_pane.or_else(|| st.hovered_pane.clone())
                                                                                    else {
                                                                                        return WorkspaceTabDropIntent::None;
                                                                                    };

                                                                                    let mut zone = st
                                                                                        .hovered_zone
                                                                                        .unwrap_or(WorkspaceTabDropZone::Center);
                                                                                    if zone != WorkspaceTabDropZone::Center
                                                                                        && let Some(geom) = target_geom
                                                                                    {
                                                                                        // Docks/splits should be mediated by the pane drop
                                                                                        // surface. If the pointer is within the tab-strip
                                                                                        // band, force a center drop so "drop on the tabstrip"
                                                                                        // does not accidentally request a split.
                                                                                        let tab_strip_max_y = geom
                                                                                            .bounds
                                                                                            .origin
                                                                                            .y
                                                                                            .0
                                                                                            + 48.0;
                                                                                        if pointer_pos_window.y.0
                                                                                            <= tab_strip_max_y
                                                                                        {
                                                                                            zone = WorkspaceTabDropZone::Center;
                                                                                        }
                                                                                    }

                                                                                    let intent =
                                                                                        resolve_workspace_tab_drop_intent(
                                                                                            st,
                                                                                            &target_pane,
                                                                                            zone,
                                                                                        );
                                                                                    if !matches!(
                                                                                        intent,
                                                                                        WorkspaceTabDropIntent::SplitAndMove { .. }
                                                                                    ) {
                                                                                        *st = WorkspaceTabDragState::default();
                                                                                    }
                                                                                    intent
                                                                                })
                                                                                .ok()
                                                                        })
                                                                        .unwrap_or(WorkspaceTabDropIntent::None);

                                                                    let should_return = matches!(
                                                                        &intent,
                                                                        WorkspaceTabDropIntent::MoveToPane { .. }
                                                                            | WorkspaceTabDropIntent::InsertToPane { .. }
                                                                            | WorkspaceTabDropIntent::SplitAndMove { .. }
                                                                    );

                                                                    match intent {
                                                                        WorkspaceTabDropIntent::None => {}
                                                                        WorkspaceTabDropIntent::MoveToPane {
                                                                            source,
                                                                            dragged_tab,
                                                                            target,
                                                                        } => {
                                                                            if let Some(cmd) =
                                                                                pane_activate_command(source.as_ref())
                                                                            {
                                                                                host.dispatch_command(
                                                                                    Some(acx.window),
                                                                                    cmd,
                                                                                );
                                                                            }
                                                                            if let Some(cmd) =
                                                                                crate::commands::tab_activate_command(
                                                                                    dragged_tab.as_ref(),
                                                                                )
                                                                            {
                                                                                host.dispatch_command(
                                                                                    Some(acx.window),
                                                                                    cmd,
                                                                                );
                                                                            }
                                                                            if let Some(cmd) =
                                                                                pane_move_active_tab_to_command(
                                                                                    target.as_ref(),
                                                                                )
                                                                            {
                                                                                host.dispatch_command(
                                                                                    Some(acx.window),
                                                                                    cmd,
                                                                                );
                                                                            }
                                                                        }
                                                                        WorkspaceTabDropIntent::InsertToPane {
                                                                            source,
                                                                            dragged_tab,
                                                                            target,
                                                                            target_tab,
                                                                            side,
                                                                        } => {
                                                                            if let Some(cmd) =
                                                                                pane_activate_command(source.as_ref())
                                                                            {
                                                                                host.dispatch_command(
                                                                                    Some(acx.window),
                                                                                    cmd,
                                                                                );
                                                                            }
                                                                            if let Some(cmd) =
                                                                                crate::commands::tab_activate_command(
                                                                                    dragged_tab.as_ref(),
                                                                                )
                                                                            {
                                                                                host.dispatch_command(
                                                                                    Some(acx.window),
                                                                                    cmd,
                                                                                );
                                                                            }
                                                                            if let Some(cmd) =
                                                                                pane_move_active_tab_to_command(
                                                                                    target.as_ref(),
                                                                                )
                                                                            {
                                                                                host.dispatch_command(
                                                                                    Some(acx.window),
                                                                                    cmd,
                                                                                );
                                                                            }

                                                                            let cmd = match side {
                                                                                WorkspaceTabInsertionSide::Before => tab_move_active_before_command(target_tab.as_ref()),
                                                                                WorkspaceTabInsertionSide::After => tab_move_active_after_command(target_tab.as_ref()),
                                                                            };
                                                                            if let Some(cmd) = cmd {
                                                                                host.dispatch_command(
                                                                                    Some(acx.window),
                                                                                    cmd,
                                                                                );
                                                                            }
                                                                        }
                                                                        WorkspaceTabDropIntent::SplitAndMove { .. } => {
                                                                            // Split-and-move requires access to the pane tree/window model.
                                                                            // The pane drop surface handles this path via `InternalDragRegion`.
                                                                        }
                                                                    }

                                                                    if should_return {
                                                                        host.request_redraw(acx.window);
                                                                        return outcome;
                                                                    }
                                                                }

                                                                match maybe_drop {
                                                                    WorkspaceTabStripDropTarget::None => {}
                                                                    WorkspaceTabStripDropTarget::Tab(target, side) => {
                                                                        dispatch_intent(
                                                                            host,
                                                                            acx.window,
                                                                            WorkspaceTabStripIntent::Activate(tab_command.clone()),
                                                                        );
                                                                        dispatch_intent(
                                                                            host,
                                                                            acx.window,
                                                                            WorkspaceTabStripIntent::ReorderActive {
                                                                                target_tab_id: target,
                                                                                side,
                                                                            },
                                                                        );
                                                                        dispatch_intent(
                                                                            host,
                                                                            acx.window,
                                                                            WorkspaceTabStripIntent::RequestRedraw,
                                                                        );
                                                                    }
                                                                    WorkspaceTabStripDropTarget::PinnedBoundary => {
                                                                        dispatch_intent(
                                                                            host,
                                                                            acx.window,
                                                                            WorkspaceTabStripIntent::Activate(tab_command.clone()),
                                                                        );
                                                                        dispatch_intent(
                                                                            host,
                                                                            acx.window,
                                                                            WorkspaceTabStripIntent::SetPinned {
                                                                                tab_id: tab_id_for_pinned_boundary.clone(),
                                                                                pinned: !tab_pinned,
                                                                            },
                                                                        );
                                                                        dispatch_intent(
                                                                            host,
                                                                            acx.window,
                                                                            WorkspaceTabStripIntent::RequestRedraw,
                                                                        );
                                                                    }
                                                                    WorkspaceTabStripDropTarget::End => {}
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
                                                            .and_then(|_| match &drop_target {
                                                                WorkspaceTabStripDropTarget::Tab(target, side)
                                                                    if target.as_ref() == tab_id.as_ref() =>
                                                                {
                                                                    Some(*side)
                                                                }
                                                                _ => None,
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

                                                        let mut chrome = cx.container(
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
                                                                }
                                                                .into(),
                                                                background: bg,
                                                                border: indicator_border,
                                                                border_color: indicator_border_color,
                                                                corner_radii: Corners::all(Px(
                                                                    tab_radius.0.max(0.0),
                                                                )),
                                                                ..Default::default()
                                                            },
                                                            |cx| {
                                                                let mut out = Vec::new();
                                                                if tab_preview {
                                                                    if let Some(test_id) =
                                                                        tab_preview_test_id.clone()
                                                                    {
                                                                        let mut layout =
                                                                            LayoutStyle::default();
                                                                        layout.position =
                                                                            PositionStyle::Absolute;
                                                                        layout.inset.top =
                                                                            InsetEdge::Px(Px(0.0));
                                                                        layout.inset.left =
                                                                            InsetEdge::Px(Px(0.0));
                                                                        layout.size.width =
                                                                            Length::Px(Px(1.0));
                                                                        layout.size.height =
                                                                            Length::Px(Px(1.0));
                                                                        out.push(cx.hit_test_gate_props(
                                                                            HitTestGateProps {
                                                                                layout,
                                                                                hit_test: false,
                                                                            },
                                                                            move |cx| {
                                                                                vec![cx.semantics(
                                                                                    SemanticsProps {
                                                                                        layout: fill_layout(),
                                                                                        role: SemanticsRole::Generic,
                                                                                        label: Some(Arc::<str>::from(
                                                                                            "Preview tab",
                                                                                        )),
                                                                                        test_id: Some(test_id),
                                                                                        ..Default::default()
                                                                                    },
                                                                                    |_cx| Vec::new(),
                                                                                )]
                                                                            },
                                                                        ));
                                                                    }
                                                                }
                                                                if tab_pinned {
                                                                    if let Some(test_id) =
                                                                        tab_pinned_test_id.clone()
                                                                    {
                                                                        let mut layout =
                                                                            LayoutStyle::default();
                                                                        layout.position =
                                                                            PositionStyle::Absolute;
                                                                        layout.inset.top =
                                                                            InsetEdge::Px(Px(0.0));
                                                                        layout.inset.left =
                                                                            InsetEdge::Px(Px(0.0));
                                                                        layout.size.width =
                                                                            Length::Px(Px(1.0));
                                                                        layout.size.height =
                                                                            Length::Px(Px(1.0));
                                                                        out.push(cx.hit_test_gate_props(
                                                                            HitTestGateProps {
                                                                                layout,
                                                                                hit_test: false,
                                                                            },
                                                                            move |cx| {
                                                                                vec![cx.semantics(
                                                                                    SemanticsProps {
                                                                                        layout: fill_layout(),
                                                                                        role: SemanticsRole::Generic,
                                                                                        label: Some(Arc::<str>::from(
                                                                                            "Pinned tab",
                                                                                        )),
                                                                                        test_id: Some(test_id),
                                                                                        ..Default::default()
                                                                                    },
                                                                                    |_cx| Vec::new(),
                                                                                )]
                                                                            },
                                                                        ));
                                                                    }
                                                                }
                                                                if tab_dirty {
                                                                    if let Some(test_id) =
                                                                        tab_dirty_test_id.clone()
                                                                    {
                                                                        let mut layout =
                                                                            LayoutStyle::default();
                                                                        layout.position =
                                                                            PositionStyle::Absolute;
                                                                        layout.inset.top =
                                                                            InsetEdge::Px(Px(0.0));
                                                                        layout.inset.left =
                                                                            InsetEdge::Px(Px(0.0));
                                                                        layout.size.width =
                                                                            Length::Px(Px(1.0));
                                                                        layout.size.height =
                                                                            Length::Px(Px(1.0));
                                                                        out.push(cx.hit_test_gate_props(
                                                                            HitTestGateProps {
                                                                                layout,
                                                                                hit_test: false,
                                                                            },
                                                                            move |cx| {
                                                                                vec![cx.semantics(
                                                                                    SemanticsProps {
                                                                                        layout: fill_layout(),
                                                                                        role: SemanticsRole::Generic,
                                                                                        label: Some(Arc::<str>::from(
                                                                                            "Dirty tab",
                                                                                        )),
                                                                                        test_id: Some(test_id),
                                                                                        ..Default::default()
                                                                                    },
                                                                                    |_cx| Vec::new(),
                                                                                )]
                                                                            },
                                                                        ));
                                                                    }
                                                                }

                                                                out.push(cx.flex(
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
                                                                        gap: Px(6.0).into(),
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
                                                                        let tab_close_test_id = tab_test_id
                                                                            .as_ref()
                                                                            .map(|id| Arc::<str>::from(format!("{id}.close")));

                                                                        let mut children = vec![
                                                                            cx.text_props(TextProps {
                                                                                layout: {
                                                                                    let mut layout =
                                                                                        LayoutStyle::default();
                                                                                    layout.size.max_width =
                                                                                        Some(Length::Px(tab_max_width));
                                                                                    layout.size.min_width =
                                                                                        Some(Length::Px(Px(0.0)));
                                                                                    layout.flex.grow = 1.0;
                                                                                    layout.flex.shrink = 1.0;
                                                                                    layout
                                                                                },
                                                                                text: label,
                                                                                style: Some({
                                                                                    if tab_preview {
                                                                                        let mut style =
                                                                                            text_style.clone();
                                                                                        style.slant = TextSlant::Italic;
                                                                                        style
                                                                                    } else {
                                                                                        text_style.clone()
                                                                                    }
                                                                                }),
                                                                                color: Some(tab_fg),
                                                                                wrap: TextWrap::None,
                                                                                overflow: TextOverflow::Ellipsis,
                                                                                align: fret_core::TextAlign::Start,
                                                                                ink_overflow: TextInkOverflow::None,
                                                                            }),
                                                                        ];

                                                                        if has_trailing_slot {
                                                                            if show_close {
                                                                                if let Some(close_command) =
                                                                                    tab_close_command.clone()
                                                                                {
                                                                                    children.push(tab_close_button(
                                                                                        cx,
                                                                                        close_command,
                                                                                        pane_activate_cmd_for_close.clone(),
                                                                                        hover_bg,
                                                                                        text_style.clone(),
                                                                                        tab_fg,
                                                                                        tab_close_test_id.clone(),
                                                                                    ));
                                                                                }
                                                                            } else if tab_dirty {
                                                                                children.push(tab_dirty_indicator(
                                                                                    cx,
                                                                                    dirty_fg,
                                                                                    text_style.clone(),
                                                                                ));
                                                                            } else {
                                                                                children.push(tab_trailing_slot_placeholder(cx));
                                                                            }
                                                                        }

                                                                        children
                                                                    },
                                                                ));

                                                                out
                                                            },
                                                        );
                                                        if let Some(test_id) = tab_chrome_test_id.clone() {
                                                            chrome = chrome.test_id(test_id);
                                                        }
                                                        vec![chrome]
                                                    },
                                                );

	                                                #[cfg(feature = "shadcn-context-menu")]
	                                                {
	                                                    let open = get_context_menu_open_model(cx, &tab_id);
	                                                    let close_cmd = tab_close_command.clone();
	                                                    let tab_id_for_menu = tab_id.clone();
	                                                    let tab_pinned_for_menu = tab_pinned;
	                                                    let has_left = has_left;
	                                                    let has_right = has_right;
	                                                    let has_others = has_others;
	                                                    let menu_test_id_base = tab_test_id
	                                                        .as_ref()
	                                                        .map(|id| Arc::<str>::from(format!("{id}.menu")));
	                                                    ContextMenu::new(open).into_element(
	                                                        cx,
	                                                        |_cx| tab_element,
	                                                        move |_cx| {
	                                                            let mut entries = Vec::new();
	                                                            if let Some(cmd) = close_cmd {
	                                                                let mut item =
	                                                                    ContextMenuItem::new("Close Tab")
	                                                                        .on_select(cmd);
	                                                                if let Some(base) =
	                                                                    menu_test_id_base.as_ref()
	                                                                {
	                                                                    item = item.test_id(Arc::<str>::from(format!(
	                                                                        "{base}.close_tab"
	                                                                    )));
	                                                                }
	                                                                entries.push(ContextMenuEntry::Item(item));
	                                                            }
	                                                            if tab_pinned_for_menu {
	                                                                if let Some(cmd) = tab_unpin_command(tab_id_for_menu.as_ref()) {
	                                                                    let mut item =
	                                                                        ContextMenuItem::new("Unpin Tab")
	                                                                            .on_select(cmd);
	                                                                    if let Some(base) =
	                                                                        menu_test_id_base.as_ref()
	                                                                    {
	                                                                        item = item.test_id(
	                                                                            Arc::<str>::from(
	                                                                                format!(
	                                                                                    "{base}.unpin"
	                                                                                ),
	                                                                            ),
	                                                                        );
	                                                                    }
	                                                                    entries.push(
	                                                                        ContextMenuEntry::Item(item),
	                                                                    );
	                                                                }
	                                                            } else if let Some(cmd) = tab_pin_command(tab_id_for_menu.as_ref()) {
	                                                                let mut item =
	                                                                    ContextMenuItem::new("Pin Tab")
	                                                                        .on_select(cmd);
	                                                                if let Some(base) =
	                                                                    menu_test_id_base.as_ref()
	                                                                {
	                                                                    item = item.test_id(Arc::<str>::from(format!(
	                                                                        "{base}.pin"
	                                                                    )));
	                                                                }
	                                                                entries.push(ContextMenuEntry::Item(item));
	                                                            }
	                                                            {
	                                                                let mut item =
	                                                                    ContextMenuItem::new("Close Other Tabs")
	                                                                        .disabled(!has_others)
	                                                                        .on_select(CommandId::new(
	                                                                            crate::commands::CMD_WORKSPACE_TAB_CLOSE_OTHERS,
	                                                                        ));
	                                                                if let Some(base) =
	                                                                    menu_test_id_base.as_ref()
	                                                                {
	                                                                    item = item.test_id(
	                                                                        Arc::<str>::from(
	                                                                            format!(
	                                                                                "{base}.close_others"
	                                                                            ),
	                                                                        ),
	                                                                    );
	                                                                }
	                                                                entries.push(
	                                                                    ContextMenuEntry::Item(item),
	                                                                );
	                                                            }
	                                                            {
	                                                                let mut item = ContextMenuItem::new(
	                                                                    "Close Tabs to the Left",
	                                                                )
	                                                                .disabled(!has_left)
	                                                                .on_select(CommandId::new(
	                                                                    crate::commands::CMD_WORKSPACE_TAB_CLOSE_LEFT,
	                                                                ));
	                                                                if let Some(base) =
	                                                                    menu_test_id_base.as_ref()
	                                                                {
	                                                                    item = item.test_id(
	                                                                        Arc::<str>::from(
	                                                                            format!(
	                                                                                "{base}.close_left"
	                                                                            ),
	                                                                        ),
	                                                                    );
	                                                                }
	                                                                entries.push(
	                                                                    ContextMenuEntry::Item(item),
	                                                                );
	                                                            }
	                                                            {
	                                                                let mut item = ContextMenuItem::new(
	                                                                    "Close Tabs to the Right",
	                                                                )
	                                                                .disabled(!has_right)
	                                                                .on_select(CommandId::new(
	                                                                    crate::commands::CMD_WORKSPACE_TAB_CLOSE_RIGHT,
	                                                                ));
	                                                                if let Some(base) =
	                                                                    menu_test_id_base.as_ref()
	                                                                {
	                                                                    item = item.test_id(
	                                                                        Arc::<str>::from(
	                                                                            format!(
	                                                                                "{base}.close_right"
	                                                                            ),
	                                                                        ),
	                                                                    );
	                                                                }
	                                                                entries.push(
	                                                                    ContextMenuEntry::Item(item),
	                                                                );
	                                                            }
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

                            // A stable drop surface that represents the "header space" to the right
                            // of the visible tabs (drop at end). This mirrors dockview's
                            // `VoidContainer` and Zed's explicit drop target element.
                            //
                            // Important: this surface must remain visible while overflowed, so it
                            // lives outside the scroll content.
                            //
                            // Note: this element is intentionally not focusable/roving; it exists
                            // only for hit testing, DnD routing, and automation.
                            let end_drop_test_id = root_test_id.as_ref().map(|root| {
                                Arc::<str>::from(format!("{root}.drop_end"))
                            });
                            let end_drop_drag_model = drag_model.clone();
                            let end_drop_pinned_by_id = pinned_by_id.clone();
                            let end_drop_canonical_tab_order = canonical_tab_order.clone();
                            let end_drop = cx.keyed("workspace-tab-strip-drop-end", |cx| {
                                let mut layout = LayoutStyle::default();
                                layout.size.height = Length::Fill;
                                layout.size.width = Length::Px(Px(60.0));
                                layout.size.min_width = Some(Length::Px(Px(24.0)));
                                layout.flex.grow = 0.0;
                                layout.flex.shrink = 0.0;
                                let drag_model = end_drop_drag_model.clone();
                                let pinned_by_id = end_drop_pinned_by_id.clone();
                                let canonical_tab_order = end_drop_canonical_tab_order.clone();
                                let on_internal_drag: OnInternalDrag =
                                    Arc::new(move |host, acx, drag| {
                                        if !matches!(
                                            drag.kind,
                                            fret_core::InternalDragKind::Over
                                                | fret_core::InternalDragKind::Drop
                                        ) {
                                            return false;
                                        }

                                        let _ = host.models_mut().update(&drag_model, |st| {
                                            if st.pointer != Some(drag.pointer_id) {
                                                return;
                                            }
                                            st.dragging = true;
                                            let Some(dragged) = st.dragged_tab.clone() else {
                                                st.drop_target = WorkspaceTabStripDropTarget::None;
                                                return;
                                            };

                                            let next = resolve_end_drop_target_in_canonical_order(
                                                pinned_by_id.as_ref(),
                                                canonical_tab_order.as_ref(),
                                                dragged.as_ref(),
                                            )
                                            .map(|id| {
                                                WorkspaceTabStripDropTarget::Tab(
                                                    id,
                                                    WorkspaceTabInsertionSide::After,
                                                )
                                            })
                                            .unwrap_or(WorkspaceTabStripDropTarget::None);
                                            st.drop_target = next;
                                        });

                                        host.request_redraw(acx.window);
                                        true
                                    });

                                let mut el = cx.internal_drag_region(
                                    InternalDragRegionProps {
                                        layout,
                                        enabled: true,
                                    },
                                    |cx| {
                                        cx.internal_drag_region_on_internal_drag(on_internal_drag);
                                        Vec::new()
                                    },
                                );
                                if let Some(id) = end_drop_test_id.clone() {
                                    el = el.test_id(id);
                                }
                                end_drop_target_element.set(Some(el.id));
                                el
                            });

                            let scroll_step = Px(120.0);
                            let scroll_x = scroll_handle.offset().x;
                            let scroll_max_x = scroll_handle.max_offset().x;
                            let can_scroll_left = scroll_x.0 > 0.5;
                            let can_scroll_right = scroll_x.0 + 0.5 < scroll_max_x.0;
                            let scroll_handle_for_wheel = scroll_handle.clone();
                            let scroll_handle_for_controls = scroll_handle.clone();

                            #[cfg(feature = "shadcn-context-menu")]
                            let overflow_button_text_style = text_style.clone();

                            let rects = {
                                let elements = tab_elements.borrow();
                                collect_tab_hit_rects(cx, elements.as_slice())
                            };
                            let pinned_boundary_rect_now = bounds_for_optional_element_id(
                                cx,
                                pinned_boundary_element.get(),
                            );
                            let end_drop_target_rect_now = bounds_for_optional_element_id(
                                cx,
                                end_drop_target_element.get(),
                            );
                            let overflow_control_rect_now = bounds_for_optional_element_id(
                                cx,
                                overflow_control_element.get(),
                            );
                            let scroll_left_control_rect_now = bounds_for_optional_element_id(
                                cx,
                                scroll_left_control_element.get(),
                            );
                            let scroll_right_control_rect_now = bounds_for_optional_element_id(
                                cx,
                                scroll_right_control_element.get(),
                            );

                            #[cfg(feature = "shadcn-context-menu")]
                            let (overflow_is_overflowing, overflow_button_test_id, overflow_entries) = {
                                let is_overflowing = scroll_max_x.0 > 0.5;
                                let viewport_now = scroll_element
                                    .get()
                                    .and_then(|id| cx.last_bounds_for_element(id));
                                let viewport = viewport_now.or(cached_scroll_viewport);
                                let tab_rects = if rects.is_empty() {
                                    cached_tab_rects.as_slice()
                                } else {
                                    rects.as_slice()
                                };
                                let (button_test_id, entries) = compute_overflow_menu_entries(
                                    cx,
                                    root_test_id.as_ref(),
                                    &tabs,
                                    tab_rects,
                                    viewport,
                                    is_overflowing,
                                    reveal_hint_model.clone(),
                                    text_style.clone(),
                                    inactive_fg,
                                );
                                (is_overflowing, button_test_id, entries)
                            };

                            let viewport_now = scroll_element
                                .get()
                                .and_then(|id| cx.last_bounds_for_element(id));
                            let viewport_for_hit = viewport_now.or(cached_scroll_viewport);
                            let rects_for_hit = if rects.is_empty() {
                                cached_tab_rects.clone()
                            } else {
                                rects.clone()
                            };
                            if !rects.is_empty() || viewport_now.is_some() {
                                let rects_for_cache = rects.clone();
                                cx.with_state(WorkspaceTabStripState::default, |state| {
                                    if !rects_for_cache.is_empty() {
                                        state.last_tab_rects = rects_for_cache;
                                    }
                                    if let Some(viewport) = viewport_now {
                                        state.last_scroll_viewport = Some(viewport);
                                    }
                                });
                            }

                            // Keep geometry synced even when we're not actively dragging so the first
                            // drag move can resolve drop targets immediately (no "first move has no
                            // rects" gap).
                            let should_sync_rects = true;
                            let should_clear = drag_snapshot.dragged_tab.as_ref().is_some_and(|dragged| {
                                !rects_for_hit.iter().any(|r| r.id.as_ref() == dragged.as_ref())
                            });
                            let rects_changed = rects_for_hit != drag_snapshot.tab_rects;
                            let pinned_boundary_changed =
                                pinned_boundary_rect_now != drag_snapshot.pinned_boundary_rect;
                            let viewport_changed = viewport_for_hit != drag_snapshot.scroll_viewport_rect;
                            let end_drop_changed =
                                end_drop_target_rect_now != drag_snapshot.end_drop_target_rect;
                            let overflow_control_changed =
                                overflow_control_rect_now != drag_snapshot.overflow_control_rect;
                            let scroll_left_control_changed =
                                scroll_left_control_rect_now
                                    != drag_snapshot.scroll_left_control_rect;
                            let scroll_right_control_changed =
                                scroll_right_control_rect_now
                                    != drag_snapshot.scroll_right_control_rect;

                            if should_clear
                                || (should_sync_rects
                                    && (rects_changed
                                        || pinned_boundary_changed
                                        || viewport_changed
                                        || end_drop_changed
                                        || overflow_control_changed
                                        || scroll_left_control_changed
                                        || scroll_right_control_changed))
                            {
                                let rects_for_model = rects_for_hit.clone();
                                let pinned_boundary_rect_for_model = pinned_boundary_rect_now;
                                let viewport_for_model = viewport_for_hit;
                                let end_drop_target_rect_for_model = end_drop_target_rect_now;
                                let overflow_control_rect_for_model = overflow_control_rect_now;
                                let scroll_left_control_rect_for_model =
                                    scroll_left_control_rect_now;
                                let scroll_right_control_rect_for_model =
                                    scroll_right_control_rect_now;
                                let _ = cx.app.models_mut().update(&drag_model, move |st| {
                                    if should_clear {
                                        *st = WorkspaceTabStripDragState::default();
                                        return;
                                    }

                                    st.tab_rects = rects_for_model;
                                    st.pinned_boundary_rect = pinned_boundary_rect_for_model;
                                    st.scroll_viewport_rect = viewport_for_model;
                                    st.end_drop_target_rect = end_drop_target_rect_for_model;
                                    st.overflow_control_rect = overflow_control_rect_for_model;
                                    st.scroll_left_control_rect = scroll_left_control_rect_for_model;
                                    st.scroll_right_control_rect =
                                        scroll_right_control_rect_for_model;
                                    match st.drop_target.clone() {
                                        WorkspaceTabStripDropTarget::None => {}
                                        WorkspaceTabStripDropTarget::PinnedBoundary => {
                                            if st.pinned_boundary_rect.is_none() {
                                                st.drop_target = WorkspaceTabStripDropTarget::None;
                                            }
                                        }
                                        WorkspaceTabStripDropTarget::Tab(target, _side) => {
                                            if !st.tab_rects.iter().any(|r| r.id.as_ref() == target.as_ref()) {
                                                st.drop_target = WorkspaceTabStripDropTarget::None;
                                            }
                                        }
                                        WorkspaceTabStripDropTarget::End => {
                                            st.drop_target = WorkspaceTabStripDropTarget::None;
                                        }
                                    }
                                });
                            }

                            // While dragging, recompute the local drop target every frame so it stays in
                            // sync with edge auto-scroll and scroll-to-reveal changes.
                            if drag_snapshot.dragging
                                && let Some(pointer_id) = drag_snapshot.pointer
                                && let Some(dragged) = drag_snapshot.dragged_tab.as_deref()
                            {
                                if let Some(session) = cx.app.drag(pointer_id) {
                                    let session_kind = session.kind;
                                    let session_dragging = session.dragging;
                                    let session_current_window = session.current_window;
                                    let session_position = session.position;

                                    if session_kind == DRAG_KIND_WORKSPACE_TAB
                                        && session_dragging
                                        && session_current_window == cx.window
                                    {
                                        let overflow_rect = overflow_control_rect_now;
                                        let next = compute_workspace_tab_strip_drop_target(
                                            session_position,
                                            dragged,
                                            &rects_for_hit,
                                            pinned_boundary_rect_now,
                                            end_drop_target_rect_now,
                                            viewport_for_hit,
                                            overflow_rect,
                                            scroll_left_control_rect_now,
                                            scroll_right_control_rect_now,
                                        );
                                    let next = match next {
                                        WorkspaceTabStripDropTarget::End => resolve_end_drop_target_in_canonical_order(
                                            pinned_by_id.as_ref(),
                                            canonical_tab_order.as_ref(),
                                            dragged,
                                        )
                                        .map(|id| {
                                            WorkspaceTabStripDropTarget::Tab(
                                                id,
                                                WorkspaceTabInsertionSide::After,
                                            )
                                        })
                                        .unwrap_or(WorkspaceTabStripDropTarget::None),
                                        other => other,
                                    };
                                    if next != drag_snapshot.drop_target {
                                        let _ = cx.app.models_mut().update(&drag_model, |st| {
                                            if st.pointer != Some(pointer_id) {
                                                return;
                                            }
                                            st.drop_target = next.clone();
                                        });
                                    }

                                    if let Some(viewport) = viewport_for_hit {
                                        let current = scroll_handle.offset();
                                        let max_x = scroll_handle.max_offset().x;
                                        let delta = compute_tab_strip_edge_auto_scroll_delta_x(
                                            viewport,
                                            session_position,
                                            current.x,
                                            max_x,
                                        );
                                        if delta.0.abs() > 0.01 {
                                            let next_x = (current.x.0 + delta.0).clamp(0.0, max_x.0);
                                            if (next_x - current.x.0).abs() > 0.01 {
                                                scroll_handle.set_offset(Point::new(Px(next_x), current.y));
                                            }
                                        }
                                    }
                                    }
                                }
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
                                    && tab_drag_snapshot.hovered_zone.is_some();

                                if let (true, Some(pointer_id)) = (should_compute, tab_drag_snapshot.pointer) {
                                    let session = cx.app.drag(pointer_id);
                                    let rects_for_cross = if rects.is_empty() {
                                        cached_tab_rects.clone()
                                    } else {
                                        rects.clone()
                                    };
                                    let mut next_tab: Option<Arc<str>> = None;
                                    let mut next_side: Option<WorkspaceTabInsertionSide> = None;
                                    let next_rects = rects_for_cross;

                                    if let Some(session) = session
                                        && session.kind == DRAG_KIND_WORKSPACE_TAB
                                        && session.dragging
                                        && session.current_window == cx.window
                                        && let Some(dragged) = tab_drag_snapshot.dragged_tab.as_deref()
                                    {
                                        let mut drop = compute_workspace_tab_strip_drop_target(
                                            session.position,
                                            dragged,
                                            &next_rects,
                                            pinned_boundary_rect_now,
                                            end_drop_target_rect_now,
                                            viewport_for_hit,
                                            overflow_control_rect_now,
                                            scroll_left_control_rect_now,
                                            scroll_right_control_rect_now,
                                        );

                                        // Cross-pane drag policy: dropping in end-drop / header space inserts at the
                                        // end of the canonical order, regardless of scroll position.
                                        if matches!(drop, WorkspaceTabStripDropTarget::End) {
                                            drop = canonical_tab_order
                                                .last()
                                                .cloned()
                                                .map(|id| {
                                                    WorkspaceTabStripDropTarget::Tab(
                                                        id,
                                                        WorkspaceTabInsertionSide::After,
                                                    )
                                                })
                                                .unwrap_or(WorkspaceTabStripDropTarget::None);
                                        }

                                        // Do not surface pinned-boundary drops as "insert next to tab" for
                                        // cross-pane drags. Pinned is a workspace policy affordance.
                                        if matches!(drop, WorkspaceTabStripDropTarget::PinnedBoundary) {
                                            drop = WorkspaceTabStripDropTarget::None;
                                        }

                                        // Cross-pane drag policy: avoid implicitly pinning tabs by inserting into
                                        // the pinned region. If the drop target lands on a pinned tab, clamp the
                                        // insertion to the pinned boundary (insert after the last pinned tab).
                                        if let WorkspaceTabStripDropTarget::Tab(target, _side) = &drop
                                            && pinned_by_id.get(target).copied().unwrap_or(false)
                                        {
                                            let pinned_count =
                                                tabs.iter().take_while(|t| t.pinned).count();
                                            if pinned_count > 0
                                                && let Some(last_pinned) = canonical_tab_order
                                                    .get(pinned_count.saturating_sub(1))
                                            {
                                                drop = WorkspaceTabStripDropTarget::Tab(
                                                    last_pinned.clone(),
                                                    WorkspaceTabInsertionSide::After,
                                                );
                                            }
                                        }

                                        if let WorkspaceTabStripDropTarget::Tab(id, side) = drop {
                                            next_tab = Some(id);
                                            next_side = Some(side);
                                        }
                                    }

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

                            vec![cx.pointer_region(
                                PointerRegionProps {
                                    layout: fill_layout(),
                                    ..Default::default()
                                },
                                move |cx| {
                                    let on_wheel: OnWheel = {
                                        let scroll_handle = scroll_handle_for_wheel.clone();
                                        Arc::new(move |host, acx, wheel| {
                                            let max_x = scroll_handle.max_offset().x;
                                            if max_x.0 <= 0.5 {
                                                return false;
                                            }

                                            let dx = wheel.delta.x;
                                            let dy = wheel.delta.y;
                                            let delta_x = if wheel.modifiers.shift
                                                && dx.0.abs() <= 0.01
                                            {
                                                dy
                                            } else if dx.0.abs() > 0.01 {
                                                dx
                                            } else {
                                                dy
                                            };

                                            if delta_x.0.abs() <= 0.01 {
                                                return false;
                                            }

                                            let prev = scroll_handle.offset();
                                            scroll_handle.set_offset(Point::new(
                                                Px(prev.x.0 - delta_x.0),
                                                prev.y,
                                            ));
                                            let next = scroll_handle.offset();
                                            let consumed =
                                                (prev.x.0 - next.x.0).abs() > 0.001;
                                            if consumed {
                                                host.request_redraw(acx.window);
                                            }
                                            consumed
                                        })
                                    };
                                    cx.pointer_region_on_wheel(on_wheel);

                                    vec![cx.flex(
                                        FlexProps {
                                            layout: fill_layout(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap: Px(2.0).into(),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            ..Default::default()
                                        },
                                        move |cx| {
                                            let mut out = vec![
                                                tab_strip_scroll_button(
                                                    cx,
                                                    can_scroll_left,
                                                    "<",
                                                    "Scroll left",
                                                    -1.0,
                                                    scroll_step,
                                                    scroll_handle_for_controls.clone(),
                                                    scroll_button_fg,
                                                    hover_bg,
                                                    text_style.clone(),
                                                    scroll_left_control_element.clone(),
                                                ),
                                                scroll,
                                                end_drop,
                                                tab_strip_scroll_button(
                                                    cx,
                                                    can_scroll_right,
                                                    ">",
                                                    "Scroll right",
                                                    1.0,
                                                    scroll_step,
                                                    scroll_handle_for_controls.clone(),
                                                    scroll_button_fg,
                                                    hover_bg,
                                                    text_style.clone(),
                                                    scroll_right_control_element.clone(),
                                                ),
                                            ];

                                            #[cfg(feature = "shadcn-context-menu")]
                                            overflow_control_element.set(None);
                                            scroll_left_control_element.set(None);
                                            scroll_right_control_element.set(None);
                                            #[cfg(feature = "shadcn-context-menu")]
                                            if overflow_is_overflowing {
                                                let enabled = !overflow_entries.is_empty();
                                                let overflow_control_element_for_trigger =
                                                    overflow_control_element.clone();
                                                let trigger = move |cx: &mut ElementContext<'_, H>| {
                                                    overflow_control_element_for_trigger
                                                        .set(Some(cx.root_id()));
                                                    tab_strip_overflow_button(
                                                        cx,
                                                        enabled,
                                                        scroll_button_fg,
                                                        hover_bg,
                                                        overflow_button_text_style.clone(),
                                                        overflow_button_test_id.clone(),
                                                    )
                                                };

                                                let entries = overflow_entries;
                                                let overflow_open = get_overflow_menu_open_model(cx);
                                                let menu =
                                                    DropdownMenu::new_controllable(cx, Some(overflow_open), false)
                                                    .align(DropdownMenuAlign::End)
                                                    .side(DropdownMenuSide::Bottom)
                                                    .into_element(cx, trigger, move |_cx| entries);
                                                out.push(menu);
                                            }

                                            out
                                        },
                                    )]
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
                                let hint = cx
                                    .app
                                    .models_mut()
                                    .read(&reveal_hint_model, |st| st.clone())
                                    .ok();

                                let hint_matches_active = hint
                                    .as_ref()
                                    .and_then(|st| st.tab_id.as_ref())
                                    .zip(active.as_ref())
                                    .is_some_and(|(hint_id, active_id)| {
                                        hint_id.as_ref() == active_id.as_ref()
                                    });

                                let margin = if hint_matches_active
                                    && hint
                                        .as_ref()
                                        .and_then(|st| st.reason)
                                        == Some(ActivateReason::Keyboard)
                                {
                                    Px(12.0)
                                } else {
                                    Px(0.0)
                                };

                                scroll_rect_into_view_x_with_margin(
                                    &scroll_handle,
                                    viewport,
                                    tab_rect,
                                    margin,
                                );

                                if hint.is_some_and(|st| st.tab_id.is_some() || st.reason.is_some())
                                {
                                    let _ = cx.app.models_mut().update(
                                        &reveal_hint_model,
                                        |st| {
                                            st.tab_id = None;
                                            st.reason = None;
                                        },
                                    );
                                }
                            }
                        }
                    }

                    cx.with_state(WorkspaceTabStripState::default, |state| {
                        state.last_active = active.clone();
                    });

                    // Best-effort diagnostics hook: publish interaction state into the window-level
                    // diagnostics store so `fretboard diag` can gate editor-grade invariants
                    // without relying on pixels.
                    {
                        let frame_id = cx.app.frame_id();
                        let tab_count = tabs.len();
                        let scroll_x = scroll_handle.offset().x;
                        let max_scroll_x = scroll_handle.max_offset().x;
                        let overflow = max_scroll_x.0 > 0.5;

                        let viewport_now = scroll_element
                            .get()
                            .and_then(|id| cx.last_bounds_for_element(id));
                        let viewport = viewport_now.or(cached_scroll_viewport);
                        let active_tab_rect = active_tab_element
                            .get()
                            .and_then(|id| cx.last_bounds_for_element(id));
                        let scroll_viewport_rect = viewport;
                        let active_tab_rect_diag = active_tab_rect;

                        let status = if active.is_none() {
                            fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::NoActiveTab
                        } else if viewport.is_none() {
                            fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::MissingScrollViewportRect
                        } else if active_tab_rect.is_none() {
                            fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::MissingActiveTabRect
                        } else {
                            fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::Ok
                        };

                        let active_visible = match (viewport, active_tab_rect) {
                            (Some(viewport), Some(tab)) => {
                                // `last_bounds_for_element` reports tab bounds in "unscrolled"
                                // coordinates while the viewport bounds are in window
                                // coordinates. Convert the tab rect into window coordinates by
                                // subtracting the current scroll offset.
                                let view_left = viewport.origin.x.0;
                                let view_right = viewport.origin.x.0 + viewport.size.width.0;
                                let tab_left = tab.origin.x.0 - scroll_x.0;
                                let tab_right = tab.origin.x.0 + tab.size.width.0 - scroll_x.0;
                                tab_left < view_right && tab_right > view_left
                            }
                            _ => false,
                        };

                        cx.app.with_global_mut_untracked(
                            fret_runtime::WindowInteractionDiagnosticsStore::default,
                            |store, _app| {
                                store.record_workspace_tab_strip_active_visibility(
                                    cx.window,
                                    frame_id,
                                    fret_runtime::WorkspaceTabStripActiveVisibilityDiagnostics {
                                        status,
                                        pane_id: pane_id.clone(),
                                        active_tab_id: active.clone(),
                                        tab_count,
                                        overflow,
                                        scroll_x,
                                        max_scroll_x,
                                        scroll_viewport_rect,
                                        active_tab_rect: active_tab_rect_diag,
                                        active_visible,
                                    },
                                );
                            },
                        );
                    }

                    // Editor-grade focus restore:
                    //
                    // When the tab strip is focused (keyboard-first), closing the active tab
                    // should keep focus within the strip by pre-focusing the tab that is expected
                    // to become active next.
                    {
                        use crate::commands::{CMD_WORKSPACE_TAB_CLOSE, CMD_WORKSPACE_TAB_CLOSE_PREFIX};

                        let active_for_hook = active.clone();
                        let mru_for_hook = mru.clone();
                        let canonical_for_hook = canonical_tab_order.clone();
                        let tab_elements_for_hook = tab_elements.clone();
                        let pane_id_for_hook = pane_id.clone();
                        let tab_element_registry_for_timer = tab_element_registry.clone();
                        let focus_restore_model_for_timer = focus_restore_model.clone();
                        let tab_element_registry_for_command = tab_element_registry.clone();
                        let focus_restore_model_for_command = focus_restore_model.clone();
                        let reveal_hint_model_for_command = reveal_hint_model.clone();

                        cx.timer_on_timer_for(
                            root.id,
                            Arc::new(move |host, acx, token| {
                                const MAX_ATTEMPTS: u32 = 4;

                                let Ok((pending_timer, pane_id, tab_id, attempts)) =
                                    host.models_mut().read(&focus_restore_model_for_timer, |st| {
                                        (st.timer, st.target_pane_id.clone(), st.tab_id.clone(), st.attempts)
                                    })
                                else {
                                    return false;
                                };

                                if pending_timer != Some(token) {
                                    return false;
                                }

                                let Some(tab_id) = tab_id else {
                                    return false;
                                };

                                let key = WorkspaceTabElementKey {
                                    window: acx.window,
                                    pane_id,
                                    tab_id: tab_id.clone(),
                                };

                                let target = host
                                    .models_mut()
                                    .read(&tab_element_registry_for_timer, |reg| reg.get(&key))
                                    .ok()
                                    .flatten();

                                if let Some(target) = target {
                                    host.request_focus(target);
                                    host.request_redraw(acx.window);
                                    let _ = host.models_mut().update(&focus_restore_model_for_timer, |st| {
                                        if st.timer == Some(token) {
                                            st.timer = None;
                                            st.target_pane_id = None;
                                            st.tab_id = None;
                                            st.attempts = 0;
                                        }
                                    });
                                    return false;
                                }

                                if attempts >= MAX_ATTEMPTS {
                                    let _ = host.models_mut().update(&focus_restore_model_for_timer, |st| {
                                        if st.timer == Some(token) {
                                            st.timer = None;
                                            st.target_pane_id = None;
                                            st.tab_id = None;
                                            st.attempts = 0;
                                        }
                                    });
                                    return false;
                                }

                                let retry_token = host.next_timer_token();
                                let retry_after = if attempts == 0 {
                                    Duration::from_millis(0)
                                } else {
                                    Duration::from_millis(16)
                                };

                                let _ = host.models_mut().update(&focus_restore_model_for_timer, |st| {
                                    if st.timer == Some(token) {
                                        st.timer = Some(retry_token);
                                        st.attempts = attempts.saturating_add(1);
                                    }
                                });

                                host.push_effect(Effect::SetTimer {
                                    window: Some(acx.window),
                                    token: retry_token,
                                    after: retry_after,
                                    repeat: None,
                                });

                                false
                            }),
                        );

                        cx.command_on_command_for(
                            root.id,
                            Arc::new(move |host, acx, command| {
                                let Some(active_id) = active_for_hook.clone() else {
                                    return false;
                                };

                                let cmd = command.as_str();
                                let closing_active = if cmd == CMD_WORKSPACE_TAB_CLOSE {
                                    true
                                } else if let Some(id) =
                                    cmd.strip_prefix(CMD_WORKSPACE_TAB_CLOSE_PREFIX)
                                {
                                    id.trim() == active_id.as_ref()
                                } else {
                                    false
                                };

                                if !closing_active {
                                    return false;
                                }

                                let next = utils::predict_next_active_tab_after_close(
                                    &active_id,
                                    canonical_for_hook.as_ref(),
                                    mru_for_hook.as_deref(),
                                );

                                if let Some(next_id) = next {
                                    let _ = host.models_mut().update(
                                        &reveal_hint_model_for_command,
                                        |st| {
                                            st.tab_id = Some(next_id.clone());
                                            st.reason = Some(ActivateReason::Keyboard);
                                        },
                                    );

                                    // Drop the closing tab entry (best-effort) so the registry
                                    // doesn't grow unbounded in long sessions.
                                    let closing_key = WorkspaceTabElementKey {
                                        window: acx.window,
                                        pane_id: pane_id_for_hook.clone(),
                                        tab_id: active_id.clone(),
                                    };
                                    let _ = host
                                        .models_mut()
                                        .update(&tab_element_registry_for_command, |reg| {
                                            reg.remove(&closing_key);
                                        });

                                    let target = tab_elements_for_hook
                                        .borrow()
                                        .iter()
                                        .find(|(id, _)| id.as_ref() == next_id.as_ref())
                                        .map(|(_, el)| *el);
                                    if let Some(target) = target {
                                        host.request_focus(target);
                                        host.request_redraw(acx.window);
                                    }

                                    // The tab we want to focus is typically not focusable until it
                                    // becomes the new active tab (roving focus policy). Defer an
                                    // additional focus attempt to a timer tick so it runs after
                                    // the close command has updated selection.
                                    let existing = host
                                        .models_mut()
                                        .read(&focus_restore_model_for_command, |st| st.timer)
                                        .ok()
                                        .flatten();
                                    if let Some(prev) = existing {
                                        host.push_effect(Effect::CancelTimer { token: prev });
                                    }

                                    let token = host.next_timer_token();
                                    let _ = host.models_mut().update(
                                        &focus_restore_model_for_command,
                                        |st| {
                                            st.timer = Some(token);
                                            st.target_pane_id = pane_id_for_hook.clone();
                                            st.tab_id = Some(next_id.clone());
                                            st.attempts = 0;
                                        },
                                    );
                                    host.push_effect(Effect::SetTimer {
                                        window: Some(acx.window),
                                        token,
                                        after: Duration::from_millis(0),
                                        repeat: None,
                                    });
                                }

                                false
                            }),
                        );
                    }

                    vec![root]
            },
        )
    }
}
