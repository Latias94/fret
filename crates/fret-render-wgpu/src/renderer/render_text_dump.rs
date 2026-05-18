use super::debug_dump_gate::{DumpFrameEnv, emit_dump_file, should_emit_dump_frame};
use super::{OrderedDraw, SceneEncoding, TextDrawKind, TextGlyphInstance, ViewportUniform};
use crate::text::DebugGlyphAtlasLookup;
use std::sync::atomic::AtomicBool;

const RENDER_TEXT_DUMP_ENV: DumpFrameEnv = DumpFrameEnv::new(
    "FRET_RENDER_TEXT_DUMP",
    "FRET_RENDER_TEXT_DUMP_FRAME",
    "FRET_RENDER_TEXT_DUMP_AFTER_FRAMES",
    "FRET_RENDER_TEXT_DUMP_EVERY",
    "FRET_RENDER_TEXT_DUMP_DIR",
    "render_text",
);
static RENDER_TEXT_DUMPED: AtomicBool = AtomicBool::new(false);

fn parse_env_probe_px(name: &str) -> Option<(f32, f32, f32, f32)> {
    let v = std::env::var(name).ok()?;
    let mut it = v.split([',', ' ']).filter(|s| !s.is_empty());
    let x = it.next()?.parse::<f32>().ok()?;
    let y = it.next()?.parse::<f32>().ok()?;
    let w = it.next()?.parse::<f32>().ok()?;
    let h = it.next()?.parse::<f32>().ok()?;
    Some((x, y, w, h))
}

