use std::sync::Arc;

use fret::component::prelude::*;
use fret::{advanced::prelude::*, shadcn};
use fret_core::{AppWindowId, Px, ViewportFit};
use fret_launch::{EngineFrameUpdate, imported_viewport_target::ImportedViewportRenderTarget};
use fret_render::{
    RenderTargetColorSpace, RenderTargetIngestStrategy, RenderTargetMetadata, Renderer,
    WgpuContext, write_rgba8_texture_region,
};
use fret_runtime::{FrameId, Model, TickId};
use fret_ui::element::{LayoutStyle, Length, SemanticsDecoration};

const ROOT_NAME: &str = "cookbook.external_texture_import_basics";

const TEST_ID_ROOT: &str = "cookbook.external_texture_import_basics.root";
const TEST_ID_SURFACE: &str = "cookbook.external_texture_import_basics.surface";

const TEST_ID_SIZE_640: &str = "cookbook.external_texture_import_basics.size_640";
const TEST_ID_SIZE_960: &str = "cookbook.external_texture_import_basics.size_960";
const TEST_ID_SIZE_1280: &str = "cookbook.external_texture_import_basics.size_1280";

const TEST_ID_FIT_CONTAIN: &str = "cookbook.external_texture_import_basics.fit_contain";
const TEST_ID_FIT_COVER: &str = "cookbook.external_texture_import_basics.fit_cover";
const TEST_ID_FIT_STRETCH: &str = "cookbook.external_texture_import_basics.fit_stretch";

const TEST_ID_TARGET_W: &str = "cookbook.external_texture_import_basics.target_w";
const TEST_ID_TARGET_H: &str = "cookbook.external_texture_import_basics.target_h";
const TEST_ID_FIT_CODE: &str = "cookbook.external_texture_import_basics.fit_code";
const TEST_ID_INGEST_CODE: &str = "cookbook.external_texture_import_basics.ingest_code";

const DEFAULT_TARGET_PX_SIZE: (u32, u32) = (960, 540);
const SIZE_PRESET_640: &str = "640x360";
const SIZE_PRESET_960: &str = "960x540";
const SIZE_PRESET_1280: &str = "1280x720";
const FIT_CONTAIN: &str = "contain";
const FIT_COVER: &str = "cover";
const FIT_STRETCH: &str = "stretch";

fn selected_target_px_size(value: Option<&str>) -> ((u32, u32), &'static str) {
    match value {
        Some(SIZE_PRESET_640) => ((640, 360), "640×360"),
        Some(SIZE_PRESET_1280) => ((1280, 720), "1280×720"),
        _ => (DEFAULT_TARGET_PX_SIZE, "960×540"),
    }
}

fn selected_fit(value: Option<&str>) -> ViewportFit {
    match value {
        Some(FIT_COVER) => ViewportFit::Cover,
        Some(FIT_STRETCH) => ViewportFit::Stretch,
        _ => ViewportFit::Contain,
    }
}

fn fit_code(fit: ViewportFit) -> f64 {
    match fit {
        ViewportFit::Contain => 0.0,
        ViewportFit::Cover => 1.0,
        ViewportFit::Stretch => 2.0,
    }
}

fn ingest_code(ingest: RenderTargetIngestStrategy) -> f64 {
    match ingest {
        RenderTargetIngestStrategy::Owned => 0.0,
        RenderTargetIngestStrategy::GpuCopy => 1.0,
        RenderTargetIngestStrategy::CpuUpload => 2.0,
        RenderTargetIngestStrategy::ExternalZeroCopy => 3.0,
        RenderTargetIngestStrategy::Unknown => 4.0,
    }
}

#[derive(Debug)]
struct ExternalTextureImportBasicsState {
    preset: Model<Option<Arc<str>>>,
    fit: Model<Option<Arc<str>>>,

    target: ImportedViewportRenderTarget,
    target_px_size: (u32, u32),
    texture: Option<wgpu::Texture>,

    target_w: Model<f64>,
    target_h: Model<f64>,
    ingest: Model<f64>,
}

fn init_window(app: &mut KernelApp, _window: AppWindowId) -> ExternalTextureImportBasicsState {
    ExternalTextureImportBasicsState {
        preset: app
            .models_mut()
            .insert(Some(Arc::<str>::from(SIZE_PRESET_960))),
        fit: app.models_mut().insert(Some(Arc::<str>::from(FIT_CONTAIN))),
        target: ImportedViewportRenderTarget::new(
            wgpu::TextureFormat::Rgba8UnormSrgb,
            RenderTargetColorSpace::Srgb,
        ),
        target_px_size: (1, 1),
        texture: None,
        target_w: app.models_mut().insert(DEFAULT_TARGET_PX_SIZE.0 as f64),
        target_h: app.models_mut().insert(DEFAULT_TARGET_PX_SIZE.1 as f64),
        ingest: app
            .models_mut()
            .insert(ingest_code(RenderTargetIngestStrategy::Owned)),
    }
}

