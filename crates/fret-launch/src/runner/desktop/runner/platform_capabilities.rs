use super::{WinitRunner, WinitRunnerConfig};
use crate::WinitAppDriver;
use fret_runtime::{ExternalDragPayloadKind, ExternalDragPositionQuality, PlatformCapabilities};

#[cfg(any(test, target_os = "linux"))]
fn apply_linux_windowing_capability_posture(
    caps: &mut PlatformCapabilities,
    is_wayland_session: bool,
) {
    // Linux windowing behavior varies significantly across X11/Wayland and compositors. Default
    // to best-effort until we add backend-specific detection.
    caps.ui.window_hover_detection = fret_runtime::WindowHoverDetectionQuality::BestEffort;
    caps.ui.window_set_outer_position = fret_runtime::WindowSetOuterPositionQuality::BestEffort;
    caps.ui.window_z_level = fret_runtime::WindowZLevelQuality::BestEffort;

    // Wayland compositors do not provide a reliable "window under cursor" contract and may ignore
    // programmatic window positioning/z-level hints. Prefer a predictable in-window floating
    // fallback over OS tear-off UX (ADR 0054 / ADR 0083).
    if is_wayland_session {
        caps.ui.window_tear_off = false;
        caps.ui.window_hover_detection = fret_runtime::WindowHoverDetectionQuality::None;
        caps.ui.window_z_level = fret_runtime::WindowZLevelQuality::None;
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn apply_native_clipboard_capability_posture(
    caps: &mut PlatformCapabilities,
    native_clipboard_disabled: bool,
) {
    let text_available = !native_clipboard_disabled;
    caps.clipboard.text.read = text_available;
    caps.clipboard.text.write = text_available;
    caps.clipboard.files = false;
    caps.clipboard.primary_text = cfg!(target_os = "linux") && text_available;
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn backend_platform_capabilities(
        config: &WinitRunnerConfig,
    ) -> PlatformCapabilities {
        Self::backend_platform_capabilities_with_native_clipboard_disabled(
            config,
            fret_platform_native::clipboard::native_clipboard_disabled_by_env(),
        )
    }

    pub(super) fn backend_platform_capabilities_with_native_clipboard_disabled(
        _config: &WinitRunnerConfig,
        native_clipboard_disabled: bool,
    ) -> PlatformCapabilities {
        let mut caps = PlatformCapabilities::default();

        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            caps.exec.background_work = fret_runtime::ExecBackgroundWork::Threads;
            caps.exec.wake = fret_runtime::ExecWake::Reliable;
            caps.exec.timers = fret_runtime::ExecTimers::Reliable;

            caps.ui.multi_window = true;
            caps.ui.window_tear_off = true;
            caps.ui.cursor_icons = true;
            caps.ui.window_decorations = true;
            caps.ui.window_resizable = true;
            caps.ui.window_set_visible = true;
            caps.ui.window_begin_drag = true;
            caps.ui.window_begin_resize = true;
            caps.ui.window_non_activating = true;

            // Best-effort / platform-specific window style facets.
            caps.ui.window_skip_taskbar = cfg!(target_os = "windows");
            caps.ui.window_transparent = cfg!(any(target_os = "windows", target_os = "macos"));
            caps.ui.window_opacity = cfg!(any(target_os = "windows", target_os = "macos"));
            caps.ui.window_hit_test_passthrough_all =
                cfg!(any(target_os = "windows", target_os = "macos"));
            caps.ui.window_hit_test_passthrough_regions = cfg!(target_os = "windows")
                || cfg!(all(target_os = "macos", feature = "macos-hit-test-regions"));

            // Background materials are capability-gated and intentionally conservative by default.
            // Runners should only advertise these once there is an end-to-end implementation
            // (window + compositor + renderer alpha semantics).
            caps.ui.window_background_material_system_default = false;
            caps.ui.window_background_material_mica = false;
            caps.ui.window_background_material_acrylic = false;
            caps.ui.window_background_material_vibrancy = false;

            #[cfg(target_os = "windows")]
            {
                // Windows 11 22H2+ supports `DWMWA_SYSTEMBACKDROP_TYPE` (Mica/Acrylic best-effort).
                // Keep this conservative: only advertise support once we have a stable end-to-end
                // mapping and deterministic clamping + diagnostics.
                if super::win32::supports_dwm_system_backdrop() {
                    caps.ui.window_background_material_system_default = true;
                    caps.ui.window_background_material_mica = true;
                    caps.ui.window_background_material_acrylic = true;
                }
            }

            #[cfg(target_os = "macos")]
            {
                // macOS supports `NSVisualEffectView`-backed vibrancy behind transparent windows.
                // Treat this as best-effort and keep it capability-gated so scripts can degrade
                // deterministically.
                caps.ui.window_background_material_system_default = true;
                caps.ui.window_background_material_vibrancy = true;
            }

            // Non-portable escape hatch remains opt-in and backend-defined.
            caps.ui.native_window_handle = false;

            #[cfg(any(target_os = "windows", target_os = "macos"))]
            {
                caps.ui.window_hover_detection =
                    fret_runtime::WindowHoverDetectionQuality::Reliable;
                caps.ui.window_set_outer_position =
                    fret_runtime::WindowSetOuterPositionQuality::Reliable;
                caps.ui.window_z_level = fret_runtime::WindowZLevelQuality::Reliable;
            }

            #[cfg(target_os = "linux")]
            {
                apply_linux_windowing_capability_posture(
                    &mut caps,
                    super::platform_prefs::linux_is_wayland_session(),
                );
            }

            apply_native_clipboard_capability_posture(&mut caps, native_clipboard_disabled);

            caps.dnd.external = true;
            // The portable external drag contract is token-based (ADR 0053).
            caps.dnd.external_payload = ExternalDragPayloadKind::FileToken;
            caps.dnd.external_position = ExternalDragPositionQuality::Continuous;

            // winit on macOS does not reliably provide continuous drag-over cursor positions for
            // external file drags (see `docs/known-issues.md`).
            #[cfg(target_os = "macos")]
            {
                caps.dnd.external_position = ExternalDragPositionQuality::BestEffort;
            }

            caps.ime.enabled = true;
            caps.ime.set_cursor_area = true;

            caps.fs.real_paths = true;
            caps.fs.file_dialogs = true;

            caps.shell.open_url = true;

            caps.gfx.native_gpu = true;
            caps.gfx.webgpu = false;
        }

        #[cfg(target_arch = "wasm32")]
        {
            caps.exec.background_work = fret_runtime::ExecBackgroundWork::Cooperative;
            caps.exec.wake = fret_runtime::ExecWake::BestEffort;
            caps.exec.timers = fret_runtime::ExecTimers::BestEffort;

            caps.ui.multi_window = false;
            caps.ui.window_tear_off = false;
            caps.ui.cursor_icons = false;
            caps.ui.window_hover_detection = fret_runtime::WindowHoverDetectionQuality::None;
            caps.ui.window_set_outer_position = fret_runtime::WindowSetOuterPositionQuality::None;
            caps.ui.window_z_level = fret_runtime::WindowZLevelQuality::None;
            caps.ui.window_decorations = false;
            caps.ui.window_resizable = false;
            caps.ui.window_transparent = false;
            caps.ui.window_opacity = false;
            caps.ui.window_skip_taskbar = false;
            caps.ui.window_non_activating = false;
            caps.ui.window_hit_test_passthrough_all = false;
            caps.ui.window_hit_test_passthrough_regions = false;
            caps.ui.window_set_visible = false;
            caps.ui.window_begin_drag = false;
            caps.ui.window_begin_resize = false;
            caps.ui.window_background_material_system_default = false;
            caps.ui.window_background_material_mica = false;
            caps.ui.window_background_material_acrylic = false;
            caps.ui.window_background_material_vibrancy = false;
            caps.ui.native_window_handle = false;

            caps.clipboard.text.read = false;
            caps.clipboard.text.write = false;
            caps.clipboard.files = false;
            caps.clipboard.primary_text = false;

            caps.dnd.external = false;
            caps.dnd.external_payload = ExternalDragPayloadKind::None;
            caps.dnd.external_position = ExternalDragPositionQuality::None;

            caps.ime.enabled = true;
            caps.ime.set_cursor_area = false;

            caps.fs.real_paths = false;
            caps.fs.file_dialogs = false;

            caps.shell.open_url = true;

            caps.gfx.native_gpu = false;
            caps.gfx.webgpu = true;
        }

        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            caps.exec.background_work = fret_runtime::ExecBackgroundWork::Threads;
            caps.exec.wake = fret_runtime::ExecWake::Reliable;
            caps.exec.timers = fret_runtime::ExecTimers::Reliable;

            caps.ui.multi_window = false;
            caps.ui.window_tear_off = false;
            caps.ui.cursor_icons = false;
            caps.ui.window_hover_detection = fret_runtime::WindowHoverDetectionQuality::None;
            caps.ui.window_set_outer_position = fret_runtime::WindowSetOuterPositionQuality::None;
            caps.ui.window_z_level = fret_runtime::WindowZLevelQuality::None;
            caps.ui.window_opacity = false;

            caps.clipboard.text.read = false;
            caps.clipboard.text.write = false;
            caps.clipboard.files = false;
            caps.clipboard.primary_text = false;

            caps.dnd.external = false;
            caps.dnd.external_payload = ExternalDragPayloadKind::None;
            caps.dnd.external_position = ExternalDragPositionQuality::None;

            caps.ime.enabled = true;
            caps.ime.set_cursor_area = true;

            caps.fs.real_paths = false;
            caps.fs.file_dialogs = false;

            caps.shell.open_url = false;

            caps.gfx.native_gpu = true;
            caps.gfx.webgpu = false;
        }

        caps
    }

    pub(super) fn effective_platform_capabilities(
        config: &WinitRunnerConfig,
        requested: &PlatformCapabilities,
    ) -> PlatformCapabilities {
        let available = Self::backend_platform_capabilities(config);
        Self::effective_platform_capabilities_from_available(requested, &available)
    }

    pub(super) fn effective_platform_capabilities_from_available(
        requested: &PlatformCapabilities,
        available: &PlatformCapabilities,
    ) -> PlatformCapabilities {
        let mut caps = requested.clone();

        caps.exec.background_work = caps
            .exec
            .background_work
            .clamp_to_available(available.exec.background_work);
        caps.exec.wake = caps.exec.wake.clamp_to_available(available.exec.wake);
        caps.exec.timers = caps.exec.timers.clamp_to_available(available.exec.timers);

        caps.ui.multi_window &= available.ui.multi_window;
        caps.ui.window_tear_off &= available.ui.window_tear_off;
        caps.ui.cursor_icons &= available.ui.cursor_icons;
        caps.ui.window_decorations &= available.ui.window_decorations;
        caps.ui.window_resizable &= available.ui.window_resizable;
        caps.ui.window_transparent &= available.ui.window_transparent;
        caps.ui.window_opacity &= available.ui.window_opacity;
        caps.ui.window_skip_taskbar &= available.ui.window_skip_taskbar;
        caps.ui.window_non_activating &= available.ui.window_non_activating;
        caps.ui.window_hit_test_passthrough_all &= available.ui.window_hit_test_passthrough_all;
        caps.ui.window_hit_test_passthrough_regions &=
            available.ui.window_hit_test_passthrough_regions;
        caps.ui.window_set_visible &= available.ui.window_set_visible;
        caps.ui.window_begin_drag &= available.ui.window_begin_drag;
        caps.ui.window_begin_resize &= available.ui.window_begin_resize;
        caps.ui.window_background_material_system_default &=
            available.ui.window_background_material_system_default;
        caps.ui.window_background_material_mica &= available.ui.window_background_material_mica;
        caps.ui.window_background_material_acrylic &=
            available.ui.window_background_material_acrylic;
        caps.ui.window_background_material_vibrancy &=
            available.ui.window_background_material_vibrancy;
        caps.ui.native_window_handle &= available.ui.native_window_handle;
        caps.ui.window_hover_detection = caps
            .ui
            .window_hover_detection
            .clamp_to_available(available.ui.window_hover_detection);
        caps.ui.window_set_outer_position = caps
            .ui
            .window_set_outer_position
            .clamp_to_available(available.ui.window_set_outer_position);
        caps.ui.window_z_level = caps
            .ui
            .window_z_level
            .clamp_to_available(available.ui.window_z_level);

        caps.clipboard.text.read &= available.clipboard.text.read;
        caps.clipboard.text.write &= available.clipboard.text.write;
        caps.clipboard.files &= available.clipboard.files;
        caps.clipboard.primary_text &= available.clipboard.primary_text;

        caps.dnd.external &= available.dnd.external;
        caps.dnd.external_payload =
            match (caps.dnd.external_payload, available.dnd.external_payload) {
                (ExternalDragPayloadKind::None, _) => ExternalDragPayloadKind::None,
                (_, ExternalDragPayloadKind::None) => ExternalDragPayloadKind::None,
                (requested, available) if requested == available => requested,
                // Narrow to the backend's portable contract if the requested mode isn't supported.
                (_, available) => available,
            };
        caps.dnd.external_position = if caps.dnd.external {
            caps.dnd
                .external_position
                .clamp_to_available(available.dnd.external_position)
        } else {
            ExternalDragPositionQuality::None
        };

        caps.ime.enabled &= available.ime.enabled;
        caps.ime.set_cursor_area &= available.ime.set_cursor_area;

        caps.fs.real_paths &= available.fs.real_paths;
        caps.fs.file_dialogs &= available.fs.file_dialogs;

        caps.shell.open_url &= available.shell.open_url;

        caps.gfx.native_gpu &= available.gfx.native_gpu;
        caps.gfx.webgpu &= available.gfx.webgpu;

        caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WinitEventContext, WinitRenderContext};
    use fret_app::App;
    use fret_core::Event;

    struct TestDriver;
    struct TestWindowState;

    impl WinitAppDriver for TestDriver {
        type WindowState = TestWindowState;

        fn create_window_state(
            &mut self,
            _app: &mut App,
            _window: fret_core::AppWindowId,
        ) -> Self::WindowState {
            TestWindowState
        }

        fn handle_event(
            &mut self,
            _context: WinitEventContext<'_, Self::WindowState>,
            _event: &Event,
        ) {
        }

        fn render(&mut self, _context: WinitRenderContext<'_, Self::WindowState>) {}
    }

    #[test]
    fn linux_windowing_capability_posture_keeps_x11_as_best_effort() {
        let mut caps = PlatformCapabilities::default();
        caps.ui.multi_window = true;
        caps.ui.window_tear_off = true;

        apply_linux_windowing_capability_posture(&mut caps, false);

        assert!(caps.ui.multi_window);
        assert!(caps.ui.window_tear_off);
        assert_eq!(
            caps.ui.window_hover_detection,
            fret_runtime::WindowHoverDetectionQuality::BestEffort
        );
        assert_eq!(
            caps.ui.window_set_outer_position,
            fret_runtime::WindowSetOuterPositionQuality::BestEffort
        );
        assert_eq!(
            caps.ui.window_z_level,
            fret_runtime::WindowZLevelQuality::BestEffort
        );
    }

    #[test]
    fn linux_windowing_capability_posture_disables_tear_off_on_wayland() {
        let mut caps = PlatformCapabilities::default();
        caps.ui.multi_window = true;
        caps.ui.window_tear_off = true;

        apply_linux_windowing_capability_posture(&mut caps, true);

        assert!(caps.ui.multi_window);
        assert!(!caps.ui.window_tear_off);
        assert_eq!(
            caps.ui.window_hover_detection,
            fret_runtime::WindowHoverDetectionQuality::None
        );
        assert_eq!(
            caps.ui.window_set_outer_position,
            fret_runtime::WindowSetOuterPositionQuality::BestEffort
        );
        assert_eq!(
            caps.ui.window_z_level,
            fret_runtime::WindowZLevelQuality::None
        );
    }

    #[test]
    fn native_clipboard_capability_posture_disables_text_files_and_primary_selection() {
        let mut caps = PlatformCapabilities::default();
        caps.clipboard.text.read = true;
        caps.clipboard.text.write = true;
        caps.clipboard.files = true;
        caps.clipboard.primary_text = true;

        apply_native_clipboard_capability_posture(&mut caps, true);

        assert!(!caps.clipboard.text.read);
        assert!(!caps.clipboard.text.write);
        assert!(!caps.clipboard.files);
        assert!(!caps.clipboard.primary_text);
    }

    #[test]
    fn native_clipboard_capability_posture_advertises_linux_primary_selection_when_enabled() {
        let mut caps = PlatformCapabilities::default();
        caps.clipboard.text.read = false;
        caps.clipboard.text.write = false;
        caps.clipboard.files = true;
        caps.clipboard.primary_text = false;

        apply_native_clipboard_capability_posture(&mut caps, false);

        assert!(caps.clipboard.text.read);
        assert!(caps.clipboard.text.write);
        assert!(!caps.clipboard.files);
        assert_eq!(caps.clipboard.primary_text, cfg!(target_os = "linux"));
    }

    #[test]
    fn backend_platform_capabilities_honor_native_clipboard_disabled() {
        let caps =
            WinitRunner::<TestDriver>::backend_platform_capabilities_with_native_clipboard_disabled(
                &WinitRunnerConfig::default(),
                true,
            );

        assert!(!caps.clipboard.text.read);
        assert!(!caps.clipboard.text.write);
        assert!(!caps.clipboard.files);
        assert!(!caps.clipboard.primary_text);
    }

    #[test]
    fn effective_platform_capabilities_clamp_primary_selection_to_backend() {
        let mut requested = PlatformCapabilities::default();
        requested.clipboard.text.read = true;
        requested.clipboard.text.write = true;
        requested.clipboard.primary_text = true;

        let mut available = PlatformCapabilities::default();
        available.clipboard.text.read = true;
        available.clipboard.text.write = true;
        available.clipboard.primary_text = false;

        let caps = WinitRunner::<TestDriver>::effective_platform_capabilities_from_available(
            &requested, &available,
        );

        assert!(caps.clipboard.text.read);
        assert!(caps.clipboard.text.write);
        assert!(!caps.clipboard.primary_text);
    }
}
