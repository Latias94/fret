use fret_core::{Color, Corners, DrawOrder, Paint, Point, Px, Rect, Scene, SceneOp, Size};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ShadowInsets {
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) right: f32,
    pub(crate) bottom: f32,
}

pub(crate) fn split_box_shadow_layers(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0_u32;
    let mut start = 0_usize;
    for (idx, ch) in s.char_indices() {
        match ch {
            '(' => depth = depth.saturating_add(1),
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                out.push(s[start..idx].trim());
                start = idx + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        out.push(s[start..].trim());
    }
    out.into_iter().filter(|p| !p.is_empty()).collect()
}

fn parse_px(s: &str) -> Option<f32> {
    let s = s.trim();
    let v = s.strip_suffix("px").unwrap_or(s);
    v.parse::<f32>().ok()
}

pub(crate) fn parse_box_shadow_layer(layer: &str) -> Option<(String, f32, f32, f32, f32)> {
    let layer = layer.trim();
    if layer.is_empty() || layer == "none" {
        return None;
    }

    let (color, rest) = if layer.starts_with('#') {
        let mut it = layer.splitn(2, char::is_whitespace);
        let color = it.next()?.trim().to_string();
        (color, it.next().unwrap_or("").trim())
    } else if let Some(paren) = layer.find('(') {
        let mut depth = 0_u32;
        let mut end = None;
        for (idx, ch) in layer.char_indices().skip(paren) {
            match ch {
                '(' => depth = depth.saturating_add(1),
                ')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        end = Some(idx);
                        break;
                    }
                }
                _ => {}
            }
        }
        let end = end?;
        let color = layer[..=end].trim().to_string();
        (color, layer[end + 1..].trim())
    } else {
        let mut it = layer.splitn(2, char::is_whitespace);
        let color = it.next()?.trim().to_string();
        (color, it.next().unwrap_or("").trim())
    };

    let parts: Vec<&str> = rest.split_whitespace().filter(|p| !p.is_empty()).collect();
    if parts.len() < 4 {
        return None;
    }
    let x = parse_px(parts[0])?;
    let y = parse_px(parts[1])?;
    let blur = parse_px(parts[2])?;
    let spread = parse_px(parts[3])?;
    Some((color, x, y, blur, spread))
}

pub(crate) fn shadow_insets_for_rect(panel: Rect, shadow: Rect) -> ShadowInsets {
    let panel_right = panel.origin.x.0 + panel.size.width.0;
    let panel_bottom = panel.origin.y.0 + panel.size.height.0;
    let shadow_right = shadow.origin.x.0 + shadow.size.width.0;
    let shadow_bottom = shadow.origin.y.0 + shadow.size.height.0;

    ShadowInsets {
        left: shadow.origin.x.0 - panel.origin.x.0,
        top: shadow.origin.y.0 - panel.origin.y.0,
        right: shadow_right - panel_right,
        bottom: shadow_bottom - panel_bottom,
    }
}

pub(crate) fn shadow_insets_for_box_shadow_layer(
    x: f32,
    y: f32,
    blur: f32,
    spread: f32,
) -> ShadowInsets {
    let delta = spread + blur;
    ShadowInsets {
        left: x - delta,
        top: y - delta,
        right: x + delta,
        bottom: y + delta,
    }
}

pub(crate) fn shadow_insets_score(a: ShadowInsets, b: ShadowInsets) -> f32 {
    (a.left - b.left).abs()
        + (a.top - b.top).abs()
        + (a.right - b.right).abs()
        + (a.bottom - b.bottom).abs()
}

pub(crate) fn shadow_insets_from_box_shadow(
    box_shadow: &str,
    mut color_alpha: impl FnMut(&str) -> Option<f32>,
) -> Vec<ShadowInsets> {
    if box_shadow.is_empty() || box_shadow == "none" {
        return Vec::new();
    }

    let mut out = Vec::new();
    for layer in split_box_shadow_layers(box_shadow) {
        let Some((color, x, y, blur, spread)) = parse_box_shadow_layer(layer) else {
            continue;
        };
        if color_alpha(&color).is_some_and(|alpha| alpha <= 0.01) {
            continue;
        }
        if x.abs() <= 0.01 && y.abs() <= 0.01 && blur.abs() <= 0.01 && spread.abs() <= 0.01 {
            continue;
        }
        out.push(shadow_insets_for_box_shadow_layer(x, y, blur, spread));
    }
    out
}

