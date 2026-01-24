use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
struct PendingCapture {
    out_dir: PathBuf,
    bundle_dir_name: String,
    tick_id: u64,
    frame_id: u64,
    scale_factor: f32,
}

#[derive(Debug)]
pub(crate) struct InFlightCapture {
    out_path: PathBuf,
    manifest_path: PathBuf,
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
    last_trigger_mtime: Option<SystemTime>,
    pending_by_window_ffi: HashMap<u64, PendingCapture>,
}

impl DiagScreenshotCapture {
    pub(crate) fn from_env() -> Option<Self> {
        let enabled = env_flag_default_false("FRET_DIAG_SCREENSHOTS");
        if !enabled {
            return None;
        }

        let out_dir_env = std::env::var_os("FRET_DIAG_DIR").filter(|v| !v.is_empty());
        let out_dir = out_dir_env
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target").join("fret-diag"));

        let request_path = std::env::var_os("FRET_DIAG_SCREENSHOT_REQUEST_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("screenshots.request.json"));

        let trigger_path = std::env::var_os("FRET_DIAG_SCREENSHOT_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("screenshots.touch"));

        Some(Self {
            request_path,
            trigger_path,
            last_trigger_mtime: None,
            pending_by_window_ffi: HashMap::new(),
        })
    }

    pub(crate) fn poll(&mut self) {
        let modified = match std::fs::metadata(&self.trigger_path).and_then(|m| m.modified()) {
            Ok(m) => m,
            Err(_) => return,
        };

        if self.last_trigger_mtime.is_some_and(|prev| prev >= modified) {
            return;
        }
        self.last_trigger_mtime = Some(modified);

        let bytes = match std::fs::read(&self.request_path) {
            Ok(b) => b,
            Err(_) => return,
        };

        let req = match parse_request_json(&bytes) {
            Ok(r) => r,
            Err(err) => {
                tracing::warn!(error = %err, "diag screenshot: failed to parse request");
                return;
            }
        };

        for item in req.windows {
            self.pending_by_window_ffi.insert(
                item.window_ffi,
                PendingCapture {
                    out_dir: req.out_dir.clone(),
                    bundle_dir_name: req.bundle_dir_name.clone(),
                    tick_id: item.tick_id,
                    frame_id: item.frame_id,
                    scale_factor: item.scale_factor,
                },
            );
        }
    }

    pub(crate) fn begin_capture_for_window(
        &mut self,
        device: &wgpu::Device,
        window_ffi: u64,
        source_texture: &wgpu::Texture,
        source_format: wgpu::TextureFormat,
        surface_size_px: (u32, u32),
    ) -> Option<(wgpu::CommandBuffer, InFlightCapture)> {
        let pending = self.pending_by_window_ffi.remove(&window_ffi)?;

        let (width_px, height_px) = surface_size_px;
        if width_px == 0 || height_px == 0 {
            return None;
        }

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

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ParsedRequest {
    out_dir: PathBuf,
    bundle_dir_name: String,
    windows: Vec<ParsedWindowRequest>,
}

#[derive(Debug, Clone)]
struct ParsedWindowRequest {
    window_ffi: u64,
    tick_id: u64,
    frame_id: u64,
    scale_factor: f32,
}

fn parse_request_json(bytes: &[u8]) -> Result<ParsedRequest, String> {
    let v: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| format!("invalid json: {e}"))?;
    let schema = v
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "missing schema_version".to_string())?;
    if schema != 1 {
        return Err(format!("unsupported schema_version: {schema}"));
    }

    let out_dir = v
        .get("out_dir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing out_dir".to_string())?;
    let bundle_dir_name = v
        .get("bundle_dir_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing bundle_dir_name".to_string())?;
    let windows = v
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "missing windows".to_string())?;

    let mut parsed_windows = Vec::new();
    for w in windows {
        let window_ffi = w
            .get("window")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "windows[i].window missing".to_string())?;
        let tick_id = w
            .get("tick_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "windows[i].tick_id missing".to_string())?;
        let frame_id = w
            .get("frame_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "windows[i].frame_id missing".to_string())?;
        let scale_factor = w
            .get("scale_factor")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;

        parsed_windows.push(ParsedWindowRequest {
            window_ffi,
            tick_id,
            frame_id,
            scale_factor,
        });
    }

    Ok(ParsedRequest {
        out_dir: PathBuf::from(out_dir),
        bundle_dir_name: bundle_dir_name.to_string(),
        windows: parsed_windows,
    })
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
