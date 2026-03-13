use super::{OrderedDraw, SceneEncoding, TextDrawKind, TextVertex};
use crate::text::{DebugGlyphAtlasLookup, GlyphQuadKind};

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
fn parse_env_u64(name: &str) -> Option<u64> {
    std::env::var(name).ok().and_then(|v| v.parse::<u64>().ok())
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_env_probe_px(name: &str) -> Option<(f32, f32, f32, f32)> {
    let v = std::env::var(name).ok()?;
    let mut it = v.split([',', ' ']).filter(|s| !s.is_empty());
    let x = it.next()?.parse::<f32>().ok()?;
    let y = it.next()?.parse::<f32>().ok()?;
    let w = it.next()?.parse::<f32>().ok()?;
    let h = it.next()?.parse::<f32>().ok()?;
    Some((x, y, w, h))
}

#[cfg(not(target_arch = "wasm32"))]
fn dump_dir_from_env() -> PathBuf {
    std::env::var_os("FRET_RENDER_TEXT_DUMP_DIR")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".fret").join("render_text"))
}

#[cfg(not(target_arch = "wasm32"))]
fn should_dump_frame(frame_index: u64) -> bool {
    if std::env::var_os("FRET_RENDER_TEXT_DUMP")
        .filter(|v| !v.is_empty())
        .is_none()
    {
        return false;
    }

    if let Some(frame) = parse_env_u64("FRET_RENDER_TEXT_DUMP_FRAME") {
        return frame_index == frame;
    }

    let after = parse_env_u64("FRET_RENDER_TEXT_DUMP_AFTER_FRAMES").unwrap_or(1);
    if frame_index < after {
        return false;
    }

    if let Some(every) = parse_env_u64("FRET_RENDER_TEXT_DUMP_EVERY") {
        return every > 0 && (frame_index - after).is_multiple_of(every);
    }

    static DUMPED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    !DUMPED.swap(true, std::sync::atomic::Ordering::SeqCst)
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), serde(rename_all = "snake_case"))]
enum JsonAtlasKind {
    Mask,
    Color,
    Subpixel,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
struct JsonProbeRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl JsonProbeRect {
    fn intersects_bounds(&self, b: &JsonBoundsPx) -> bool {
        let ax0 = self.x;
        let ay0 = self.y;
        let ax1 = self.x + self.w;
        let ay1 = self.y + self.h;
        let bx0 = b.min_x;
        let by0 = b.min_y;
        let bx1 = b.max_x;
        let by1 = b.max_y;
        ax0 < bx1 && ax1 > bx0 && ay0 < by1 && ay1 > by0
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
struct JsonBoundsPx {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl JsonBoundsPx {
    fn from_vertices(vertices: &[TextVertex]) -> Option<Self> {
        let first = vertices.first()?;
        let mut min_x = first.pos_px[0];
        let mut max_x = first.pos_px[0];
        let mut min_y = first.pos_px[1];
        let mut max_y = first.pos_px[1];
        for v in vertices {
            let x = v.pos_px[0];
            let y = v.pos_px[1];
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
        Some(Self {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }
}

#[derive(Debug, serde::Serialize)]
struct JsonTextDrawDump {
    ordered_draw_ix: usize,
    atlas_kind: JsonAtlasKind,
    atlas_page: u16,
    paint_index: u32,
    uniform_index: u32,
    scissor: [u32; 4],
    first_vertex: u32,
    vertex_count: u32,
    bounds_px: Option<JsonBoundsPx>,
}

#[derive(Debug, serde::Serialize)]
struct JsonGlyphProbeDump {
    ordered_draw_ix: usize,
    atlas_kind: JsonAtlasKind,
    atlas_page: u16,
    paint_index: u32,
    uniform_index: u32,
    vertex_ix: u32,
    bounds_px: JsonBoundsPx,
    uv: [f32; 4],
    atlas_xywh: [u32; 4],
    glyph: Option<DebugGlyphAtlasLookup>,
}

#[derive(Debug, serde::Serialize)]
struct JsonRenderTextDump<'a> {
    schema_version: u32,
    frame_index: u64,
    viewport_size: [u32; 2],
    probe_px: Option<JsonProbeRect>,
    text_draws: &'a [JsonTextDrawDump],
    probe_hits: &'a [JsonGlyphProbeDump],
}

#[derive(Default)]
pub(super) struct RenderTextDumpState {
    text_draws: Vec<JsonTextDrawDump>,
    probe_hits: Vec<JsonGlyphProbeDump>,
    bytes: Vec<u8>,
}

fn atlas_kind_for_text_draw(kind: TextDrawKind) -> (JsonAtlasKind, GlyphQuadKind) {
    match kind {
        TextDrawKind::Mask | TextDrawKind::MaskOutline => {
            (JsonAtlasKind::Mask, GlyphQuadKind::Mask)
        }
        TextDrawKind::Color => (JsonAtlasKind::Color, GlyphQuadKind::Color),
        TextDrawKind::Subpixel | TextDrawKind::SubpixelOutline => {
            (JsonAtlasKind::Subpixel, GlyphQuadKind::Subpixel)
        }
    }
}

fn uv_to_atlas_xywh(u0: f32, v0: f32, u1: f32, v1: f32, w: u32, h: u32) -> [u32; 4] {
    let wf = w as f32;
    let hf = h as f32;
    if wf <= 0.0 || hf <= 0.0 {
        return [0, 0, 0, 0];
    }
    let x = (u0 * wf).round().max(0.0) as u32;
    let y = (v0 * hf).round().max(0.0) as u32;
    let ww = ((u1 - u0) * wf).round().max(0.0) as u32;
    let hh = ((v1 - v0) * hf).round().max(0.0) as u32;
    [x, y, ww, hh]
}

impl RenderTextDumpState {
    fn clear_scratch(&mut self) {
        self.text_draws.clear();
        self.probe_hits.clear();
        self.bytes.clear();
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn maybe_dump_render_text_json(
        &mut self,
        text_system: &crate::text::TextSystem,
        frame_index: u64,
        viewport_size: (u32, u32),
        encoding: &SceneEncoding,
    ) {
        if !should_dump_frame(frame_index) {
            return;
        }

        let probe_px = parse_env_probe_px("FRET_RENDER_TEXT_DUMP_PROBE_PX")
            .map(|(x, y, w, h)| JsonProbeRect { x, y, w, h });

        self.clear_scratch();

        for (ordered_draw_ix, draw) in encoding.ordered_draws.iter().enumerate() {
            let OrderedDraw::Text(draw) = draw else {
                continue;
            };

            let (atlas_kind_json, atlas_kind) = atlas_kind_for_text_draw(draw.kind);

            let first = draw.first_vertex as usize;
            let count = draw.vertex_count as usize;
            let end = first
                .saturating_add(count)
                .min(encoding.text_vertices.len());
            let vertices = &encoding.text_vertices[first..end];
            let bounds_px = JsonBoundsPx::from_vertices(vertices);

            self.text_draws.push(JsonTextDrawDump {
                ordered_draw_ix,
                atlas_kind: atlas_kind_json,
                atlas_page: draw.atlas_page,
                paint_index: draw.paint_index,
                uniform_index: draw.uniform_index,
                scissor: [
                    draw.scissor.x,
                    draw.scissor.y,
                    draw.scissor.w,
                    draw.scissor.h,
                ],
                first_vertex: draw.first_vertex,
                vertex_count: draw.vertex_count,
                bounds_px,
            });

            let Some(probe) = probe_px else {
                continue;
            };

            let (atlas_w, atlas_h) = text_system.debug_atlas_dims(atlas_kind);
            if draw.vertex_count < 6 {
                continue;
            }
            let glyph_count = (draw.vertex_count as usize) / 6;
            for g_ix in 0..glyph_count {
                let base = first.saturating_add(g_ix.saturating_mul(6));
                let end6 = base.saturating_add(6);
                if end6 > encoding.text_vertices.len() {
                    break;
                }
                let glyph_vs = &encoding.text_vertices[base..end6];
                let Some(glyph_bounds) = JsonBoundsPx::from_vertices(glyph_vs) else {
                    continue;
                };
                if !probe.intersects_bounds(&glyph_bounds) {
                    continue;
                }

                let u0 = glyph_vs[0].uv[0];
                let v0 = glyph_vs[0].uv[1];
                let u1 = glyph_vs[2].uv[0];
                let v1 = glyph_vs[2].uv[1];
                let atlas_xywh = uv_to_atlas_xywh(u0, v0, u1, v1, atlas_w, atlas_h);
                let glyph = text_system.debug_lookup_glyph_atlas_entry(
                    atlas_kind,
                    draw.atlas_page,
                    atlas_xywh[0],
                    atlas_xywh[1],
                    atlas_xywh[2],
                    atlas_xywh[3],
                );

                self.probe_hits.push(JsonGlyphProbeDump {
                    ordered_draw_ix,
                    atlas_kind: atlas_kind_json,
                    atlas_page: draw.atlas_page,
                    paint_index: draw.paint_index,
                    uniform_index: draw.uniform_index,
                    vertex_ix: (base as u32).saturating_sub(draw.first_vertex),
                    bounds_px: glyph_bounds,
                    uv: [u0, v0, u1, v1],
                    atlas_xywh,
                    glyph,
                });
            }
        }

        let dump = JsonRenderTextDump {
            schema_version: 1,
            frame_index,
            viewport_size: [viewport_size.0, viewport_size.1],
            probe_px,
            text_draws: &self.text_draws,
            probe_hits: &self.probe_hits,
        };

        let dir = dump_dir_from_env();
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join(format!("render_text.frame{frame_index}.json"));
        self.bytes.clear();
        if serde_json::to_writer_pretty(&mut self.bytes, &dump).is_ok() {
            let _ = std::fs::write(file, &self.bytes);
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(super) fn maybe_dump_render_text_json(
        &mut self,
        _text_system: &crate::text::TextSystem,
        _frame_index: u64,
        _viewport_size: (u32, u32),
        _encoding: &SceneEncoding,
    ) {
    }
}

impl crate::renderer::Renderer {
    pub(super) fn maybe_dump_render_text_json(
        &mut self,
        frame_index: u64,
        viewport_size: (u32, u32),
        encoding: &SceneEncoding,
    ) {
        let dump_state = &mut self.render_text_dump_state;
        let text_system = &self.text_system;
        dump_state.maybe_dump_render_text_json(text_system, frame_index, viewport_size, encoding);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_text_dump_state_clear_scratch_keeps_capacity() {
        let mut state = RenderTextDumpState::default();
        state.text_draws.push(JsonTextDrawDump {
            ordered_draw_ix: 0,
            atlas_kind: JsonAtlasKind::Mask,
            atlas_page: 0,
            paint_index: 0,
            uniform_index: 0,
            scissor: [0, 0, 1, 1],
            first_vertex: 0,
            vertex_count: 6,
            bounds_px: None,
        });
        state.probe_hits.push(JsonGlyphProbeDump {
            ordered_draw_ix: 0,
            atlas_kind: JsonAtlasKind::Mask,
            atlas_page: 0,
            paint_index: 0,
            uniform_index: 0,
            vertex_ix: 0,
            bounds_px: JsonBoundsPx {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 1.0,
                max_y: 1.0,
            },
            uv: [0.0, 0.0, 1.0, 1.0],
            atlas_xywh: [0, 0, 1, 1],
            glyph: None,
        });
        state.bytes.extend_from_slice(&[1, 2, 3]);

        let draws_cap = state.text_draws.capacity();
        let hits_cap = state.probe_hits.capacity();
        let bytes_cap = state.bytes.capacity();

        state.clear_scratch();

        assert!(state.text_draws.is_empty());
        assert!(state.probe_hits.is_empty());
        assert!(state.bytes.is_empty());
        assert!(state.text_draws.capacity() >= draws_cap);
        assert!(state.probe_hits.capacity() >= hits_cap);
        assert!(state.bytes.capacity() >= bytes_cap);
    }
}
