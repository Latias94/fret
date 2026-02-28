use fret_core::{ColorRange, RectPx, YuvMatrix};

fn rect_or_full(width: u32, height: u32, update_rect_px: Option<RectPx>) -> RectPx {
    update_rect_px.unwrap_or_else(|| RectPx::full(width, height))
}

fn rect_end(value: u32, extent: u32) -> Option<u32> {
    value.checked_add(extent)
}

fn clamp_rect_to_frame(rect: RectPx, width: u32, height: u32) -> Option<RectPx> {
    let x1 = rect_end(rect.x, rect.w)?;
    let y1 = rect_end(rect.y, rect.h)?;
    if rect.w == 0 || rect.h == 0 || rect.x >= width || rect.y >= height {
        return None;
    }
    if x1 > width || y1 > height {
        return None;
    }
    Some(rect)
}

fn expand_rect_to_even(rect: RectPx, width: u32, height: u32) -> Option<RectPx> {
    let x1 = rect_end(rect.x, rect.w)?;
    let y1 = rect_end(rect.y, rect.h)?;
    let x0 = rect.x & !1;
    let y0 = rect.y & !1;
    let mut x1 = (x1 + 1) & !1;
    let mut y1 = (y1 + 1) & !1;
    x1 = x1.min(width);
    y1 = y1.min(height);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some(RectPx::new(x0, y0, x1 - x0, y1 - y0))
}

fn chroma_dims_420(width: u32, height: u32) -> (u32, u32) {
    (width.div_ceil(2), height.div_ceil(2))
}

#[doc(hidden)]
pub fn normalize_update_rect_420(
    width: u32,
    height: u32,
    update_rect_px: Option<RectPx>,
) -> Result<RectPx, String> {
    let rect = rect_or_full(width, height, update_rect_px);
    let rect = clamp_rect_to_frame(rect, width, height)
        .ok_or_else(|| "invalid update_rect".to_string())?;
    expand_rect_to_even(rect, width, height).ok_or_else(|| "invalid update_rect".to_string())
}

