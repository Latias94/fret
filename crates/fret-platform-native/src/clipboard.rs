use fret_platform::clipboard::{Clipboard, ClipboardError, ClipboardErrorKind};

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
