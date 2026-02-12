use std::path::PathBuf;

use renderdog::{InAppError, RenderDog};

pub struct RenderDocCapture {
    api: RenderDog,
    pending: bool,
    capture_dir: PathBuf,
    autocapture_after_frames: Option<u32>,
    frame_index: u32,
}

impl RenderDocCapture {
    // RenderDoc capture usage (Windows):
    //
    // 1) Make RenderDoc available:
    //    - Recommended: launch the app via qrenderdoc/renderdoccmd (injection), then `RenderDog::new()` can connect.
    //    - Or set `FRET_RENDERDOC_DLL` to an absolute `renderdoc.dll` path (useful for Scoop installs).
    //
    // 2) Enable capture:
    //    - Set `FRET_RENDERDOC=1` to enable the integration.
    //    - Optional: set `FRET_RENDERDOC_AUTOCAPTURE=1` to capture the first rendered frame automatically.
    //    - Optional: set `FRET_RENDERDOC_AUTOCAPTURE_AFTER_FRAMES=<n>` to capture the Nth rendered frame
    //      (useful when the first frame is not representative).
    //    - Optional: set `FRET_RENDERDOC_CAPTURE_DIR=<path>` to control where `.rdc` files are written.
    //
    // 3) If capture hotkey doesn't work, prefer Vulkan backend for debugging:
    //    - Set `FRET_WGPU_BACKEND=vulkan` and rerun the demo, then press `F12` to request a capture.
    //
    // Example:
    //   $env:FRET_RENDERDOC=1
    //   $env:FRET_RENDERDOC_DLL="C:\\Users\\Frankorz\\scoop\\apps\\renderdoc\\current\\renderdoc.dll"
    //   $env:FRET_RENDERDOC_AUTOCAPTURE=1
    //   $env:FRET_WGPU_BACKEND="vulkan"
    //   cargo run -p fret-demo --bin fret-demo -- effects_demo
    //   cargo run -p fret-demo --bin fret-demo -- linked_cursor_demo
    pub fn try_init() -> Option<Self> {
        #[cfg(windows)]
        {
            Self::try_preload_windows_dll_from_env();
        }

        let api = RenderDog::new().ok()?;

        let capture_dir = std::env::var_os("FRET_RENDERDOC_CAPTURE_DIR")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".fret").join("renderdoc"));

        let _ = std::fs::create_dir_all(&capture_dir);
        // RenderDoc expects a UTF-8 template string.
        if let Some(template) = capture_dir.join("fret").to_str() {
            let _ = api.inner().set_capture_file_path_template(template);
        } else {
            tracing::warn!(
                capture_dir = ?capture_dir,
                "renderdoc capture requested but capture dir is not valid UTF-8; leaving template unchanged"
            );
        }

        let autocapture_after_frames = std::env::var("FRET_RENDERDOC_AUTOCAPTURE_AFTER_FRAMES")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|v| *v > 0)
            .or_else(|| {
                std::env::var_os("FRET_RENDERDOC_AUTOCAPTURE")
                    .filter(|v| !v.is_empty())
                    .map(|_| 1)
            });

        Some(Self {
            api,
            pending: false,
            capture_dir,
            autocapture_after_frames,
            frame_index: 0,
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
        self.frame_index = self.frame_index.saturating_add(1);
        if let Some(after) = self.autocapture_after_frames
            && self.frame_index >= after
        {
            self.autocapture_after_frames = None;
            self.pending = true;
        }

        if !std::mem::take(&mut self.pending) {
            return false;
        }

        let _ = self.api.inner().start_frame_capture(None, None);
        true
    }

    pub fn end_capture(&mut self) {
        let _ = self.api.inner().end_frame_capture(None, None);

        let Ok(count) = self.api.inner().get_num_captures() else {
            tracing::warn!("renderdoc capture ended but capture list is unavailable");
            return;
        };
        if count == 0 {
            tracing::warn!(
                capture_dir = ?self.capture_dir,
                "renderdoc capture ended but no captures were recorded"
            );
            return;
        }

        match self.api.inner().get_capture(count - 1) {
            Ok((path, _timestamp_s)) => tracing::info!(capture = %path, "renderdoc capture saved"),
            Err(InAppError::InvalidCaptureIndex) => tracing::warn!(
                capture_dir = ?self.capture_dir,
                count,
                "renderdoc capture ended but capture index is invalid"
            ),
            Err(_) => {
                // Keep the log forgiving: the capture may still be saved by RenderDoc even if metadata lookup fails.
                tracing::info!(
                    capture_dir = ?self.capture_dir,
                    count,
                    "renderdoc capture saved"
                );
            }
        }
    }
}
