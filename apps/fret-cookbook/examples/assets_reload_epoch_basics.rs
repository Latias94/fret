use fret::prelude::*;
use fret_ui::element::{ImageProps, LayoutStyle, Length, SizeStyle, SvgIconProps};
use fret_ui_assets::ui::ImageSourceElementContextExt as _;
use std::path::PathBuf;
use std::sync::Arc;

mod act {
    fret::actions!([BumpReload = "cookbook.assets_reload_epoch_basics.bump_reload.v1"]);
}

const TEST_ID_ROOT: &str = "cookbook.assets_reload_epoch_basics.root";
const TEST_ID_BUMP_RELOAD: &str = "cookbook.assets_reload_epoch_basics.bump_reload";
const TEST_ID_IMAGE_STATUS: &str = "cookbook.assets_reload_epoch_basics.image.status";
const TEST_ID_SVG_STATUS: &str = "cookbook.assets_reload_epoch_basics.svg.status";

fn repo_root_from_manifest_dir() -> PathBuf {
    // Cookbook examples should not depend on the process CWD (fretboard/dev runners may vary it).
    // Resolve paths relative to the workspace root via `CARGO_MANIFEST_DIR`.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn repo_path(rel: &str) -> Arc<PathBuf> {
    Arc::new(repo_root_from_manifest_dir().join(rel))
}

struct AssetsReloadEpochBasicsView {
    window: AppWindowId,
    bumps: Model<u64>,
    applied_bumps: u64,
    file_image: fret_ui_assets::ImageSource,
    svg_file: fret_ui_assets::SvgFileSource,
}

