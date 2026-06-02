use fret_core::Color;
use fret_ui::Theme;

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: lerp(a.a, b.a, t),
    }
}

fn is_effectively_transparent(c: Color) -> bool {
    c.a <= 0.02
}

fn opaque_over(theme: &Theme, fg: Color) -> Color {
    // Approximate the effective color a translucent surface would produce when rendered over the
    // theme background, then make it opaque so cached layers don't leak stale pixels.
    let bg = theme.color_token("background");
    let t = fg.a;
    let mut out = mix(bg, fg, t);
    out.a = 1.0;
    out
}

fn editor_fallback_input_bg(theme: &Theme) -> Color {
    // Shadcn themes sometimes set `component.input.bg` to fully transparent. For editor controls we
    // need a stable, non-transparent surface so frames are visible and we don't expose stale
    // pixels from cached overlay layers.
    let bg = theme.color_token("background");
    let muted = theme
        .color_by_key("muted")
        .unwrap_or_else(|| theme.color_token("muted"));
    let mut out = mix(bg, muted, 0.10);
    out.a = 1.0;
    out
}

pub(crate) fn sanitize_editor_surface_bg(theme: &Theme, bg: Color) -> Color {
    if is_effectively_transparent(bg) {
        return editor_fallback_input_bg(theme);
    }

    // Even when not fully transparent (e.g. shadcn `bg-input/30`), keep editor input surfaces
    // opaque to reduce ghosting/artifacts under paint caching and overlay reuse.
    if bg.a < 0.98 {
        return opaque_over(theme, bg);
    }

    bg
}
