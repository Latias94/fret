use std::fmt;

#[cfg(target_os = "macos")]
use winit::dpi::PhysicalPosition;

pub(super) fn macos_window_log(_args: fmt::Arguments<'_>) {
    #[cfg(target_os = "macos")]
    {
        use std::{
            io::Write,
            sync::{Mutex, OnceLock},
        };

        if std::env::var_os("FRET_MACOS_WINDOW_LOG").is_none() {
            return;
        }

        static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

        let file = LOG_FILE.get_or_init(|| {
            let _ = std::fs::create_dir_all("target");
            let path = std::path::Path::new("target").join("fret-macos-window.log");
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)
                .expect("open fret-macos-window.log");
            let _ = writeln!(
                file,
                "[session] pid={} time={:?}",
                std::process::id(),
                std::time::SystemTime::now()
            );
            Mutex::new(file)
        });

        let Ok(mut file) = file.lock() else {
            return;
        };

        let _ = writeln!(file, "{}", _args);
    }
}

#[cfg(target_os = "macos")]
pub(super) fn macos_dockfloating_parenting_enabled() -> bool {
    use std::sync::OnceLock;

    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("FRET_MACOS_DOCKFLOAT_PARENT").is_some_and(|v| !v.is_empty())
    })
}

pub(super) fn dock_tearoff_log(_args: fmt::Arguments<'_>) {
    #[cfg(target_os = "macos")]
    {
        use std::{
            io::Write,
            sync::{Mutex, OnceLock},
        };

        if std::env::var_os("FRET_DOCK_TEAROFF_LOG").is_none() {
            return;
        }

        static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

        let file = LOG_FILE.get_or_init(|| {
            let _ = std::fs::create_dir_all("target");
            let path = std::path::Path::new("target").join("fret-dock-tearoff.log");
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)
                .expect("open fret-dock-tearoff.log");
            let _ = writeln!(
                file,
                "[session] pid={} time={:?}",
                std::process::id(),
                std::time::SystemTime::now()
            );
            Mutex::new(file)
        });

        let Ok(mut file) = file.lock() else {
            return;
        };

        let _ = writeln!(file, "{}", _args);
    }
}