fn should_dump_frame(frame_index: u64) -> bool {
    should_emit_dump_frame(frame_index, RENDER_TEXT_DUMP_ENV, &RENDER_TEXT_DUMPED)
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
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
    fn from_instance(uniform: &ViewportUniform, instance: &TextGlyphInstance) -> Option<Self> {
        let rect = instance.local_rect;
        let pts = [
            transform_text_point(uniform, rect[0], rect[1]),
            transform_text_point(uniform, rect[2], rect[1]),
            transform_text_point(uniform, rect[2], rect[3]),
            transform_text_point(uniform, rect[0], rect[3]),
        ];
        let first = pts.first()?;
        let mut min_x = first.0;
        let mut max_x = first.0;
        let mut min_y = first.1;
        let mut max_y = first.1;
        for (x, y) in pts {
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

fn transform_text_point(uniform: &ViewportUniform, x: f32, y: f32) -> (f32, f32) {
    let row0 = uniform.text_transform0;
    let row1 = uniform.text_transform1;
    (
        row0[0] * x + row0[1] * y + row0[2],
        row1[0] * x + row1[1] * y + row1[2],
    )
}

#[derive(Debug, serde::Serialize)]
struct JsonTextDrawDump {
    ordered_draw_ix: usize,
    atlas_kind: JsonAtlasKind,
    atlas_page: u16,
    paint_index: u32,
    uniform_index: u32,
    scissor: [u32; 4],
    first_instance: u32,
    instance_count: u32,
    bounds_px: Option<JsonBoundsPx>,
}

#[derive(Debug, serde::Serialize)]
struct JsonGlyphProbeDump {
    ordered_draw_ix: usize,
    atlas_kind: JsonAtlasKind,
    atlas_page: u16,
    paint_index: u32,
    uniform_index: u32,
    instance_ix: u32,
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

fn atlas_kind_for_text_draw(kind: TextDrawKind) -> JsonAtlasKind {
    match kind {
        TextDrawKind::Mask | TextDrawKind::MaskOutline => JsonAtlasKind::Mask,
        TextDrawKind::Color => JsonAtlasKind::Color,
        TextDrawKind::Subpixel | TextDrawKind::SubpixelOutline => JsonAtlasKind::Subpixel,
    }
}

fn atlas_dims_for_text_draw(
    text_system: &crate::text::TextSystem,
    kind: TextDrawKind,
) -> (u32, u32) {
    match kind {
        TextDrawKind::Mask | TextDrawKind::MaskOutline => text_system.debug_mask_atlas_dims(),
        TextDrawKind::Color => text_system.debug_color_atlas_dims(),
        TextDrawKind::Subpixel | TextDrawKind::SubpixelOutline => {
            text_system.debug_subpixel_atlas_dims()
        }
    }
}

fn lookup_glyph_atlas_entry_for_text_draw(
    text_system: &crate::text::TextSystem,
    kind: TextDrawKind,
    page: u16,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Option<DebugGlyphAtlasLookup> {
    match kind {
        TextDrawKind::Mask | TextDrawKind::MaskOutline => {
            text_system.debug_lookup_mask_glyph_atlas_entry(page, x, y, w, h)
        }
        TextDrawKind::Color => text_system.debug_lookup_color_glyph_atlas_entry(page, x, y, w, h),
        TextDrawKind::Subpixel | TextDrawKind::SubpixelOutline => {
            text_system.debug_lookup_subpixel_glyph_atlas_entry(page, x, y, w, h)
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

            let atlas_kind_json = atlas_kind_for_text_draw(draw.kind);
            let Some(uniform) = encoding.uniforms.get(draw.uniform_index as usize) else {
                continue;
            };

            let first = draw.first_instance as usize;
            let count = draw.instance_count as usize;
            let end = first
                .saturating_add(count)
                .min(encoding.text_glyph_instances.len());
            let instances = &encoding.text_glyph_instances[first..end];
            let bounds_px = instances.first().and_then(|_| {
                instances
                    .iter()
                    .filter_map(|instance| JsonBoundsPx::from_instance(uniform, instance))
                    .reduce(|mut a, b| {
                        a.min_x = a.min_x.min(b.min_x);
                        a.min_y = a.min_y.min(b.min_y);
                        a.max_x = a.max_x.max(b.max_x);
                        a.max_y = a.max_y.max(b.max_y);
                        a
                    })
            });

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
                first_instance: draw.first_instance,
                instance_count: draw.instance_count,
                bounds_px,
            });

            let Some(probe) = probe_px else {
                continue;
            };

            let (atlas_w, atlas_h) = atlas_dims_for_text_draw(text_system, draw.kind);
            if draw.instance_count == 0 {
                continue;
            }
            for (g_ix, instance) in instances.iter().enumerate() {
                let Some(glyph_bounds) = JsonBoundsPx::from_instance(uniform, instance) else {
                    continue;
                };
                if !probe.intersects_bounds(&glyph_bounds) {
                    continue;
                }

                let u0 = instance.uv[0];
                let v0 = instance.uv[1];
                let u1 = instance.uv[2];
                let v1 = instance.uv[3];
                let atlas_xywh = uv_to_atlas_xywh(u0, v0, u1, v1, atlas_w, atlas_h);
                let glyph = lookup_glyph_atlas_entry_for_text_draw(
                    text_system,
                    draw.kind,
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
                    instance_ix: (first.saturating_add(g_ix)) as u32,
                    bounds_px: glyph_bounds,
                    uv: [u0, v0, u1, v1],
                    atlas_xywh,
                    glyph,
                });
            }
        }

        let dump = JsonRenderTextDump {
            schema_version: 2,
            frame_index,
            viewport_size: [viewport_size.0, viewport_size.1],
            probe_px,
            text_draws: &self.text_draws,
            probe_hits: &self.probe_hits,
        };

        self.bytes.clear();
        if serde_json::to_writer_pretty(&mut self.bytes, &dump).is_ok() {
            emit_dump_file(
                RENDER_TEXT_DUMP_ENV,
                format!("render_text.frame{frame_index}.json"),
                &self.bytes,
            );
        }
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
            first_instance: 0,
            instance_count: 1,
            bounds_px: None,
        });
        state.probe_hits.push(JsonGlyphProbeDump {
            ordered_draw_ix: 0,
            atlas_kind: JsonAtlasKind::Mask,
            atlas_page: 0,
            paint_index: 0,
            uniform_index: 0,
            instance_ix: 0,
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
