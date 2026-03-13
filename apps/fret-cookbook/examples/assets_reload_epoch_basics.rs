use fret::{FretApp, advanced::prelude::*, shadcn};
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
    applied_bumps: u64,
    file_image: fret_ui_assets::ImageSource,
    svg_file: fret_ui_assets::SvgFileSource,
}

impl View for AssetsReloadEpochBasicsView {
    fn init(app: &mut KernelApp, window: AppWindowId) -> Self {
        // Optional: configure budgets explicitly so this example is self-contained.
        fret_ui_assets::UiAssets::configure(app, fret_ui_assets::UiAssetsBudgets::default());

        let file_image =
            fret_ui_assets::ImageSource::from_path(repo_path("assets/textures/test.jpg"));
        let svg_file =
            fret_ui_assets::SvgFileSource::from_path(repo_path("assets/demo/icon-search.svg"));

        Self {
            window,
            applied_bumps: 0,
            file_image,
            svg_file,
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();
        let bumps_state = cx.state().local::<u64>();

        cx.actions()
            .local_update::<act::BumpReload, u64>(&bumps_state, |value| {
                *value = value.wrapping_add(1);
            });

        let bumps = cx.state().watch(&bumps_state).layout().value_or(0);
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
        .items_center();

        let file_image_state = cx.use_image_source_state(&self.file_image);
        let image_panel = render_image_panel(cx, &theme, file_image_state);

        // SVG file loading is synchronous (cached) so we can surface useful error strings directly.
        cx.observe_global::<fret_ui_assets::UiAssetsReloadEpoch>(Invalidation::Paint);
        let svg_file_state = fret_ui_assets::read_svg_file_cached(&mut *cx.app, &self.svg_file);
        let svg_panel = render_svg_panel(cx, &theme, svg_file_state);

        let images = fret_ui_assets::UiAssets::image_stats(&mut *cx.app);
        let svgs = fret_ui_assets::UiAssets::svg_stats(&mut *cx.app);
        let stats = shadcn::Alert::build(|cx, out| {
            out.push_ui(cx, shadcn::AlertTitle::new("Budgets + cache stats"));
            out.push_ui(
                cx,
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
            );
        });

        let content = ui::v_flex(|cx| ui::children![cx; actions, image_panel, svg_panel, stats])
            .gap(Space::N4)
            .items_stretch();

        let card = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Assets reload epoch basics"),
                        shadcn::card_description(
                            "Load a file image + SVG icon and trigger a ViewCache-safe dev reload by bumping UiAssetsReloadEpoch.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| ui::children![cx; content]),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(900.0));

        fret_cookbook::scaffold::centered_page_muted_ui(cx, TEST_ID_ROOT, card).into()
    }
}

fn render_image_panel(
    _cx: &mut UiCx<'_>,
    theme: &ThemeSnapshot,
    st: fret_ui_assets::ImageSourceState,
) -> impl IntoUiElement<KernelApp> + use<> {
    let status = match st.status {
        fret_ui_assets::image_asset_state::ImageLoadingStatus::Idle => "idle",
        fret_ui_assets::image_asset_state::ImageLoadingStatus::Loading => "loading",
        fret_ui_assets::image_asset_state::ImageLoadingStatus::Loaded => "ready",
        fret_ui_assets::image_asset_state::ImageLoadingStatus::Error => "error",
    };
    let border = ColorRef::Color(theme.color_token("border"));

    let image_box = ui::container(move |cx| {
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
    .border_color(border)
    .rounded(Radius::Lg)
    .w_px(Px(240.0))
    .h_px(Px(180.0))
    .overflow_hidden();

    let body = ui::v_flex_build(move |cx, out| {
        out.push_ui(
            cx,
            ui::h_flex(|cx| {
                ui::children![cx;
                    shadcn::Label::new("File image status:"),
                    shadcn::Badge::new(status)
                        .variant(shadcn::BadgeVariant::Secondary)
                        .test_id(TEST_ID_IMAGE_STATUS),
                ]
            })
            .gap(Space::N2)
            .items_center(),
        );
        out.push_ui(cx, image_box);

        if let Some(msg) = st.error {
            let alert = shadcn::Alert::build(|cx, out| {
                out.push_ui(cx, shadcn::AlertTitle::new("Image decode/upload failed"));
                out.push_ui(cx, shadcn::AlertDescription::new(msg));
            })
            .variant(shadcn::AlertVariant::Destructive);
            out.push_ui(cx, alert);
        }
    })
    .gap(Space::N3)
    .items_start();

    shadcn::card(|cx| {
        ui::children![cx;
            shadcn::card_header(|cx| {
                ui::children![cx;
                    shadcn::card_title("Image from disk"),
                    shadcn::card_description(
                        "Loads `assets/textures/test.jpg` via ImageSource + ImageAssetCache (async decode, cached upload).",
                    ),
                ]
            }),
            shadcn::card_content(|cx| ui::children![cx; body]),
        ]
    })
    .ui()
    .w_full()
}

fn render_svg_panel(
    _cx: &mut UiCx<'_>,
    theme: &ThemeSnapshot,
    st: fret_ui_assets::SvgFileState,
) -> impl IntoUiElement<KernelApp> + use<> {
    let status = if st.error.is_some() {
        "error"
    } else if st.bytes.is_some() {
        "ready"
    } else {
        "missing"
    };
    let border = ColorRef::Color(theme.color_token("border"));
    let destructive = ColorRef::Color(theme.color_token("destructive"));
    let foreground = theme.color_token("foreground");

    let box_el = ui::container(move |cx| {
        if let Some(err) = st.error.clone() {
            ui::children![cx;
                ui::text(format!("Failed to read SVG: {err}"))
                    .text_color(destructive)]
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
            props.color = foreground;
            ui::children![cx; cx.svg_icon_props(props)]
        } else {
            ui::children![cx; cx.spinner()]
        }
    })
    .border_1()
    .border_color(border)
    .rounded(Radius::Lg)
    .p(Space::N4)
    .w_px(Px(240.0))
    .h_px(Px(180.0));

    let body = ui::v_flex(move |cx| {
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
    .items_start();

    shadcn::card(|cx| {
        ui::children![cx;
            shadcn::card_header(|cx| {
                ui::children![cx;
                    shadcn::card_title("SVG icon from disk"),
                    shadcn::card_description(
                        "Loads `assets/demo/icon-search.svg` via SvgFileSource + UiAssetsReloadEpoch.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| ui::children![cx; body]),
        ]
    })
    .ui()
    .w_full()
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-assets-reload-epoch-basics")
        .window("cookbook-assets-reload-epoch-basics", (960.0, 780.0))
        .config_files(false)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<AssetsReloadEpochBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
