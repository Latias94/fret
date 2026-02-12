#[cfg(not(target_arch = "wasm32"))]
use fret_core::{ExternalDropFileData, ExternalDropReadError};
use fret_core::{FileDialogDataEvent, FileDialogOptions, FileDialogSelection, FileDialogToken};
use fret_platform::external_drop::ExternalDropReadLimits;
use fret_platform::file_dialog::{FileDialogError, FileDialogProvider};

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "windows", target_os = "macos", target_os = "linux")
))]
use fret_core::ExternalDragFile;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "windows", target_os = "macos", target_os = "linux")
))]
use std::collections::HashMap;

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "windows", target_os = "macos", target_os = "linux")
))]
#[derive(Debug)]
pub struct NativeFileDialog {
    next_token: u64,
    selections: HashMap<FileDialogToken, Vec<PathBuf>>,
}

#[cfg(not(all(
    not(target_arch = "wasm32"),
    any(target_os = "windows", target_os = "macos", target_os = "linux")
)))]
#[derive(Debug)]
pub struct NativeFileDialog;

pub type DesktopFileDialog = NativeFileDialog;

impl NativeFileDialog {
    pub fn new() -> Self {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        ))]
        {
            Self {
                next_token: 1,
                selections: HashMap::new(),
            }
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            Self
        }
    }
}

impl Default for NativeFileDialog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "windows", target_os = "macos", target_os = "linux")
))]
impl NativeFileDialog {
    fn allocate_token(&mut self) -> FileDialogToken {
        let token = FileDialogToken(self.next_token);
        self.next_token = self.next_token.saturating_add(1);
        token
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeFileDialog {
    pub fn paths(&self, token: FileDialogToken) -> Option<&[PathBuf]> {
        #[cfg(all(
            any(target_os = "windows", target_os = "macos", target_os = "linux"),
            not(target_arch = "wasm32")
        ))]
        {
            self.selections.get(&token).map(|v| v.as_slice())
        }

        #[cfg(not(all(
            any(target_os = "windows", target_os = "macos", target_os = "linux"),
            not(target_arch = "wasm32")
        )))]
        {
            let _ = token;
            None
        }
    }

    pub fn read_paths(
        token: FileDialogToken,
        paths: Vec<PathBuf>,
        limits: ExternalDropReadLimits,
    ) -> FileDialogDataEvent {
        let mut files: Vec<ExternalDropFileData> = Vec::new();
        let mut errors: Vec<ExternalDropReadError> = Vec::new();
        let mut total: u64 = 0;

        for path in paths.into_iter().take(limits.max_files) {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string());

            let meta_len = match std::fs::metadata(&path) {
                Ok(m) => Some(m.len()),
                Err(err) => {
                    errors.push(ExternalDropReadError {
                        name,
                        message: format!("metadata failed: {err}"),
                    });
                    continue;
                }
            };

            if let Some(len) = meta_len
                && len > limits.max_file_bytes
            {
                errors.push(ExternalDropReadError {
                    name,
                    message: format!(
                        "file too large ({} bytes > max_file_bytes {})",
                        len, limits.max_file_bytes
                    ),
                });
                continue;
            }

            if total >= limits.max_total_bytes {
                errors.push(ExternalDropReadError {
                    name,
                    message: format!(
                        "selection too large (total {} >= max_total_bytes {})",
                        total, limits.max_total_bytes
                    ),
                });
                break;
            }

            let bytes = match std::fs::read(&path) {
                Ok(bytes) => bytes,
                Err(err) => {
                    errors.push(ExternalDropReadError {
                        name,
                        message: format!("read failed: {err}"),
                    });
                    continue;
                }
            };

            if bytes.len() as u64 > limits.max_file_bytes {
                errors.push(ExternalDropReadError {
                    name,
                    message: format!(
                        "file too large ({} bytes > max_file_bytes {})",
                        bytes.len(),
                        limits.max_file_bytes
                    ),
                });
                continue;
            }

            let next_total = total.saturating_add(bytes.len() as u64);
            if next_total > limits.max_total_bytes {
                errors.push(ExternalDropReadError {
                    name,
                    message: format!(
                        "selection too large (next_total {} > max_total_bytes {})",
                        next_total, limits.max_total_bytes
                    ),
                });
                break;
            }

            total = next_total;
            files.push(ExternalDropFileData { name, bytes });
        }

        FileDialogDataEvent {
            token,
            files,
            errors,
        }
    }
}

impl FileDialogProvider for NativeFileDialog {
    fn open_files(
        &mut self,
        options: &FileDialogOptions,
    ) -> Result<Option<FileDialogSelection>, FileDialogError> {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        ))]
        {
            let mut dialog = rfd::FileDialog::new();

            if let Some(title) = &options.title {
                dialog = dialog.set_title(title);
            }

            for filter in &options.filters {
                dialog = dialog.add_filter(&filter.name, &filter.extensions);
            }

            let selected: Option<Vec<PathBuf>> = if options.multiple {
                dialog.pick_files()
            } else {
                dialog.pick_file().map(|p| vec![p])
            };

            let Some(paths) = selected else {
                return Ok(None);
            };

            let token = self.allocate_token();
            let files = paths
                .iter()
                .map(|path| ExternalDragFile {
                    name: path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.to_string_lossy().to_string()),
                })
                .collect::<Vec<_>>();

            self.selections.insert(token, paths);

            Ok(Some(FileDialogSelection { token, files }))
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            let _ = options;
            Err(FileDialogError {
                kind: fret_platform::file_dialog::FileDialogErrorKind::Unsupported,
            })
        }
    }

    fn read_all(
        &mut self,
        token: FileDialogToken,
        limits: ExternalDropReadLimits,
    ) -> Option<FileDialogDataEvent> {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        ))]
        {
            let paths = self.selections.get(&token)?.clone();
            Some(Self::read_paths(token, paths, limits))
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            let _ = token;
            let _ = limits;
            None
        }
    }

    fn release(&mut self, token: FileDialogToken) {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        ))]
        {
            self.selections.remove(&token);
        }

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(target_os = "windows", target_os = "macos", target_os = "linux")
        )))]
        {
            let _ = token;
        }
    }
}