pub(crate) fn maybe_dump_shadow_candidates(
    label: &str,
    expected: &[ShadowInsets],
    candidates: &[ShadowInsets],
) {
    if std::env::var("FRET_DEBUG_SHADOW_INSETS").is_err() {
        return;
    }
    eprintln!("-- shadow insets debug: {label}");
    eprintln!("expected: {expected:?}");
    let mut sorted = candidates.to_vec();
    sorted.sort_by(|a, b| a.top.partial_cmp(&b.top).unwrap_or(Ordering::Equal));
    for (idx, cand) in sorted.iter().take(16).enumerate() {
        eprintln!("cand[{idx}] {cand:?}");
    }
}

pub(crate) fn assert_shadow_insets_match(
    web_name: &str,
    web_theme_name: &str,
    expected: &[ShadowInsets],
    candidates: &[ShadowInsets],
) {
    assert!(
        !expected.is_empty(),
        "{web_name} {web_theme_name}: web golden did not expose any drop shadow layers"
    );
    assert!(
        candidates.len() >= expected.len(),
        "{web_name} {web_theme_name}: not enough shadow candidates (expected ≥{}, got {})",
        expected.len(),
        candidates.len()
    );

    let chosen: Vec<ShadowInsets> = match expected.len() {
        1 => {
            let exp = expected[0];
            let mut best = candidates[0];
            let mut best_score = f32::INFINITY;
            for cand in candidates {
                let score = shadow_insets_score(*cand, exp);
                if score < best_score {
                    best_score = score;
                    best = *cand;
                }
            }
            vec![best]
        }
        2 => {
            let exp0 = expected[0];
            let exp1 = expected[1];
            let mut best0 = candidates[0];
            let mut best1 = candidates[1];
            let mut best_score = f32::INFINITY;

            for (i, cand0) in candidates.iter().enumerate() {
                for (j, cand1) in candidates.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let score =
                        shadow_insets_score(*cand0, exp0) + shadow_insets_score(*cand1, exp1);
                    if score < best_score {
                        best_score = score;
                        best0 = *cand0;
                        best1 = *cand1;
                    }
                }
            }

            vec![best0, best1]
        }
        n => panic!("{web_name} {web_theme_name}: unsupported shadow layer count {n}"),
    };

    let tol = 1.0;
    for (idx, (exp, act)) in expected.iter().zip(chosen.iter()).enumerate() {
        let assert_one = |edge: &str, actual: f32, expected: f32| {
            let delta = (actual - expected).abs();
            assert!(
                delta <= tol,
                "{web_name} {web_theme_name} shadow[{idx}] {edge}: expected≈{expected} (±{tol}) got={actual} (Δ={delta})"
            );
        };
        assert_one("left", act.left, exp.left);
        assert_one("top", act.top, exp.top);
        assert_one("right", act.right, exp.right);
        assert_one("bottom", act.bottom, exp.bottom);
    }
}

fn rect_area(rect: Rect) -> f32 {
    rect.size.width.0.max(0.0) * rect.size.height.0.max(0.0)
}

fn rect_intersection_area(a: Rect, b: Rect) -> f32 {
    let x0 = a.origin.x.0.max(b.origin.x.0);
    let y0 = a.origin.y.0.max(b.origin.y.0);
    let x1 = (a.origin.x.0 + a.size.width.0).min(b.origin.x.0 + b.size.width.0);
    let y1 = (a.origin.y.0 + a.size.height.0).min(b.origin.y.0 + b.size.height.0);
    let w = (x1 - x0).max(0.0);
    let h = (y1 - y0).max(0.0);
    w * h
}