#[cfg(target_os = "macos")]
pub(super) fn macos_cursor_trace_enabled() -> bool {
    use std::sync::OnceLock;

    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED
        .get_or_init(|| std::env::var_os("FRET_MACOS_CURSOR_TRACE").is_some_and(|v| !v.is_empty()))
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub(super) fn macos_is_left_mouse_down() -> bool {
    use objc::runtime::Class;
    use objc::{msg_send, sel, sel_impl};
    unsafe {
        let Some(class) = Class::get("NSEvent") else {
            return false;
        };
        let buttons: u64 = msg_send![class, pressedMouseButtons];
        (buttons & 1) != 0
    }
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub(super) fn macos_mouse_location() -> Option<cocoa::foundation::NSPoint> {
    use cocoa::foundation::NSPoint;
    use objc::runtime::Class;
    use objc::{msg_send, sel, sel_impl};
    unsafe {
        let Some(class) = Class::get("NSEvent") else {
            return None;
        };
        let point: NSPoint = msg_send![class, mouseLocation];
        Some(point)
    }
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct MacCursorTransform {
    scale_factor: f64,
    x_offset: f64,
    y_offset: f64,
    y_flipped: Option<bool>,
    last_winit_y: Option<f64>,
    last_cocoa_y: Option<f64>,
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct MacCursorScreenKey {
    origin_x: i32,
    origin_y: i32,
    width: i32,
    height: i32,
    scale_milli: i32,
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
impl MacCursorScreenKey {
    fn from_frame(frame: cocoa::foundation::NSRect, scale_factor: f64) -> Self {
        Self {
            origin_x: frame.origin.x.round() as i32,
            origin_y: frame.origin.y.round() as i32,
            width: frame.size.width.round() as i32,
            height: frame.size.height.round() as i32,
            scale_milli: (scale_factor * 1000.0).round() as i32,
        }
    }

    fn unknown(scale_factor: f64) -> Self {
        Self {
            origin_x: 0,
            origin_y: 0,
            width: 0,
            height: 0,
            scale_milli: (scale_factor * 1000.0).round() as i32,
        }
    }
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn macos_screen_key_for_point(point: cocoa::foundation::NSPoint) -> Option<MacCursorScreenKey> {
    use cocoa::base::id;
    use cocoa::foundation::NSRect;
    use objc::runtime::Class;
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let Some(class) = Class::get("NSScreen") else {
            return None;
        };
        let screens: id = msg_send![class, screens];
        if screens.is_null() {
            return None;
        }
        let count: usize = msg_send![screens, count];
        for idx in 0..count {
            let screen: id = msg_send![screens, objectAtIndex: idx];
            if screen.is_null() {
                continue;
            }
            let frame: NSRect = msg_send![screen, frame];
            let min_x = frame.origin.x;
            let min_y = frame.origin.y;
            let max_x = min_x + frame.size.width;
            let max_y = min_y + frame.size.height;
            if point.x >= min_x && point.x < max_x && point.y >= min_y && point.y < max_y {
                let scale_factor: f64 = msg_send![screen, backingScaleFactor];
                return Some(MacCursorScreenKey::from_frame(frame, scale_factor));
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
impl MacCursorTransform {
    fn update_from_sample(
        &mut self,
        winit_screen_pos: PhysicalPosition<f64>,
        cocoa_mouse_location: cocoa::foundation::NSPoint,
        scale_factor: f64,
    ) {
        let cocoa_x = cocoa_mouse_location.x * scale_factor;
        let cocoa_y = cocoa_mouse_location.y * scale_factor;

        if self.y_flipped.is_none()
            && let (Some(prev_winit_y), Some(prev_cocoa_y)) = (self.last_winit_y, self.last_cocoa_y)
        {
            let dy_winit = winit_screen_pos.y - prev_winit_y;
            let dy_cocoa = cocoa_y - prev_cocoa_y;
            if dy_winit.abs() > 0.5 && dy_cocoa.abs() > 0.5 {
                self.y_flipped = Some(dy_winit * dy_cocoa < 0.0);
            }
        }

        self.last_winit_y = Some(winit_screen_pos.y);
        self.last_cocoa_y = Some(cocoa_y);

        self.scale_factor = scale_factor;
        self.x_offset = winit_screen_pos.x - cocoa_x;

        let y_flipped = self.y_flipped.unwrap_or(true);
        self.y_offset = if y_flipped {
            winit_screen_pos.y + cocoa_y
        } else {
            winit_screen_pos.y - cocoa_y
        };

        if macos_cursor_trace_enabled() {
            dock_tearoff_log(format_args!(
                "[cursor-calibrate] winit=({:.1},{:.1}) cocoa=({:.1},{:.1}) scale={:.3} flipped={:?} x_off={:.1} y_off={:.1}",
                winit_screen_pos.x,
                winit_screen_pos.y,
                cocoa_x,
                cocoa_y,
                self.scale_factor,
                self.y_flipped,
                self.x_offset,
                self.y_offset,
            ));
        }
    }

    fn map(&self, cocoa_mouse_location: cocoa::foundation::NSPoint) -> PhysicalPosition<f64> {
        let cocoa_x = cocoa_mouse_location.x * self.scale_factor;
        let cocoa_y = cocoa_mouse_location.y * self.scale_factor;
        let x = cocoa_x + self.x_offset;
        let y = if self.y_flipped.unwrap_or(true) {
            self.y_offset - cocoa_y
        } else {
            cocoa_y + self.y_offset
        };
        let out = PhysicalPosition::new(x, y);
        if macos_cursor_trace_enabled() {
            dock_tearoff_log(format_args!(
                "[cursor-map] cocoa=({:.1},{:.1}) scale={:.3} flipped={:?} out=({:.1},{:.1})",
                cocoa_x, cocoa_y, self.scale_factor, self.y_flipped, out.x, out.y
            ));
        }
        out
    }
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
#[derive(Default)]
pub(super) struct MacCursorTransformTable {
    by_screen: HashMap<MacCursorScreenKey, MacCursorTransform>,
    last_used: Option<MacCursorScreenKey>,
}

impl<D: super::WinitAppDriver> super::WinitRunner<D> {
    #[cfg(target_os = "macos")]
    pub(super) fn macos_bootstrap_cursor_transform_from_active_drag(&mut self) -> bool {
        let Some(pointer_id) = self.dock_drag_pointer_id() else {
            return false;
        };
        let Some(drag) = self.app.drag(pointer_id) else {
            return false;
        };
        let window = drag.current_window;
        let Some(screen_pos) = self.cursor_screen_pos_fallback_for_window(window) else {
            return false;
        };
        let scale_factor = self
            .windows
            .get(window)
            .map(|s| s.window.scale_factor())
            .unwrap_or(1.0);
        self.macos_calibrate_cursor_transform_from_window_sample(screen_pos, scale_factor);
        true
    }

    #[cfg(target_os = "macos")]
    pub(super) fn macos_refresh_cursor_screen_pos_for_dock_drag(&mut self) {
        if self.dock_drag_pointer_id().is_none() && self.dock_tearoff_follow.is_none() {
            return;
        }
        if self.macos_refresh_cursor_screen_pos_from_nsevent() {
            return;
        }
        if self.macos_bootstrap_cursor_transform_from_active_drag() {
            let _ = self.macos_refresh_cursor_screen_pos_from_nsevent();
        }
    }

    #[cfg(target_os = "macos")]
    fn macos_calibrate_cursor_transform_from_window_sample(
        &mut self,
        winit_screen_pos: PhysicalPosition<f64>,
        scale_factor: f64,
    ) {
        let Some(cocoa_pos) = macos_mouse_location() else {
            return;
        };
        self.macos_cursor_transform.update_from_window_sample(
            winit_screen_pos,
            cocoa_pos,
            scale_factor,
        );
    }

    #[cfg(target_os = "macos")]
    fn macos_refresh_cursor_screen_pos_from_nsevent(&mut self) -> bool {
        let Some(cocoa_pos) = macos_mouse_location() else {
            return false;
        };
        let Some(mapped) = self.macos_cursor_transform.map(cocoa_pos) else {
            return false;
        };
        self.cursor_screen_pos = Some(mapped);
        true
    }
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
impl MacCursorTransformTable {
    fn update_from_window_sample(
        &mut self,
        winit_screen_pos: PhysicalPosition<f64>,
        cocoa_pos: cocoa::foundation::NSPoint,
        scale_factor: f64,
    ) {
        let key = macos_screen_key_for_point(cocoa_pos).unwrap_or_else(|| {
            // If we can't resolve the screen (AppKit oddities), still store a transform so we can
            // map `NSEvent::mouseLocation` during cross-window drags without integrating deltas.
            MacCursorScreenKey::unknown(scale_factor)
        });
        let transform = self
            .by_screen
            .entry(key)
            .or_insert_with(MacCursorTransform::default);
        transform.update_from_sample(winit_screen_pos, cocoa_pos, scale_factor);
        self.last_used = Some(key);
    }

    fn map_with_key_hint(
        &mut self,
        cocoa_pos: cocoa::foundation::NSPoint,
        key_hint: Option<MacCursorScreenKey>,
    ) -> Option<PhysicalPosition<f64>> {
        let hint_hit = key_hint.is_some_and(|k| self.by_screen.contains_key(&k));
        let last_hit = self
            .last_used
            .is_some_and(|k| self.by_screen.contains_key(&k));
        let selection = if hint_hit {
            "key"
        } else if last_hit {
            "last"
        } else {
            "any"
        };

        let transform = key_hint
            .and_then(|k| self.by_screen.get(&k).copied())
            .or_else(|| self.last_used.and_then(|k| self.by_screen.get(&k).copied()))
            .or_else(|| self.by_screen.values().next().copied())?;

        let out = transform.map(cocoa_pos);

        if macos_cursor_trace_enabled() {
            dock_tearoff_log(format_args!(
                "[cursor-refresh] cocoa=({:.1},{:.1}) selection={} key={:?} last={:?} transforms={}",
                cocoa_pos.x,
                cocoa_pos.y,
                selection,
                key_hint,
                self.last_used,
                self.by_screen.len(),
            ));
        }

        if let Some(key) = key_hint {
            self.last_used = Some(key);
        }

        Some(out)
    }

    fn map(&mut self, cocoa_pos: cocoa::foundation::NSPoint) -> Option<PhysicalPosition<f64>> {
        self.map_with_key_hint(cocoa_pos, macos_screen_key_for_point(cocoa_pos))
    }
}
