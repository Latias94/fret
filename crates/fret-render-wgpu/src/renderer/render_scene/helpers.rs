use super::super::*;

pub(super) fn set_scissor_rect_absolute(
    rp: &mut wgpu::RenderPass<'_>,
    scissor: ScissorRect,
    dst_origin: (u32, u32),
    dst_size: (u32, u32),
) -> bool {
    if scissor.w == 0 || scissor.h == 0 || dst_size.0 == 0 || dst_size.1 == 0 {
        return false;
    }

    let x0 = scissor.x;
    let y0 = scissor.y;
    let x1 = scissor.x.saturating_add(scissor.w);
    let y1 = scissor.y.saturating_add(scissor.h);

    let lx0 = x0.saturating_sub(dst_origin.0).min(dst_size.0);
    let ly0 = y0.saturating_sub(dst_origin.1).min(dst_size.1);
    let lx1 = x1.saturating_sub(dst_origin.0).min(dst_size.0);
    let ly1 = y1.saturating_sub(dst_origin.1).min(dst_size.1);

    let w = lx1.saturating_sub(lx0);
    let h = ly1.saturating_sub(ly0);
    if w == 0 || h == 0 {
        return false;
    }

    rp.set_scissor_rect(lx0, ly0, w, h);
    true
}
