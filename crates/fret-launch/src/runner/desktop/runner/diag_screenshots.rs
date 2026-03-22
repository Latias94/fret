use std::collections::HashMap;
use std::path::{Path, PathBuf};

use fret_diag_protocol::{
    DiagScreenshotRequestV1, DiagScreenshotResultEntryV1, DiagScreenshotResultFileV1,
};

#[derive(Debug, Clone)]
struct PendingCapture {
    out_dir: PathBuf,
    bundle_dir_name: String,
    request_id: Option<String>,
    tick_id: u64,
    frame_id: u64,
    scale_factor: f32,
}

#[derive(Debug)]
pub(crate) struct InFlightCapture {
    out_path: PathBuf,
    manifest_path: PathBuf,
    bundle_dir_name: String,
    request_id: Option<String>,
    window_ffi: u64,
    tick_id: u64,
    frame_id: u64,
    scale_factor: f32,
    width_px: u32,
    height_px: u32,
    padded_bytes_per_row: u32,
    unpadded_bytes_per_row: u32,
    source_format: wgpu::TextureFormat,
    buffer: wgpu::Buffer,
}

#[derive(Debug)]
pub(crate) struct DiagScreenshotCapture {
    request_path: PathBuf,
    trigger_path: PathBuf,
    result_path: PathBuf,
    result_trigger_path: PathBuf,
    last_trigger_stamp: Option<u64>,
    pending_by_window_ffi: HashMap<u64, PendingCapture>,
}

impl DiagScreenshotCapture {
    fn read_config_json() -> Option<serde_json::Value> {
        let path = std::env::var_os("FRET_DIAG_CONFIG_PATH").filter(|v| !v.is_empty());
        let path = path?;
        let Ok(bytes) = std::fs::read(&path) else {
            return None;
        };
        serde_json::from_slice::<serde_json::Value>(&bytes).ok()
    }

