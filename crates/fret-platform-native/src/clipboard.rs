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
    #[cfg(all(
        not(target_arch = "wasm32"),
        any(target_os = "windows", target_os = "macos", target_os = "linux")
    ))]
    inner: Option<arboard::Clipboard>,
}

pub type DesktopClipboard = NativeClipboard;

impl Default for NativeClipboard {
    fn default() -> Self {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        ))]
        {
            Self {
                inner: arboard::Clipboard::new().ok(),
            }
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            Self {}
        }
    }
}

impl Clipboard for NativeClipboard {
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        ))]
        {
            let Some(cb) = self.inner.as_mut() else {
                return Err(ClipboardError {
                    kind: ClipboardErrorKind::Unavailable,
                });
            };
            cb.set_text(text.to_string()).map_err(|_| ClipboardError {
                kind: ClipboardErrorKind::BackendError,
            })
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            let _ = text;
            Err(ClipboardError {
                kind: ClipboardErrorKind::Unavailable,
            })
        }
    }

    fn get_text(&mut self) -> Result<Option<String>, ClipboardError> {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        ))]
        {
            let Some(cb) = self.inner.as_mut() else {
                return Err(ClipboardError {
                    kind: ClipboardErrorKind::Unavailable,
                });
            };
            match cb.get_text() {
                Ok(text) => Ok(Some(text)),
                Err(_) => Err(ClipboardError {
                    kind: ClipboardErrorKind::BackendError,
                }),
            }
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            Err(ClipboardError {
                kind: ClipboardErrorKind::Unavailable,
            })
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
                let Some(cb) = self.inner.as_mut() else {
                    return Err(ClipboardError {
                        kind: ClipboardErrorKind::Unavailable,
                    });
                };
                cb.set()
                    .clipboard(LinuxClipboardKind::Primary)
                    .text(text.to_string())
                    .map_err(|_| ClipboardError {
                        kind: ClipboardErrorKind::BackendError,
                    })
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
                Err(ClipboardError {
                    kind: ClipboardErrorKind::Unavailable,
                })
            }
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            let _ = text;
            Err(ClipboardError {
                kind: ClipboardErrorKind::Unavailable,
            })
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
                let Some(cb) = self.inner.as_mut() else {
                    return Err(ClipboardError {
                        kind: ClipboardErrorKind::Unavailable,
                    });
                };
                cb.get()
                    .clipboard(LinuxClipboardKind::Primary)
                    .text()
                    .map(Some)
                    .map_err(|_| ClipboardError {
                        kind: ClipboardErrorKind::BackendError,
                    })
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
                Err(ClipboardError {
                    kind: ClipboardErrorKind::Unavailable,
                })
            }
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            Err(ClipboardError {
                kind: ClipboardErrorKind::Unavailable,
            })
        }
    }
}
