use std::collections::HashSet;

use crate::runner::streaming_upload::StreamingUploadStats;
use fret_app::Effect;
use fret_core::AppWindowId;
use fret_core::time::Instant;
use winit::event_loop::ActiveEventLoop;

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn dispatch_effect_queue(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        effects: Vec<Effect>,
        stats: &mut StreamingUploadStats,
        window_state_dirty: &mut HashSet<AppWindowId>,
        now: Instant,
    ) -> bool {
        for effect in effects {
            match effect {
                Effect::Redraw(window) => {
                    self.handle_effect_redraw(window);
                }
                Effect::ImeAllow { window, enabled } => {
                    self.handle_ime_allow(window, enabled, window_state_dirty);
                }
                Effect::ImeRequestVirtualKeyboard { window, visible } => {
                    self.handle_ime_request_virtual_keyboard(window, visible);
                }
                Effect::ImeSetCursorArea { window, rect } => {
                    self.handle_ime_set_cursor_area(window, rect, window_state_dirty);
                }
                Effect::WindowMetricsSetInsets {
                    window,
                    safe_area_insets,
                    occlusion_insets,
                } => {
                    self.apply_window_metrics_insets_request(
                        window,
                        safe_area_insets,
                        occlusion_insets,
                    );
                }
                Effect::WindowMetricsSetPreferences {
                    window,
                    color_scheme,
                    prefers_reduced_motion,
                    text_scale_factor,
                } => {
                    self.apply_window_metrics_preferences_request(
                        window,
                        color_scheme,
                        prefers_reduced_motion,
                        text_scale_factor,
                    );
                }
                Effect::CursorSetIcon { window, icon } => {
                    self.handle_cursor_set_icon(window, icon, window_state_dirty);
                }
                Effect::RequestAnimationFrame(window) => {
                    self.handle_request_animation_frame(window);
                }
                Effect::DiagInjectEvent { window, event } => {
                    self.handle_diag_inject_event(window, event);
                }
                Effect::SetTimer { .. } => {
                    self.handle_set_timer_effect(now, &effect);
                }
                Effect::CancelTimer { token } => {
                    self.handle_cancel_timer_effect(token);
                }
                Effect::QuitApp => {
                    if self.handle_quit_app_effect(event_loop) {
                        return true;
                    }
                }
                Effect::ShowAboutPanel => {
                    self.handle_show_about_panel();
                }
                Effect::HideApp => {
                    self.handle_hide_app();
                }
                Effect::HideOtherApps => {
                    self.handle_hide_other_apps();
                }
                Effect::UnhideAllApps => {
                    self.handle_unhide_all_apps();
                }
                Effect::Command { window, command } => {
                    self.handle_command_effect(window, command);
                }
                Effect::SetMenuBar { window, menu_bar } => {
                    self.handle_set_menu_bar_effect(window, menu_bar);
                }
                Effect::DiagClipboardForceUnavailable { window, enabled } => {
                    self.apply_diag_clipboard_force_unavailable(window, enabled);
                }
                Effect::DiagIncomingOpenInject { window, items } => {
                    self.handle_diag_incoming_open_inject(window, items);
                }
                Effect::ClipboardWriteText {
                    window,
                    token,
                    text,
                } => {
                    self.handle_clipboard_write_text(window, token, text);
                }
                Effect::ClipboardReadText { window, token } => {
                    self.handle_clipboard_read_text(window, token);
                }
                Effect::PrimarySelectionSetText { text } => {
                    self.handle_primary_selection_set_text(text);
                }
                Effect::PrimarySelectionGetText { window, token } => {
                    self.handle_primary_selection_get_text(window, token);
                }
                Effect::ExternalDropReadAll { window, token } => {
                    self.handle_external_drop_read_all(window, token);
                }
                Effect::ExternalDropReadAllWithLimits {
                    window,
                    token,
                    limits,
                } => {
                    self.handle_external_drop_read_all_with_limits(window, token, limits);
                }
                Effect::ExternalDropRelease { token } => {
                    self.handle_external_drop_release(token);
                }
                Effect::OpenUrl { url, .. } => {
                    self.handle_open_url(url);
                }
                Effect::ShareSheetShow {
                    window,
                    token,
                    items,
                } => {
                    self.handle_share_sheet_show(window, token, items);
                }
                Effect::FileDialogOpen { window, options } => {
                    self.handle_file_dialog_open(window, options);
                }
                Effect::FileDialogReadAll { window, token } => {
                    self.handle_file_dialog_read_all(window, token);
                }
                Effect::FileDialogReadAllWithLimits {
                    window,
                    token,
                    limits,
                } => {
                    self.handle_file_dialog_read_all_with_limits(window, token, limits);
                }
                Effect::FileDialogRelease { token } => {
                    self.handle_file_dialog_release(token);
                }
                Effect::IncomingOpenReadAll { window, token } => {
                    self.handle_incoming_open_read_all(window, token);
                }
                Effect::IncomingOpenReadAllWithLimits {
                    window,
                    token,
                    limits,
                } => {
                    self.handle_incoming_open_read_all_with_limits(window, token, limits);
                }
                Effect::IncomingOpenRelease { token } => {
                    self.handle_incoming_open_release(token);
                }
                Effect::TextAddFontAssets { requests } => {
                    self.handle_text_add_font_assets(requests);
                }
                Effect::TextRescanSystemFonts => {
                    self.handle_text_rescan_system_fonts();
                }
                Effect::ImageRegisterRgba8 {
                    window,
                    token,
                    width,
                    height,
                    bytes,
                    color_info,
                    alpha_mode,
                } => {
                    self.handle_image_register_rgba8(
                        window, token, width, height, bytes, color_info, alpha_mode,
                    );
                }
                Effect::ImageUpdateRgba8 {
                    window,
                    token,
                    image,
                    stream_generation,
                    width,
                    height,
                    update_rect_px,
                    bytes_per_row,
                    bytes,
                    color_info,
                    alpha_mode,
                } => {
                    self.handle_image_update_rgba8(
                        stats,
                        window,
                        token,
                        image,
                        stream_generation,
                        width,
                        height,
                        update_rect_px,
                        bytes_per_row,
                        bytes,
                        color_info,
                        alpha_mode,
                    );
                }
                Effect::ImageUpdateNv12 {
                    window,
                    token,
                    image,
                    stream_generation,
                    width,
                    height,
                    update_rect_px,
                    y_bytes_per_row,
                    y_plane,
                    uv_bytes_per_row,
                    uv_plane,
                    color_info,
                    alpha_mode: _,
                } => {
                    self.handle_image_update_nv12(
                        stats,
                        window,
                        token,
                        image,
                        stream_generation,
                        width,
                        height,
                        update_rect_px,
                        y_bytes_per_row,
                        y_plane,
                        uv_bytes_per_row,
                        uv_plane,
                        color_info,
                    );
                }
                Effect::ImageUpdateI420 {
                    window,
                    token,
                    image,
                    stream_generation,
                    width,
                    height,
                    update_rect_px,
                    y_bytes_per_row,
                    y_plane,
                    u_bytes_per_row,
                    u_plane,
                    v_bytes_per_row,
                    v_plane,
                    color_info,
                    alpha_mode: _,
                } => {
                    self.handle_image_update_i420(
                        stats,
                        window,
                        token,
                        image,
                        stream_generation,
                        width,
                        height,
                        update_rect_px,
                        y_bytes_per_row,
                        y_plane,
                        u_bytes_per_row,
                        u_plane,
                        v_bytes_per_row,
                        v_plane,
                        color_info,
                    );
                }
                Effect::ImageUnregister { image } => {
                    self.handle_image_unregister(image);
                }
                Effect::ViewportInput(event) => {
                    self.handle_viewport_input_effect(event);
                }
                Effect::Dock(op) => {
                    self.handle_dock_effect(op);
                }
                Effect::Window(req) => {
                    if self.handle_window_request_effect(event_loop, req, now) {
                        return true;
                    }
                }
            }
        }

        false
    }
}
