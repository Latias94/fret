pub const SOURCE: &str = include_str!("image_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{ImageColorSpace, Px};
use fret_ui_ai as ui_ai;
use fret_ui_assets as ui_assets;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::OnceLock;
use ui_assets::ui::ImageSourceElementContextExt as _;

fn demo_image_source() -> &'static ui_assets::ImageSource {
    static SOURCE: OnceLock<ui_assets::ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        // Keep the snippet self-contained instead of depending on UI Gallery driver globals.
        ui_assets::ImageSource::rgba8(96, 96, demo_avatar_rgba8(96, 96), ImageColorSpace::Srgb)
    })
}

fn demo_avatar_rgba8(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;
    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.5;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = ((dx * dx + dy * dy).sqrt()) / (width.min(height) as f32 * 0.5);

            let (mut r, mut g, mut b) = if distance <= 0.45 {
                (248u8, 215u8, 184u8)
            } else {
                let tint = (30.0 + 120.0 * fx) as u8;
                (18u8, tint, (90.0 + 120.0 * (1.0 - fy)) as u8)
            };

            let eye_band = y > height / 3 && y < height / 2;
            let left_eye = x > width / 3 - 6 && x < width / 3 + 2;
            let right_eye = x > (width * 2) / 3 - 2 && x < (width * 2) / 3 + 6;
            let mouth_band = y > (height * 2) / 3 && y < (height * 2) / 3 + 4;
            let mouth = x > width / 3 && x < (width * 2) / 3;

            if eye_band && (left_eye || right_eye) {
                r = 28;
                g = 28;
                b = 36;
            } else if mouth_band && mouth {
                r = 120;
                g = 64;
                b = 72;
            }

            out[idx] = r;
            out[idx + 1] = g;
            out[idx + 2] = b;
            out[idx + 3] = 255;
        }
    }

    out
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let state = cx.use_image_source_state(demo_image_source());
    let status_line = cx
        .text(format!("status={:?}", state.status))
        .test_id("ui-ai-image-demo-status");

    let border = cx.with_theme(|theme| theme.color_token("border"));
    let image = state.image.map(|id| {
        ui_ai::Image::new(id)
            .alt("Generated image")
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border)),
            )
            .refine_layout(LayoutRefinement::default().w_px(Px(300.0)).h_px(Px(300.0)))
            .test_id("ui-ai-image-demo-image")
            .into_element(cx)
    });

    let image = image.unwrap_or_else(|| {
        cx.text("Loading image...")
            .test_id("ui-ai-image-demo-loading")
    });

    let props = cx.with_theme(|theme| {
        let chrome = ChromeRefinement::default().p(Space::N4);
        let layout = LayoutRefinement::default().w_full().min_w_0().min_h_0();
        decl_style::container_props(theme, chrome, layout)
    });

    ui::v_flex(move |cx| {
        vec![
            cx.text("Image (AI Elements): generated image presentation surface."),
            status_line,
            cx.container(props, move |cx| {
                vec![
                    ui::h_flex(move |_cx| vec![image])
                        .layout(LayoutRefinement::default().w_full())
                        .justify_center()
                        .items_center()
                        .into_element(cx),
                ]
            }),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N3)
    .into_element(cx)
}
// endregion: example
