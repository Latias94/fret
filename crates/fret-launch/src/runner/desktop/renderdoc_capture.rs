use std::path::PathBuf;

use renderdoc::{RenderDoc, V100};

pub struct RenderDocCapture {
    api: RenderDoc<V100>,
    pending: bool,
    capture_dir: PathBuf,
}

impl RenderDocCapture {
    pub fn try_init() -> Option<Self> {
        #[cfg(windows)]
        {
            Self::try_preload_windows_dll_from_env();
        }

        let mut api = RenderDoc::new().ok()?;

        let capture_dir = std::env::var_os("FRET_RENDERDOC_CAPTURE_DIR")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".fret").join("renderdoc"));

        let _ = std::fs::create_dir_all(&capture_dir);
        RenderDoc::<V100>::set_log_file_path_template(&mut api, capture_dir.join("fret"));

        let pending = std::env::var_os("FRET_RENDERDOC_AUTOCAPTURE")
            .filter(|v| !v.is_empty())
            .is_some();

        Some(Self {
            api,
            pending,
            capture_dir,
        })
    }

    #[cfg(windows)]
    fn try_preload_windows_dll_from_env() {
        use std::os::windows::ffi::OsStrExt as _;
        use windows_sys::Win32::System::LibraryLoader::LoadLibraryW;

        let Some(path) = std::env::var_os("FRET_RENDERDOC_DLL").filter(|v| !v.is_empty()) else {
            return;
        };

        let wide = path
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();

        unsafe {
            let _ = LoadLibraryW(wide.as_ptr());
        }
    }

    pub fn request_capture(&mut self) {
        self.pending = true;
    }

    pub fn begin_capture_if_requested(&mut self) -> bool {
        if !std::mem::take(&mut self.pending) {
            return false;
        }

        RenderDoc::<V100>::start_frame_capture(
            &mut self.api,
            std::ptr::null::<std::ffi::c_void>(),
            std::ptr::null(),
        );
        true
    }

    pub fn end_capture(&mut self) {
        RenderDoc::<V100>::end_frame_capture(
            &mut self.api,
            std::ptr::null::<std::ffi::c_void>(),
            std::ptr::null(),
        );

        let count = RenderDoc::<V100>::get_num_captures(&self.api);
        if count == 0 {
            tracing::warn!(
                capture_dir = ?self.capture_dir,
                "renderdoc capture ended but no captures were recorded"
            );
            return;
        }

        if let Some((path, _time)) = RenderDoc::<V100>::get_capture(&self.api, count - 1) {
            tracing::info!(capture = ?path, "renderdoc capture saved");
        } else {
            tracing::info!(
                capture_dir = ?self.capture_dir,
                count,
                "renderdoc capture saved"
            );
        }
    }
}
