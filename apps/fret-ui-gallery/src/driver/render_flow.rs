use crate::spec::*;
use crate::ui;
use fret_app::{App, Model};
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use fret_core::{AppWindowId, Px, SemanticsRole};
use fret_runtime::WindowCommandAvailabilityService;
use fret_ui::Invalidation;
use fret_ui::declarative;
use fret_ui::element::{AnyElement, LayoutStyle, Length, SemanticsProps, SpacerProps};
use fret_ui_kit::OverlayController;
use fret_ui_shadcn as shadcn;
use fret_workspace::WorkspaceFrame;
use std::sync::Arc;

use super::{
    UiGalleryDriver, UiGalleryWindowState, chrome, debug_hud, debug_stats, inspector, menubar,
    settings_sheet, shell, status_bar, toaster, ui_gallery_bisect_flags,
};

pub(super) struct PreparedFrame {
    pub(super) bisect: u32,
    pub(super) cache_sidebar: bool,
    pub(super) cache_content: bool,
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
    pub(super) chrome_show_workspace_tab_strip: Model<bool>,
    pub(super) menu_bar_seq: Model<u64>,
    pub(super) inspector_enabled: Model<bool>,
    pub(super) inspector_last_pointer: Model<Option<fret_core::Point>>,
    pub(super) inspector_status: Option<status_bar::InspectorStatus>,
    pub(super) show_debug_hud: bool,
    pub(super) debug_hud_lines: Vec<Arc<str>>,
    pub(super) show_status_bar: bool,
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
    let prev_edit_can_undo = app
        .models()
        .get_copied(&state.settings_edit_can_undo)
        .unwrap_or_default();
    if prev_edit_can_undo != availability.edit_can_undo {
        let _ = app.models_mut().update(&state.settings_edit_can_undo, |v| {
            *v = availability.edit_can_undo
        });
    }

    let prev_edit_can_redo = app
        .models()
        .get_copied(&state.settings_edit_can_redo)
        .unwrap_or_default();
    if prev_edit_can_redo != availability.edit_can_redo {
        let _ = app.models_mut().update(&state.settings_edit_can_redo, |v| {
            *v = availability.edit_can_redo
        });
    }

    let cache_enabled = app
        .models()
        .get_copied(&state.view_cache_enabled)
        .unwrap_or(false);
    let cache_shell = app
        .models()
        .get_copied(&state.view_cache_cache_shell)
        .unwrap_or(false);
    let cache_content = app
        .models()
        .get_copied(&state.view_cache_cache_content)
        .unwrap_or(true);

    let cache_sidebar = cache_shell;
    let cache_content = cache_shell && cache_content;

    if state.ui.view_cache_enabled() != cache_enabled {
        state.ui.set_view_cache_enabled(cache_enabled);
        if let Some(root) = state.root {
            // Flipping the view-cache flag should not require a full relayout: it affects how the
            // tree is painted/cached, not the geometry.
            state.ui.invalidate(root, Invalidation::Paint);
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
    let chrome_show_workspace_tab_strip = state.chrome_show_workspace_tab_strip.clone();
    let menu_bar_seq = state.menu_bar_seq.clone();
    let inspector_enabled = state.inspector_enabled.clone();
    let inspector_last_pointer = state.inspector_last_pointer.clone();

    let inspector_on = app.models().get_copied(&inspector_enabled).unwrap_or(false);
    // Perf suites set `FRET_DIAG_RENDERER_PERF=1`. Avoid enabling the UI-tree debug HUD/stats in
    // that mode because it perturbs steady-state perf measurements.
    let perf_mode = std::env::var_os("FRET_DIAG_RENDERER_PERF").is_some_and(|v| !v.is_empty());
    let hud_on = inspector_on
        || std::env::var_os("FRET_UI_DEBUG_STATS").is_some_and(|v| !v.is_empty())
        || (!perf_mode && std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty()));
    let diag_enabled =
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| svc.is_enabled());
    state.ui.set_debug_enabled(diag_enabled || hud_on);
    if hud_on {
        app.request_redraw(window);
    }
    let show_status_bar = std::env::var_os("FRET_UI_GALLERY_STATUS_BAR")
        .is_some_and(|v| !v.is_empty())
        || hud_on
        || diag_enabled;

    UiGalleryDriver::sync_shadcn_theme(app, state);
    UiGalleryDriver::sync_motion_preset(app, state);
    let last_debug_stats = state.ui.debug_stats();

    let debug_hud = debug_stats::compute_debug_hud_bundle(
        app,
        &state.ui,
        window,
        &mut state.debug_hud,
        &inspector_enabled,
        &inspector_last_pointer,
        hud_on,
    );

    PreparedFrame {
        bisect,
        cache_sidebar,
        cache_content,
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
        chrome_show_workspace_tab_strip,
        menu_bar_seq,
        inspector_enabled,
        inspector_last_pointer,
        inspector_status: debug_hud.inspector_status,
        show_debug_hud: debug_hud.show,
        debug_hud_lines: debug_hud.lines,
        show_status_bar,
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
        frame.cache_sidebar,
        &frame.nav_query,
        &frame.selected_page,
        &frame.workspace_tabs,
    );
    let content = shell::content_view(
        cx,
        &theme,
        frame.bisect,
        frame.cache_content,
        &frame.selected_page,
        frame.content_models.as_ref(),
    );

    let show_tab_strip = cx
        .get_model_copied(&frame.chrome_show_workspace_tab_strip, Invalidation::Layout)
        .unwrap_or(false);
    let tab_strip = if show_tab_strip && (frame.bisect & BISECT_DISABLE_TAB_STRIP) == 0 {
        Some(chrome::tab_strip_view(
            cx,
            false,
            &frame.selected_page,
            &frame.workspace_tabs,
            &frame.workspace_dirty_tabs,
        ))
    } else {
        None
    };

    let menubar_handle = std::cell::RefCell::new(None);
    let in_window_menu_bar =
        menubar::build_in_window_menu_bar(cx, &frame.menu_bar_seq, &menubar_handle);

    let top_bar = chrome::top_bar_view(cx, in_window_menu_bar, tab_strip);

    let mut center_layout = fret_ui::element::LayoutStyle::default();
    center_layout.size.width = fret_ui::element::Length::Fill;
    center_layout.size.height = fret_ui::element::Length::Fill;
    center_layout.flex.grow = 1.0;

    let center = cx
        .flex(
            fret_ui::element::FlexProps {
                layout: center_layout,
                direction: fret_core::Axis::Horizontal,
                ..Default::default()
            },
            |_cx| vec![sidebar, content],
        )
        .test_id("ui-gallery-workspace-center");

    let mut frame_el = WorkspaceFrame::new(center).top(top_bar);
    if frame.show_status_bar {
        let status_bar = status_bar::status_bar_view(
            cx,
            frame.content_models.as_ref(),
            frame.inspector_status.as_ref(),
            frame.layout_time_us,
            frame.paint_time_us,
        );
        frame_el = frame_el.bottom(status_bar);
    }
    let frame_el = frame_el.into_element(cx);

    let mut frame_semantics_layout = LayoutStyle::default();
    frame_semantics_layout.size.width = Length::Fill;
    frame_semantics_layout.size.height = Length::Fill;
    let panel = cx.semantics(
        SemanticsProps {
            layout: frame_semantics_layout,
            role: SemanticsRole::Panel,
            label: Some(Arc::from("fret-ui-gallery")),
            test_id: Some(Arc::from("ui-gallery-workspace-frame")),
            ..Default::default()
        },
        |_cx| [frame_el],
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
        frame.chrome_show_workspace_tab_strip.clone(),
        &mut content,
    );

    let command_palette = cx.keyed("ui_gallery.global_command_palette", |cx| {
        shadcn::CommandDialog::new_with_host_commands(
            cx,
            frame.content_models.cmdk_open.clone(),
            frame.content_models.cmdk_query.clone(),
        )
        .a11y_label("Command palette")
        .empty_text("No results found.")
        .into_element(cx, |cx| {
            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Px(Px(0.0));
            layout.size.height = Length::Px(Px(0.0));
            layout.flex.grow = 0.0;
            layout.flex.shrink = 0.0;
            cx.spacer(SpacerProps {
                layout,
                min: Px(0.0),
            })
        })
    });
    content.push(command_palette);

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

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{
        AppWindowId, Event, Modifiers, MouseButton, MouseButtons, PathCommand, PathConstraints,
        PathId, PathMetrics, PathService, PathStyle, Point, PointerEvent, PointerId, PointerType,
        Px, Rect, Size, SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService,
    };
    use fret_runtime::{FrameId, TickId};

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
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

    impl fret_core::MaterialService for FakeServices {
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

