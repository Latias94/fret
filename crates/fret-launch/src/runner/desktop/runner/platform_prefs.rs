use super::*;
use fret_core::{ColorScheme, ContrastPreference, ForcedColorsMode};
use winit::window::Window;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct DesktopEnvironmentSnapshot {
    pub(super) color_scheme: Option<ColorScheme>,
    pub(super) prefers_reduced_motion: Option<bool>,
    pub(super) text_scale_factor: Option<f32>,
    pub(super) prefers_reduced_transparency: Option<bool>,
    pub(super) accent_color: Option<fret_core::Color>,
    pub(super) contrast_preference: Option<ContrastPreference>,
    pub(super) forced_colors_mode: Option<ForcedColorsMode>,
}

#[cfg(target_os = "linux")]
mod linux_portal_settings {
    use fret_core::time::{Duration, Instant};
    use std::sync::{Mutex, OnceLock};

    use zbus::blocking::{Connection, Proxy};
    use zbus::zvariant::OwnedValue;

    const SETTINGS_SERVICE: &str = "org.freedesktop.portal.Desktop";
    const SETTINGS_PATH: &str = "/org/freedesktop/portal/desktop";
    const SETTINGS_INTERFACE: &str = "org.freedesktop.portal.Settings";

    pub const APPEARANCE_NAMESPACE: &str = "org.freedesktop.appearance";

    struct PortalCache {
        connection: Option<Connection>,
        next_retry_at: Instant,
    }

    static CACHE: OnceLock<Mutex<PortalCache>> = OnceLock::new();