fn rect_expand(rect: Rect, delta: f32) -> Rect {
    if delta >= 0.0 {
        Rect::new(
            Point::new(Px(rect.origin.x.0 - delta), Px(rect.origin.y.0 - delta)),
            Size::new(
                Px(rect.size.width.0 + delta * 2.0),
                Px(rect.size.height.0 + delta * 2.0),
            ),
        )
    } else {
        let d = -delta;
        Rect::new(
            Point::new(Px(rect.origin.x.0 + d), Px(rect.origin.y.0 + d)),
            Size::new(
                Px((rect.size.width.0 - d * 2.0).max(0.0)),
                Px((rect.size.height.0 - d * 2.0).max(0.0)),
            ),
        )
    }
}

fn shadow_rrect_outer_rect(rect: Rect, offset: Point, spread: f32, blur_radius: f32) -> Rect {
    let mut outer = rect_expand(rect, spread + blur_radius.max(0.0));
    outer.origin.x = Px(outer.origin.x.0 + offset.x.0);
    outer.origin.y = Px(outer.origin.y.0 + offset.y.0);
    outer
}

fn paint_solid_color(paint: fret_core::scene::PaintBindingV1) -> Color {
    match paint.paint {
        Paint::Solid(color) => color,
        _ => Color::TRANSPARENT,
    }
}

fn has_border(border: &[f32; 4]) -> bool {
    border.iter().any(|v| *v > 0.01)
}

pub(crate) fn fret_drop_shadow_insets_candidates(
    scene: &Scene,
    panel_rect: Rect,
) -> Vec<ShadowInsets> {
    let panel_area = rect_area(panel_rect).max(1.0);
    let mut out = Vec::new();

    for op in scene.ops() {
        match *op {
            SceneOp::Quad {
                rect,
                background,
                border,
                ..
            } => {
                let background = paint_solid_color(background);
                let border = [border.top.0, border.right.0, border.bottom.0, border.left.0];
                if has_border(&border) {
                    continue;
                }
                if background.a <= 0.0001 || background.a >= 0.95 {
                    continue;
                }
                if rect_intersection_area(rect, panel_rect) / panel_area <= 0.01 {
                    continue;
                }
                let insets = shadow_insets_for_rect(panel_rect, rect);
                let extends_outside = insets.left < -0.01
                    || insets.top < -0.01
                    || insets.right > 0.01
                    || insets.bottom > 0.01;
                if !extends_outside {
                    continue;
                }

                out.push(insets);
            }
            SceneOp::ShadowRRect {
                rect,
                offset,
                spread,
                blur_radius,
                color,
                ..
            } => {
                if color.a <= 0.0001 {
                    continue;
                }
                let outer_rect =
                    shadow_rrect_outer_rect(rect, offset, spread.0, blur_radius.0.max(0.0));
                if rect_intersection_area(outer_rect, panel_rect) / panel_area <= 0.01 {
                    continue;
                }
                let insets = shadow_insets_for_rect(panel_rect, outer_rect);
                let extends_outside = insets.left < -0.01
                    || insets.top < -0.01
                    || insets.right > 0.01
                    || insets.bottom > 0.01;
                if !extends_outside {
                    continue;
                }

                out.push(insets);
            }
            _ => {}
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fret_drop_shadow_insets_candidates_include_shadow_rrect_ops() {
        let panel = Rect::new(
            Point::new(Px(10.0), Px(10.0)),
            Size::new(Px(30.0), Px(20.0)),
        );
        let mut scene = Scene::default();
        scene.push(SceneOp::ShadowRRect {
            order: DrawOrder(0),
            rect: panel,
            corner_radii: Corners::all(Px(8.0)),
            offset: Point::new(Px(0.0), Px(1.0)),
            spread: Px(0.0),
            blur_radius: Px(3.0),
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.12,
            },
        });

        let candidates = fret_drop_shadow_insets_candidates(&scene, panel);
        assert_eq!(candidates.len(), 1);
        let shadow = candidates[0];
        assert!((shadow.left - -3.0).abs() <= 0.01);
        assert!((shadow.top - -2.0).abs() <= 0.01);
        assert!((shadow.right - 3.0).abs() <= 0.01);
        assert!((shadow.bottom - 4.0).abs() <= 0.01);
    }
}