fn view(
    cx: &mut ElementContext<'_, KernelApp>,
    st: &mut ExternalTextureImportBasicsState,
) -> ViewElements {
    let theme = cx.theme().snapshot();

    let preset = st.preset.paint_in(cx).value_or_default();
    let fit_value = st.fit.paint_in(cx).value_or_default();
    let fit = selected_fit(fit_value.as_deref());
    let target_w = st.target_w.paint_in(cx).value_or_default();
    let target_h = st.target_h.paint_in(cx).value_or_default();
    let ingest = st.ingest.paint_in(cx).value_or_default();

    let (target_px_size, preset_label) = selected_target_px_size(preset.as_deref());

    let header = shadcn::card_header(|cx| {
        ui::children![cx;
            shadcn::card_title("Tier A interop: external texture import (basics)"),
            shadcn::card_description(
                "Presenting a per-frame imported wgpu::TextureView as a stable RenderTargetId via EngineFrameUpdate deltas (ADR 0234).",
            ),
        ]
    });

    let size_controls = shadcn::ToggleGroup::single(&st.preset)
        .deselectable(false)
        .variant(shadcn::ToggleVariant::Outline)
        .spacing(Space::N2)
        .items([
            shadcn::ToggleGroupItem::new(SIZE_PRESET_640, [cx.text("640×360")])
                .a11y_label("Target size 640 by 360")
                .test_id(TEST_ID_SIZE_640),
            shadcn::ToggleGroupItem::new(SIZE_PRESET_960, [cx.text("960×540")])
                .a11y_label("Target size 960 by 540")
                .test_id(TEST_ID_SIZE_960),
            shadcn::ToggleGroupItem::new(SIZE_PRESET_1280, [cx.text("1280×720")])
                .a11y_label("Target size 1280 by 720")
                .test_id(TEST_ID_SIZE_1280),
        ])
        .refine_layout(LayoutRefinement::default().flex_none());

    let fit_controls = shadcn::ToggleGroup::single(&st.fit)
        .deselectable(false)
        .variant(shadcn::ToggleVariant::Outline)
        .spacing(Space::N2)
        .items([
            shadcn::ToggleGroupItem::new(FIT_CONTAIN, [cx.text("Fit: Contain")])
                .a11y_label("Viewport fit contain")
                .test_id(TEST_ID_FIT_CONTAIN),
            shadcn::ToggleGroupItem::new(FIT_COVER, [cx.text("Cover")])
                .a11y_label("Viewport fit cover")
                .test_id(TEST_ID_FIT_COVER),
            shadcn::ToggleGroupItem::new(FIT_STRETCH, [cx.text("Stretch")])
                .a11y_label("Viewport fit stretch")
                .test_id(TEST_ID_FIT_STRETCH),
        ])
        .refine_layout(LayoutRefinement::default().flex_none());

    let controls = ui::h_flex(|cx| {
        ui::children![
            cx;
            size_controls,
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical),
            fit_controls,
        ]
    })
    .gap(Space::N2)
    .items_center()
    .wrap();

    let target_w_badge = shadcn::Badge::new(format!("target.w: {target_w:.0}"))
        .variant(shadcn::BadgeVariant::Secondary)
        .a11y(
            SemanticsDecoration::default()
                .role(SemanticsRole::Meter)
                .test_id(TEST_ID_TARGET_W)
                .numeric_value(target_w),
        );
    let target_h_badge = shadcn::Badge::new(format!("target.h: {target_h:.0}"))
        .variant(shadcn::BadgeVariant::Secondary)
        .a11y(
            SemanticsDecoration::default()
                .role(SemanticsRole::Meter)
                .test_id(TEST_ID_TARGET_H)
                .numeric_value(target_h),
        );
    let fit_badge = shadcn::Badge::new(format!("fit: {:.0}", fit_code(fit)))
        .variant(shadcn::BadgeVariant::Secondary)
        .a11y(
            SemanticsDecoration::default()
                .role(SemanticsRole::Meter)
                .test_id(TEST_ID_FIT_CODE)
                .numeric_value(fit_code(fit)),
        );
    let ingest_badge = shadcn::Badge::new(format!("ingest: {ingest:.0}"))
        .variant(shadcn::BadgeVariant::Secondary)
        .a11y(
            SemanticsDecoration::default()
                .role(SemanticsRole::Meter)
                .test_id(TEST_ID_INGEST_CODE)
                .numeric_value(ingest),
        );

    let info = ui::h_flex(|cx| {
        ui::children![
            cx;
            ui::text(format!("target_px_size preset: {preset_label}")).text_xs(),
            target_w_badge,
            target_h_badge,
            fit_badge,
            ingest_badge,
        ]
    })
    .gap(Space::N2)
    .items_center()
    .wrap();

    let hint = shadcn::Alert::new(ui::children![
        cx;
        shadcn::AlertTitle::new("Key idea"),
        shadcn::AlertDescription::new(
            "Instead of mutating the renderer's render target registry directly, this example emits explicit per-frame EngineFrameUpdate target updates (ImportedViewportRenderTarget). This keeps registry mutation staged through the runner.",
        ),
    ]);

    let mut surface_style = LayoutStyle::default();
    surface_style.size.width = Length::Fill;
    surface_style.size.height = Length::Fill;

    let surface = cx
        .viewport_surface_props(fret_ui::element::ViewportSurfaceProps {
            layout: surface_style,
            target: st.target.id(),
            target_px_size,
            fit,
            opacity: 1.0,
        })
        .test_id(TEST_ID_SURFACE);

    let surface_panel = ui::container(|_cx| vec![surface])
        .bg(ColorRef::Color(theme.color_token("muted")))
        .rounded(Radius::Md)
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .w_full()
        .h_px(Px(420.0));

    let content =
        ui::v_flex(|cx| ui::children![cx; controls, info, hint, surface_panel]).gap(Space::N3);

    let card = shadcn::card(|cx| {
        ui::children![cx;
            header,
            shadcn::card_content(|cx| ui::single(cx, content)),
        ]
    })
    .ui()
    .w_full()
    .h_full()
    .max_w(Px(1100.0));

    fret_cookbook::scaffold::centered_page_background(cx, TEST_ID_ROOT, card)
}