    pub(crate) fn from_env() -> Option<Self> {
        let config = Self::read_config_json();
        let enabled = env_flag_default_false("FRET_DIAG_GPU_SCREENSHOTS")
            || config
                .as_ref()
                .and_then(|v| v.get("screenshots_enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
        if !enabled {
            return None;
        }

        let out_dir_env = std::env::var_os("FRET_DIAG_DIR").filter(|v| !v.is_empty());
        let out_dir = out_dir_env
            .map(PathBuf::from)
            .or_else(|| {
                config
                    .as_ref()
                    .and_then(|v| v.get("out_dir"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| PathBuf::from("target").join("fret-diag"));

        let config_paths = config.as_ref().and_then(|v| v.get("paths"));
        let resolve_config_path = |key: &str| -> Option<PathBuf> {
            let raw = config_paths
                .and_then(|p| p.get(key))
                .and_then(|v| v.as_str())
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())?;
            let p = PathBuf::from(raw);
            Some(if p.is_absolute() { p } else { out_dir.join(p) })
        };

        let request_path = std::env::var_os("FRET_DIAG_SCREENSHOT_REQUEST_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| resolve_config_path("screenshot_request_path"))
            .unwrap_or_else(|| out_dir.join("screenshots.request.json"));

        let trigger_path = std::env::var_os("FRET_DIAG_SCREENSHOT_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| resolve_config_path("screenshot_trigger_path"))
            .unwrap_or_else(|| out_dir.join("screenshots.touch"));

        let result_path = std::env::var_os("FRET_DIAG_SCREENSHOT_RESULT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| resolve_config_path("screenshot_result_path"))
            .unwrap_or_else(|| out_dir.join("screenshots.result.json"));

        let result_trigger_path = std::env::var_os("FRET_DIAG_SCREENSHOT_RESULT_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| resolve_config_path("screenshot_result_trigger_path"))
            .unwrap_or_else(|| out_dir.join("screenshots.result.touch"));

        tracing::debug!(
            enabled,
            request_path = %request_path.display(),
            trigger_path = %trigger_path.display(),
            result_path = %result_path.display(),
            result_trigger_path = %result_trigger_path.display(),
            "diag screenshot: configured capture surface"
        );

        Some(Self {
            request_path,
            trigger_path,
            result_path,
            result_trigger_path,
            last_trigger_stamp: None,
            pending_by_window_ffi: HashMap::new(),
        })
    }

    pub(crate) fn poll(&mut self) {
        let stamp = match read_touch_stamp(&self.trigger_path) {
            Some(stamp) => stamp,
            None => return,
        };
        if self.last_trigger_stamp.is_some_and(|prev| prev >= stamp) {
            return;
        }
        self.last_trigger_stamp = Some(stamp);

        tracing::debug!(
            stamp,
            trigger_path = %self.trigger_path.display(),
            request_path = %self.request_path.display(),
            "diag screenshot: observed trigger stamp"
        );

        let bytes = match std::fs::read(&self.request_path) {
            Ok(b) => b,
            Err(_) => return,
        };

        let req: DiagScreenshotRequestV1 = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(error = %err, "diag screenshot: failed to parse request");
                return;
            }
        };
        if req.schema_version != 1 {
            tracing::warn!(
                schema_version = req.schema_version,
                "diag screenshot: unsupported schema_version"
            );
            return;
        }

        let out_dir = PathBuf::from(&req.out_dir);
        tracing::debug!(
            out_dir = %out_dir.display(),
            bundle_dir_name = %req.bundle_dir_name,
            request_id = ?req.request_id.as_deref(),
            windows = req.windows.len(),
            "diag screenshot: parsed request"
        );
        for item in req.windows {
            tracing::debug!(
                window = item.window,
                tick_id = item.tick_id,
                frame_id = item.frame_id,
                scale_factor = item.scale_factor,
                "diag screenshot: queued window capture"
            );
            self.pending_by_window_ffi.insert(
                item.window,
                PendingCapture {
                    out_dir: out_dir.clone(),
                    bundle_dir_name: req.bundle_dir_name.clone(),
                    request_id: req.request_id.clone(),
                    tick_id: item.tick_id,
                    frame_id: item.frame_id,
                    scale_factor: item.scale_factor as f32,
                },
            );
        }
    }

    pub(crate) fn has_pending_for_window(&self, window_ffi: u64) -> bool {
        self.pending_by_window_ffi.contains_key(&window_ffi)
    }

    pub(crate) fn begin_capture_for_window(
        &mut self,
        device: &wgpu::Device,
        window_ffi: u64,
        source_texture: &wgpu::Texture,
        source_format: wgpu::TextureFormat,
        surface_size_px: (u32, u32),
    ) -> Option<(wgpu::CommandBuffer, InFlightCapture)> {
        let (width_px, height_px) = surface_size_px;
        if width_px == 0 || height_px == 0 {
            // Avoid dropping the request on transient zero-sized surfaces (minimized/just-created).
            // We'll retry on the next frame once the surface is configured to a real size.
            if let Some(pending) = self.pending_by_window_ffi.get(&window_ffi) {
                tracing::debug!(
                    window = window_ffi,
                    bundle_dir_name = pending.bundle_dir_name.as_str(),
                    "diag screenshot: surface has zero size; delaying capture"
                );
            }
            return None;
        }

        let pending = self.pending_by_window_ffi.remove(&window_ffi)?;
        tracing::debug!(
            window = window_ffi,
            width_px,
            height_px,
            format = ?source_format,
            "diag screenshot: begin capture"
        );

        let bytes_per_pixel: u32 = 4;
        let unpadded_bytes_per_row = width_px.saturating_mul(bytes_per_pixel);
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(256).saturating_mul(256);
        let buffer_size = padded_bytes_per_row as u64 * height_px as u64;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("diag screenshot readback buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("diag screenshot readback encoder"),
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: source_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height_px),
                },
            },
            wgpu::Extent3d {
                width: width_px,
                height: height_px,
                depth_or_array_layers: 1,
            },
        );
        let cmd = encoder.finish();

        let screenshots_dir = pending
            .out_dir
            .join("screenshots")
            .join(&pending.bundle_dir_name);
        let _ = std::fs::create_dir_all(&screenshots_dir);

        let file_name = format!(
            "window-{window_ffi}-tick-{tick}-frame-{frame}.png",
            tick = pending.tick_id,
            frame = pending.frame_id
        );
        let out_path = screenshots_dir.join(file_name);
        let manifest_path = screenshots_dir.join("manifest.json");

        Some((
            cmd,
            InFlightCapture {
                out_path,
                manifest_path,
                bundle_dir_name: pending.bundle_dir_name,
                request_id: pending.request_id,
                window_ffi,
                tick_id: pending.tick_id,
                frame_id: pending.frame_id,
                scale_factor: pending.scale_factor,
                width_px,
                height_px,
                padded_bytes_per_row,
                unpadded_bytes_per_row,
                source_format,
                buffer,
            },
        ))
    }

    pub(crate) fn finish_capture(
        &mut self,
        device: &wgpu::Device,
        inflight: InFlightCapture,
    ) -> Result<(), String> {
        use std::sync::mpsc;

        let slice = inflight.buffer.slice(..);
        let (tx, rx) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv()
            .map_err(|_| "diag screenshot: map_async channel closed".to_string())?
            .map_err(|e| format!("diag screenshot: map_async failed: {e:?}"))?;

        let mapped = slice.get_mapped_range();
        let mut tight =
            vec![0u8; inflight.unpadded_bytes_per_row as usize * inflight.height_px as usize];
        for row in 0..inflight.height_px as usize {
            let src = row * inflight.padded_bytes_per_row as usize;
            let dst = row * inflight.unpadded_bytes_per_row as usize;
            tight[dst..dst + inflight.unpadded_bytes_per_row as usize]
                .copy_from_slice(&mapped[src..src + inflight.unpadded_bytes_per_row as usize]);
        }
        drop(mapped);
        inflight.buffer.unmap();

        match inflight.source_format {
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
                for px in tight.chunks_exact_mut(4) {
                    px.swap(0, 2);
                }
            }
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => {}
            other => {
                return Err(format!(
                    "diag screenshot: unsupported surface format for readback: {other:?}"
                ));
            }
        }

        let img = image::RgbaImage::from_raw(inflight.width_px, inflight.height_px, tight)
            .ok_or_else(|| "diag screenshot: failed to construct RgbaImage".to_string())?;
        img.save(&inflight.out_path)
            .map_err(|e| format!("diag screenshot: failed to save png: {e}"))?;

        update_manifest(
            &inflight.manifest_path,
            ManifestEntry {
                file_name: inflight
                    .out_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("screenshot.png")
                    .to_string(),
                window_ffi: inflight.window_ffi,
                tick_id: inflight.tick_id,
                frame_id: inflight.frame_id,
                scale_factor: inflight.scale_factor,
                width_px: inflight.width_px,
                height_px: inflight.height_px,
            },
        )?;

        update_result(
            &self.result_path,
            &self.result_trigger_path,
            ResultEntry {
                request_id: inflight.request_id,
                bundle_dir_name: inflight.bundle_dir_name,
                window_ffi: inflight.window_ffi,
                tick_id: inflight.tick_id,
                frame_id: inflight.frame_id,
                scale_factor: inflight.scale_factor,
                file: inflight
                    .out_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("screenshot.png")
                    .to_string(),
                width_px: inflight.width_px,
                height_px: inflight.height_px,
            },
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ManifestEntry {
    file_name: String,
    window_ffi: u64,
    tick_id: u64,
    frame_id: u64,
    scale_factor: f32,
    width_px: u32,
    height_px: u32,
}

fn update_manifest(path: &Path, entry: ManifestEntry) -> Result<(), String> {
    let mut root: serde_json::Value = if path.is_file() {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        serde_json::from_slice(&bytes).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if root
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(1)
        != 1
    {
        root = serde_json::json!({});
    }

    if !root.is_object() {
        root = serde_json::json!({});
    }

    if root.get("schema_version").is_none() {
        root["schema_version"] = serde_json::json!(1);
    }
    root["generated_unix_ms"] = serde_json::json!(
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or_default()
    );

    if !root.get("images").is_some_and(|v| v.is_array()) {
        root["images"] = serde_json::json!([]);
    }
    let images = root
        .get_mut("images")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| "diag screenshot: invalid manifest images".to_string())?;

    images.retain(|img| img.get("file").and_then(|v| v.as_str()) != Some(entry.file_name.as_str()));

    images.push(serde_json::json!({
        "file": entry.file_name,
        "window": entry.window_ffi,
        "tick_id": entry.tick_id,
        "frame_id": entry.frame_id,
        "scale_factor": entry.scale_factor,
        "width_px": entry.width_px,
        "height_px": entry.height_px,
    }));

    images.sort_by(|a, b| {
        a.get("file")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(b.get("file").and_then(|v| v.as_str()).unwrap_or(""))
    });

    let bytes = serde_json::to_vec_pretty(&root).map_err(|e| e.to_string())?;
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
struct ResultEntry {
    request_id: Option<String>,
    bundle_dir_name: String,
    window_ffi: u64,
    tick_id: u64,
    frame_id: u64,
    scale_factor: f32,
    file: String,
    width_px: u32,
    height_px: u32,
}

fn update_result(path: &Path, trigger_path: &Path, entry: ResultEntry) -> Result<(), String> {
    let mut root: DiagScreenshotResultFileV1 = if path.is_file() {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        serde_json::from_slice(&bytes).unwrap_or_default()
    } else {
        DiagScreenshotResultFileV1::default()
    };

    if root.schema_version != 1 {
        root = DiagScreenshotResultFileV1::default();
    }

    let now_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default();
    root.schema_version = 1;
    root.updated_unix_ms = Some(now_unix_ms);

    if let Some(ref request_id) = entry.request_id {
        root.completed.retain(|v| {
            v.request_id.as_deref() != Some(request_id.as_str()) || v.window != entry.window_ffi
        });
    }

    root.completed.push(DiagScreenshotResultEntryV1 {
        request_id: entry.request_id,
        bundle_dir_name: entry.bundle_dir_name,
        window: entry.window_ffi,
        tick_id: entry.tick_id,
        frame_id: entry.frame_id,
        scale_factor: entry.scale_factor,
        file: entry.file,
        width_px: entry.width_px,
        height_px: entry.height_px,
        completed_unix_ms: now_unix_ms,
    });

    if root.completed.len() > 200 {
        let drain = root.completed.len().saturating_sub(200);
        root.completed.drain(0..drain);
    }

    let bytes = serde_json::to_vec_pretty(&root).map_err(|e| e.to_string())?;
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    std::fs::write(path, bytes).map_err(|e| e.to_string())?;
    touch_stamp_file(trigger_path, now_unix_ms);
    Ok(())
}

fn touch_stamp_file(path: &Path, stamp: u64) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, format!("{stamp}\n").as_bytes());
}

fn read_touch_stamp(path: &Path) -> Option<u64> {
    let bytes = std::fs::read(path).ok()?;
    let text = std::str::from_utf8(&bytes).ok()?;
    text.lines()
        .rev()
        .find_map(|line| line.trim().parse::<u64>().ok())
}

fn env_flag_default_false(name: &str) -> bool {
    let Ok(v) = std::env::var(name) else {
        return false;
    };
    let v = v.trim().to_ascii_lowercase();
    if v.is_empty() {
        return true;
    }
    !matches!(v.as_str(), "0" | "false" | "no" | "off")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let unique = format!(
            "fret-diag-screenshot-tests-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_request(
        request_path: &Path,
        trigger_path: &Path,
        stamp: u64,
        request_id: &str,
        window: u64,
    ) {
        let req = DiagScreenshotRequestV1 {
            schema_version: 1,
            out_dir: "target/fret-diag".to_string(),
            bundle_dir_name: "bundle".to_string(),
            request_id: Some(request_id.to_string()),
            windows: vec![fret_diag_protocol::DiagScreenshotWindowRequestV1 {
                window,
                tick_id: 7,
                frame_id: 11,
                scale_factor: 2.0,
            }],
        };
        let bytes = serde_json::to_vec_pretty(&req).expect("serialize request");
        std::fs::write(request_path, bytes).expect("write request");
        touch_stamp_file(trigger_path, stamp);
    }

    #[test]
    fn poll_reads_touch_stamp_and_tracks_pending_window() {
        let dir = temp_dir("poll_reads_touch_stamp");
        let request_path = dir.join("screenshots.request.json");
        let trigger_path = dir.join("screenshots.touch");
        let result_path = dir.join("screenshots.result.json");
        let result_trigger_path = dir.join("screenshots.result.touch");

        write_request(&request_path, &trigger_path, 1, "req-1", 41);

        let mut capture = DiagScreenshotCapture {
            request_path,
            trigger_path,
            result_path,
            result_trigger_path,
            last_trigger_stamp: None,
            pending_by_window_ffi: HashMap::new(),
        };

        capture.poll();

        assert!(capture.has_pending_for_window(41));
        assert_eq!(capture.last_trigger_stamp, Some(1));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn poll_ignores_stale_touch_stamp_and_accepts_newer_one() {
        let dir = temp_dir("poll_ignores_stale_touch_stamp");
        let request_path = dir.join("screenshots.request.json");
        let trigger_path = dir.join("screenshots.touch");
        let result_path = dir.join("screenshots.result.json");
        let result_trigger_path = dir.join("screenshots.result.touch");

        write_request(&request_path, &trigger_path, 2, "req-1", 41);

        let mut capture = DiagScreenshotCapture {
            request_path: request_path.clone(),
            trigger_path: trigger_path.clone(),
            result_path,
            result_trigger_path,
            last_trigger_stamp: None,
            pending_by_window_ffi: HashMap::new(),
        };

        capture.poll();
        assert!(capture.has_pending_for_window(41));

        capture.pending_by_window_ffi.clear();
        write_request(&request_path, &trigger_path, 2, "req-2", 42);
        capture.poll();
        assert!(
            !capture.has_pending_for_window(42),
            "same touch stamp should not be treated as a new screenshot request"
        );

        write_request(&request_path, &trigger_path, 3, "req-3", 43);
        capture.poll();
        assert!(capture.has_pending_for_window(43));
        assert_eq!(capture.last_trigger_stamp, Some(3));
        let _ = std::fs::remove_dir_all(dir);
    }
}
