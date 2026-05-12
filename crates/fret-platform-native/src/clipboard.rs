use std::ffi::OsStr;

use fret_platform::clipboard::{Clipboard, ClipboardError, ClipboardErrorKind};

#[cfg(all(
    unix,
    not(any(
        target_os = "macos",
        target_os = "android",
        target_os = "ios",
        target_os = "emscripten"
    ))
))]
use arboard::{GetExtLinux as _, LinuxClipboardKind, SetExtLinux as _};

pub struct NativeClipboard {
    disabled: bool,
    #[cfg(all(
        not(target_arch = "wasm32"),
        any(target_os = "windows", target_os = "macos", target_os = "linux")
    ))]
    inner: Option<arboard::Clipboard>,
}

pub type DesktopClipboard = NativeClipboard;

pub const FRET_CLIPBOARD_DISABLE_ENV: &str = "FRET_CLIPBOARD_DISABLE";

pub fn native_clipboard_disabled_by_env() -> bool {
    clipboard_disable_env_enabled(std::env::var_os(FRET_CLIPBOARD_DISABLE_ENV).as_deref())
}

fn clipboard_disable_env_enabled(value: Option<&OsStr>) -> bool {
    let Some(value) = value else {
        return false;
    };
    let value = value.to_string_lossy();
    let value = value.trim();
    !value.is_empty()
        && value != "0"
        && !value.eq_ignore_ascii_case("false")
        && !value.eq_ignore_ascii_case("no")
        && !value.eq_ignore_ascii_case("off")
}

fn unavailable_error(message: impl Into<Option<String>>) -> ClipboardError {
    ClipboardError {
        kind: ClipboardErrorKind::Unavailable,
        message: message.into(),
    }
}

fn backend_error(message: impl Into<Option<String>>) -> ClipboardError {
    ClipboardError {
        kind: ClipboardErrorKind::BackendError,
        message: message.into(),
    }
}

impl Default for NativeClipboard {
    fn default() -> Self {
        Self::from_env()
    }
}

impl NativeClipboard {
    pub fn from_env() -> Self {
        if native_clipboard_disabled_by_env() {
            Self::unavailable()
        } else {
            Self::lazy()
        }
    }

    pub fn lazy() -> Self {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        ))]
        {
            Self {
                disabled: false,
                inner: None,
            }
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            Self { disabled: false }
        }
    }

    pub fn unavailable() -> Self {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        ))]
        {
            Self {
                disabled: true,
                inner: None,
            }
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            Self { disabled: true }
        }
    }

    #[cfg(all(
        not(target_arch = "wasm32"),
        any(target_os = "windows", target_os = "macos", target_os = "linux")
    ))]
    fn clipboard_mut(&mut self) -> Result<&mut arboard::Clipboard, ClipboardError> {
        if self.disabled {
            return Err(unavailable_error(Some(format!(
                "native clipboard disabled by {FRET_CLIPBOARD_DISABLE_ENV}"
            ))));
        }

        if self.inner.is_none() {
            let clipboard = arboard::Clipboard::new().map_err(|err| {
                unavailable_error(Some(format!("native clipboard backend unavailable: {err}")))
            })?;
            self.inner = Some(clipboard);
        }

        self.inner
            .as_mut()
            .ok_or_else(|| unavailable_error(Some("native clipboard backend unavailable".into())))
    }
}

impl Clipboard for NativeClipboard {
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        ))]
        {
            let cb = self.clipboard_mut()?;
            cb.set_text(text.to_string())
                .map_err(|err| backend_error(Some(err.to_string())))
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            let _ = text;
            Err(unavailable_error(None))
        }
    }

    fn get_text(&mut self) -> Result<Option<String>, ClipboardError> {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        ))]
        {
            let cb = self.clipboard_mut()?;
            match cb.get_text() {
                Ok(text) => Ok(Some(text)),
                Err(err) => Err(backend_error(Some(err.to_string()))),
            }
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            Err(unavailable_error(None))
        }
    }
}

impl NativeClipboard {
    pub fn set_primary_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        ))]
        {
            #[cfg(all(
                unix,
                not(any(
                    target_os = "macos",
                    target_os = "android",
                    target_os = "ios",
                    target_os = "emscripten"
                ))
            ))]
            {
                let cb = self.clipboard_mut()?;
                cb.set()
                    .clipboard(LinuxClipboardKind::Primary)
                    .text(text.to_string())
                    .map_err(|err| backend_error(Some(err.to_string())))
            }

            #[cfg(not(all(
                unix,
                not(any(
                    target_os = "macos",
                    target_os = "android",
                    target_os = "ios",
                    target_os = "emscripten"
                ))
            )))]
            {
                let _ = text;
                Err(unavailable_error(None))
            }
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            let _ = text;
            Err(unavailable_error(None))
        }
    }

    pub fn get_primary_text(&mut self) -> Result<Option<String>, ClipboardError> {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        ))]
        {
            #[cfg(all(
                unix,
                not(any(
                    target_os = "macos",
                    target_os = "android",
                    target_os = "ios",
                    target_os = "emscripten"
                ))
            ))]
            {
                let cb = self.clipboard_mut()?;
                cb.get()
                    .clipboard(LinuxClipboardKind::Primary)
                    .text()
                    .map(Some)
                    .map_err(|err| backend_error(Some(err.to_string())))
            }

            #[cfg(not(all(
                unix,
                not(any(
                    target_os = "macos",
                    target_os = "android",
                    target_os = "ios",
                    target_os = "emscripten"
                ))
            )))]
            {
                Err(unavailable_error(None))
            }
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            Err(unavailable_error(None))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_disable_env_enabled_accepts_only_truthy_values() {
        assert!(!clipboard_disable_env_enabled(None));
        assert!(!clipboard_disable_env_enabled(Some(OsStr::new(""))));
        assert!(!clipboard_disable_env_enabled(Some(OsStr::new("0"))));
        assert!(!clipboard_disable_env_enabled(Some(OsStr::new("false"))));
        assert!(!clipboard_disable_env_enabled(Some(OsStr::new("NO"))));
        assert!(!clipboard_disable_env_enabled(Some(OsStr::new("off"))));

        assert!(clipboard_disable_env_enabled(Some(OsStr::new("1"))));
        assert!(clipboard_disable_env_enabled(Some(OsStr::new("true"))));
        assert!(clipboard_disable_env_enabled(Some(OsStr::new("disabled"))));
    }

    #[test]
    fn unavailable_clipboard_reports_text_unavailable_without_backend_init() {
        let mut clipboard = NativeClipboard::unavailable();

        assert_eq!(
            clipboard.set_text("hello").unwrap_err().kind,
            ClipboardErrorKind::Unavailable
        );
        assert_eq!(
            clipboard.get_text().unwrap_err().kind,
            ClipboardErrorKind::Unavailable
        );
    }

    #[test]
    fn unavailable_clipboard_reports_primary_selection_unavailable() {
        let mut clipboard = NativeClipboard::unavailable();

        assert_eq!(
            clipboard.set_primary_text("hello").unwrap_err().kind,
            ClipboardErrorKind::Unavailable
        );
        assert_eq!(
            clipboard.get_primary_text().unwrap_err().kind,
            ClipboardErrorKind::Unavailable
        );
    }
}
