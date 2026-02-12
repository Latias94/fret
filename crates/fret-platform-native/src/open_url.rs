use fret_platform::open_url::{OpenUrl, OpenUrlError, OpenUrlErrorKind};

#[derive(Debug, Default)]
pub struct NativeOpenUrl;

pub type DesktopOpenUrl = NativeOpenUrl;

impl OpenUrl for NativeOpenUrl {
    fn open_url(&mut self, url: &str) -> Result<(), OpenUrlError> {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        ))]
        {
            webbrowser::open(url).map_err(|_| OpenUrlError {
                kind: OpenUrlErrorKind::BackendError,
            })?;
            Ok(())
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            let _ = url;
            Err(OpenUrlError {
                kind: OpenUrlErrorKind::Unsupported,
            })
        }
    }
}