impl View for AssetsReloadEpochBasicsView {
    fn init(app: &mut App, window: AppWindowId) -> Self {
        // Optional: configure budgets explicitly so this example is self-contained.
        fret_ui_assets::UiAssets::configure(app, fret_ui_assets::UiAssetsBudgets::default());

        let file_image =
            fret_ui_assets::ImageSource::from_path(repo_path("assets/textures/test.jpg"));
        let svg_file =
            fret_ui_assets::SvgFileSource::from_path(repo_path("assets/demo/icon-search.svg"));

        Self {
            window,
            bumps: app.models_mut().insert(0),
            applied_bumps: 0,
            file_image,
            svg_file,
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();

        cx.on_action_notify_models::<act::BumpReload>({
            let bumps = self.bumps.clone();
            move |models| {
                models
                    .update(&bumps, |v| {
                        *v = v.wrapping_add(1);
                    })
                    .is_ok()
            }
        });

        let bumps = cx.watch_model(&self.bumps).layout().copied_or(0);
        if bumps != self.applied_bumps {
            fret_ui_assets::bump_ui_assets_reload_epoch(&mut *cx.app);
            self.applied_bumps = bumps;
            cx.app.request_redraw(self.window);
            cx.app
                .push_effect(Effect::RequestAnimationFrame(self.window));
        }

        let epoch = cx
            .app
            .global::<fret_ui_assets::UiAssetsReloadEpoch>()
            .copied()
            .map(|v| v.0)
            .unwrap_or(0);

        let actions = ui::h_flex(|cx| {
            ui::children![
                cx;
                shadcn::Button::new("Bump assets reload epoch")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .action(act::BumpReload)
                    .test_id(TEST_ID_BUMP_RELOAD),
                shadcn::Badge::new(format!("epoch: {epoch}"))
                    .variant(shadcn::BadgeVariant::Secondary),
                shadcn::Badge::new("Tip: edit files under `assets/` then click reload.")
                    .variant(shadcn::BadgeVariant::Secondary),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let file_image_state = cx.use_image_source_state(&self.file_image);
        let image_panel = render_image_panel(cx, &theme, file_image_state);

        // SVG file loading is synchronous (cached) so we can surface useful error strings directly.
        cx.observe_global::<fret_ui_assets::UiAssetsReloadEpoch>(Invalidation::Paint);
        let svg_file_state = fret_ui_assets::read_svg_file_cached(&mut *cx.app, &self.svg_file);
        let svg_panel = render_svg_panel(cx, &theme, svg_file_state);

        let images = fret_ui_assets::UiAssets::image_stats(&mut *cx.app);
        let svgs = fret_ui_assets::UiAssets::svg_stats(&mut *cx.app);
        let stats = shadcn::Alert::new(ui::children![
            cx;
            shadcn::AlertTitle::new("Budgets + cache stats"),
            shadcn::AlertDescription::new(format!(
                "Images: ready={} pending={} failed={} bytes={} / {} | SVGs: ready={} bytes={} / {}",
                images.ready_count,
                images.pending_count,
                images.failed_count,
                images.bytes_ready,
                images.bytes_budget,
                svgs.ready_count,
                svgs.bytes_ready,
                svgs.bytes_budget
            )),
        ])
        .into_element(cx);

        let content = ui::v_flex(|cx| ui::children![cx; actions, image_panel, svg_panel, stats])
            .gap(Space::N4)
            .items_stretch()
            .into_element(cx);

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Assets reload epoch basics"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new(
                            "Load a file image + SVG icon and trigger a ViewCache-safe dev reload by bumping UiAssetsReloadEpoch.",
                        ),
                    );
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|_cx, out| {
                    out.push(content);
                }),
            );
        })
        .ui()
        .w_full()
        .max_w(Px(900.0))
        .into_element(cx);

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn render_image_panel(
    cx: &mut ElementContext<'_, App>,
    theme: &ThemeSnapshot,
    st: fret_ui_assets::ImageSourceState,
) -> AnyElement {
    let status = match st.status {
        fret_ui_assets::image_asset_state::ImageLoadingStatus::Idle => "idle",
        fret_ui_assets::image_asset_state::ImageLoadingStatus::Loading => "loading",
        fret_ui_assets::image_asset_state::ImageLoadingStatus::Loaded => "ready",
        fret_ui_assets::image_asset_state::ImageLoadingStatus::Error => "error",
    };

    let image_box = ui::container(|cx| {
        if let Some(image) = st.image {
            let mut img = ImageProps::new(image);
            img.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            };
            [cx.image_props(img)]
        } else {
            [cx.spinner()]
        }
    })
    .border_1()
    .border_color(ColorRef::Color(theme.color_token("border")))
    .rounded(Radius::Lg)
    .w_px(Px(240.0))
    .h_px(Px(180.0))
    .overflow_hidden()
    .into_element(cx);

    let mut body: Vec<AnyElement> = Vec::new();
    body.push(
        ui::h_flex(|cx| {
            ui::children![cx;
                shadcn::Label::new("File image status:"),
                shadcn::Badge::new(status)
                    .variant(shadcn::BadgeVariant::Secondary)
                    .test_id(TEST_ID_IMAGE_STATUS),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx),
    );
    body.push(image_box);

    if let Some(msg) = st.error {
        body.push(
            shadcn::Alert::new(ui::children![cx;
                shadcn::AlertTitle::new("Image decode/upload failed"),
                shadcn::AlertDescription::new(msg),
            ])
            .variant(shadcn::AlertVariant::Destructive)
            .into_element(cx),
        );
    }

    let body = ui::v_flex(|_cx| body)
        .gap(Space::N3)
        .items_start()
        .into_element(cx);

    shadcn::Card::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::CardHeader::build(|cx, out| {
                out.push_ui(cx, shadcn::CardTitle::new("Image from disk"));
                out.push_ui(
                    cx,
                    shadcn::CardDescription::new(
                        "Loads `assets/textures/test.jpg` via ImageSource + ImageAssetCache (async decode, cached upload).",
                    ),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::CardContent::build(|_cx, out| {
                out.push(body);
            }),
        );
    })
    .ui()
    .w_full()
    .into_element(cx)
}

fn render_svg_panel(
    cx: &mut ElementContext<'_, App>,
    theme: &ThemeSnapshot,
    st: fret_ui_assets::SvgFileState,
) -> AnyElement {
    let status = if st.error.is_some() {
        "error"
    } else if st.bytes.is_some() {
        "ready"
    } else {
        "missing"
    };

    let box_el = ui::container(|cx| {
        if let Some(err) = st.error.clone() {
            [ui::text(format!("Failed to read SVG: {err}"))
                .text_color(ColorRef::Color(theme.color_token("destructive")))
                .into_element(cx)]
        } else if let Some(bytes) = st.bytes.clone() {
            let mut props = SvgIconProps::new(fret_ui::SvgSource::Bytes(bytes));
            props.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            };
            props.fit = fret_core::SvgFit::Contain;
            props.color = theme.color_token("foreground");
            [cx.svg_icon_props(props)]
        } else {
            [cx.spinner()]
        }
    })
    .border_1()
    .border_color(ColorRef::Color(theme.color_token("border")))
    .rounded(Radius::Lg)
    .p(Space::N4)
    .w_px(Px(240.0))
    .h_px(Px(180.0))
    .into_element(cx);

    let body = ui::v_flex(|cx| {
        ui::children![cx;
            ui::h_flex(|cx| {
                ui::children![cx;
                    shadcn::Label::new("SVG status:"),
                    shadcn::Badge::new(status)
                        .variant(shadcn::BadgeVariant::Secondary)
                        .test_id(TEST_ID_SVG_STATUS),
                ]
            })
            .gap(Space::N2)
            .items_center(),
            box_el,
        ]
    })
    .gap(Space::N3)
    .items_start()
    .into_element(cx);

    shadcn::Card::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::CardHeader::build(|cx, out| {
                out.push_ui(cx, shadcn::CardTitle::new("SVG icon from disk"));
                out.push_ui(
                    cx,
                    shadcn::CardDescription::new(
                        "Loads `assets/demo/icon-search.svg` via SvgFileSource + UiAssetsReloadEpoch.",
                    ),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::CardContent::build(|_cx, out| {
                out.push(body);
            }),
        );
    })
    .ui()
    .w_full()
    .into_element(cx)
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-assets-reload-epoch-basics")
        .window("cookbook-assets-reload-epoch-basics", (960.0, 780.0))
        .config_files(false)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<AssetsReloadEpochBasicsView>()
        .map_err(anyhow::Error::from)
}
