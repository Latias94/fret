use fret_platform::open_url::{OpenUrl, OpenUrlError, OpenUrlErrorKind};

#[derive(Debug, Default)]
pub struct WinitOpenUrl;

impl OpenUrl for WinitOpenUrl {
    fn open_url(&mut self, url: &str) -> Result<(), OpenUrlError> {
        webbrowser::open(url).map_err(|_| OpenUrlError {
            kind: OpenUrlErrorKind::BackendError,
        })?;
        Ok(())
    }
}

