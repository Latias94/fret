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
        AppWindowId, Event, Modifiers, PathCommand, PathConstraints, PathId, PathMetrics,
        PathService, PathStyle, Point, PointerEvent, PointerId, PointerType, Px, Rect, Size, SvgId,
        SvgService, TextBlobId, TextConstraints, TextMetrics, TextService,
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

    fn node_by_test_id<'a>(
        snap: &'a fret_core::SemanticsSnapshot,
        test_id: &str,
    ) -> &'a fret_core::SemanticsNode {
        snap.nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some(test_id))
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
        let node_id = node_id_by_test_id(rendered, test_id);
        rendered
            .state
            .ui
            .debug_node_visual_bounds(node_id)
            .unwrap_or_else(|| panic!("missing visual bounds for test_id={test_id}"))
    }

    fn render_gallery_page(page: &str) -> RenderedGalleryPage {
        let window = AppWindowId::default();
        let mut app = App::new();
        let state = UiGalleryDriver::build_ui(&mut app, window);
        let _ = app.models_mut().update(&state.selected_page, |selected| {
            *selected = Arc::<str>::from(page)
        });

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(1080.0), Px(720.0)),
        );
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

    fn assert_page_bottom_clamps_to_viewport_bottom(page: &str, page_root_test_id: &str) {
        let mut rendered = render_gallery_page(page);
        let initial_page_bounds = visual_bounds_by_test_id(&rendered, page_root_test_id);
        let mut last_page_y = initial_page_bounds.origin.y.0;
        let mut moved = false;
        let mut stable_frames = 0usize;

        for _ in 0..48 {
            let viewport_bounds =
                visual_bounds_by_test_id(&rendered, "ui-gallery-content-viewport");
            let wheel_pos = Point::new(
                Px(viewport_bounds.origin.x.0 + viewport_bounds.size.width.0 * 0.5),
                Px(viewport_bounds.origin.y.0 + viewport_bounds.size.height.0 * 0.5),
            );

            rendered.state.ui.dispatch_event(
                &mut rendered.app,
                &mut rendered.services,
                &Event::Pointer(PointerEvent::Wheel {
                    position: wheel_pos,
                    delta: Point::new(Px(0.0), Px(-2000.0)),
                    modifiers: Modifiers::default(),
                    pointer_id: PointerId(0),
                    pointer_type: PointerType::Mouse,
                }),
            );

            render_gallery_frame(&mut rendered);

            let page_bounds = visual_bounds_by_test_id(&rendered, page_root_test_id);
            if page_bounds.origin.y.0 < last_page_y - 0.5 {
                moved = true;
                stable_frames = 0;
            } else {
                stable_frames += 1;
            }
            last_page_y = page_bounds.origin.y.0;

            let viewport_bottom = viewport_bounds.origin.y.0 + viewport_bounds.size.height.0;
            let page_bottom = page_bounds.origin.y.0 + page_bounds.size.height.0;
            if moved && (page_bottom - viewport_bottom).abs() <= 2.0 {
                return;
            }
            if moved && stable_frames >= 3 {
                break;
            }
        }

        let viewport_bounds = visual_bounds_by_test_id(&rendered, "ui-gallery-content-viewport");
        let page_bounds = visual_bounds_by_test_id(&rendered, page_root_test_id);

        let viewport_bottom = viewport_bounds.origin.y.0 + viewport_bounds.size.height.0;
        let page_bottom = page_bounds.origin.y.0 + page_bounds.size.height.0;

        assert!(
            moved,
            "expected wheel scrolling to move the gallery page for page={page}: initial_page={initial_page_bounds:?} final_page={page_bounds:?} viewport={viewport_bounds:?}"
        );

        assert!(
            (page_bottom - viewport_bottom).abs() <= 2.0,
            "expected gallery content to clamp cleanly at the viewport bottom for page={page}: viewport={viewport_bounds:?} page={page_bounds:?} page_bottom={page_bottom} viewport_bottom={viewport_bottom}"
        );
    }

    fn scroll_test_id_into_gallery_viewport(
        rendered: &mut RenderedGalleryPage,
        target_test_id: &str,
    ) {
        for _ in 0..64 {
            let gallery_viewport =
                visual_bounds_by_test_id(rendered, "ui-gallery-content-viewport");
            let target_bounds = visual_bounds_by_test_id(rendered, target_test_id);
            let target_center_y = target_bounds.origin.y.0 + target_bounds.size.height.0 * 0.5;
            let visible_top = gallery_viewport.origin.y.0;
            let visible_bottom = visible_top + gallery_viewport.size.height.0;

            if target_center_y >= visible_top + 4.0 && target_center_y <= visible_bottom - 4.0 {
                return;
            }

            let wheel_pos = Point::new(
                Px(gallery_viewport.origin.x.0 + gallery_viewport.size.width.0 * 0.5),
                Px(gallery_viewport.origin.y.0 + gallery_viewport.size.height.0 * 0.5),
            );
            let delta_y = if target_center_y > visible_bottom {
                Px(-480.0)
            } else {
                Px(480.0)
            };
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

        let gallery_viewport = visual_bounds_by_test_id(rendered, "ui-gallery-content-viewport");
        let target_bounds = visual_bounds_by_test_id(rendered, target_test_id);
        panic!(
            "expected target to become visible inside gallery viewport before interaction: target_test_id={target_test_id} gallery_viewport={gallery_viewport:?} target_bounds={target_bounds:?}"
        );
    }

    #[test]
    fn gallery_component_pages_scroll_to_bottom_without_height_drift() {
        let cases = [
            (PAGE_ACCORDION, "ui-gallery-accordion"),
            (PAGE_AVATAR, "ui-gallery-avatar"),
            (PAGE_CARD, "ui-gallery-card"),
            (PAGE_DIALOG, "ui-gallery-dialog"),
            (PAGE_INPUT, "ui-gallery-input"),
            (PAGE_INPUT_GROUP, "ui-gallery-input-group"),
            (PAGE_MENUBAR, "ui-gallery-menubar-component"),
            (PAGE_NAVIGATION_MENU, "ui-gallery-navigation-menu"),
            (PAGE_POPOVER, "ui-gallery-popover"),
            (PAGE_SCROLL_AREA, "ui-gallery-scroll-area"),
            (PAGE_SELECT, "ui-gallery-select"),
            (PAGE_SHEET, "ui-gallery-sheet"),
            (PAGE_SIDEBAR, "ui-gallery-sidebar"),
            (PAGE_SONNER, "ui-gallery-sonner"),
            (PAGE_TABS, "ui-gallery-tabs"),
            (PAGE_TEXTAREA, "ui-gallery-textarea"),
        ];

        for (page, page_root_test_id) in cases {
            assert_page_bottom_clamps_to_viewport_bottom(page, page_root_test_id);
        }
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
