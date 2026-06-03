use fret_app::{CreateWindowKind, WindowRequest};
use fret_core::time::Instant;
use tracing::error;
use winit::event_loop::ActiveEventLoop;

use super::macos_cursor::dock_tearoff_log;
use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_window_request_effect(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        req: WindowRequest,
        now: Instant,
    ) -> bool {
        match req {
            WindowRequest::Close(window) => self.handle_window_close_request(window, event_loop),
            WindowRequest::Create(create) => {
                if matches!(create.kind, CreateWindowKind::DockFloating { .. }) {
                    dock_tearoff_log(format_args!(
                        "[effect-window-create] kind={:?} anchor={:?}",
                        create.kind, create.anchor
                    ));
                }
                let new_window = match self.create_window_from_request(event_loop, &create) {
                    Ok(id) => id,
                    Err(e) => {
                        error!(error = ?e, "failed to create window from request");
                        return false;
                    }
                };
                self.handle_created_docking_window(&create, new_window, now);

                self.driver
                    .window_created(&mut self.app, &create, new_window);

                self.app.request_redraw(new_window);
                false
            }
            WindowRequest::SetVisible { window, visible } => {
                self.apply_window_visibility_request(window, visible);
                false
            }
            WindowRequest::SetInnerSize { window, size } => {
                self.apply_window_inner_size_request(window, size);
                false
            }
            WindowRequest::SetOuterPosition { window, position } => {
                self.apply_window_outer_position_request(window, position);
                false
            }
            WindowRequest::Raise {
                window,
                sender: sender_id,
            } => {
                self.apply_window_raise_request(window, sender_id, now);
                false
            }
            WindowRequest::BeginDrag { window } => {
                self.begin_window_drag_request(window);
                false
            }
            WindowRequest::BeginResize { window, direction } => {
                self.begin_window_resize_request(window, direction);
                false
            }
            WindowRequest::SetStyle { window, style } => {
                self.apply_window_style_request(window, style);
                false
            }
        }
    }
}
