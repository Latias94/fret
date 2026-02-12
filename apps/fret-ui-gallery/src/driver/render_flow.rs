use crate::spec::*;
use crate::ui;
use fret_app::{App, Model};
use fret_core::{AppWindowId, SemanticsRole};
use fret_runtime::WindowCommandAvailabilityService;
use fret_ui::Invalidation;
use fret_ui::declarative;
use fret_ui::element::{AnyElement, SemanticsDecoration};
use fret_ui_kit::OverlayController;
use fret_workspace::WorkspaceFrame;
use std::sync::Arc;

use super::{
    UiGalleryDriver, UiGalleryWindowState, chrome, debug_hud, debug_stats, inspector, menubar,
    settings_sheet, shell, status_bar, toaster, ui_gallery_bisect_flags,
};

pub(super) struct PreparedFrame {
    pub(super) bisect: u32,
    pub(super) cache_shell: bool,
    pub(super) content_models: Arc<ui::UiGalleryModels>,
    pub(super) selected_page: Model<Arc<str>>,
    pub(super) workspace_tabs: Model<Vec<Arc<str>>>,
    pub(super) workspace_dirty_tabs: Model<Vec<Arc<str>>>,
    pub(super) nav_query: Model<String>,
    pub(super) settings_open: Model<bool>,
    pub(super) settings_menu_bar_os: Model<Option<Arc<str>>>,
    pub(super) settings_menu_bar_os_open: Model<bool>,
    pub(super) settings_menu_bar_in_window: Model<Option<Arc<str>>>,
    pub(super) settings_menu_bar_in_window_open: Model<bool>,
    pub(super) settings_edit_can_undo: Model<bool>,
    pub(super) settings_edit_can_redo: Model<bool>,
    pub(super) menu_bar_seq: Model<u64>,
    pub(super) inspector_enabled: Model<bool>,
    pub(super) inspector_last_pointer: Model<Option<fret_core::Point>>,
    pub(super) inspector_status: Option<status_bar::InspectorStatus>,
    pub(super) show_debug_hud: bool,
    pub(super) debug_hud_lines: Vec<Arc<str>>,
    pub(super) layout_time_us: u128,
    pub(super) paint_time_us: u128,
}

pub(super) fn begin_frame(
    app: &mut App,
    window: AppWindowId,
    state: &mut UiGalleryWindowState,
) -> PreparedFrame {
    OverlayController::begin_frame(app, window);
    let bisect = ui_gallery_bisect_flags();

    UiGalleryDriver::sync_undo_availability(app, window, &state.undo_doc);

    #[cfg(target_arch = "wasm32")]
    UiGalleryDriver::sync_page_router_from_external_history(app, window, state);

    let availability = app
        .global::<WindowCommandAvailabilityService>()
        .and_then(|svc| svc.snapshot(window))
        .copied()
        .unwrap_or_default();
    let _ = app.models_mut().update(&state.settings_edit_can_undo, |v| {
        *v = availability.edit_can_undo
    });
    let _ = app.models_mut().update(&state.settings_edit_can_redo, |v| {
        *v = availability.edit_can_redo
    });

    let cache_enabled = app
        .models()
        .get_copied(&state.view_cache_enabled)
        .unwrap_or(false);
    let cache_shell = app
        .models()
        .get_copied(&state.view_cache_cache_shell)
        .unwrap_or(false);

    if state.ui.view_cache_enabled() != cache_enabled {
        state.ui.set_view_cache_enabled(cache_enabled);
        if let Some(root) = state.root {
            state.ui.invalidate(root, Invalidation::Layout);
        }
    }

    let content_models = Arc::new(state.content_models());
    let selected_page = state.selected_page.clone();
    let workspace_tabs = state.workspace_tabs.clone();
    let workspace_dirty_tabs = state.workspace_dirty_tabs.clone();
    let nav_query = state.nav_query.clone();
    let settings_open = state.settings_open.clone();
    let settings_menu_bar_os = state.settings_menu_bar_os.clone();
    let settings_menu_bar_os_open = state.settings_menu_bar_os_open.clone();
    let settings_menu_bar_in_window = state.settings_menu_bar_in_window.clone();
    let settings_menu_bar_in_window_open = state.settings_menu_bar_in_window_open.clone();
    let settings_edit_can_undo = state.settings_edit_can_undo.clone();
    let settings_edit_can_redo = state.settings_edit_can_redo.clone();
    let menu_bar_seq = state.menu_bar_seq.clone();
    let inspector_enabled = state.inspector_enabled.clone();
    let inspector_last_pointer = state.inspector_last_pointer.clone();

    let inspector_on = app.models().get_copied(&inspector_enabled).unwrap_or(false);
    let debug_on = inspector_on
        || std::env::var_os("FRET_UI_DEBUG_STATS").is_some_and(|v| !v.is_empty())
        || std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty());
    state.ui.set_debug_enabled(debug_on);
    if debug_on {
        app.request_redraw(window);
    }

    UiGalleryDriver::sync_shadcn_theme(app, state);
    let last_debug_stats = state.ui.debug_stats();

    let debug_hud = debug_stats::compute_debug_hud_bundle(
        app,
        &state.ui,
        window,
        &mut state.debug_hud,
        &inspector_enabled,
        &inspector_last_pointer,
        debug_on,
    );

    PreparedFrame {
        bisect,
        cache_shell,
        content_models,
        selected_page,
        workspace_tabs,
        workspace_dirty_tabs,
        nav_query,
        settings_open,
        settings_menu_bar_os,
        settings_menu_bar_os_open,
        settings_menu_bar_in_window,
        settings_menu_bar_in_window_open,
        settings_edit_can_undo,
        settings_edit_can_redo,
        menu_bar_seq,
        inspector_enabled,
        inspector_last_pointer,
        inspector_status: debug_hud.inspector_status,
        show_debug_hud: debug_hud.show,
        debug_hud_lines: debug_hud.lines,
        layout_time_us: last_debug_stats.layout_time.as_micros(),
        paint_time_us: last_debug_stats.paint_time.as_micros(),
    }
}

