use fret_platform::open_url::{OpenUrl, OpenUrlError, OpenUrlErrorKind};

#[derive(Debug, Default)]
pub struct DesktopOpenUrl;

impl OpenUrl for DesktopOpenUrl {
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