fn record_engine_frame(
    app: &mut KernelApp,
    _window: AppWindowId,
    _ui: &mut UiTree<KernelApp>,
    st: &mut ExternalTextureImportBasicsState,
    context: &WgpuContext,
    renderer: &mut Renderer,
    _scale_factor: f32,
    _tick_id: TickId,
    frame_id: FrameId,
) -> EngineFrameUpdate {
    let preset = app
        .models()
        .read(&st.preset, Clone::clone)
        .unwrap_or_default();
    let (desired, _) = selected_target_px_size(preset.as_deref());

    let mut update = EngineFrameUpdate::default();

    let needs_alloc = st.texture.is_none() || st.target_px_size != desired;
    if needs_alloc {
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cookbook external texture import target"),
            size: wgpu::Extent3d {
                width: desired.0.max(1),
                height: desired.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: st.target.format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        if !st.target.is_registered() {
            let mut metadata = RenderTargetMetadata::default();
            metadata.requested_ingest_strategy = RenderTargetIngestStrategy::Owned;
            metadata.ingest_strategy = RenderTargetIngestStrategy::Owned;
            st.target
                .ensure_registered_with_metadata(renderer, view.clone(), desired, metadata);
        }

        st.texture = Some(texture);
        st.target_px_size = desired;
    }

    let (w, h) = st.target_px_size;
    let w = w.max(1);
    let h = h.max(1);

    let mut rgba = vec![0u8; (w as usize).saturating_mul(h as usize).saturating_mul(4)];
    for y in 0..h {
        for x in 0..w {
            let i = ((y as usize)
                .saturating_mul(w as usize)
                .saturating_add(x as usize))
            .saturating_mul(4);

            let fx = (x as f32 + 0.5) / w as f32;
            let fy = (y as f32 + 0.5) / h as f32;
            let t = (frame_id.0 as f32 * 0.03).sin() * 0.5 + 0.5;
            let checker = ((x / 32) ^ (y / 32)) & 1;
            let bump = if checker == 0 { 0.9 } else { 0.55 };

            let r = (255.0 * (0.10 + 0.85 * fx) * bump) as u8;
            let g = (255.0 * (0.12 + 0.75 * fy) * bump) as u8;
            let b = (255.0 * (0.15 + 0.70 * t) * bump) as u8;

            rgba[i] = r;
            rgba[i + 1] = g;
            rgba[i + 2] = b;
            rgba[i + 3] = 255;
        }
    }

    let texture = st
        .texture
        .as_ref()
        .expect("texture must exist after allocation");
    write_rgba8_texture_region(
        &context.queue,
        texture,
        (0, 0),
        (w, h),
        w.saturating_mul(4),
        &rgba,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let requested = RenderTargetIngestStrategy::Owned;
    let effective = RenderTargetIngestStrategy::Owned;
    st.target.push_update_with_ingest_strategies(
        &mut update,
        view,
        (w, h),
        RenderTargetMetadata::default(),
        requested,
        effective,
    );

    let _ = app.models_mut().update(&st.target_w, |v| *v = w as f64);
    let _ = app.models_mut().update(&st.target_h, |v| *v = h as f64);
    let _ = app
        .models_mut()
        .update(&st.ingest, |v| *v = ingest_code(effective));

    update
}

fn configure_driver(
    driver: UiAppDriver<ExternalTextureImportBasicsState>,
) -> UiAppDriver<ExternalTextureImportBasicsState> {
    driver.record_engine_frame(record_engine_frame)
}

fn main() -> anyhow::Result<()> {
    let builder = ui_app_with_hooks(ROOT_NAME, init_window, view, configure_driver)
        .with_main_window("cookbook-external-texture-import-basics", (1120.0, 780.0))
        .setup((shadcn::app::install, fret_icons_lucide::app::install))
        .setup(fret_cookbook::install_cookbook_defaults)
        .with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096);

    #[cfg(feature = "cookbook-diag")]
    let builder = builder.with_default_diagnostics();

    builder.run().map_err(anyhow::Error::from)
}
