use std::{collections::HashMap, path::PathBuf};

use fret_core::{
    ExternalDropDataEvent, ExternalDropFileData, ExternalDropReadError, ExternalDropToken,
};
use fret_platform::external_drop::{ExternalDropProvider, ExternalDropReadLimits};

#[derive(Debug)]
pub struct WinitExternalDrop {
    next_token: u64,
    payloads: HashMap<ExternalDropToken, Vec<PathBuf>>,
}

impl WinitExternalDrop {
    pub fn new() -> Self {
        Self {
            next_token: 1,
            payloads: HashMap::new(),
        }
    }

    pub fn allocate_token(&mut self) -> ExternalDropToken {
        let token = ExternalDropToken(self.next_token);
        self.next_token = self.next_token.saturating_add(1);
        token
    }

    pub fn set_payload_paths(&mut self, token: ExternalDropToken, paths: Vec<PathBuf>) {
        self.payloads.insert(token, paths);
    }

    pub fn paths(&self, token: ExternalDropToken) -> Option<&[PathBuf]> {
        self.payloads.get(&token).map(|v| v.as_slice())
    }

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

impl Default for WinitExternalDrop {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalDropProvider for WinitExternalDrop {
    fn read_all(
        &mut self,
        token: ExternalDropToken,
        limits: ExternalDropReadLimits,
    ) -> Option<ExternalDropDataEvent> {
        let paths = self.payloads.get(&token)?.clone();
        Some(Self::read_paths(token, paths, limits))
    }

    fn release(&mut self, token: ExternalDropToken) {
        self.payloads.remove(&token);
    }
}