pub(super) fn render_root(
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    state: &mut UiGalleryWindowState,
    bounds: fret_core::Rect,
    frame: &PreparedFrame,
) -> fret_core::NodeId {
    declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
        .render_root("fret-ui-gallery", |cx| render_root_contents(cx, frame))
}

pub(super) fn end_frame(
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    state: &mut UiGalleryWindowState,
    bounds: fret_core::Rect,
    frame: &PreparedFrame,
    root: fret_core::NodeId,
) {
    state.ui.set_root(root);
    if (frame.bisect & BISECT_DISABLE_OVERLAY_CONTROLLER) == 0 {
        OverlayController::render(&mut state.ui, app, services, window, bounds);
    }
    state.root = Some(root);
}

fn render_root_contents(
    cx: &mut fret_ui::ElementContext<'_, App>,
    frame: &PreparedFrame,
) -> Vec<AnyElement> {
    if (frame.bisect & BISECT_MINIMAL_ROOT) != 0 {
        return vec![cx.text("Hello, fret-ui-gallery")];
    }

    let theme = cx.theme().clone();

    let sidebar = shell::sidebar_view(
        cx,
        &theme,
        frame.bisect,
        frame.cache_shell,
        &frame.nav_query,
        &frame.selected_page,
        &frame.workspace_tabs,
    );
    let content = shell::content_view(
        cx,
        &theme,
        frame.bisect,
        frame.cache_shell,
        &frame.selected_page,
        frame.content_models.as_ref(),
    );

    let tab_strip = chrome::tab_strip_view(
        cx,
        (frame.bisect & BISECT_DISABLE_TAB_STRIP) != 0,
        &frame.selected_page,
        &frame.workspace_tabs,
        &frame.workspace_dirty_tabs,
    );

    let menubar_handle = std::cell::RefCell::new(None);
    let in_window_menu_bar =
        menubar::build_in_window_menu_bar(cx, &frame.menu_bar_seq, &menubar_handle);

    let top_bar = chrome::top_bar_view(cx, in_window_menu_bar, tab_strip);

    let status_bar = status_bar::status_bar_view(
        cx,
        frame.content_models.as_ref(),
        frame.inspector_status.as_ref(),
        frame.layout_time_us,
        frame.paint_time_us,
    );

    let mut center_layout = fret_ui::element::LayoutStyle::default();
    center_layout.size.width = fret_ui::element::Length::Fill;
    center_layout.size.height = fret_ui::element::Length::Fill;
    center_layout.flex.grow = 1.0;

    let center = cx.flex(
        fret_ui::element::FlexProps {
            layout: center_layout,
            direction: fret_core::Axis::Horizontal,
            ..Default::default()
        },
        |_cx| vec![sidebar, content],
    );

    let frame_el = WorkspaceFrame::new(center)
        .top(top_bar)
        .bottom(status_bar)
        .into_element(cx);

    let panel = frame_el.attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Panel)
            .label("fret-ui-gallery"),
    );
    menubar::attach_in_window_menubar_handlers(cx, panel.id, &menubar_handle);

    let mut content: Vec<AnyElement> = vec![
        panel,
        toaster::toaster_view(
            cx,
            frame.content_models.as_ref(),
            (frame.bisect & BISECT_DISABLE_TOASTER) != 0,
        ),
    ];

    settings_sheet::push_settings_sheet(
        cx,
        frame.settings_open.clone(),
        frame.settings_menu_bar_os.clone(),
        frame.settings_menu_bar_os_open.clone(),
        frame.settings_menu_bar_in_window.clone(),
        frame.settings_menu_bar_in_window_open.clone(),
        frame.settings_edit_can_undo.clone(),
        frame.settings_edit_can_redo.clone(),
        &mut content,
    );

    debug_hud::maybe_push_debug_hud(
        cx,
        theme.clone(),
        frame.show_debug_hud,
        frame.debug_hud_lines.clone(),
        &mut content,
    );

    inspector::wrap_content_if_enabled(
        cx,
        &frame.inspector_enabled,
        &frame.inspector_last_pointer,
        content,
    )
}
