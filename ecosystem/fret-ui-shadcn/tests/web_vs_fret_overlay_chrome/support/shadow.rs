use super::*;

use std::cmp::Ordering;

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

#[derive(Debug, Clone, Copy)]
pub(crate) struct ShadowInsets {
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) right: f32,
    pub(crate) bottom: f32,
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
    if expected.is_empty() {
        return;
    }
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
        assert_close(
            &format!("{web_name} {web_theme_name} shadow[{idx}] left"),
            act.left,
            exp.left,
            tol,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} shadow[{idx}] top"),
            act.top,
            exp.top,
            tol,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} shadow[{idx}] right"),
            act.right,
            exp.right,
            tol,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} shadow[{idx}] bottom"),
            act.bottom,
            exp.bottom,
            tol,
        );
    }
}

pub(crate) fn web_drop_shadow_insets(node: &WebNode) -> Vec<ShadowInsets> {
    let box_shadow = node
        .computed_style
        .get("boxShadow")
        .map(String::as_str)
        .unwrap_or("");
    if box_shadow.is_empty() || box_shadow == "none" {
        return Vec::new();
    }

    let mut out = Vec::new();
    for layer in split_box_shadow_layers(box_shadow) {
        let Some((color, x, y, blur, spread)) = parse_box_shadow_layer(layer) else {
            continue;
        };
        if let Some(rgba) = parse_css_color(&color)
            && rgba.a <= 0.01
        {
            continue;
        }
        if x.abs() <= 0.01 && y.abs() <= 0.01 && blur.abs() <= 0.01 && spread.abs() <= 0.01 {
            continue;
        }
        out.push(shadow_insets_for_box_shadow_layer(x, y, blur, spread));
    }
    out
}

pub(crate) fn fret_drop_shadow_insets_candidates(
    scene: &Scene,
    panel_rect: Rect,
) -> Vec<ShadowInsets> {
    let panel_area = rect_area(panel_rect).max(1.0);
    let mut out = Vec::new();

    for op in scene.ops() {
        let SceneOp::Quad {
            rect,
            background,
            border,
            ..
        } = *op
        else {
            continue;
        };

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

        out.push(shadow_insets_for_rect(panel_rect, rect));
    }

    out
}
