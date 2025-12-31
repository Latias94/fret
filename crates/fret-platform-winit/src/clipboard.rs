use fret_platform::clipboard::{Clipboard, ClipboardError, ClipboardErrorKind};

pub struct WinitClipboard {
    inner: Option<arboard::Clipboard>,
}

impl Default for WinitClipboard {
    fn default() -> Self {
        Self {
            inner: arboard::Clipboard::new().ok(),
        }
    }
}

impl Clipboard for WinitClipboard {
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        let Some(cb) = self.inner.as_mut() else {
            return Err(ClipboardError {
                kind: ClipboardErrorKind::Unavailable,
            });
        };
        cb.set_text(text.to_string()).map_err(|_| ClipboardError {
            kind: ClipboardErrorKind::BackendError,
        })
    }

    fn get_text(&mut self) -> Result<Option<String>, ClipboardError> {
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
}
