use super::*;

pub(super) fn map_scissor_to_size(
    scissor: Option<ScissorRect>,
    src_size: (u32, u32),
    dst_size: (u32, u32),
) -> Option<ScissorRect> {
    let scissor = scissor?;
    if scissor.w == 0 || scissor.h == 0 {
        return None;
    }

    let src_w = src_size.0.max(1);
    let src_h = src_size.1.max(1);
    let dst_w = dst_size.0.max(1);
    let dst_h = dst_size.1.max(1);

    let x0 = scissor.x;
    let y0 = scissor.y;
    let x1 = x0.saturating_add(scissor.w);
    let y1 = y0.saturating_add(scissor.h);

    let sx0 = x0.saturating_mul(dst_w) / src_w;
    let sy0 = y0.saturating_mul(dst_h) / src_h;
    let sx1 = x1.saturating_mul(dst_w).div_ceil(src_w);
    let sy1 = y1.saturating_mul(dst_h).div_ceil(src_h);

    let sx0 = sx0.min(dst_w);
    let sy0 = sy0.min(dst_h);
    let sx1 = sx1.min(dst_w);
    let sy1 = sy1.min(dst_h);

    if sx1 <= sx0 || sy1 <= sy0 {
        return None;
    }

    Some(ScissorRect {
        x: sx0,
        y: sy0,
        w: sx1 - sx0,
        h: sy1 - sy0,
    })
}

pub(super) fn map_scissor_downsample_nearest(
    scissor: Option<ScissorRect>,
    scale: u32,
    dst_size: (u32, u32),
) -> Option<ScissorRect> {
    let scissor = scissor?;
    if scissor.w == 0 || scissor.h == 0 {
        return None;
    }
    let scale = scale.max(1);
    if scale <= 1 {
        return map_scissor_to_size(Some(scissor), dst_size, dst_size);
    }

    let dst_w = dst_size.0.max(1);
    let dst_h = dst_size.1.max(1);

    let x0 = scissor.x;
    let y0 = scissor.y;
    let x1 = x0.saturating_add(scissor.w);
    let y1 = y0.saturating_add(scissor.h);

    let sx0 = x0 / scale;
    let sy0 = y0 / scale;
    let sx1 = x1.div_ceil(scale);
    let sy1 = y1.div_ceil(scale);

    let sx0 = sx0.min(dst_w);
    let sy0 = sy0.min(dst_h);
    let sx1 = sx1.min(dst_w);
    let sy1 = sy1.min(dst_h);

    if sx1 <= sx0 || sy1 <= sy0 {
        return None;
    }

    Some(ScissorRect {
        x: sx0,
        y: sy0,
        w: sx1 - sx0,
        h: sy1 - sy0,
    })
}
