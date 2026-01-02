#[cfg(not(target_arch = "wasm32"))]
use fret_core::ExternalDropFileData;
#[cfg(not(target_arch = "wasm32"))]
use fret_core::ExternalDropReadError;
use fret_core::{ExternalDropDataEvent, ExternalDropToken};
use fret_platform::external_drop::{ExternalDropProvider, ExternalDropReadLimits};

#[cfg(not(target_arch = "wasm32"))]
use std::{collections::HashMap, path::PathBuf};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct DesktopExternalDrop {
    next_token: u64,
    payloads: HashMap<ExternalDropToken, Vec<PathBuf>>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct DesktopExternalDrop;

impl DesktopExternalDrop {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                next_token: 1,
                payloads: HashMap::new(),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            Self
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn allocate_token(&mut self) -> ExternalDropToken {
        let token = ExternalDropToken(self.next_token);
        self.next_token = self.next_token.saturating_add(1);
        token
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_payload_paths(&mut self, token: ExternalDropToken, paths: Vec<PathBuf>) {
        self.payloads.insert(token, paths);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn paths(&self, token: ExternalDropToken) -> Option<&[PathBuf]> {
        self.payloads.get(&token).map(|v| v.as_slice())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_paths(
        token: ExternalDropToken,
        paths: Vec<PathBuf>,
        limits: ExternalDropReadLimits,
    ) -> ExternalDropDataEvent {
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
                        "drop too large (total {} >= max_total_bytes {})",
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
                        "drop too large (next_total {} > max_total_bytes {})",
                        next_total, limits.max_total_bytes
                    ),
                });
                break;
            }

            total = next_total;
            files.push(ExternalDropFileData { name, bytes });
        }

        ExternalDropDataEvent {
            token,
            files,
            errors,
        }
    }
}

impl Default for DesktopExternalDrop {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalDropProvider for DesktopExternalDrop {
    fn read_all(
        &mut self,
        token: ExternalDropToken,
        limits: ExternalDropReadLimits,
    ) -> Option<ExternalDropDataEvent> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let paths = self.payloads.get(&token)?.clone();
            Some(Self::read_paths(token, paths, limits))
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = token;
            let _ = limits;
            None
        }
    }

    fn release(&mut self, token: ExternalDropToken) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.payloads.remove(&token);
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = token;
        }
    }
}
