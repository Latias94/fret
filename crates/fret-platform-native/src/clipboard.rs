use fret_platform::clipboard::{Clipboard, ClipboardError, ClipboardErrorKind};

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
))]
use arboard::{GetExtLinux as _, LinuxClipboardKind, SetExtLinux as _};

pub struct NativeClipboard {
    #[cfg(not(target_arch = "wasm32"))]
    inner: Option<arboard::Clipboard>,
}

pub type DesktopClipboard = NativeClipboard;

impl Default for NativeClipboard {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                inner: arboard::Clipboard::new().ok(),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            Self {}
        }
    }
}

impl Clipboard for NativeClipboard {
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        #[cfg(not(target_arch = "wasm32"))]
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

        #[cfg(target_arch = "wasm32")]
        {
            let _ = text;
            Err(ClipboardError {
                kind: ClipboardErrorKind::Unavailable,
            })
        }
    }

    fn get_text(&mut self) -> Result<Option<String>, ClipboardError> {
        #[cfg(not(target_arch = "wasm32"))]
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

        #[cfg(target_arch = "wasm32")]
        {
            Err(ClipboardError {
                kind: ClipboardErrorKind::Unavailable,
            })
        }
    }
}

impl NativeClipboard {
    pub fn set_primary_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            #[cfg(all(
                unix,
                not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
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
                not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
            )))]
            {
                let _ = text;
                Err(ClipboardError {
                    kind: ClipboardErrorKind::Unavailable,
                })
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = text;
            Err(ClipboardError {
                kind: ClipboardErrorKind::Unavailable,
            })
        }
    }

    pub fn get_primary_text(&mut self) -> Result<Option<String>, ClipboardError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            #[cfg(all(
                unix,
                not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
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
                not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
            )))]
            {
                Err(ClipboardError {
                    kind: ClipboardErrorKind::Unavailable,
                })
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            Err(ClipboardError {
                kind: ClipboardErrorKind::Unavailable,
            })
        }
    }
}
