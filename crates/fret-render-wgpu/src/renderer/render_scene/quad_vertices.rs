use super::super::*;

pub(super) fn build_plan_quad_vertices_into(
    plan: &RenderPlan,
    viewport_size: (u32, u32),
    vertices: &mut Vec<ViewportVertex>,
    bases: &mut Vec<Option<u32>>,
) {
    vertices.clear();
    bases.clear();
    bases.resize(plan.passes.len(), None);

    for (pass_index, planned_pass) in plan.passes.iter().enumerate() {
        match planned_pass {
            RenderPlanPass::PathMsaaBatch(path_pass) => {
                let union = path_pass.union_scissor.0;
                if union.w == 0 || union.h == 0 {
                    continue;
                }

                let x0 = union.x as f32;
                let y0 = union.y as f32;
                let x1 = (union.x + union.w) as f32;
                let y1 = (union.y + union.h) as f32;

                let vw = viewport_size.0.max(1) as f32;
                let vh = viewport_size.1.max(1) as f32;
                let u0 = x0 / vw;
                let v0 = y0 / vh;
                let u1 = x1 / vw;
                let v1 = y1 / vh;

                let base = vertices.len().min(u32::MAX as usize) as u32;
                bases[pass_index] = Some(base);
                vertices.extend_from_slice(&[
                    ViewportVertex {
                        pos_px: [x0, y0],
                        uv: [u0, v0],
                        opacity: 1.0,
                        _pad: [0.0; 3],
                    },
                    ViewportVertex {
                        pos_px: [x1, y0],
                        uv: [u1, v0],
                        opacity: 1.0,
                        _pad: [0.0; 3],
                    },
                    ViewportVertex {
                        pos_px: [x1, y1],
                        uv: [u1, v1],
                        opacity: 1.0,
                        _pad: [0.0; 3],
                    },
                    ViewportVertex {
                        pos_px: [x0, y0],
                        uv: [u0, v0],
                        opacity: 1.0,
                        _pad: [0.0; 3],
                    },
                    ViewportVertex {
                        pos_px: [x1, y1],
                        uv: [u1, v1],
                        opacity: 1.0,
                        _pad: [0.0; 3],
                    },
                    ViewportVertex {
                        pos_px: [x0, y1],
                        uv: [u0, v1],
                        opacity: 1.0,
                        _pad: [0.0; 3],
                    },
                ]);
            }
            RenderPlanPass::CompositePremul(pass) => {
                let (x0, y0, x1, y1) = if let Some(scissor) = pass.dst_scissor.map(|s| s.0) {
                    (
                        scissor.x as f32,
                        scissor.y as f32,
                        (scissor.x + scissor.w) as f32,
                        (scissor.y + scissor.h) as f32,
                    )
                } else {
                    let ox = pass.dst_origin.0 as f32;
                    let oy = pass.dst_origin.1 as f32;
                    (
                        ox,
                        oy,
                        ox + pass.dst_size.0 as f32,
                        oy + pass.dst_size.1 as f32,
                    )
                };

                let src_ox = pass.src_origin.0 as f32;
                let src_oy = pass.src_origin.1 as f32;
                let src_w = pass.src_size.0.max(1) as f32;
                let src_h = pass.src_size.1.max(1) as f32;
                let u0 = ((x0 - src_ox) / src_w).clamp(0.0, 1.0);
                let v0 = ((y0 - src_oy) / src_h).clamp(0.0, 1.0);
                let u1 = ((x1 - src_ox) / src_w).clamp(0.0, 1.0);
                let v1 = ((y1 - src_oy) / src_h).clamp(0.0, 1.0);

                let base = vertices.len().min(u32::MAX as usize) as u32;
                bases[pass_index] = Some(base);
                let opacity = pass.opacity.clamp(0.0, 1.0);
                vertices.extend_from_slice(&[
                    ViewportVertex {
                        pos_px: [x0, y0],
                        uv: [u0, v0],
                        opacity,
                        _pad: [0.0; 3],
                    },
                    ViewportVertex {
                        pos_px: [x1, y0],
                        uv: [u1, v0],
                        opacity,
                        _pad: [0.0; 3],
                    },
                    ViewportVertex {
                        pos_px: [x1, y1],
                        uv: [u1, v1],
                        opacity,
                        _pad: [0.0; 3],
                    },
                    ViewportVertex {
                        pos_px: [x0, y0],
                        uv: [u0, v0],
                        opacity,
                        _pad: [0.0; 3],
                    },
                    ViewportVertex {
                        pos_px: [x1, y1],
                        uv: [u1, v1],
                        opacity,
                        _pad: [0.0; 3],
                    },
                    ViewportVertex {
                        pos_px: [x0, y1],
                        uv: [u0, v1],
                        opacity,
                        _pad: [0.0; 3],
                    },
                ]);
            }
            _ => {}
        }
    }
}

pub(super) fn upload_plan_quad_vertices(
    renderer: &mut Renderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    plan: &RenderPlan,
    viewport_size: (u32, u32),
) -> Vec<Option<u32>> {
    let mut vertices = std::mem::take(&mut renderer.plan_quad_vertices_scratch);
    let mut bases = std::mem::take(&mut renderer.plan_quad_vertex_bases_scratch);
    build_plan_quad_vertices_into(plan, viewport_size, &mut vertices, &mut bases);

    if !vertices.is_empty() {
        renderer.ensure_path_composite_vertex_buffer(device, vertices.len());
        queue.write_buffer(
            renderer.path_composite_vertices_ref(),
            0,
            bytemuck::cast_slice(&vertices),
        );
    }

    vertices.clear();
    renderer.plan_quad_vertices_scratch = vertices;
    bases
}
