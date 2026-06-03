use fret_core::{AppWindowId, Event};
use fret_runtime::PlatformCapabilities;

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    fn incoming_open_limits_cap(&self) -> fret_core::ExternalDropReadLimits {
        fret_core::ExternalDropReadLimits {
            max_total_bytes: self.config.file_dialog_max_total_bytes,
            max_file_bytes: self.config.file_dialog_max_file_bytes,
            max_files: self.config.file_dialog_max_files,
        }
    }

    fn build_incoming_open_data_event(
        &self,
        token: fret_core::IncomingOpenToken,
        limits: fret_core::ExternalDropReadLimits,
        include_limits: bool,
    ) -> Option<fret_core::IncomingOpenDataEvent> {
        if let Some(payload) = self.diag_incoming_open_payloads.get(&token) {
            let mut total_bytes: u64 = 0;
            let mut files: Vec<fret_core::ExternalDropFileData> = Vec::new();
            let mut texts: Vec<String> = Vec::new();
            let mut errors: Vec<fret_core::ExternalDropReadError> = Vec::new();

            for file in payload.files.iter().take(limits.max_files) {
                let len = file.bytes.len() as u64;
                if len > limits.max_file_bytes {
                    errors.push(fret_core::ExternalDropReadError {
                        name: file.name.clone(),
                        message: "file exceeds max_file_bytes".to_string(),
                    });
                    continue;
                }
                if total_bytes.saturating_add(len) > limits.max_total_bytes {
                    errors.push(fret_core::ExternalDropReadError {
                        name: file.name.clone(),
                        message: "read exceeds max_total_bytes".to_string(),
                    });
                    continue;
                }
                total_bytes = total_bytes.saturating_add(len);
                files.push(file.clone());
            }

            for (idx, text) in payload.texts.iter().enumerate() {
                let len = text.len() as u64;
                let name = format!("text[{idx}]");
                if len > limits.max_file_bytes {
                    errors.push(fret_core::ExternalDropReadError {
                        name,
                        message: "text exceeds max_file_bytes".to_string(),
                    });
                    continue;
                }
                if total_bytes.saturating_add(len) > limits.max_total_bytes {
                    errors.push(fret_core::ExternalDropReadError {
                        name,
                        message: "read exceeds max_total_bytes".to_string(),
                    });
                    continue;
                }
                total_bytes = total_bytes.saturating_add(len);
                texts.push(text.clone());
            }

            return Some(fret_core::IncomingOpenDataEvent {
                token,
                files,
                texts,
                errors,
                limits: include_limits.then_some(limits),
            });
        }

        let payload = self.incoming_open_path_payloads.get(&token)?;

        let mut files: Vec<fret_core::ExternalDropFileData> = Vec::new();
        let mut errors: Vec<fret_core::ExternalDropReadError> = Vec::new();
        let mut total: u64 = 0;

        for path in payload.paths.iter().take(limits.max_files) {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string());

            let meta_len = match std::fs::metadata(path) {
                Ok(m) => Some(m.len()),
                Err(err) => {
                    errors.push(fret_core::ExternalDropReadError {
                        name,
                        message: format!("metadata failed: {err}"),
                    });
                    continue;
                }
            };

            if let Some(len) = meta_len
                && len > limits.max_file_bytes
            {
                errors.push(fret_core::ExternalDropReadError {
                    name,
                    message: format!(
                        "file too large ({} bytes > max_file_bytes {})",
                        len, limits.max_file_bytes
                    ),
                });
                continue;
            }

            if total >= limits.max_total_bytes {
                errors.push(fret_core::ExternalDropReadError {
                    name,
                    message: format!(
                        "read too large (total {} >= max_total_bytes {})",
                        total, limits.max_total_bytes
                    ),
                });
                break;
            }

            let bytes = match std::fs::read(path) {
                Ok(bytes) => bytes,
                Err(err) => {
                    errors.push(fret_core::ExternalDropReadError {
                        name,
                        message: format!("read failed: {err}"),
                    });
                    continue;
                }
            };

            if bytes.len() as u64 > limits.max_file_bytes {
                errors.push(fret_core::ExternalDropReadError {
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
                errors.push(fret_core::ExternalDropReadError {
                    name,
                    message: format!(
                        "read too large (next_total {} > max_total_bytes {})",
                        next_total, limits.max_total_bytes
                    ),
                });
                break;
            }

            total = next_total;
            files.push(fret_core::ExternalDropFileData { name, bytes });
        }

        Some(fret_core::IncomingOpenDataEvent {
            token,
            files,
            texts: Vec::new(),
            errors,
            limits: include_limits.then_some(limits),
        })
    }

    pub(super) fn handle_diag_incoming_open_inject(
        &mut self,
        window: AppWindowId,
        items: Vec<fret_runtime::DiagIncomingOpenItem>,
    ) {
        let token = self.allocate_incoming_open_token();
        let mut payload = super::DiagIncomingOpenPayload::default();
        let mut request_items: Vec<fret_core::IncomingOpenItem> = Vec::new();
        for item in items {
            match item {
                fret_runtime::DiagIncomingOpenItem::File {
                    name,
                    bytes,
                    media_type,
                } => {
                    request_items.push(fret_core::IncomingOpenItem::File(
                        fret_core::ExternalDragFile {
                            name: name.clone(),
                            size_bytes: Some(bytes.len() as u64),
                            media_type,
                        },
                    ));
                    payload
                        .files
                        .push(fret_core::ExternalDropFileData { name, bytes });
                }
                fret_runtime::DiagIncomingOpenItem::Text { text, media_type } => {
                    let estimated_size_bytes = Some(text.len() as u64);
                    request_items.push(fret_core::IncomingOpenItem::Text {
                        media_type,
                        estimated_size_bytes,
                    });
                    payload.texts.push(text);
                }
            }
        }
        self.diag_incoming_open_payloads.insert(token, payload);
        self.deliver_window_event_now(
            window,
            &Event::IncomingOpenRequest {
                token,
                items: request_items,
            },
        );
    }

    fn deliver_incoming_open_read(
        &mut self,
        window: AppWindowId,
        token: fret_core::IncomingOpenToken,
        limits: fret_core::ExternalDropReadLimits,
        include_limits: bool,
    ) {
        match self.build_incoming_open_data_event(token, limits, include_limits) {
            Some(data) => self.deliver_window_event_now(window, &Event::IncomingOpenData(data)),
            None => {
                self.deliver_window_event_now(window, &Event::IncomingOpenUnavailable { token })
            }
        }
    }

    pub(super) fn handle_incoming_open_read_all(
        &mut self,
        window: AppWindowId,
        token: fret_core::IncomingOpenToken,
    ) {
        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        if !caps.shell.incoming_open {
            self.deliver_window_event_now(window, &Event::IncomingOpenUnavailable { token });
            return;
        }

        let limits = self.incoming_open_limits_cap();
        let include_limits = false;
        self.deliver_incoming_open_read(window, token, limits, include_limits);
    }

    pub(super) fn handle_incoming_open_read_all_with_limits(
        &mut self,
        window: AppWindowId,
        token: fret_core::IncomingOpenToken,
        limits: fret_core::ExternalDropReadLimits,
    ) {
        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        if !caps.shell.incoming_open {
            self.deliver_window_event_now(window, &Event::IncomingOpenUnavailable { token });
            return;
        }

        let cap = self.incoming_open_limits_cap();
        let limits = limits.capped_by(cap);
        let include_limits = true;
        self.deliver_incoming_open_read(window, token, limits, include_limits);
    }

    pub(super) fn handle_incoming_open_release(&mut self, token: fret_core::IncomingOpenToken) {
        self.diag_incoming_open_payloads.remove(&token);
        self.incoming_open_path_payloads.remove(&token);
    }
}
