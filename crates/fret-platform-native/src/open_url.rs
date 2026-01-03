use fret_platform::open_url::{OpenUrl, OpenUrlError, OpenUrlErrorKind};

#[derive(Debug, Default)]
pub struct NativeOpenUrl;

pub type DesktopOpenUrl = NativeOpenUrl;

impl OpenUrl for NativeOpenUrl {
    fn open_url(&mut self, url: &str) -> Result<(), OpenUrlError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            webbrowser::open(url).map_err(|_| OpenUrlError {
                kind: OpenUrlErrorKind::BackendError,
            })?;
            Ok(())
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = url;
            Err(OpenUrlError {
                kind: OpenUrlErrorKind::Unsupported,
            })
        }
    }
}