    fn find_node_by_test_id<'a>(
        snap: &'a fret_core::SemanticsSnapshot,
        test_id: &str,
    ) -> Option<&'a fret_core::SemanticsNode> {
        snap.nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some(test_id))
    }

    fn node_by_test_id<'a>(
        snap: &'a fret_core::SemanticsSnapshot,
        test_id: &str,
    ) -> &'a fret_core::SemanticsNode {
        find_node_by_test_id(snap, test_id)
            .unwrap_or_else(|| panic!("missing semantics test_id={test_id}"))
    }

    struct RenderedGalleryPage {
        window: AppWindowId,
        frame_index: u64,
        app: App,
        state: UiGalleryWindowState,
        services: FakeServices,
        bounds: Rect,
    }

    fn render_gallery_frame(rendered: &mut RenderedGalleryPage) {
        rendered.frame_index = rendered.frame_index.saturating_add(1);
        rendered.app.set_tick_id(TickId(rendered.frame_index));
        rendered.app.set_frame_id(FrameId(rendered.frame_index));

        let frame = begin_frame(&mut rendered.app, rendered.window, &mut rendered.state);
        let root = render_root(
            &mut rendered.app,
            &mut rendered.services,
            rendered.window,
            &mut rendered.state,
            rendered.bounds,
            &frame,
        );
        end_frame(
            &mut rendered.app,
            &mut rendered.services,
            rendered.window,
            &mut rendered.state,
            rendered.bounds,
            &frame,
            root,
        );
        rendered.state.ui.request_semantics_snapshot();
        rendered.state.ui.layout_all(
            &mut rendered.app,
            &mut rendered.services,
            rendered.bounds,
            1.0,
        );
    }

    fn node_id_by_test_id(rendered: &RenderedGalleryPage, test_id: &str) -> fret_core::NodeId {
        let snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after layout");
        node_by_test_id(snapshot, test_id).id
    }

    fn visual_bounds_by_test_id(rendered: &RenderedGalleryPage, test_id: &str) -> Rect {
        let snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after layout");
        let node = node_by_test_id(snapshot, test_id);
        rendered
            .state
            .ui
            .debug_node_visual_bounds(node.id)
            .or_else(|| rendered.state.ui.debug_node_bounds(node.id))
            .or(Some(node.bounds))
            .unwrap_or_else(|| panic!("missing visual/layout bounds for test_id={test_id}"))
    }

    fn visual_bounds_by_test_id_if_present(
        rendered: &RenderedGalleryPage,
        test_id: &str,
    ) -> Option<Rect> {
        let snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after layout");
        let node = find_node_by_test_id(snapshot, test_id)?;
        rendered
            .state
            .ui
            .debug_node_visual_bounds(node.id)
            .or_else(|| rendered.state.ui.debug_node_bounds(node.id))
            .or(Some(node.bounds))
    }

    fn render_gallery_page_with_bounds(page: &str, bounds: Rect) -> RenderedGalleryPage {
        let window = AppWindowId::default();
        let mut app = App::new();
        let state = UiGalleryDriver::build_ui(&mut app, window);
        let _ = app.models_mut().update(&state.selected_page, |selected| {
            *selected = Arc::<str>::from(page)
        });

        let services = FakeServices;

        let mut rendered = RenderedGalleryPage {
            window,
            frame_index: 0,
            app,
            state,
            services,
            bounds,
        };
        render_gallery_frame(&mut rendered);
        render_gallery_frame(&mut rendered);
        rendered
    }

    fn render_gallery_page(page: &str) -> RenderedGalleryPage {
        render_gallery_page_with_bounds(
            page,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(1080.0), Px(720.0)),
            ),
        )
    }

    fn scroll_gallery_page_to_bottom(
        rendered: &mut RenderedGalleryPage,
        page: &str,
        tracked_test_id: &str,
    ) -> (Rect, Rect, Rect) {
        let initial_page_bounds = visual_bounds_by_test_id(&rendered, tracked_test_id);
        let mut last_page_y = initial_page_bounds.origin.y.0;
        let mut last_gallery_scroll_y: Option<f64> = None;
        let mut last_gallery_scroll_y_max: Option<f64> = None;
        let mut moved = false;
        let mut stable_frames = 0usize;

        for _ in 0..48 {
            wheel_gallery_viewport(rendered, Px(-2000.0));

            let page_bounds = visual_bounds_by_test_id(&rendered, tracked_test_id);
            let snapshot = rendered
                .state
                .ui
                .semantics_snapshot()
                .expect("expected semantics snapshot while scrolling gallery page to bottom");
            let gallery_scroll = node_by_test_id(snapshot, "ui-gallery-content-viewport")
                .extra
                .scroll;
            let current_gallery_scroll_y = gallery_scroll.y.unwrap_or(0.0);
            let current_gallery_scroll_y_max = gallery_scroll.y_max.unwrap_or(0.0);
            let page_moved = page_bounds.origin.y.0 < last_page_y - 0.5;
            let scroll_changed = last_gallery_scroll_y
                .is_some_and(|last| (current_gallery_scroll_y - last).abs() > 0.5)
                || last_gallery_scroll_y_max
                    .is_some_and(|last| (current_gallery_scroll_y_max - last).abs() > 0.5);

            if page_moved {
                moved = true;
                stable_frames = 0;
            } else if moved && scroll_changed {
                stable_frames = 0;
            } else {
                stable_frames += 1;
            }
            last_page_y = page_bounds.origin.y.0;
            last_gallery_scroll_y = Some(current_gallery_scroll_y);
            last_gallery_scroll_y_max = Some(current_gallery_scroll_y_max);

            if moved && stable_frames >= 3 {
                break;
            }
        }

        let viewport_bounds = visual_bounds_by_test_id(&rendered, "ui-gallery-content-viewport");
        let page_bounds = visual_bounds_by_test_id(&rendered, tracked_test_id);

        assert!(
            moved,
            "expected wheel scrolling to move the gallery page for page={page}: initial_page={initial_page_bounds:?} final_page={page_bounds:?} viewport={viewport_bounds:?}"
        );

        (initial_page_bounds, viewport_bounds, page_bounds)
    }

    fn assert_page_bottom_clamps_to_viewport_bottom(page: &str, tracked_test_id: &str) {
        let mut rendered = render_gallery_page(page);
        let (_, viewport_bounds, before_page_bounds) =
            scroll_gallery_page_to_bottom(&mut rendered, page, tracked_test_id);
        let before_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after scrolling gallery page to bottom");
        let before_gallery_scroll = node_by_test_id(before_snapshot, "ui-gallery-content-viewport")
            .extra
            .scroll;

        for _ in 0..3 {
            wheel_gallery_viewport(&mut rendered, Px(-240.0));
        }

        let after_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after extra bottom scroll input");
        let after_gallery_scroll = node_by_test_id(after_snapshot, "ui-gallery-content-viewport")
            .extra
            .scroll;
        let after_page_bounds = visual_bounds_by_test_id(&rendered, tracked_test_id);

        assert!(
            (after_gallery_scroll.y.unwrap_or(0.0) - before_gallery_scroll.y.unwrap_or(0.0)).abs()
                <= 0.01,
            "expected the gallery viewport to stay pinned once page scrolling reached the bottom: page={page} tracked_test_id={tracked_test_id} viewport={viewport_bounds:?} before_page_bounds={before_page_bounds:?} after_page_bounds={after_page_bounds:?} before_gallery_scroll={before_gallery_scroll:?} after_gallery_scroll={after_gallery_scroll:?}"
        );
        assert!(
            (after_page_bounds.origin.y.0 - before_page_bounds.origin.y.0).abs() <= 0.01
                && (after_page_bounds.size.height.0 - before_page_bounds.size.height.0).abs()
                    <= 0.01,
            "expected page geometry to remain stable after scrolling to the bottom: page={page} tracked_test_id={tracked_test_id} viewport={viewport_bounds:?} before_page_bounds={before_page_bounds:?} after_page_bounds={after_page_bounds:?} before_gallery_scroll={before_gallery_scroll:?} after_gallery_scroll={after_gallery_scroll:?}"
        );
    }

    fn assert_preview_card_content_contains_page_bottom(page: &str, page_root_test_id: &str) {
        let mut rendered = render_gallery_page(page);
        let (initial_page_bounds, viewport_bounds, _) =
            scroll_gallery_page_to_bottom(&mut rendered, page, "ui-gallery-preview-card");

        let page_bounds = visual_bounds_by_test_id(&rendered, page_root_test_id);

        let card_bounds = visual_bounds_by_test_id(&rendered, "ui-gallery-preview-card");
        let header_bounds = visual_bounds_by_test_id(&rendered, "ui-gallery-preview-card-header");
        let content_bounds = visual_bounds_by_test_id(&rendered, "ui-gallery-preview-card-content");

        let card_bottom = card_bounds.origin.y.0 + card_bounds.size.height.0;
        let content_bottom = content_bounds.origin.y.0 + content_bounds.size.height.0;
        let page_bottom = page_bounds.origin.y.0 + page_bounds.size.height.0;

        assert!(
            content_bottom + 1.0 >= page_bottom,
            "expected preview card content to contain the scrolled page bottom for page={page}: initial_page={initial_page_bounds:?} viewport={viewport_bounds:?} card={card_bounds:?} header={header_bounds:?} content={content_bounds:?} page={page_bounds:?}"
        );
        assert!(
            card_bottom + 1.0 >= content_bottom,
            "expected preview card root to contain its content after scrolling to bottom for page={page}: initial_page={initial_page_bounds:?} viewport={viewport_bounds:?} card={card_bounds:?} header={header_bounds:?} content={content_bounds:?} page={page_bounds:?}"
        );
    }

    fn rects_intersect(a: Rect, b: Rect) -> bool {
        let a_right = a.origin.x.0 + a.size.width.0;
        let a_bottom = a.origin.y.0 + a.size.height.0;
        let b_right = b.origin.x.0 + b.size.width.0;
        let b_bottom = b.origin.y.0 + b.size.height.0;

        a.origin.x.0 < b_right
            && b.origin.x.0 < a_right
            && a.origin.y.0 < b_bottom
            && b.origin.y.0 < a_bottom
    }

    fn wheel_gallery_viewport(rendered: &mut RenderedGalleryPage, delta_y: Px) {
        let viewport_bounds = visual_bounds_by_test_id(rendered, "ui-gallery-content-viewport");
        let wheel_pos = gallery_steering_wheel_position(viewport_bounds, delta_y.0 < 0.0);
        rendered.state.ui.dispatch_event(
            &mut rendered.app,
            &mut rendered.services,
            &Event::Pointer(PointerEvent::Wheel {
                position: wheel_pos,
                delta: Point::new(Px(0.0), delta_y),
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        render_gallery_frame(rendered);
    }

    fn scroll_test_id_into_gallery_viewport(
        rendered: &mut RenderedGalleryPage,
        target_test_id: &str,
    ) {
        let mut last_gallery_scroll_y: Option<f64> = None;
        let mut stable_frames = 0usize;

        for _ in 0..96 {
            let gallery_viewport =
                visual_bounds_by_test_id(rendered, "ui-gallery-content-viewport");
            let gallery_snapshot =
                rendered.state.ui.semantics_snapshot().expect(
                    "expected semantics snapshot while scrolling target into gallery viewport",
                );
            let gallery_scroll = node_by_test_id(gallery_snapshot, "ui-gallery-content-viewport")
                .extra
                .scroll;
            let target_node = find_node_by_test_id(gallery_snapshot, target_test_id)
                .map(|node| (node.id, node.bounds));
            let target_bounds = target_node.and_then(|(node_id, node_bounds)| {
                rendered
                    .state
                    .ui
                    .debug_node_visual_bounds(node_id)
                    .or_else(|| rendered.state.ui.debug_node_bounds(node_id))
                    .or(Some(node_bounds))
            });

            if let Some(target_bounds) = target_bounds {
                let target_center_y = target_bounds.origin.y.0 + target_bounds.size.height.0 * 0.5;
                let visible_top = gallery_viewport.origin.y.0;
                let visible_bottom = visible_top + gallery_viewport.size.height.0;

                if target_center_y >= visible_top + 4.0 && target_center_y <= visible_bottom - 4.0 {
                    return;
                }

                let did_scroll = target_node.is_some_and(|(node_id, _)| {
                    rendered
                        .state
                        .ui
                        .scroll_node_into_view(&mut rendered.app, node_id)
                });
                render_gallery_frame(rendered);

                let after_snapshot = rendered.state.ui.semantics_snapshot().expect(
                    "expected semantics snapshot after scroll-node-into-view while preparing gallery interaction",
                );
                let after_gallery_scroll =
                    node_by_test_id(after_snapshot, "ui-gallery-content-viewport")
                        .extra
                        .scroll;
                let current_gallery_scroll_y = after_gallery_scroll.y.unwrap_or(0.0);
                if let Some(last) = last_gallery_scroll_y {
                    if (current_gallery_scroll_y - last).abs() <= 0.01 {
                        stable_frames += 1;
                    } else {
                        stable_frames = 0;
                    }
                }
                last_gallery_scroll_y = Some(current_gallery_scroll_y);

                if did_scroll {
                    continue;
                }

                let scroll_down = target_center_y > visible_bottom;
                let wheel_pos = gallery_steering_wheel_position(gallery_viewport, scroll_down);
                let delta_y = if scroll_down { Px(-480.0) } else { Px(480.0) };
                rendered.state.ui.dispatch_event(
                    &mut rendered.app,
                    &mut rendered.services,
                    &Event::Pointer(PointerEvent::Wheel {
                        position: wheel_pos,
                        delta: Point::new(Px(0.0), delta_y),
                        modifiers: Modifiers::default(),
                        pointer_id: PointerId(0),
                        pointer_type: PointerType::Mouse,
                    }),
                );
                render_gallery_frame(rendered);
                continue;
            }

            let current_gallery_scroll_y = gallery_scroll.y.unwrap_or(0.0);
            if let Some(last) = last_gallery_scroll_y {
                if (current_gallery_scroll_y - last).abs() <= 0.01 {
                    stable_frames += 1;
                } else {
                    stable_frames = 0;
                }
            }
            last_gallery_scroll_y = Some(current_gallery_scroll_y);

            if stable_frames >= 3 {
                let matching_test_ids: Vec<String> = gallery_snapshot
                    .nodes
                    .iter()
                    .filter_map(|node| node.test_id.as_ref())
                    .filter(|test_id| {
                        (target_test_id.contains("markdown") && test_id.contains("markdown"))
                            || (target_test_id.contains("code-editor")
                                && test_id.contains("code-editor"))
                    })
                    .take(24)
                    .cloned()
                    .collect();
                panic!(
                    "expected target to appear in semantics after scrolling gallery viewport: target_test_id={target_test_id} gallery_viewport={gallery_viewport:?} gallery_scroll={gallery_scroll:?} matching_test_ids={matching_test_ids:?}"
                );
            }

            let wheel_pos = gallery_steering_wheel_position(gallery_viewport, true);
            rendered.state.ui.dispatch_event(
                &mut rendered.app,
                &mut rendered.services,
                &Event::Pointer(PointerEvent::Wheel {
                    position: wheel_pos,
                    delta: Point::new(Px(0.0), Px(-480.0)),
                    modifiers: Modifiers::default(),
                    pointer_id: PointerId(0),
                    pointer_type: PointerType::Mouse,
                }),
            );
            render_gallery_frame(rendered);
        }

        let gallery_viewport = visual_bounds_by_test_id(rendered, "ui-gallery-content-viewport");
        let target_bounds = visual_bounds_by_test_id_if_present(rendered, target_test_id);
        let gallery_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot before final gallery visibility panic");
        let gallery_scroll = node_by_test_id(gallery_snapshot, "ui-gallery-content-viewport")
            .extra
            .scroll;
        panic!(
            "expected target to become visible inside gallery viewport before interaction: target_test_id={target_test_id} gallery_viewport={gallery_viewport:?} gallery_scroll={gallery_scroll:?} target_bounds={target_bounds:?}"
        );
    }

    fn gallery_steering_wheel_position(gallery_viewport: Rect, scroll_down: bool) -> Point {
        let inset_x = gallery_viewport.size.width.0.min(40.0);
        let inset_y = gallery_viewport.size.height.0.min(40.0);
        let x = gallery_viewport.origin.x.0 + inset_x;
        let y = if scroll_down {
            gallery_viewport.origin.y.0 + inset_y
        } else {
            gallery_viewport.origin.y.0 + gallery_viewport.size.height.0 - inset_y
        };
        Point::new(Px(x), Px(y))
    }

    fn click_test_id_center(rendered: &mut RenderedGalleryPage, target_test_id: &str) {
        let target_bounds = visual_bounds_by_test_id(rendered, target_test_id);
        let position = Point::new(
            Px(target_bounds.origin.x.0 + target_bounds.size.width.0 * 0.5),
            Px(target_bounds.origin.y.0 + target_bounds.size.height.0 * 0.5),
        );

        rendered.state.ui.dispatch_event(
            &mut rendered.app,
            &mut rendered.services,
            &Event::Pointer(PointerEvent::Down {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(1),
                pointer_type: PointerType::Mouse,
            }),
        );
        render_gallery_frame(rendered);

        rendered.state.ui.dispatch_event(
            &mut rendered.app,
            &mut rendered.services,
            &Event::Pointer(PointerEvent::Up {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_id: PointerId(1),
                pointer_type: PointerType::Mouse,
            }),
        );
        render_gallery_frame(rendered);
    }

    fn wheel_test_id_center(
        rendered: &mut RenderedGalleryPage,
        target_test_id: &str,
        delta: Point,
        steps: usize,
    ) {
        for _ in 0..steps {
            let target_bounds = visual_bounds_by_test_id(rendered, target_test_id);
            let position = Point::new(
                Px(target_bounds.origin.x.0 + target_bounds.size.width.0 * 0.5),
                Px(target_bounds.origin.y.0 + target_bounds.size.height.0 * 0.5),
            );
            rendered.state.ui.dispatch_event(
                &mut rendered.app,
                &mut rendered.services,
                &Event::Pointer(PointerEvent::Wheel {
                    position,
                    delta,
                    modifiers: Modifiers::default(),
                    pointer_id: PointerId(0),
                    pointer_type: PointerType::Mouse,
                }),
            );
            render_gallery_frame(rendered);
        }
    }

    fn touch_pan_position(
        rendered: &mut RenderedGalleryPage,
        start: Point,
        delta: Point,
        steps: usize,
    ) {
        let pointer_id = PointerId(2);
        rendered.state.ui.dispatch_event(
            &mut rendered.app,
            &mut rendered.services,
            &Event::Pointer(PointerEvent::Down {
                position: start,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id,
                pointer_type: PointerType::Touch,
            }),
        );
        render_gallery_frame(rendered);

        for step in 1..=steps.max(1) {
            let progress = step as f32 / steps.max(1) as f32;
            let position = Point::new(
                Px(start.x.0 + delta.x.0 * progress),
                Px(start.y.0 + delta.y.0 * progress),
            );
            rendered.state.ui.dispatch_event(
                &mut rendered.app,
                &mut rendered.services,
                &Event::Pointer(PointerEvent::Move {
                    position,
                    buttons: MouseButtons {
                        left: true,
                        ..Default::default()
                    },
                    modifiers: Modifiers::default(),
                    pointer_id,
                    pointer_type: PointerType::Touch,
                }),
            );
            render_gallery_frame(rendered);
        }

        let end = Point::new(Px(start.x.0 + delta.x.0), Px(start.y.0 + delta.y.0));
        rendered.state.ui.dispatch_event(
            &mut rendered.app,
            &mut rendered.services,
            &Event::Pointer(PointerEvent::Up {
                position: end,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_id,
                pointer_type: PointerType::Touch,
            }),
        );
        render_gallery_frame(rendered);
    }

    fn touch_pan_test_id_center(
        rendered: &mut RenderedGalleryPage,
        target_test_id: &str,
        delta: Point,
        steps: usize,
    ) {
        let target_bounds = visual_bounds_by_test_id(rendered, target_test_id);
        let start = Point::new(
            Px(target_bounds.origin.x.0 + target_bounds.size.width.0 * 0.5),
            Px(target_bounds.origin.y.0 + target_bounds.size.height.0 * 0.5),
        );
        touch_pan_position(rendered, start, delta, steps);
    }

    fn scrollbar_thumb_by_test_id(rendered: &RenderedGalleryPage, scrollbar_test_id: &str) -> Rect {
        let snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot while locating scrollbar thumb");
        let scroll = node_by_test_id(snapshot, scrollbar_test_id).extra.scroll;
        let track = visual_bounds_by_test_id(rendered, scrollbar_test_id);
        let track_padding = 1.0_f32.min((track.size.height.0 * 0.5).max(0.0));
        let viewport = track.size.height.0.max(0.0);
        let max_offset = scroll.y_max.unwrap_or(0.0).max(0.0) as f32;
        let content = (viewport + max_offset).max(viewport);
        let inner_track_h = (track.size.height.0 - track_padding * 2.0).max(0.0);
        let ratio = if content <= 0.0 {
            1.0
        } else {
            (viewport / content).clamp(0.0, 1.0)
        };
        let min_thumb_h = 18.0_f32.min(inner_track_h);
        let thumb_h = (inner_track_h * ratio).max(min_thumb_h).min(inner_track_h);
        let max_thumb_y = (inner_track_h - thumb_h).max(0.0);
        let offset = scroll.y.unwrap_or(0.0).max(0.0) as f32;
        let t = if max_offset <= 0.0 {
            0.0
        } else {
            (offset / max_offset).clamp(0.0, 1.0)
        };
        Rect::new(
            Point::new(
                track.origin.x,
                Px(track.origin.y.0 + track_padding + max_thumb_y * t),
            ),
            Size::new(track.size.width, Px(thumb_h)),
        )
    }

    fn assert_scrollbar_thumb_drag_advances_inner_viewport_without_advancing_gallery_page(
        page: &str,
        scrollbar_test_id: &str,
    ) {
        let mut rendered = render_gallery_page(page);
        scroll_test_id_into_gallery_viewport(&mut rendered, scrollbar_test_id);

        let before_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot before scrollbar thumb drag");
        let before_gallery_scroll = node_by_test_id(before_snapshot, "ui-gallery-content-viewport")
            .extra
            .scroll;
        let before_inner_scroll = node_by_test_id(before_snapshot, scrollbar_test_id)
            .extra
            .scroll;
        let scrollbar_node = node_by_test_id(before_snapshot, scrollbar_test_id).id;
        let thumb = scrollbar_thumb_by_test_id(&rendered, scrollbar_test_id);
        let start = Point::new(
            Px(thumb.origin.x.0 + thumb.size.width.0 * 0.5),
            Px(thumb.origin.y.0 + thumb.size.height.0 * 0.5),
        );
        let pointer_id = PointerId(7);

        rendered.state.ui.dispatch_event(
            &mut rendered.app,
            &mut rendered.services,
            &Event::Pointer(PointerEvent::Down {
                position: start,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id,
                pointer_type: PointerType::Mouse,
            }),
        );
        render_gallery_frame(&mut rendered);

        assert_eq!(
            rendered.state.ui.captured_for(pointer_id),
            Some(scrollbar_node),
            "expected thumb drag to capture the pointer on scrollbar down: page={page} scrollbar_test_id={scrollbar_test_id} thumb={thumb:?}"
        );

        let delta = Point::new(Px(0.0), Px(40.0));
        for step in 1..=10 {
            let progress = step as f32 / 10.0;
            let position = Point::new(
                Px(start.x.0 + delta.x.0 * progress),
                Px(start.y.0 + delta.y.0 * progress),
            );
            rendered.state.ui.dispatch_event(
                &mut rendered.app,
                &mut rendered.services,
                &Event::Pointer(PointerEvent::Move {
                    position,
                    buttons: MouseButtons {
                        left: true,
                        ..Default::default()
                    },
                    modifiers: Modifiers::default(),
                    pointer_id,
                    pointer_type: PointerType::Mouse,
                }),
            );
            render_gallery_frame(&mut rendered);
        }

        let end = Point::new(Px(start.x.0 + delta.x.0), Px(start.y.0 + delta.y.0));
        rendered.state.ui.dispatch_event(
            &mut rendered.app,
            &mut rendered.services,
            &Event::Pointer(PointerEvent::Up {
                position: end,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_id,
                pointer_type: PointerType::Mouse,
            }),
        );
        render_gallery_frame(&mut rendered);

        let after_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after scrollbar thumb drag");
        let after_gallery_scroll = node_by_test_id(after_snapshot, "ui-gallery-content-viewport")
            .extra
            .scroll;
        let after_inner_scroll = node_by_test_id(after_snapshot, scrollbar_test_id)
            .extra
            .scroll;

        assert!(
            rendered.state.ui.captured_for(pointer_id).is_none(),
            "expected thumb drag to release pointer capture after mouse up: page={page} scrollbar_test_id={scrollbar_test_id}"
        );
        assert!(
            after_inner_scroll.y.unwrap_or(0.0) > before_inner_scroll.y.unwrap_or(0.0) + 0.01,
            "expected scrollbar thumb drag to advance the inner scroll state: page={page} scrollbar_test_id={scrollbar_test_id} before_inner_scroll={before_inner_scroll:?} after_inner_scroll={after_inner_scroll:?} before_gallery_scroll={before_gallery_scroll:?} after_gallery_scroll={after_gallery_scroll:?} thumb={thumb:?} scrollbar_node={scrollbar_node:?}"
        );
        assert!(
            (after_gallery_scroll.y.unwrap_or(0.0) - before_gallery_scroll.y.unwrap_or(0.0)).abs()
                <= 0.01,
            "expected gallery page scroll to remain stable while dragging inner scrollbar thumb: page={page} scrollbar_test_id={scrollbar_test_id} before_gallery_scroll={before_gallery_scroll:?} after_gallery_scroll={after_gallery_scroll:?} before_inner_scroll={before_inner_scroll:?} after_inner_scroll={after_inner_scroll:?} thumb={thumb:?} scrollbar_node={scrollbar_node:?}"
        );
    }

    fn wait_until_test_id_exists(
        rendered: &mut RenderedGalleryPage,
        target_test_id: &str,
        max_frames: usize,
    ) {
        for _ in 0..=max_frames {
            let snapshot = rendered
                .state
                .ui
                .semantics_snapshot()
                .expect("expected semantics snapshot while waiting for test id");
            if find_node_by_test_id(snapshot, target_test_id).is_some() {
                return;
            }
            render_gallery_frame(rendered);
        }

        let snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after waiting for test id");
        let present = snapshot
            .nodes
            .iter()
            .filter_map(|node| node.test_id.as_deref())
            .collect::<Vec<_>>();
        panic!(
            "expected test_id={target_test_id} to appear within {max_frames} frames; present_test_ids={present:?}"
        );
    }
    fn hit_chain_at(rendered: &mut RenderedGalleryPage, position: Point) -> Vec<String> {
        let snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot for hit-chain debug");
        let hit = rendered.state.ui.debug_hit_test(position).hit;
        let mut out = Vec::new();
        let mut node = hit;
        while let Some(id) = node {
            let kind = rendered
                .state
                .ui
                .debug_declarative_instance_kind(&mut rendered.app, rendered.window, id)
                .unwrap_or("non-declarative");
            let test_id = snapshot
                .nodes
                .iter()
                .find(|n| n.id == id)
                .and_then(|n| n.test_id.as_deref())
                .unwrap_or("-");
            out.push(format!("{:?}:{kind}:{test_id}", id));
            node = rendered.state.ui.node_parent(id);
        }
        out
    }

    fn assert_inner_viewport_vertical_touch_pan_is_owned_by_editor(
        page: &str,
        viewport_test_id: &str,
    ) {
        let mut rendered = render_gallery_page(page);
        scroll_test_id_into_gallery_viewport(&mut rendered, viewport_test_id);

        let before_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot before inner editor touch pan");
        let before_gallery_scroll = node_by_test_id(before_snapshot, "ui-gallery-content-viewport")
            .extra
            .scroll;
        let before_inner_scroll = node_by_test_id(before_snapshot, viewport_test_id)
            .extra
            .scroll;
        let before_viewport_bounds = visual_bounds_by_test_id(&rendered, viewport_test_id);
        let before_center = Point::new(
            Px(before_viewport_bounds.origin.x.0 + before_viewport_bounds.size.width.0 * 0.5),
            Px(before_viewport_bounds.origin.y.0 + before_viewport_bounds.size.height.0 * 0.5),
        );
        let before_hit_chain = hit_chain_at(&mut rendered, before_center);

        assert!(
            before_inner_scroll.y_max.unwrap_or(0.0) > 0.01,
            "expected editor viewport to have vertical overflow before touch pan: page={page} viewport_test_id={viewport_test_id} before_inner_scroll={before_inner_scroll:?} before_gallery_scroll={before_gallery_scroll:?} bounds={before_viewport_bounds:?} center={before_center:?} hit_chain={before_hit_chain:?}"
        );

        touch_pan_test_id_center(
            &mut rendered,
            viewport_test_id,
            Point::new(Px(0.0), Px(-8.0)),
            1,
        );

        let after_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after inner editor touch pan");
        let after_gallery_scroll = node_by_test_id(after_snapshot, "ui-gallery-content-viewport")
            .extra
            .scroll;
        let after_inner_scroll = node_by_test_id(after_snapshot, viewport_test_id)
            .extra
            .scroll;
        let after_viewport_bounds = visual_bounds_by_test_id(&rendered, viewport_test_id);
        let after_center = Point::new(
            Px(after_viewport_bounds.origin.x.0 + after_viewport_bounds.size.width.0 * 0.5),
            Px(after_viewport_bounds.origin.y.0 + after_viewport_bounds.size.height.0 * 0.5),
        );
        let after_hit_chain = hit_chain_at(&mut rendered, after_center);

        assert!(
            after_inner_scroll.y.unwrap_or(0.0) > before_inner_scroll.y.unwrap_or(0.0) + 0.01,
            "expected touch pan over the editor viewport to advance the editor's own scroll state: page={page} viewport_test_id={viewport_test_id} before_inner_scroll={before_inner_scroll:?} after_inner_scroll={after_inner_scroll:?} before_gallery_scroll={before_gallery_scroll:?} after_gallery_scroll={after_gallery_scroll:?} before_bounds={before_viewport_bounds:?} after_bounds={after_viewport_bounds:?} before_center={before_center:?} after_center={after_center:?} before_hit_chain={before_hit_chain:?} after_hit_chain={after_hit_chain:?}"
        );
        assert!(
            (after_gallery_scroll.y.unwrap_or(0.0) - before_gallery_scroll.y.unwrap_or(0.0)).abs()
                <= 0.01,
            "expected the outer gallery viewport not to consume touch pan while the inner editor viewport can still scroll: page={page} viewport_test_id={viewport_test_id} before_gallery_scroll={before_gallery_scroll:?} after_gallery_scroll={after_gallery_scroll:?} before_inner_scroll={before_inner_scroll:?} after_inner_scroll={after_inner_scroll:?}"
        );
        assert!(
            (after_viewport_bounds.origin.y.0 - before_viewport_bounds.origin.y.0).abs() <= 0.01
                && (after_viewport_bounds.size.height.0 - before_viewport_bounds.size.height.0)
                    .abs()
                    <= 0.01,
            "expected inner editor touch scrolling to keep the editor viewport geometry stable inside the page: page={page} viewport_test_id={viewport_test_id} before_bounds={before_viewport_bounds:?} after_bounds={after_viewport_bounds:?} before_gallery_scroll={before_gallery_scroll:?} after_gallery_scroll={after_gallery_scroll:?}"
        );
    }

    fn assert_inner_viewport_vertical_scroll_is_owned_by_editor(
        page: &str,
        viewport_test_id: &str,
    ) {
        let mut rendered = render_gallery_page(page);
        scroll_test_id_into_gallery_viewport(&mut rendered, viewport_test_id);

        let before_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot before inner editor scroll");
        let before_gallery_scroll = node_by_test_id(before_snapshot, "ui-gallery-content-viewport")
            .extra
            .scroll;
        let before_inner_scroll = node_by_test_id(before_snapshot, viewport_test_id)
            .extra
            .scroll;
        let before_viewport_bounds = visual_bounds_by_test_id(&rendered, viewport_test_id);
        let before_center = Point::new(
            Px(before_viewport_bounds.origin.x.0 + before_viewport_bounds.size.width.0 * 0.5),
            Px(before_viewport_bounds.origin.y.0 + before_viewport_bounds.size.height.0 * 0.5),
        );
        let before_hit_chain = hit_chain_at(&mut rendered, before_center);

        assert!(
            before_inner_scroll.y_max.unwrap_or(0.0) > 0.01,
            "expected editor viewport to have vertical overflow before wheel input: page={page} viewport_test_id={viewport_test_id} before_inner_scroll={before_inner_scroll:?} before_gallery_scroll={before_gallery_scroll:?} bounds={before_viewport_bounds:?} center={before_center:?} hit_chain={before_hit_chain:?}"
        );

        wheel_test_id_center(
            &mut rendered,
            viewport_test_id,
            Point::new(Px(0.0), Px(-240.0)),
            1,
        );

        let after_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after inner editor wheel input");
        let after_gallery_scroll = node_by_test_id(after_snapshot, "ui-gallery-content-viewport")
            .extra
            .scroll;
        let after_inner_scroll = node_by_test_id(after_snapshot, viewport_test_id)
            .extra
            .scroll;
        let after_viewport_bounds = visual_bounds_by_test_id(&rendered, viewport_test_id);
        let after_center = Point::new(
            Px(after_viewport_bounds.origin.x.0 + after_viewport_bounds.size.width.0 * 0.5),
            Px(after_viewport_bounds.origin.y.0 + after_viewport_bounds.size.height.0 * 0.5),
        );
        let after_hit_chain = hit_chain_at(&mut rendered, after_center);

        assert!(
            after_inner_scroll.y.unwrap_or(0.0) > before_inner_scroll.y.unwrap_or(0.0) + 0.01,
            "expected wheel input over the editor viewport to advance the editor's own scroll state: page={page} viewport_test_id={viewport_test_id} before_inner_scroll={before_inner_scroll:?} after_inner_scroll={after_inner_scroll:?} before_gallery_scroll={before_gallery_scroll:?} after_gallery_scroll={after_gallery_scroll:?} before_bounds={before_viewport_bounds:?} after_bounds={after_viewport_bounds:?} before_center={before_center:?} after_center={after_center:?} before_hit_chain={before_hit_chain:?} after_hit_chain={after_hit_chain:?}"
        );
        assert!(
            (after_gallery_scroll.y.unwrap_or(0.0) - before_gallery_scroll.y.unwrap_or(0.0)).abs()
                <= 0.01,
            "expected the outer gallery viewport not to consume wheel input while the inner editor viewport can still scroll: page={page} viewport_test_id={viewport_test_id} before_gallery_scroll={before_gallery_scroll:?} after_gallery_scroll={after_gallery_scroll:?} before_inner_scroll={before_inner_scroll:?} after_inner_scroll={after_inner_scroll:?}"
        );
        assert!(
            (after_viewport_bounds.origin.y.0 - before_viewport_bounds.origin.y.0).abs() <= 0.01
                && (after_viewport_bounds.size.height.0 - before_viewport_bounds.size.height.0)
                    .abs()
                    <= 0.01,
            "expected inner editor scrolling to keep the editor viewport geometry stable inside the page: page={page} viewport_test_id={viewport_test_id} before_bounds={before_viewport_bounds:?} after_bounds={after_viewport_bounds:?} before_gallery_scroll={before_gallery_scroll:?} after_gallery_scroll={after_gallery_scroll:?}"
        );
    }

    fn assert_overlay_inner_viewport_vertical_scroll_is_owned(
        page: &str,
        trigger_test_id: &str,
        viewport_test_id: &str,
        bounds: Option<Rect>,
    ) {
        let mut rendered = bounds
            .map(|bounds| render_gallery_page_with_bounds(page, bounds))
            .unwrap_or_else(|| render_gallery_page(page));
        scroll_test_id_into_gallery_viewport(&mut rendered, trigger_test_id);
        click_test_id_center(&mut rendered, trigger_test_id);
        wait_until_test_id_exists(&mut rendered, viewport_test_id, 12);

        let before_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot before overlay inner wheel scroll");
        let before_gallery_scroll = node_by_test_id(before_snapshot, "ui-gallery-content-viewport")
            .extra
            .scroll;
        let before_inner_scroll = node_by_test_id(before_snapshot, viewport_test_id)
            .extra
            .scroll;

        assert!(
            before_inner_scroll.y_max.unwrap_or(0.0) > 0.01,
            "expected overlay viewport to have vertical overflow before wheel input: page={page} trigger_test_id={trigger_test_id} viewport_test_id={viewport_test_id} before_inner_scroll={before_inner_scroll:?} before_gallery_scroll={before_gallery_scroll:?}"
        );

        wheel_test_id_center(
            &mut rendered,
            viewport_test_id,
            Point::new(Px(0.0), Px(-240.0)),
            1,
        );

        let after_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after overlay inner wheel scroll");
        let after_gallery_scroll = node_by_test_id(after_snapshot, "ui-gallery-content-viewport")
            .extra
            .scroll;
        let after_inner_scroll = node_by_test_id(after_snapshot, viewport_test_id)
            .extra
            .scroll;

        assert!(
            after_inner_scroll.y.unwrap_or(0.0) > before_inner_scroll.y.unwrap_or(0.0) + 0.01,
            "expected wheel input over overlay viewport to advance the overlay's own scroll state: page={page} trigger_test_id={trigger_test_id} viewport_test_id={viewport_test_id} before_inner_scroll={before_inner_scroll:?} after_inner_scroll={after_inner_scroll:?} before_gallery_scroll={before_gallery_scroll:?} after_gallery_scroll={after_gallery_scroll:?}"
        );
        assert!(
            (after_gallery_scroll.y.unwrap_or(0.0) - before_gallery_scroll.y.unwrap_or(0.0)).abs()
                <= 0.01,
            "expected the outer gallery viewport not to consume wheel input while the opened overlay viewport can still scroll: page={page} trigger_test_id={trigger_test_id} viewport_test_id={viewport_test_id} before_gallery_scroll={before_gallery_scroll:?} after_gallery_scroll={after_gallery_scroll:?} before_inner_scroll={before_inner_scroll:?} after_inner_scroll={after_inner_scroll:?}"
        );
    }

    fn assert_overlay_inner_viewport_vertical_touch_pan_is_owned(
        page: &str,
        trigger_test_id: &str,
        viewport_test_id: &str,
        bounds: Option<Rect>,
    ) {
        let mut rendered = bounds
            .map(|bounds| render_gallery_page_with_bounds(page, bounds))
            .unwrap_or_else(|| render_gallery_page(page));
        scroll_test_id_into_gallery_viewport(&mut rendered, trigger_test_id);
        click_test_id_center(&mut rendered, trigger_test_id);
        wait_until_test_id_exists(&mut rendered, viewport_test_id, 12);

        let before_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot before overlay inner touch pan");
        let before_gallery_scroll = node_by_test_id(before_snapshot, "ui-gallery-content-viewport")
            .extra
            .scroll;
        let before_inner_scroll = node_by_test_id(before_snapshot, viewport_test_id)
            .extra
            .scroll;

        assert!(
            before_inner_scroll.y_max.unwrap_or(0.0) > 0.01,
            "expected overlay viewport to have vertical overflow before touch pan: page={page} trigger_test_id={trigger_test_id} viewport_test_id={viewport_test_id} before_inner_scroll={before_inner_scroll:?} before_gallery_scroll={before_gallery_scroll:?}"
        );

        touch_pan_test_id_center(
            &mut rendered,
            viewport_test_id,
            Point::new(Px(0.0), Px(-48.0)),
            3,
        );

        let after_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after overlay inner touch pan");
        let after_gallery_scroll = node_by_test_id(after_snapshot, "ui-gallery-content-viewport")
            .extra
            .scroll;
        let after_inner_scroll = node_by_test_id(after_snapshot, viewport_test_id)
            .extra
            .scroll;

        assert!(
            after_inner_scroll.y.unwrap_or(0.0) > before_inner_scroll.y.unwrap_or(0.0) + 0.01,
            "expected touch pan over overlay viewport to advance the overlay's own scroll state: page={page} trigger_test_id={trigger_test_id} viewport_test_id={viewport_test_id} before_inner_scroll={before_inner_scroll:?} after_inner_scroll={after_inner_scroll:?} before_gallery_scroll={before_gallery_scroll:?} after_gallery_scroll={after_gallery_scroll:?}"
        );
        assert!(
            (after_gallery_scroll.y.unwrap_or(0.0) - before_gallery_scroll.y.unwrap_or(0.0)).abs()
                <= 0.01,
            "expected the outer gallery viewport not to consume touch pan while the opened overlay viewport can still scroll: page={page} trigger_test_id={trigger_test_id} viewport_test_id={viewport_test_id} before_gallery_scroll={before_gallery_scroll:?} after_gallery_scroll={after_gallery_scroll:?} before_inner_scroll={before_inner_scroll:?} after_inner_scroll={after_inner_scroll:?}"
        );
    }

    #[test]
    fn gallery_component_pages_scroll_to_bottom_without_height_drift() {
        let cases = [
            (PAGE_ACCORDION, "ui-gallery-accordion"),
            (PAGE_AVATAR, "ui-gallery-avatar"),
            (PAGE_BUTTON_GROUP, "ui-gallery-button-group"),
            (PAGE_CALENDAR, "ui-gallery-calendar"),
            (PAGE_CARD, "ui-gallery-card"),
            (PAGE_CONTEXT_MENU, "ui-gallery-context-menu"),
            (PAGE_DIALOG, "ui-gallery-dialog"),
            (PAGE_DROPDOWN_MENU, "ui-gallery-dropdown-menu"),
            (PAGE_FIELD, "ui-gallery-field"),
            (PAGE_HOVER_CARD, "ui-gallery-hover-card"),
            (PAGE_INPUT, "ui-gallery-input"),
            (PAGE_INPUT_GROUP, "ui-gallery-input-group"),
            (PAGE_MENUBAR, "ui-gallery-menubar-component"),
            (PAGE_NAVIGATION_MENU, "ui-gallery-navigation-menu"),
            (PAGE_POPOVER, "ui-gallery-popover"),
            (PAGE_PROGRESS, "ui-gallery-progress"),
            (PAGE_SCROLL_AREA, "ui-gallery-scroll-area"),
            (PAGE_SELECT, "ui-gallery-select"),
            (PAGE_SHEET, "ui-gallery-sheet"),
            (PAGE_SIDEBAR, "ui-gallery-sidebar"),
            (PAGE_SONNER, "ui-gallery-sonner"),
            (PAGE_TABS, "ui-gallery-tabs"),
            (PAGE_TEXTAREA, "ui-gallery-textarea"),
            (PAGE_TOGGLE_GROUP, "ui-gallery-toggle-group"),
        ];

        for (page, page_root_test_id) in cases {
            assert_page_bottom_clamps_to_viewport_bottom(page, page_root_test_id);
        }
    }

    #[test]
    fn gallery_preview_card_contains_avatar_and_card_page_content_at_bottom() {
        let cases = [
            (PAGE_AVATAR, "ui-gallery-avatar"),
            (PAGE_CARD, "ui-gallery-card"),
        ];

        for (page, page_root_test_id) in cases {
            assert_preview_card_content_contains_page_bottom(page, page_root_test_id);
        }
    }

    #[test]
    fn gallery_card_core_examples_keep_upstream_aligned_targets_present() {
        let mut rendered = render_gallery_page(PAGE_CARD);

        for target in [
            "ui-gallery-card-demo-title",
            "ui-gallery-card-demo-sign-up",
            "ui-gallery-card-demo-login",
            "ui-gallery-card-demo-login-google",
            "ui-gallery-card-size-sm-action",
            "ui-gallery-card-image-featured",
            "ui-gallery-card-image-view-event",
            "ui-gallery-card-rtl-login",
            "ui-gallery-card-rtl-login-with-google",
        ] {
            scroll_test_id_into_gallery_viewport(&mut rendered, target);
            let bounds = visual_bounds_by_test_id(&rendered, target);
            assert!(
                bounds.size.width.0 > 0.0 && bounds.size.height.0 > 0.0,
                "expected Card page target to render with non-zero bounds: target={target} bounds={bounds:?}"
            );
        }
    }

    #[test]
    fn gallery_calendar_core_examples_keep_upstream_aligned_targets_present() {
        let mut rendered = render_gallery_page(PAGE_CALENDAR);

        for target in [
            "ui-gallery-calendar-demo-content",
            "ui-gallery-calendar-usage-content",
            "ui-gallery-calendar-hijri-content",
            "ui-gallery-calendar-basic-content",
            "ui-gallery-calendar-range-content",
            "ui-gallery-calendar-caption-content",
            "ui-gallery-calendar-presets-content",
            "ui-gallery-calendar-time-content",
            "ui-gallery-calendar-booked-content",
            "ui-gallery-calendar-custom-cell-content",
            "ui-gallery-calendar-week-numbers-content",
            "ui-gallery-calendar-rtl-content",
        ] {
            scroll_test_id_into_gallery_viewport(&mut rendered, target);
            let bounds = visual_bounds_by_test_id(&rendered, target);
            assert!(
                bounds.size.width.0 > 0.0 && bounds.size.height.0 > 0.0,
                "expected Calendar page target to render with non-zero bounds: target={target} bounds={bounds:?}"
            );
        }
    }

    #[test]
    fn gallery_card_compositions_keep_consistent_card_widths() {
        let mut rendered = render_gallery_page(PAGE_CARD);

        scroll_test_id_into_gallery_viewport(
            &mut rendered,
            "ui-gallery-card-compositions-footer-only",
        );

        let content_only =
            visual_bounds_by_test_id(&rendered, "ui-gallery-card-compositions-content-only");
        let header_only =
            visual_bounds_by_test_id(&rendered, "ui-gallery-card-compositions-header-only");
        let footer_only =
            visual_bounds_by_test_id(&rendered, "ui-gallery-card-compositions-footer-only");
        let header_content =
            visual_bounds_by_test_id(&rendered, "ui-gallery-card-compositions-header-content");

        let expected_width = content_only.size.width.0;
        for (name, bounds) in [
            ("header_only", header_only),
            ("footer_only", footer_only),
            ("header_content", header_content),
        ] {
            assert!(
                (bounds.size.width.0 - expected_width).abs() <= 1.0,
                "expected Card compositions sample '{name}' to keep the shared card width: expected≈{expected_width} actual={} content_only={content_only:?} header_only={header_only:?} footer_only={footer_only:?} header_content={header_content:?}",
                bounds.size.width.0,
            );
        }
    }

    fn assert_notes_section_keeps_stable_height_while_scrolling_into_view(
        page: &str,
        notes_test_id: &str,
    ) {
        let mut rendered = render_gallery_page(page);
        let mut baseline_notes_height: Option<f32> = None;
        let mut notes_became_visible = false;

        for _ in 0..48 {
            let viewport_bounds =
                visual_bounds_by_test_id(&rendered, "ui-gallery-content-viewport");
            let content_bounds =
                visual_bounds_by_test_id(&rendered, "ui-gallery-preview-card-content");
            let notes_bounds = visual_bounds_by_test_id(&rendered, notes_test_id);

            if rects_intersect(viewport_bounds, notes_bounds) {
                let containment_epsilon = 1.0;
                let content_left = content_bounds.origin.x.0;
                let content_top = content_bounds.origin.y.0;
                let content_right = content_left + content_bounds.size.width.0;
                let content_bottom = content_top + content_bounds.size.height.0;
                let notes_left = notes_bounds.origin.x.0;
                let notes_top = notes_bounds.origin.y.0;
                let notes_right = notes_left + notes_bounds.size.width.0;
                let notes_bottom = notes_top + notes_bounds.size.height.0;
                let current_notes_height = notes_bounds.size.height.0;

                notes_became_visible = true;
                if let Some(baseline) = baseline_notes_height {
                    assert!(
                        (current_notes_height - baseline).abs() <= 0.01,
                        "expected notes section height to remain stable while scrolling: page={page} content_bounds={content_bounds:?} notes_bounds={notes_bounds:?} viewport={viewport_bounds:?} baseline_notes_height={baseline} current_notes_height={current_notes_height}"
                    );
                } else {
                    baseline_notes_height = Some(current_notes_height);
                }
                assert!(
                    notes_left >= content_left - containment_epsilon
                        && notes_top >= content_top - containment_epsilon
                        && notes_right <= content_right + containment_epsilon
                        && notes_bottom <= content_bottom + containment_epsilon,
                    "expected notes section to remain contained by preview-card content while scrolling: page={page} content_bounds={content_bounds:?} notes_bounds={notes_bounds:?} viewport={viewport_bounds:?} containment_epsilon={containment_epsilon}"
                );
            }

            let viewport_bottom = viewport_bounds.origin.y.0 + viewport_bounds.size.height.0;
            let notes_bottom = notes_bounds.origin.y.0 + notes_bounds.size.height.0;
            if notes_became_visible && (notes_bottom - viewport_bottom).abs() <= 2.0 {
                break;
            }

            wheel_gallery_viewport(&mut rendered, Px(-240.0));
        }

        assert!(
            notes_became_visible,
            "expected notes section to become visible while scrolling the gallery: page={page} notes_test_id={notes_test_id}"
        );
    }

    #[test]
    fn notes_sections_keep_stable_height_while_scrolling_into_view() {
        let cases = [
            (PAGE_ACCORDION, "ui-gallery-accordion-notes-content"),
            (PAGE_ALERT, "ui-gallery-alert-notes-content"),
            (PAGE_ALERT_DIALOG, "ui-gallery-alert-dialog-notes-content"),
            (PAGE_AVATAR, "ui-gallery-avatar-notes-content"),
            (PAGE_BUTTON_GROUP, "ui-gallery-button-group-notes-content"),
            (PAGE_CALENDAR, "ui-gallery-calendar-notes-content"),
            (PAGE_CARD, "ui-gallery-card-section-notes-content"),
            (PAGE_COMBOBOX, "ui-gallery-combobox-notes-content"),
            (PAGE_COMMAND, "ui-gallery-command-notes-content"),
            (PAGE_CONTEXT_MENU, "ui-gallery-context-menu-notes-content"),
            (PAGE_DATA_TABLE, "ui-gallery-data-table-notes-content"),
            (PAGE_DIALOG, "ui-gallery-dialog-notes-content"),
            (PAGE_DROPDOWN_MENU, "ui-gallery-dropdown-menu-notes-content"),
            (PAGE_FIELD, "ui-gallery-field-notes-content"),
            (PAGE_HOVER_CARD, "ui-gallery-hover-card-notes-content"),
            (PAGE_INPUT, "ui-gallery-input-notes-content"),
            (PAGE_INPUT_GROUP, "ui-gallery-input-group-notes-content"),
            (PAGE_MENUBAR, "ui-gallery-menubar-notes-content"),
            (
                PAGE_NAVIGATION_MENU,
                "ui-gallery-navigation-menu-notes-content",
            ),
            (PAGE_POPOVER, "ui-gallery-popover-notes-content"),
            (PAGE_PROGRESS, "ui-gallery-progress-notes-content"),
            (PAGE_SHEET, "ui-gallery-sheet-notes-content"),
            (PAGE_SIDEBAR, "ui-gallery-sidebar-notes-content"),
            (PAGE_TEXTAREA, "ui-gallery-textarea-notes-content"),
            (PAGE_TOGGLE_GROUP, "ui-gallery-toggle-group-notes-content"),
            (PAGE_TYPOGRAPHY, "ui-gallery-typography-notes-content"),
        ];

        for (page, notes_test_id) in cases {
            assert_notes_section_keeps_stable_height_while_scrolling_into_view(page, notes_test_id);
        }
    }

    #[test]
    fn nested_scroll_area_vertical_wheel_bubbles_from_inner_x_viewport_to_outer_y_viewport() {
        let mut rendered = render_gallery_page(PAGE_SCROLL_AREA);
        let outer_viewport_test_id = "ui-gallery-scroll-area-nested-outer-viewport";
        let inner_viewport_test_id = "ui-gallery-scroll-area-nested-inner-viewport";

        scroll_test_id_into_gallery_viewport(&mut rendered, inner_viewport_test_id);

        let before_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot before nested scroll routing check");
        let before_outer_scroll = node_by_test_id(before_snapshot, outer_viewport_test_id)
            .extra
            .scroll;
        let before_inner_scroll = node_by_test_id(before_snapshot, inner_viewport_test_id)
            .extra
            .scroll;

        assert!(
            before_outer_scroll.y_max.unwrap_or(0.0) > 0.01,
            "expected nested outer viewport to be vertically scrollable before routing check: outer_scroll={before_outer_scroll:?}"
        );
        assert!(
            before_inner_scroll.x_max.unwrap_or(0.0) > 0.01,
            "expected nested inner viewport to be horizontally scrollable before routing check: inner_scroll={before_inner_scroll:?}"
        );

        for _ in 0..6 {
            let viewport = visual_bounds_by_test_id(&rendered, inner_viewport_test_id);
            let wheel_pos = Point::new(
                Px(viewport.origin.x.0 + viewport.size.width.0 * 0.5),
                Px(viewport.origin.y.0 + viewport.size.height.0 * 0.5),
            );
            rendered.state.ui.dispatch_event(
                &mut rendered.app,
                &mut rendered.services,
                &Event::Pointer(PointerEvent::Wheel {
                    position: wheel_pos,
                    delta: Point::new(Px(0.0), Px(-240.0)),
                    modifiers: Modifiers::default(),
                    pointer_id: PointerId(0),
                    pointer_type: PointerType::Mouse,
                }),
            );
            render_gallery_frame(&mut rendered);
        }

        let after_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after nested scroll routing wheel input");
        let after_outer_scroll = node_by_test_id(after_snapshot, outer_viewport_test_id)
            .extra
            .scroll;
        let after_inner_scroll = node_by_test_id(after_snapshot, inner_viewport_test_id)
            .extra
            .scroll;

        assert!(
            after_outer_scroll.y.unwrap_or(0.0) > before_outer_scroll.y.unwrap_or(0.0) + 0.01,
            "expected vertical wheel input over the inner X-scroll viewport to bubble into the outer Y-scroll viewport: before_outer={before_outer_scroll:?} after_outer={after_outer_scroll:?} before_inner={before_inner_scroll:?} after_inner={after_inner_scroll:?}"
        );
        assert!(
            (after_inner_scroll.x.unwrap_or(0.0) - before_inner_scroll.x.unwrap_or(0.0)).abs()
                <= 0.01,
            "expected inner X-scroll viewport not to consume a dominant vertical wheel gesture: before_inner={before_inner_scroll:?} after_inner={after_inner_scroll:?} before_outer={before_outer_scroll:?} after_outer={after_outer_scroll:?}"
        );
    }

    #[test]
    fn nested_scroll_area_vertical_touch_pan_bubbles_from_inner_x_viewport_to_outer_y_viewport() {
        let mut rendered = render_gallery_page(PAGE_SCROLL_AREA);
        let outer_viewport_test_id = "ui-gallery-scroll-area-nested-outer-viewport";
        let inner_viewport_test_id = "ui-gallery-scroll-area-nested-inner-viewport";

        scroll_test_id_into_gallery_viewport(&mut rendered, outer_viewport_test_id);
        scroll_test_id_into_gallery_viewport(&mut rendered, inner_viewport_test_id);

        let before_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot before nested scroll routing touch pan");
        let before_outer_scroll = node_by_test_id(before_snapshot, outer_viewport_test_id)
            .extra
            .scroll;
        let before_inner_scroll = node_by_test_id(before_snapshot, inner_viewport_test_id)
            .extra
            .scroll;

        touch_pan_test_id_center(
            &mut rendered,
            inner_viewport_test_id,
            Point::new(Px(0.0), Px(-96.0)),
            3,
        );

        let after_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after nested scroll routing touch pan");
        let after_outer_scroll = node_by_test_id(after_snapshot, outer_viewport_test_id)
            .extra
            .scroll;
        let after_inner_scroll = node_by_test_id(after_snapshot, inner_viewport_test_id)
            .extra
            .scroll;

        assert!(
            after_outer_scroll.y.unwrap_or(0.0) > before_outer_scroll.y.unwrap_or(0.0) + 0.01,
            "expected vertical touch pan over the inner X-scroll viewport to bubble into the outer Y-scroll viewport: before_outer={before_outer_scroll:?} after_outer={after_outer_scroll:?} before_inner={before_inner_scroll:?} after_inner={after_inner_scroll:?}"
        );
        assert!(
            (after_inner_scroll.x.unwrap_or(0.0) - before_inner_scroll.x.unwrap_or(0.0)).abs()
                <= 0.01,
            "expected inner X-scroll viewport not to consume a dominant vertical touch pan: before_inner={before_inner_scroll:?} after_inner={after_inner_scroll:?} before_outer={before_outer_scroll:?} after_outer={after_outer_scroll:?}"
        );
    }

    #[test]
    fn expand_at_bottom_scroll_area_gains_scroll_range_and_scrolls_after_toggle() {
        let mut rendered = render_gallery_page(PAGE_SCROLL_AREA);
        let viewport_test_id = "ui-gallery-scroll-area-expand-at-bottom-viewport";
        let toggle_test_id = "ui-gallery-scroll-area-expand-at-bottom-toggle";

        scroll_test_id_into_gallery_viewport(&mut rendered, viewport_test_id);

        let before_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot before expand-at-bottom toggle");
        let before_scroll = node_by_test_id(before_snapshot, viewport_test_id)
            .extra
            .scroll;

        assert!(
            before_scroll.y_max.unwrap_or(0.0) <= 0.01,
            "expected expand-at-bottom viewport to start without vertical overflow: before_scroll={before_scroll:?}"
        );

        scroll_test_id_into_gallery_viewport(&mut rendered, toggle_test_id);
        click_test_id_center(&mut rendered, toggle_test_id);
        scroll_test_id_into_gallery_viewport(&mut rendered, viewport_test_id);

        let expanded_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after expand-at-bottom toggle");
        let expanded_scroll = node_by_test_id(expanded_snapshot, viewport_test_id)
            .extra
            .scroll;

        assert!(
            expanded_scroll.y_max.unwrap_or(0.0) > before_scroll.y_max.unwrap_or(0.0) + 0.01,
            "expected expand-at-bottom toggle to grow the viewport scroll range immediately: before_scroll={before_scroll:?} expanded_scroll={expanded_scroll:?}"
        );
        assert!(
            expanded_scroll.y.unwrap_or(0.0) <= before_scroll.y.unwrap_or(0.0) + 0.01,
            "expected toggling extra rows at the bottom not to spuriously move the current offset: before_scroll={before_scroll:?} expanded_scroll={expanded_scroll:?}"
        );

        for _ in 0..8 {
            let viewport = visual_bounds_by_test_id(&rendered, viewport_test_id);
            let wheel_pos = Point::new(
                Px(viewport.origin.x.0 + viewport.size.width.0 * 0.5),
                Px(viewport.origin.y.0 + viewport.size.height.0 * 0.5),
            );
            rendered.state.ui.dispatch_event(
                &mut rendered.app,
                &mut rendered.services,
                &Event::Pointer(PointerEvent::Wheel {
                    position: wheel_pos,
                    delta: Point::new(Px(0.0), Px(-240.0)),
                    modifiers: Modifiers::default(),
                    pointer_id: PointerId(0),
                    pointer_type: PointerType::Mouse,
                }),
            );
            render_gallery_frame(&mut rendered);
        }

        let after_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after scrolling expanded viewport");
        let after_scroll = node_by_test_id(after_snapshot, viewport_test_id)
            .extra
            .scroll;

        assert!(
            after_scroll.y.unwrap_or(0.0) > expanded_scroll.y.unwrap_or(0.0) + 0.01,
            "expected wheel input to scroll the expand-at-bottom viewport after growth at the bottom: before_scroll={before_scroll:?} expanded_scroll={expanded_scroll:?} after_scroll={after_scroll:?}"
        );
    }

    #[test]
    fn expand_at_bottom_scroll_area_clamps_offset_when_collapsing_after_bottom_scroll() {
        let mut rendered = render_gallery_page(PAGE_SCROLL_AREA);
        let viewport_test_id = "ui-gallery-scroll-area-expand-at-bottom-viewport";
        let toggle_test_id = "ui-gallery-scroll-area-expand-at-bottom-toggle";

        scroll_test_id_into_gallery_viewport(&mut rendered, viewport_test_id);
        scroll_test_id_into_gallery_viewport(&mut rendered, toggle_test_id);
        click_test_id_center(&mut rendered, toggle_test_id);
        scroll_test_id_into_gallery_viewport(&mut rendered, viewport_test_id);

        wheel_test_id_center(
            &mut rendered,
            viewport_test_id,
            Point::new(Px(0.0), Px(-240.0)),
            12,
        );

        let before_collapse_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot before collapsing expand-at-bottom viewport");
        let before_collapse_scroll = node_by_test_id(before_collapse_snapshot, viewport_test_id)
            .extra
            .scroll;

        assert!(
            before_collapse_scroll.y_max.unwrap_or(0.0) > 0.01,
            "expected expanded viewport to expose vertical overflow before collapse: scroll={before_collapse_scroll:?}"
        );
        assert!(
            before_collapse_scroll.y.unwrap_or(0.0) + 0.5
                >= before_collapse_scroll.y_max.unwrap_or(0.0),
            "expected repeated wheel input to reach the bottom before collapse: scroll={before_collapse_scroll:?}"
        );

        scroll_test_id_into_gallery_viewport(&mut rendered, toggle_test_id);
        click_test_id_center(&mut rendered, toggle_test_id);
        scroll_test_id_into_gallery_viewport(&mut rendered, viewport_test_id);

        let collapsed_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after collapsing expand-at-bottom viewport");
        let collapsed_scroll = node_by_test_id(collapsed_snapshot, viewport_test_id)
            .extra
            .scroll;

        assert!(
            collapsed_scroll.y_max.unwrap_or(0.0) <= 0.01,
            "expected collapsing the extra rows to remove vertical overflow immediately: before={before_collapse_scroll:?} after={collapsed_scroll:?}"
        );
        assert!(
            collapsed_scroll.y.unwrap_or(0.0) <= 0.01,
            "expected collapsing after bottom scroll to clamp the offset back to the top immediately: before={before_collapse_scroll:?} after={collapsed_scroll:?}"
        );

        wheel_test_id_center(
            &mut rendered,
            viewport_test_id,
            Point::new(Px(0.0), Px(-240.0)),
            4,
        );

        let after_repeat_snapshot =
            rendered.state.ui.semantics_snapshot().expect(
                "expected semantics snapshot after repeated wheel input on collapsed viewport",
            );
        let after_repeat_scroll = node_by_test_id(after_repeat_snapshot, viewport_test_id)
            .extra
            .scroll;

        assert!(
            after_repeat_scroll.y_max.unwrap_or(0.0) <= 0.01,
            "expected collapsed viewport to keep zero vertical overflow after repeated wheel input: collapsed={collapsed_scroll:?} repeated={after_repeat_scroll:?}"
        );
        assert!(
            (after_repeat_scroll.y.unwrap_or(0.0) - collapsed_scroll.y.unwrap_or(0.0)).abs()
                <= 0.01,
            "expected repeated wheel input on the collapsed viewport not to reintroduce offset drift: collapsed={collapsed_scroll:?} repeated={after_repeat_scroll:?}"
        );
    }

    #[cfg(feature = "gallery-dev")]
    #[test]
    fn editor_inner_viewports_scroll_without_advancing_gallery_page() {
        let cases = [
            (
                PAGE_MARKDOWN_EDITOR_SOURCE,
                "ui-gallery-markdown-editor-viewport",
            ),
            (PAGE_CODE_EDITOR_MVP, "ui-gallery-code-editor-mvp-viewport"),
            (
                PAGE_CODE_EDITOR_MVP,
                "ui-gallery-code-editor-a11y-selection-wrap-gate-viewport",
            ),
        ];

        for (page, viewport_test_id) in cases {
            assert_inner_viewport_vertical_scroll_is_owned_by_editor(page, viewport_test_id);
        }
    }

    #[cfg(feature = "gallery-dev")]
    #[test]
    fn editor_inner_viewports_touch_pan_without_advancing_gallery_page() {
        let cases = [
            (
                PAGE_MARKDOWN_EDITOR_SOURCE,
                "ui-gallery-markdown-editor-viewport",
            ),
            (PAGE_CODE_EDITOR_MVP, "ui-gallery-code-editor-mvp-viewport"),
            (
                PAGE_CODE_EDITOR_MVP,
                "ui-gallery-code-editor-a11y-selection-wrap-gate-viewport",
            ),
        ];

        for (page, viewport_test_id) in cases {
            assert_inner_viewport_vertical_touch_pan_is_owned_by_editor(page, viewport_test_id);
        }
    }

    #[test]
    fn dialog_and_drawer_inner_viewports_scroll_without_advancing_gallery_page() {
        let cases = [
            (
                PAGE_DIALOG,
                "ui-gallery-dialog-scrollable-trigger",
                "ui-gallery-dialog-scrollable-viewport",
            ),
            (
                PAGE_DIALOG,
                "ui-gallery-dialog-sticky-footer-trigger",
                "ui-gallery-dialog-sticky-footer-viewport",
            ),
            (
                PAGE_DRAWER,
                "ui-gallery-drawer-scrollable-trigger",
                "ui-gallery-drawer-scrollable-viewport",
            ),
        ];

        for (page, trigger_test_id, viewport_test_id) in cases {
            assert_overlay_inner_viewport_vertical_scroll_is_owned(
                page,
                trigger_test_id,
                viewport_test_id,
                None,
            );
        }
    }

    #[test]
    fn dialog_and_drawer_inner_viewports_touch_pan_without_advancing_gallery_page() {
        let cases = [
            (
                PAGE_DIALOG,
                "ui-gallery-dialog-scrollable-trigger",
                "ui-gallery-dialog-scrollable-viewport",
            ),
            (
                PAGE_DIALOG,
                "ui-gallery-dialog-sticky-footer-trigger",
                "ui-gallery-dialog-sticky-footer-viewport",
            ),
            (
                PAGE_DRAWER,
                "ui-gallery-drawer-scrollable-trigger",
                "ui-gallery-drawer-scrollable-viewport",
            ),
        ];

        for (page, trigger_test_id, viewport_test_id) in cases {
            assert_overlay_inner_viewport_vertical_touch_pan_is_owned(
                page,
                trigger_test_id,
                viewport_test_id,
                None,
            );
        }
    }

    #[cfg(feature = "gallery-dev")]
    #[test]
    fn overlay_sheet_inner_viewport_scroll_without_advancing_gallery_page() {
        assert_overlay_inner_viewport_vertical_scroll_is_owned(
            PAGE_OVERLAY,
            "ui-gallery-sheet-trigger",
            "ui-gallery-sheet-scroll-viewport",
            Some(Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(1080.0), Px(360.0)),
            )),
        );
    }

    #[cfg(feature = "gallery-dev")]
    #[test]
    fn overlay_sheet_inner_viewport_touch_pan_without_advancing_gallery_page() {
        assert_overlay_inner_viewport_vertical_touch_pan_is_owned(
            PAGE_OVERLAY,
            "ui-gallery-sheet-trigger",
            "ui-gallery-sheet-scroll-viewport",
            Some(Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(1080.0), Px(360.0)),
            )),
        );
    }

    #[test]
    fn scroll_area_drag_baseline_scrollbar_thumb_drag_advances_inner_viewport_without_advancing_gallery_page()
     {
        assert_scrollbar_thumb_drag_advances_inner_viewport_without_advancing_gallery_page(
            PAGE_SCROLL_AREA,
            "ui-gallery-scroll-area-drag-baseline-y-scrollbar",
        );
    }

    #[cfg(feature = "gallery-material3")]
    #[test]
    fn material3_top_app_bar_exit_until_collapsed_reacts_to_inner_scroll_viewport() {
        let mut rendered = render_gallery_page(PAGE_MATERIAL3_TOP_APP_BAR);
        let viewport_test_id =
            "ui-gallery-material3-top-app-bar-exit-until-collapsed-scroll-viewport";

        scroll_test_id_into_gallery_viewport(&mut rendered, viewport_test_id);

        let before_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after initial render");
        let before_viewport_scroll = node_by_test_id(before_snapshot, viewport_test_id)
            .extra
            .scroll;
        let before_viewport = visual_bounds_by_test_id(&rendered, viewport_test_id);
        assert!(
            before_viewport.size.height.0 > 0.01,
            "expected top app bar demo viewport to have non-zero initial height; bounds={before_viewport:?}"
        );

        for _ in 0..8 {
            let viewport = visual_bounds_by_test_id(&rendered, viewport_test_id);
            let wheel_pos = Point::new(
                Px(viewport.origin.x.0 + viewport.size.width.0 * 0.5),
                Px(viewport.origin.y.0 + viewport.size.height.0 * 0.5),
            );
            rendered.state.ui.dispatch_event(
                &mut rendered.app,
                &mut rendered.services,
                &Event::Pointer(PointerEvent::Wheel {
                    position: wheel_pos,
                    delta: Point::new(Px(0.0), Px(-240.0)),
                    modifiers: Modifiers::default(),
                    pointer_id: PointerId(0),
                    pointer_type: PointerType::Mouse,
                }),
            );
            render_gallery_frame(&mut rendered);
        }

        let after_snapshot = rendered
            .state
            .ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after wheel scrolling");
        let after_viewport_scroll = node_by_test_id(after_snapshot, viewport_test_id)
            .extra
            .scroll;
        let after_viewport = visual_bounds_by_test_id(&rendered, viewport_test_id);
        assert!(
            after_viewport_scroll.y.unwrap_or(0.0) > before_viewport_scroll.y.unwrap_or(0.0) + 0.01,
            "expected wheel input to advance the inner scroll viewport before checking app-bar collapse: before_scroll={:?} after_scroll={:?} before_bounds={before_viewport:?} after_bounds={after_viewport:?}",
            before_viewport_scroll,
            after_viewport_scroll,
        );
        assert!(
            after_viewport.origin.y.0 + 4.0 < before_viewport.origin.y.0
                || after_viewport.size.height.0 > before_viewport.size.height.0 + 4.0,
            "expected exit-until-collapsed bar to free more viewport space after inner scrolling: before={before_viewport:?} after={after_viewport:?} before_scroll={:?} after_scroll={:?}",
            before_viewport_scroll,
            after_viewport_scroll,
        );
    }
}