    fn with_settings_proxy<T>(f: impl FnOnce(&Proxy<'_>) -> zbus::Result<T>) -> Option<T> {
        let cache_lock = CACHE.get_or_init(|| {
            Mutex::new(PortalCache {
                connection: None,
                next_retry_at: Instant::now(),
            })
        });

        let mut cache = cache_lock.lock().ok()?;
        let now = Instant::now();

        if cache.connection.is_none() && now < cache.next_retry_at {
            return None;
        }

        if cache.connection.is_none() {
            match Connection::session() {
                Ok(connection) => cache.connection = Some(connection),
                Err(_) => {
                    cache.next_retry_at = now + Duration::from_secs(5);
                    return None;
                }
            }
        }

        let connection = cache.connection.as_ref()?;
        drop(cache);

        let proxy = Proxy::new(
            connection,
            SETTINGS_SERVICE,
            SETTINGS_PATH,
            SETTINGS_INTERFACE,
        )
        .ok()?;
        f(&proxy).ok()
    }

    fn read_owned_value(namespace: &str, key: &str) -> Option<OwnedValue> {
        with_settings_proxy(|proxy| proxy.call("Read", &(namespace, key)))
    }

    pub fn read_u32(namespace: &str, key: &str) -> Option<u32> {
        let value = read_owned_value(namespace, key)?;

        if let Ok(v) = u32::try_from(&value) {
            return Some(v);
        }
        if let Ok(v) = i32::try_from(&value) {
            return u32::try_from(v).ok();
        }
        if let Ok(v) = bool::try_from(&value) {
            return Some(if v { 1 } else { 0 });
        }

        None
    }

    pub fn read_f64(namespace: &str, key: &str) -> Option<f64> {
        let value = read_owned_value(namespace, key)?;

        if let Ok(v) = f64::try_from(&value) {
            return Some(v);
        }
        if let Ok(v) = f32::try_from(&value) {
            return Some(v as f64);
        }
        if let Ok(v) = u32::try_from(&value) {
            return Some(v as f64);
        }
        if let Ok(v) = i32::try_from(&value) {
            return Some(v as f64);
        }

        None
    }

    pub fn read_bool(namespace: &str, key: &str) -> Option<bool> {
        let value = read_owned_value(namespace, key)?;

        if let Ok(v) = bool::try_from(&value) {
            return Some(v);
        }
        if let Ok(v) = u32::try_from(&value) {
            return Some(v != 0);
        }
        if let Ok(v) = i32::try_from(&value) {
            return Some(v != 0);
        }

        None
    }
}

#[cfg(target_os = "linux")]
pub(super) static LINUX_PORTAL_ENV_DIRTY: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub(super) fn read_desktop_environment_snapshot(window: &dyn Window) -> DesktopEnvironmentSnapshot {
    DesktopEnvironmentSnapshot {
        color_scheme: read_desktop_color_scheme(window),
        prefers_reduced_motion: read_desktop_prefers_reduced_motion(),
        text_scale_factor: read_desktop_text_scale_factor(),
        prefers_reduced_transparency: read_desktop_prefers_reduced_transparency(),
        accent_color: read_desktop_accent_color(),
        contrast_preference: read_desktop_contrast_preference(),
        forced_colors_mode: read_desktop_forced_colors_mode(),
    }
}

fn read_desktop_color_scheme(window: &dyn Window) -> Option<ColorScheme> {
    let from_window = window.theme().map(|theme| match theme {
        winit::window::Theme::Light => ColorScheme::Light,
        winit::window::Theme::Dark => ColorScheme::Dark,
    });

    #[cfg(target_os = "linux")]
    {
        from_window.or_else(read_linux_portal_color_scheme)
    }

    #[cfg(not(target_os = "linux"))]
    {
        from_window
    }
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn update_window_environment_for_window_ref(
        &mut self,
        window: fret_core::AppWindowId,
        winit_window: &dyn Window,
    ) -> bool {
        let snapshot = read_desktop_environment_snapshot(winit_window);
        let metrics = self.app.global::<WindowMetricsService>();

        let needs_scheme = !metrics.is_some_and(|svc| svc.color_scheme_is_known(window))
            || metrics.and_then(|svc| svc.color_scheme(window)) != snapshot.color_scheme;
        let needs_motion = !metrics.is_some_and(|svc| svc.prefers_reduced_motion_is_known(window))
            || metrics.and_then(|svc| svc.prefers_reduced_motion(window))
                != snapshot.prefers_reduced_motion;
        let needs_text_scale = !metrics.is_some_and(|svc| svc.text_scale_factor_is_known(window))
            || metrics.and_then(|svc| svc.text_scale_factor(window)) != snapshot.text_scale_factor;
        let needs_transparency = !metrics
            .is_some_and(|svc| svc.prefers_reduced_transparency_is_known(window))
            || metrics.and_then(|svc| svc.prefers_reduced_transparency(window))
                != snapshot.prefers_reduced_transparency;
        let needs_accent = !metrics.is_some_and(|svc| svc.accent_color_is_known(window))
            || metrics.and_then(|svc| svc.accent_color(window)) != snapshot.accent_color;
        let needs_contrast = !metrics.is_some_and(|svc| svc.contrast_preference_is_known(window))
            || metrics.and_then(|svc| svc.contrast_preference(window))
                != snapshot.contrast_preference;
        let needs_forced = !metrics.is_some_and(|svc| svc.forced_colors_mode_is_known(window))
            || metrics.and_then(|svc| svc.forced_colors_mode(window))
                != snapshot.forced_colors_mode;

        if !(needs_scheme
            || needs_motion
            || needs_text_scale
            || needs_transparency
            || needs_accent
            || needs_contrast
            || needs_forced)
        {
            return false;
        }

        self.app
            .with_global_mut(WindowMetricsService::default, |svc, _app| {
                if needs_scheme {
                    svc.set_color_scheme(window, snapshot.color_scheme);
                }
                if needs_motion {
                    svc.set_prefers_reduced_motion(window, snapshot.prefers_reduced_motion);
                }
                if needs_text_scale {
                    svc.set_text_scale_factor(window, snapshot.text_scale_factor);
                }
                if needs_transparency {
                    svc.set_prefers_reduced_transparency(
                        window,
                        snapshot.prefers_reduced_transparency,
                    );
                }
                if needs_accent {
                    svc.set_accent_color(window, snapshot.accent_color);
                }
                if needs_contrast {
                    svc.set_contrast_preference(window, snapshot.contrast_preference);
                }
                if needs_forced {
                    svc.set_forced_colors_mode(window, snapshot.forced_colors_mode);
                }
            });

        true
    }

    pub(super) fn poll_window_environment_if_due(&mut self, now: Instant) {
        #[cfg(target_os = "linux")]
        let linux_dirty = LINUX_PORTAL_ENV_DIRTY.swap(false, std::sync::atomic::Ordering::SeqCst);
        #[cfg(not(target_os = "linux"))]
        let linux_dirty = false;

        if now < self.next_environment_poll_at && !linux_dirty {
            return;
        }
        self.next_environment_poll_at = now + Duration::from_millis(500);

        let windows: Vec<(fret_core::AppWindowId, Arc<dyn Window>)> = self
            .windows
            .iter()
            .map(|(id, state)| (id, state.window.clone()))
            .collect();
        for (id, window_ref) in windows {
            if self.update_window_environment_for_window_ref(id, window_ref.as_ref()) {
                self.app.request_redraw(id);
            }
        }
    }

    #[cfg(target_os = "linux")]
    pub(super) fn maybe_start_linux_portal_settings_listener(&mut self, waker: EventLoopProxy) {
        if self.linux_portal_settings_listener_started {
            return;
        }
        self.linux_portal_settings_listener_started = true;

        std::thread::spawn(move || {
            use zbus::blocking::{Connection, Proxy};

            const SETTINGS_SERVICE: &str = "org.freedesktop.portal.Desktop";
            const SETTINGS_PATH: &str = "/org/freedesktop/portal/desktop";
            const SETTINGS_INTERFACE: &str = "org.freedesktop.portal.Settings";

            let Ok(connection) = Connection::session() else {
                return;
            };
            let Ok(proxy) = Proxy::new(
                &connection,
                SETTINGS_SERVICE,
                SETTINGS_PATH,
                SETTINGS_INTERFACE,
            ) else {
                return;
            };
            let Ok(signals) = proxy.receive_signal("SettingChanged") else {
                return;
            };

            for msg in signals {
                let Ok((namespace, key, _value)) =
                    msg.body()
                        .deserialize::<(String, String, zbus::zvariant::OwnedValue)>()
                else {
                    continue;
                };
                if namespace != linux_portal_settings::APPEARANCE_NAMESPACE {
                    continue;
                }
                if !matches!(
                    key.as_str(),
                    "color-scheme" | "contrast" | "reduce-motion" | "reduced-motion"
                ) {
                    continue;
                }
                if LINUX_PORTAL_ENV_DIRTY.swap(true, std::sync::atomic::Ordering::SeqCst) {
                    continue;
                }
                waker.wake_up();
            }
        });
    }
}

#[cfg(target_os = "linux")]
fn read_linux_portal_color_scheme() -> Option<ColorScheme> {
    // Best-effort fallback for compositors/toolkits where `winit::window::Window::theme()` is
    // unavailable (`None`). The portal uses an enum-like integer value.
    //
    // - 0: no preference / unknown
    // - 1: prefer dark
    // - 2: prefer light
    let value = linux_portal_settings::read_u32(
        linux_portal_settings::APPEARANCE_NAMESPACE,
        "color-scheme",
    )?;

    match value {
        1 => Some(ColorScheme::Dark),
        2 => Some(ColorScheme::Light),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn read_desktop_prefers_reduced_motion() -> Option<bool> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SPI_GETCLIENTAREAANIMATION, SystemParametersInfoW,
    };
    use windows_sys::core::BOOL;

    unsafe {
        let mut enabled: BOOL = 0;
        let ok = SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            std::ptr::addr_of_mut!(enabled) as *mut _,
            0,
        );
        (ok != 0).then_some(enabled == 0)
    }
}