fn yuv_coeffs(matrix: YuvMatrix) -> (f32, f32, f32, f32, f32) {
    // (rv, gu, gv, bu, vu_scale) where:
    // r = y + rv * v
    // g = y - gu * u - gv * v
    // b = y + bu * u
    // y,u,v are normalized into y∈[0,1], u/v∈[-0.5,0.5]
    match matrix {
        YuvMatrix::Bt601 => (1.402_0, 0.344_136, 0.714_136, 1.772_0, 1.0),
        YuvMatrix::Bt709 => (1.574_8, 0.187_324, 0.468_124, 1.855_6, 1.0),
        YuvMatrix::Bt2020 => (1.474_6, 0.164_55, 0.571_35, 1.881_4, 1.0),
    }
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn yuv_to_rgb(y: u8, u: u8, v: u8, range: ColorRange, matrix: YuvMatrix) -> (u8, u8, u8) {
    let y = y as f32;
    let u = u as f32;
    let v = v as f32;

    let (y, u, v) = match range {
        ColorRange::Full => {
            let y = y / 255.0;
            let u = (u - 128.0) / 255.0;
            let v = (v - 128.0) / 255.0;
            (y, u, v)
        }
        ColorRange::Limited => {
            // ITU-R BT.601/709/2020 limited range conventions:
            // Y: [16, 235] -> [0, 1]
            // UV: [16, 240] -> [-0.5, 0.5]
            let y = (y - 16.0) / 219.0;
            let u = (u - 128.0) / 224.0;
            let v = (v - 128.0) / 224.0;
            (y, u, v)
        }
    };

    let (rv, gu, gv, bu, _vu_scale) = yuv_coeffs(matrix);
    let r = clamp01(y + rv * v);
    let g = clamp01(y - gu * u - gv * v);
    let b = clamp01(y + bu * u);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

#[doc(hidden)]
pub fn nv12_to_rgba8_rect(input: Nv12ToRgba8RectInput<'_>) -> Result<(RectPx, Vec<u8>), String> {
    let Nv12ToRgba8RectInput {
        width,
        height,
        update_rect_px,
        y_bytes_per_row,
        y_plane,
        uv_bytes_per_row,
        uv_plane,
        range,
        matrix,
    } = input;

    let rect = normalize_update_rect_420(width, height, update_rect_px)?;

    let (cw, ch) = chroma_dims_420(width, height);
    let y_min_bpr = width;
    let uv_min_bpr = cw.saturating_mul(2);
    if y_bytes_per_row < y_min_bpr || uv_bytes_per_row < uv_min_bpr {
        return Err("undersized bytes_per_row for nv12 planes".to_string());
    }

    let y_expected = (y_bytes_per_row as usize).saturating_mul(height as usize);
    let uv_expected = (uv_bytes_per_row as usize).saturating_mul(ch as usize);
    if y_plane.len() != y_expected || uv_plane.len() != uv_expected {
        return Err("invalid nv12 plane byte length".to_string());
    }

    let out_w = rect.w as usize;
    let out_h = rect.h as usize;
    let mut out = vec![0u8; out_w.saturating_mul(out_h).saturating_mul(4)];

    for yy in 0..rect.h {
        let y_src = rect.y + yy;
        let uv_src = y_src / 2;
        let y_row = (y_src as usize).saturating_mul(y_bytes_per_row as usize);
        let uv_row = (uv_src as usize).saturating_mul(uv_bytes_per_row as usize);

        for xx in 0..rect.w {
            let x_src = rect.x + xx;
            let uv_x = x_src / 2;

            let yv = y_plane[y_row + x_src as usize];
            let uv_off = uv_row + (uv_x as usize).saturating_mul(2);
            let u = uv_plane[uv_off];
            let v = uv_plane[uv_off + 1];
            let (r, g, b) = yuv_to_rgb(yv, u, v, range, matrix);

            let dst_px = (yy as usize)
                .saturating_mul(out_w)
                .saturating_add(xx as usize)
                .saturating_mul(4);
            out[dst_px] = r;
            out[dst_px + 1] = g;
            out[dst_px + 2] = b;
            out[dst_px + 3] = 255;
        }
    }

    Ok((rect, out))
}

#[doc(hidden)]
pub struct Nv12ToRgba8RectInput<'a> {
    pub width: u32,
    pub height: u32,
    pub update_rect_px: Option<RectPx>,
    pub y_bytes_per_row: u32,
    pub y_plane: &'a [u8],
    pub uv_bytes_per_row: u32,
    pub uv_plane: &'a [u8],
    pub range: ColorRange,
    pub matrix: YuvMatrix,
}

#[doc(hidden)]
pub fn i420_to_rgba8_rect(input: I420ToRgba8RectInput<'_>) -> Result<(RectPx, Vec<u8>), String> {
    let I420ToRgba8RectInput {
        width,
        height,
        update_rect_px,
        y_bytes_per_row,
        y_plane,
        u_bytes_per_row,
        u_plane,
        v_bytes_per_row,
        v_plane,
        range,
        matrix,
    } = input;

    let rect = normalize_update_rect_420(width, height, update_rect_px)?;

    let (cw, ch) = chroma_dims_420(width, height);
    if y_bytes_per_row < width || u_bytes_per_row < cw || v_bytes_per_row < cw {
        return Err("undersized bytes_per_row for i420 planes".to_string());
    }

    let y_expected = (y_bytes_per_row as usize).saturating_mul(height as usize);
    let u_expected = (u_bytes_per_row as usize).saturating_mul(ch as usize);
    let v_expected = (v_bytes_per_row as usize).saturating_mul(ch as usize);
    if y_plane.len() != y_expected || u_plane.len() != u_expected || v_plane.len() != v_expected {
        return Err("invalid i420 plane byte length".to_string());
    }

    let out_w = rect.w as usize;
    let out_h = rect.h as usize;
    let mut out = vec![0u8; out_w.saturating_mul(out_h).saturating_mul(4)];

    for yy in 0..rect.h {
        let y_src = rect.y + yy;
        let uv_src = y_src / 2;
        let y_row = (y_src as usize).saturating_mul(y_bytes_per_row as usize);
        let u_row = (uv_src as usize).saturating_mul(u_bytes_per_row as usize);
        let v_row = (uv_src as usize).saturating_mul(v_bytes_per_row as usize);

        for xx in 0..rect.w {
            let x_src = rect.x + xx;
            let uv_x = x_src / 2;

            let yv = y_plane[y_row + x_src as usize];
            let u = u_plane[u_row + uv_x as usize];
            let v = v_plane[v_row + uv_x as usize];
            let (r, g, b) = yuv_to_rgb(yv, u, v, range, matrix);

            let dst_px = (yy as usize)
                .saturating_mul(out_w)
                .saturating_add(xx as usize)
                .saturating_mul(4);
            out[dst_px] = r;
            out[dst_px + 1] = g;
            out[dst_px + 2] = b;
            out[dst_px + 3] = 255;
        }
    }

    Ok((rect, out))
}

#[doc(hidden)]
pub struct I420ToRgba8RectInput<'a> {
    pub width: u32,
    pub height: u32,
    pub update_rect_px: Option<RectPx>,
    pub y_bytes_per_row: u32,
    pub y_plane: &'a [u8],
    pub u_bytes_per_row: u32,
    pub u_plane: &'a [u8],
    pub v_bytes_per_row: u32,
    pub v_plane: &'a [u8],
    pub range: ColorRange,
    pub matrix: YuvMatrix,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_rect_expands_to_even_boundaries() {
        let rect = normalize_update_rect_420(10, 10, Some(RectPx::new(1, 1, 3, 3))).unwrap();
        assert_eq!(rect, RectPx::new(0, 0, 4, 4));
    }

    #[test]
    fn nv12_conversion_produces_rgba_bytes() {
        let width = 4;
        let height = 4;
        let y_bpr = width;
        let (cw, ch) = chroma_dims_420(width, height);
        let uv_bpr = cw * 2;

        let y_plane = vec![128u8; (y_bpr * height) as usize];
        let uv_plane = vec![128u8; (uv_bpr * ch) as usize];

        let (rect, rgba) = nv12_to_rgba8_rect(Nv12ToRgba8RectInput {
            width,
            height,
            update_rect_px: None,
            y_bytes_per_row: y_bpr,
            y_plane: &y_plane,
            uv_bytes_per_row: uv_bpr,
            uv_plane: &uv_plane,
            range: ColorRange::Full,
            matrix: YuvMatrix::Bt709,
        })
        .unwrap();
        assert_eq!(rect, RectPx::full(width, height));
        assert_eq!(rgba.len(), (width * height * 4) as usize);
        assert!(rgba.chunks_exact(4).all(|px| px[3] == 255));
    }

    #[test]
    fn i420_conversion_produces_rgba_bytes() {
        let width = 4;
        let height = 4;
        let y_bpr = width;
        let (cw, ch) = chroma_dims_420(width, height);

        let y_plane = vec![128u8; (y_bpr * height) as usize];
        let u_plane = vec![128u8; (cw * ch) as usize];
        let v_plane = vec![128u8; (cw * ch) as usize];

        let (rect, rgba) = i420_to_rgba8_rect(I420ToRgba8RectInput {
            width,
            height,
            update_rect_px: None,
            y_bytes_per_row: y_bpr,
            y_plane: &y_plane,
            u_bytes_per_row: cw,
            u_plane: &u_plane,
            v_bytes_per_row: cw,
            v_plane: &v_plane,
            range: ColorRange::Full,
            matrix: YuvMatrix::Bt709,
        })
        .unwrap();
        assert_eq!(rect, RectPx::full(width, height));
        assert_eq!(rgba.len(), (width * height * 4) as usize);
        assert!(rgba.chunks_exact(4).all(|px| px[3] == 255));
    }
}
