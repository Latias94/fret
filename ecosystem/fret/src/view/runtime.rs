use fret_core::AppWindowId;
use fret_ui::action::{OnCommand, OnCommandAvailability};
use fret_ui::{ElementContext, UiHost};

use super::{AppUi, View};

struct ViewRuntimeActionHooksOwner;
struct AppUiRenderRootActionHooksOwner;

#[doc(hidden)]
pub struct ViewWindowState<V: View> {
    pub view: V,
    pub(crate) cached_handlers: Option<(OnCommand, OnCommandAvailability)>,
    pub(crate) cached_action_root: Option<fret_ui::GlobalElementId>,
}

/// Keepalive state for manual `render_root(...)` surfaces that opt into `AppUi`.
///
/// This mirrors the cached action-handler lifecycle used by the `View` runtime so manual `UiTree`
/// / `FnDriver` hosts can reuse the same grouped `AppUi` authoring surface without reintroducing
/// low-level action-hook bookkeeping at each call site.
#[derive(Default)]
pub struct AppUiRenderRootState {
    pub(crate) cached_handlers: Option<(OnCommand, OnCommandAvailability)>,
    pub(crate) cached_action_root: Option<fret_ui::GlobalElementId>,
}

fn clear_app_ui_action_handlers_for_owner<Owner: 'static, H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    action_root: fret_ui::GlobalElementId,
) {
    cx.action_clear_on_command_for_owner::<Owner>(action_root);
    cx.action_clear_on_command_availability_for_owner::<Owner>(action_root);
}

fn install_app_ui_action_handlers_for_owner<Owner: 'static, H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    action_root: fret_ui::GlobalElementId,
    handlers: &Option<(OnCommand, OnCommandAvailability)>,
) {
    if let Some((on_command, on_command_availability)) = handlers.clone() {
        cx.action_route_fallback_root(action_root);
        cx.action_on_command_for_owner::<Owner>(action_root, on_command);
        cx.action_on_command_availability_for_owner::<Owner>(action_root, on_command_availability);
    }
}

fn render_app_ui_with_cached_handlers<'a, Owner: 'static, H: UiHost + 'static>(
    cx: &mut ElementContext<'a, H>,
    action_root_name: &str,
    cached_handlers: &mut Option<(OnCommand, OnCommandAvailability)>,
    cached_action_root: &mut Option<fret_ui::GlobalElementId>,
    render: impl for<'cx, 'el> FnOnce(&mut AppUi<'cx, 'el, H>) -> crate::Ui,
) -> crate::Ui {
    let mut render = Some(render);
    cx.named(action_root_name, |cx| {
        if let Some(action_root) = *cached_action_root {
            clear_app_ui_action_handlers_for_owner::<Owner, _>(cx, action_root);

            // Ensure handlers remain installed even when the view-cache root is reused (render
            // skipped).
            install_app_ui_action_handlers_for_owner::<Owner, _>(cx, action_root, cached_handlers);
        }

        cx.view_cache(
            fret_ui::element::ViewCacheProps {
                cache_key: 0,
                ..fret_ui::element::ViewCacheProps::default()
            }
            .contain_layout_when_bounds_known(true),
            |cx| {
                let action_root = cx.root_id();
                clear_app_ui_action_handlers_for_owner::<Owner, _>(cx, action_root);

                let mut app_ui = AppUi::new(cx, action_root);
                let render = render.take().expect("AppUi render closure should run once");
                let out = render(&mut app_ui);
                *cached_handlers = app_ui.take_action_handlers();
                *cached_action_root = Some(action_root);

                install_app_ui_action_handlers_for_owner::<Owner, _>(
                    cx,
                    action_root,
                    cached_handlers,
                );

                out
            },
        )
        .into()
    })
}

#[doc(hidden)]
pub fn view_init_window<V: View>(
    app: &mut fret_app::App,
    window: AppWindowId,
) -> ViewWindowState<V> {
    ViewWindowState {
        view: V::init(app, window),
        cached_handlers: None,
        cached_action_root: None,
    }
}

#[doc(hidden)]
pub fn view_view<'a, V: View>(
    cx: &mut ElementContext<'a, fret_app::App>,
    st: &mut ViewWindowState<V>,
) -> crate::Ui {
    let ViewWindowState {
        view,
        cached_handlers,
        cached_action_root,
    } = st;
    render_app_ui_with_cached_handlers::<ViewRuntimeActionHooksOwner, _>(
        cx,
        "__fret.view.action_root",
        cached_handlers,
        cached_action_root,
        |app_ui| view.render(app_ui),
    )
}

/// Render a manual declarative root through the grouped `AppUi` authoring surface.
///
/// This is the explicit advanced/manual-assembly bridge for `UiTree` / `FnDriver` code that still
/// owns its own window state but wants the same `AppUi` + `LocalState` authoring lane that the
/// higher-level `View` runtime uses.
pub fn render_root_with_app_ui<'a, H: UiHost + 'static>(
    cx: fret_ui::declarative::RenderRootContext<'a, H>,
    root_name: &str,
    state: &mut AppUiRenderRootState,
    render: impl for<'cx, 'el> FnOnce(&mut AppUi<'cx, 'el, H>) -> crate::Ui,
) -> fret_core::NodeId {
    let fret_ui::declarative::RenderRootContext {
        ui,
        app,
        services,
        window,
        bounds,
    } = cx;
    fret_ui::declarative::render_root(ui, app, services, window, bounds, root_name, |cx| {
        render_app_ui_with_cached_handlers::<AppUiRenderRootActionHooksOwner, _>(
            cx,
            "__fret.advanced.view.render_root_with_app_ui.action_root",
            &mut state.cached_handlers,
            &mut state.cached_action_root,
            render,
        )
    })
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[doc(hidden)]
pub fn view_record_engine_frame<V: View>(
    _app: &mut fret_app::App,
    _window: AppWindowId,
    ui: &mut fret_ui::UiTree<fret_app::App>,
    _state: &mut ViewWindowState<V>,
    _context: &crate::kernel::render::WgpuContext,
    _renderer: &mut crate::kernel::render::Renderer,
    _scale_factor: f32,
    _tick_id: fret_runtime::TickId,
    _frame_id: fret_runtime::FrameId,
) -> fret_launch::EngineFrameUpdate {
    if !ui.view_cache_enabled() {
        ui.set_view_cache_enabled(true);
    }
    fret_launch::EngineFrameUpdate::default()
}