#[cfg(target_os = "macos")]
fn read_desktop_prefers_reduced_motion() -> Option<bool> {
    use cocoa::base::id;
    use objc::runtime::Class;
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let Some(class) = Class::get("NSWorkspace") else {
            return None;
        };
        let workspace: id = msg_send![class, sharedWorkspace];
        if workspace.is_null() {
            return None;
        }
        let selector = sel!(accessibilityDisplayShouldReduceMotion);
        let responds: bool = msg_send![workspace, respondsToSelector: selector];
        if !responds {
            return None;
        }
        let value: bool = msg_send![workspace, accessibilityDisplayShouldReduceMotion];
        Some(value)
    }
}

#[cfg(target_os = "linux")]
fn read_desktop_prefers_reduced_motion() -> Option<bool> {
    // Best-effort mapping of portal appearance preference to the web vocabulary:
    // `prefers-reduced-motion`.
    //
    // Portal keys differ across versions; try both spellings.
    linux_portal_settings::read_bool(linux_portal_settings::APPEARANCE_NAMESPACE, "reduce-motion")
        .or_else(|| {
            linux_portal_settings::read_bool(
                linux_portal_settings::APPEARANCE_NAMESPACE,
                "reduced-motion",
            )
        })
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn read_desktop_prefers_reduced_motion() -> Option<bool> {
    None
}

#[cfg(target_os = "windows")]
fn read_desktop_text_scale_factor() -> Option<f32> {
    // Best-effort mapping to the web vocabulary used by `textScaleFactor`.
    //
    // Windows exposes a user-controlled "Text size" slider under Accessibility. It is stored as a
    // percentage value (e.g. 100, 125, 150).
    let pct = read_windows_reg_dword_hkcu(r"Software\Microsoft\Accessibility", "TextScaleFactor")?;
    if pct == 0 {
        return None;
    }
    Some((pct as f32 / 100.0).max(0.1))
}

#[cfg(target_os = "windows")]
fn read_desktop_prefers_reduced_transparency() -> Option<bool> {
    // Best-effort mapping to the web vocabulary: `prefers-reduced-transparency`.
    //
    // When Transparency Effects are disabled, interpret it as "prefers reduced transparency".
    let enabled = read_windows_reg_dword_hkcu(
        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
        "EnableTransparency",
    )?;
    Some(enabled == 0)
}

#[cfg(target_os = "windows")]
fn read_desktop_accent_color() -> Option<fret_core::Color> {
    // Best-effort accent color; Windows provides a "colorization" color through DWM.
    use windows_sys::Win32::Graphics::Dwm::DwmGetColorizationColor;
    use windows_sys::core::BOOL;

    unsafe {
        let mut argb: u32 = 0;
        let mut opaque: BOOL = 0;
        let hr =
            DwmGetColorizationColor(std::ptr::addr_of_mut!(argb), std::ptr::addr_of_mut!(opaque));
        if hr != 0 {
            return None;
        }

        let a = ((argb >> 24) & 0xFF) as f32 / 255.0;
        let r = ((argb >> 16) & 0xFF) as f32 / 255.0;
        let g = ((argb >> 8) & 0xFF) as f32 / 255.0;
        let b = (argb & 0xFF) as f32 / 255.0;
        Some(fret_core::Color { r, g, b, a })
    }
}

#[cfg(target_os = "windows")]
fn read_windows_reg_dword_hkcu(subkey: &str, value: &str) -> Option<u32> {
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    unsafe {
        let subkey_w = wide(subkey);
        let value_w = wide(value);

        let mut out: u32 = 0;
        let mut out_len = std::mem::size_of::<u32>() as u32;
        let status = RegGetValueW(
            HKEY_CURRENT_USER,
            subkey_w.as_ptr(),
            value_w.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            std::ptr::addr_of_mut!(out) as *mut _,
            std::ptr::addr_of_mut!(out_len),
        );
        (status == ERROR_SUCCESS).then_some(out)
    }
}

#[cfg(target_os = "macos")]
fn read_desktop_text_scale_factor() -> Option<f32> {
    None
}

#[cfg(target_os = "macos")]
fn read_desktop_prefers_reduced_transparency() -> Option<bool> {
    use cocoa::base::id;
    use objc::runtime::Class;
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let Some(class) = Class::get("NSWorkspace") else {
            return None;
        };
        let workspace: id = msg_send![class, sharedWorkspace];
        if workspace.is_null() {
            return None;
        }
        let selector = sel!(accessibilityDisplayShouldReduceTransparency);
        let responds: bool = msg_send![workspace, respondsToSelector: selector];
        if !responds {
            return None;
        }
        let value: bool = msg_send![workspace, accessibilityDisplayShouldReduceTransparency];
        Some(value)
    }
}

