use std::{sync::Arc, time::Duration};

use fret_core::time::Instant;
use fret_core::{Point, Scene};
use fret_render::SurfaceState;
use winit::{dpi::PhysicalPosition, window::Window};

pub(super) struct WindowRuntime<S> {
    pub(super) window: Arc<dyn Window>,
    pub(super) accessibility: Option<fret_runner_winit::accessibility::WinitAccessibility>,
    /// Cached semantics snapshot for the window, typically produced by the accessibility pass and
    /// reused by later platform hooks or accessibility actions in the same runner flow.
    pub(super) last_semantics_snapshot: Option<std::sync::Arc<fret_core::SemanticsSnapshot>>,
    pub(super) surface: Option<SurfaceState<'static>>,
    pub(super) scene: Scene,
    pub(super) platform: fret_runner_winit::WinitPlatform,
    /// Coalesced wheel delta awaiting delivery at the next frame boundary.
    ///
    /// When enabled via `FRET_WINIT_COALESCE_WHEEL=1`, we buffer wheel deltas and deliver at most
    /// one `PointerEvent::Wheel` per frame, applying a per-axis max-abs cap and carrying any
    /// remainder over to subsequent frames.
    pub(super) pending_wheel: Option<PendingWheelEvent>,
    #[cfg(target_os = "android")]
    pub(super) android_bottom_inset_baseline: Option<fret_core::Px>,
    /// Coalesced resizes awaiting metrics delivery at the next frame boundary.
    ///
    /// During interactive window resize, platforms may emit multiple size updates per vblank.
    /// We keep only the latest physical size so `Event::WindowResized` /
    /// `Event::WindowScaleFactorChanged` are still delivered once per `RedrawRequested`, while
    /// the underlying GPU surface can already be reconfigured at event time.
    pub(super) pending_surface_resize: Option<winit::dpi::PhysicalSize<u32>>,
    /// Last delivered (quantized) logical size for `Event::WindowResized`.
    ///
    /// This mirrors GPUI's `set_frame_size` guard (`old_size == new_size`) and helps reduce
    /// float-noise churn in window-metrics consumers during interactive resize.
    pub(super) last_delivered_window_resized: Option<(u32, u32)>,
    pub(super) is_focused: bool,
    pub(super) external_drag_files: Vec<std::path::PathBuf>,
    pub(super) external_drag_token: Option<fret_runtime::ExternalDropToken>,
    pub(super) user: S,
    #[cfg(windows)]
    pub(super) os_menu: Option<super::windows_menu::WindowsMenuBar>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PendingWheelEvent {
    pub(super) pointer_id: fret_core::PointerId,
    pub(super) position: Point,
    pub(super) delta: Point,
    pub(super) modifiers: fret_core::Modifiers,
    pub(super) pointer_type: fret_core::PointerType,
}

#[derive(Debug, Clone)]
pub(super) struct PendingFrontRequest {
    pub(super) source_window: Option<fret_core::AppWindowId>,
    pub(super) panel: Option<fret_core::PanelKey>,
    pub(super) created_at: Instant,
    pub(super) next_attempt_at: Instant,
    pub(super) attempts_left: u8,
}

#[derive(Debug, Clone)]
pub(super) struct TimerEntry {
    pub(super) window: Option<fret_core::AppWindowId>,
    pub(super) deadline: Instant,
    pub(super) repeat: Option<Duration>,
    pub(super) last_fired_tick: Option<fret_runtime::TickId>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DockTearoffFollow {
    pub(super) window: fret_core::AppWindowId,
    pub(super) source_window: fret_core::AppWindowId,
    pub(super) grab_offset: Point,
    pub(super) manual_follow: bool,
    pub(super) last_outer_pos: Option<PhysicalPosition<i32>>,
    pub(super) transparent_payload_applied: bool,
    pub(super) hit_test_passthrough_all_applied: bool,
    pub(super) always_on_top_applied: bool,
}