#[cfg(target_os = "macos")]
fn read_desktop_accent_color() -> Option<fret_core::Color> {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSAutoreleasePool;
    use cocoa::foundation::NSString;
    use objc::runtime::Class;
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::CStr;
    use std::os::raw::c_char;

    unsafe {
        let Some(class) = Class::get("NSUserDefaults") else {
            return None;
        };
        let defaults: id = msg_send![class, standardUserDefaults];
        if defaults.is_null() {
            return None;
        }

        let key: id = NSString::alloc(nil)
            .init_str("AppleHighlightColor")
            .autorelease();
        let value: id = msg_send![defaults, stringForKey: key];
        if value.is_null() {
            return None;
        }
        let c_str: *const c_char = msg_send![value, UTF8String];
        if c_str.is_null() {
            return None;
        }
        let s = CStr::from_ptr(c_str).to_string_lossy();
        parse_macos_highlight_color(&s)
    }
}

#[cfg(target_os = "macos")]
fn parse_macos_highlight_color(raw: &str) -> Option<fret_core::Color> {
    // `AppleHighlightColor` typically looks like:
    // "0.968627 0.831373 1.000000 Purple"
    let mut parts = raw.split_whitespace();
    let r: f32 = parts.next()?.parse().ok()?;
    let g: f32 = parts.next()?.parse().ok()?;
    let b: f32 = parts.next()?.parse().ok()?;
    Some(fret_core::Color {
        r: r.clamp(0.0, 1.0),
        g: g.clamp(0.0, 1.0),
        b: b.clamp(0.0, 1.0),
        a: 1.0,
    })
}

#[cfg(target_os = "linux")]
fn read_desktop_text_scale_factor() -> Option<f32> {
    // Linux does not have a single canonical source. When available, prefer portal settings.
    //
    // Portal keys differ across versions; try a couple of common spellings.
    let value = linux_portal_settings::read_f64(
        linux_portal_settings::APPEARANCE_NAMESPACE,
        "text-scaling-factor",
    )
    .or_else(|| {
        linux_portal_settings::read_f64(
            linux_portal_settings::APPEARANCE_NAMESPACE,
            "text-scale-factor",
        )
    })?;
    (value.is_finite() && value > 0.0).then_some(value as f32)
}

#[cfg(target_os = "linux")]
fn read_desktop_prefers_reduced_transparency() -> Option<bool> {
    // Best-effort mapping to `prefers-reduced-transparency`.
    //
    // Portal keys differ across versions; try both spellings.
    linux_portal_settings::read_bool(
        linux_portal_settings::APPEARANCE_NAMESPACE,
        "reduce-transparency",
    )
    .or_else(|| {
        linux_portal_settings::read_bool(
            linux_portal_settings::APPEARANCE_NAMESPACE,
            "reduced-transparency",
        )
    })
}

#[cfg(target_os = "linux")]
fn read_desktop_accent_color() -> Option<fret_core::Color> {
    None
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn read_desktop_text_scale_factor() -> Option<f32> {
    None
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn read_desktop_prefers_reduced_transparency() -> Option<bool> {
    None
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn read_desktop_accent_color() -> Option<fret_core::Color> {
    None
}

#[cfg(target_os = "windows")]
fn read_desktop_contrast_preference() -> Option<ContrastPreference> {
    use windows_sys::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
    use windows_sys::Win32::UI::WindowsAndMessaging::{SPI_GETHIGHCONTRAST, SystemParametersInfoW};

    unsafe {
        let mut hc = HIGHCONTRASTW {
            cbSize: std::mem::size_of::<HIGHCONTRASTW>() as u32,
            dwFlags: 0,
            lpszDefaultScheme: std::ptr::null_mut(),
        };
        let ok = SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            hc.cbSize,
            std::ptr::addr_of_mut!(hc) as *mut _,
            0,
        );
        if ok == 0 {
            return None;
        }
        if (hc.dwFlags & HCF_HIGHCONTRASTON) != 0 {
            Some(ContrastPreference::More)
        } else {
            Some(ContrastPreference::NoPreference)
        }
    }
}

#[cfg(target_os = "macos")]
fn read_desktop_contrast_preference() -> Option<ContrastPreference> {
    use cocoa::base::id;
    use objc::runtime::Class;
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let Some(class) = Class::get("NSWorkspace") else {
            return None;
        };
        let workspace: id = msg_send![class, sharedWorkspace];
        if workspace.is_null() {
            return None;
        }
        let selector = sel!(accessibilityDisplayShouldIncreaseContrast);
        let responds: bool = msg_send![workspace, respondsToSelector: selector];
        if !responds {
            return None;
        }
        let value: bool = msg_send![workspace, accessibilityDisplayShouldIncreaseContrast];
        Some(if value {
            ContrastPreference::More
        } else {
            ContrastPreference::NoPreference
        })
    }
}

#[cfg(target_os = "linux")]
fn read_desktop_contrast_preference() -> Option<ContrastPreference> {
    // Best-effort mapping to the web vocabulary: `prefers-contrast`.
    //
    // We intentionally keep this runner-owned and optional; when unavailable we return `None`.
    let value =
        linux_portal_settings::read_u32(linux_portal_settings::APPEARANCE_NAMESPACE, "contrast")?;

    Some(match value {
        0 => ContrastPreference::NoPreference,
        1 => ContrastPreference::More,
        2 => ContrastPreference::Less,
        3 => ContrastPreference::Custom,
        _ => return None,
    })
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn read_desktop_contrast_preference() -> Option<ContrastPreference> {
    None
}

#[cfg(target_os = "windows")]
fn read_desktop_forced_colors_mode() -> Option<ForcedColorsMode> {
    use windows_sys::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
    use windows_sys::Win32::UI::WindowsAndMessaging::{SPI_GETHIGHCONTRAST, SystemParametersInfoW};

    unsafe {
        let mut hc = HIGHCONTRASTW {
            cbSize: std::mem::size_of::<HIGHCONTRASTW>() as u32,
            dwFlags: 0,
            lpszDefaultScheme: std::ptr::null_mut(),
        };
        let ok = SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            hc.cbSize,
            std::ptr::addr_of_mut!(hc) as *mut _,
            0,
        );
        if ok == 0 {
            return None;
        }
        Some(if (hc.dwFlags & HCF_HIGHCONTRASTON) != 0 {
            ForcedColorsMode::Active
        } else {
            ForcedColorsMode::None
        })
    }
}

#[cfg(target_os = "linux")]
fn read_desktop_forced_colors_mode() -> Option<ForcedColorsMode> {
    // Best-effort mapping to the web vocabulary: `forced-colors`.
    //
    // Linux doesn't have a single canonical source. We currently infer it from portal appearance
    // contrast when available.
    let Some(contrast) = read_desktop_contrast_preference() else {
        return None;
    };

    Some(match contrast {
        ContrastPreference::More | ContrastPreference::Custom => ForcedColorsMode::Active,
        ContrastPreference::NoPreference | ContrastPreference::Less => ForcedColorsMode::None,
    })
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn read_desktop_forced_colors_mode() -> Option<ForcedColorsMode> {
    None
}

#[cfg(target_os = "linux")]
fn is_wayland_session(xdg_session_type: Option<&str>, wayland_display: Option<&str>) -> bool {
    if xdg_session_type.is_some_and(|v| v.eq_ignore_ascii_case("wayland")) {
        return true;
    }
    wayland_display.is_some_and(|v| !v.is_empty())
}

#[cfg(target_os = "linux")]
pub(super) fn linux_is_wayland_session() -> bool {
    let xdg_session_type = std::env::var("XDG_SESSION_TYPE").ok();
    let wayland_display = std::env::var("WAYLAND_DISPLAY").ok();
    is_wayland_session(xdg_session_type.as_deref(), wayland_display.as_deref())
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn is_wayland_session_true_for_xdg_session_type_wayland() {
        assert!(is_wayland_session(Some("wayland"), None));
        assert!(is_wayland_session(Some("Wayland"), None));
    }

    #[test]
    fn is_wayland_session_true_for_wayland_display() {
        assert!(is_wayland_session(None, Some("wayland-0")));
    }

    #[test]
    fn is_wayland_session_false_for_x11_and_no_wayland_display() {
        assert!(!is_wayland_session(Some("x11"), None));
        assert!(!is_wayland_session(None, Some("")));
    }
}
