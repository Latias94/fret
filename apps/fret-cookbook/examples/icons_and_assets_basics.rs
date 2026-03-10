use fret::{FretApp, advanced::prelude::*, shadcn};
use fret_core::ImageColorSpace;
use fret_icons::FrozenIconRegistry;
use fret_ui::element::{ImageProps, SvgIconProps};
use fret_ui_assets::ui::ImageSourceElementContextExt as _;
use std::path::PathBuf;
use std::sync::Arc;

mod act {
    fret::actions!([BumpReload = "cookbook.icons_and_assets_basics.bump_reload.v1"]);
}

const TEST_ID_ROOT: &str = "cookbook.icons_and_assets_basics.root";
const TEST_ID_BUMP_RELOAD: &str = "cookbook.icons_and_assets_basics.bump_reload";
const TEST_ID_PANEL_ICONS: &str = "cookbook.icons_and_assets_basics.panel.icons";
const TEST_ID_PANEL_SVG: &str = "cookbook.icons_and_assets_basics.panel.svg";
const TEST_ID_PANEL_IMAGE: &str = "cookbook.icons_and_assets_basics.panel.image";
const TEST_ID_IMAGE_STATUS: &str = "cookbook.icons_and_assets_basics.image.status";
const TEST_ID_SVG_STATUS: &str = "cookbook.icons_and_assets_basics.svg.status";

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

fn checkerboard_rgba8(width: u32, height: u32, cell: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let on = ((x / cell) + (y / cell)) % 2 == 0;
            let (r, g, b) = if on {
                (14u8, 165u8, 233u8)
            } else {
                (241u8, 245u8, 249u8)
            };
            let i = ((y * width + x) * 4) as usize;
            out[i] = r;
            out[i + 1] = g;
            out[i + 2] = b;
            out[i + 3] = 255;
        }
    }
    out
}

struct IconsAndAssetsBasicsView {
    window: AppWindowId,
    applied_assets_reload_bumps: u64,
    file_image: fret_ui_assets::ImageSource,
    memory_image: fret_ui_assets::ImageSource,
    svg_file: fret_ui_assets::SvgFileSource,
}

impl View for IconsAndAssetsBasicsView {
    fn init(app: &mut KernelApp, window: AppWindowId) -> Self {
        // Ensure the UI assets caches exist and set budgets explicitly (optional).
        fret_ui_assets::UiAssets::configure(
            app,
            fret_ui_assets::UiAssetsBudgets {
                image_budget_bytes: 32 * 1024 * 1024,
                image_max_ready_entries: 1024,
                svg_budget_bytes: 8 * 1024 * 1024,
                svg_max_ready_entries: 2048,
            },
        );

        let file_image =
            fret_ui_assets::ImageSource::from_path(repo_path("assets/textures/test.jpg"));
        let memory_image = fret_ui_assets::ImageSource::rgba8(
            128,
            128,
            checkerboard_rgba8(128, 128, 16),
            ImageColorSpace::Srgb,
        );

        // `SvgIconProps` is an icon-style SVG element (monochrome + currentColor), not a full SVG
        // image renderer. Use an icon-like SVG (stroke=currentColor, fill=none) for this demo.
        let svg_file =
            fret_ui_assets::SvgFileSource::from_path(repo_path("assets/demo/icon-search.svg"));

        Self {
            window,
            applied_assets_reload_bumps: 0,
            file_image,
            memory_image,
            svg_file,
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();
        let assets_reload_bumps_state = cx.use_local::<u64>();

        cx.on_action_notify_local_update::<act::BumpReload, u64>(
            &assets_reload_bumps_state,
            |value| {
                *value = value.wrapping_add(1);
            },
        );

        let bumps = assets_reload_bumps_state.watch(cx).layout().value_or(0);
        if bumps != self.applied_assets_reload_bumps {
            fret_ui_assets::bump_ui_assets_reload_epoch(&mut *cx.app);
            self.applied_assets_reload_bumps = bumps;
            cx.app.request_redraw(self.window);
            cx.app
                .push_effect(Effect::RequestAnimationFrame(self.window));
        }

        let header = shadcn::CardHeader::build(|cx, out| {
            out.push_ui(cx, shadcn::CardTitle::new("Icons + assets basics"));
            out.push_ui(
                cx,
                shadcn::CardDescription::new(
                    "Icon packs (lucide), semantic ui.* aliases, and file-based SVG/images via fret-ui-assets.",
                ),
            );
        });

        let actions = ui::h_flex(|cx| {
            ui::children![cx;
                shadcn::Button::new("Bump assets reload epoch")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .icon(IconId::new_static("ui.reset"))
                    .action(act::BumpReload)
                    .test_id(TEST_ID_BUMP_RELOAD),
                shadcn::Badge::new("Tip: edit the files under `assets/` and click reload.")
                    .variant(shadcn::BadgeVariant::Secondary)
            ]
        })
        .gap(Space::N2)
        .items_center()
        .justify_center()
        .w_full();

        let icon_row = |title: &'static str, ids: [IconId; 3]| {
            let buttons = ui::h_flex(move |cx| {
                ui::children![cx;
                    shadcn::Button::new("Search")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .leading_icon(ids[0].clone()),
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .leading_icon(ids[1].clone()),
                    shadcn::Button::new("Copy")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .leading_icon(ids[2].clone()),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .w_full();

            ui::v_flex(move |cx| ui::children![cx; shadcn::Label::new(title), buttons])
                .gap(Space::N2)
                .w_full()
        };

        let icons_panel = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Icons"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new(
                            "IconId is renderer-agnostic. Packs register data; components consume semantic ids (ui.*).",
                        ),
                    );
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(
                        cx,
                        ui::v_flex(|cx: &mut ElementContext<'_, KernelApp>| {
                            let frozen = cx.app.global::<FrozenIconRegistry>().cloned();
                            let preload = cx
                                .app
                                .global::<icon::IconSvgPreloadDiagnostics>()
                                .copied();
                            let frozen_len = frozen.as_ref().map(|v| v.len()).unwrap_or(0);
                            let preload_entries = preload.map(|v| v.entries).unwrap_or(0);
                            let preload_bytes = preload.map(|v| v.bytes_ready).unwrap_or(0);

                            ui::children![
                                cx;
                                ui::h_flex(|cx| {
                                    ui::children![
                                        cx;
                                        shadcn::Badge::new(format!("frozen icons: {frozen_len}"))
                                            .variant(shadcn::BadgeVariant::Secondary),
                                        shadcn::Badge::new(format!(
                                            "preloaded: {preload_entries} ({} KB)",
                                            preload_bytes / 1024
                                        ))
                                        .variant(shadcn::BadgeVariant::Secondary),
                                    ]
                                })
                                .gap(Space::N2)
                                .items_center(),
                                icon_row(
                                    "Semantic ids (ui.*)",
                                    [
                                        IconId::new_static("ui.search"),
                                        IconId::new_static("ui.close"),
                                        IconId::new_static("ui.copy"),
                                    ],
                                ),
                                icon_row(
                                    "Vendor ids (lucide.*)",
                                    [
                                        IconId::new_static("lucide.search"),
                                        IconId::new_static("lucide.x"),
                                        IconId::new_static("lucide.copy"),
                                    ],
                                ),
                                icon_row(
                                    "Vendor ids (lucide.*)",
                                    [
                                        IconId::new_static("lucide.search"),
                                        IconId::new_static("lucide.x"),
                                        IconId::new_static("lucide.copy"),
                                    ],
                                ),
                            ]
                        })
                        .gap(Space::N4)
                        .w_full(),
                    );
                }),
            );
        })
        .ui()
        .w_full()
        .test_id(TEST_ID_PANEL_ICONS);

        let file_image_state = cx.use_image_source_state(&self.file_image);
        let memory_image_state = cx.use_image_source_state(&self.memory_image);

        let image_status = match file_image_state.status {
            fret_ui_assets::image_asset_state::ImageLoadingStatus::Idle => "idle",
            fret_ui_assets::image_asset_state::ImageLoadingStatus::Loading => "loading",
            fret_ui_assets::image_asset_state::ImageLoadingStatus::Loaded => "ready",
            fret_ui_assets::image_asset_state::ImageLoadingStatus::Error => "error",
        };

        let render_image = |cx: &mut ElementContext<'_, KernelApp>,
                            title: &str,
                            st: &fret_ui_assets::ImageSourceState|
         -> AnyElement {
            let box_el = ui::container(|cx| {
                if let Some(image) = st.image {
                    let mut props = ImageProps::new(image);
                    props.layout =
                        style::layout_style(&theme, LayoutRefinement::default().size_full());
                    [cx.image_props(props)]
                } else {
                    [cx.spinner()]
                }
            })
            .border_1()
            .border_color(ColorRef::Color(theme.color_token("border")))
            .rounded(Radius::Lg)
            .w_px(Px(160.0))
            .h_px(Px(160.0))
            .overflow_hidden();

            ui::v_flex(|cx| ui::children![cx; shadcn::Label::new(title), box_el])
                .gap(Space::N2)
                .w_full()
                .into_element(cx)
        };

        let image_panel = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Images"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new(
                            "File-based decode is async; in-memory RGBA8 is immediate and useful for deterministic demos.",
                        ),
                    );
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(
                        cx,
                        ui::v_flex(|cx| {
                            ui::children![
                                cx;
                                ui::h_flex(|cx| {
                                    ui::children![
                                        cx;
                                        shadcn::Label::new("File image status:"),
                                        shadcn::Badge::new(image_status)
                                            .variant(shadcn::BadgeVariant::Secondary)
                                            .test_id(TEST_ID_IMAGE_STATUS),
                                    ]
                                })
                                .gap(Space::N2)
                                .items_center(),
                                ui::h_flex(|cx| {
                                    ui::children![
                                        cx;
                                        render_image(cx, "From path: `assets/textures/test.jpg`", &file_image_state),
                                        render_image(cx, "From RGBA8 buffer", &memory_image_state),
                                    ]
                                })
                                .gap(Space::N4)
                                .items_center()
                                .w_full(),
                            ]
                        })
                        .gap(Space::N3)
                        .w_full(),
                    );
                }),
            );
        })
        .ui()
        .w_full()
        .test_id(TEST_ID_PANEL_IMAGE);

        // SVG file loading is synchronous (cached) so we can surface useful error strings directly.
        cx.observe_global::<fret_ui_assets::UiAssetsReloadEpoch>(Invalidation::Paint);
        let svg_file_state = fret_ui_assets::read_svg_file_cached(&mut *cx.app, &self.svg_file);
        let svg_status = if svg_file_state.error.is_some() {
            "error"
        } else if svg_file_state.bytes.is_some() {
            "ready"
        } else {
            "missing"
        };

        let svg_box = ui::container(|cx| {
            if let Some(err) = svg_file_state.error.clone() {
                ui::children![cx;
                    ui::text(format!("Failed to read SVG: {err}"))
                        .text_color(ColorRef::Color(theme.color_token("destructive")))
                ]
            } else if let Some(bytes) = svg_file_state.bytes.clone() {
                let mut props = SvgIconProps::new(fret_ui::SvgSource::Bytes(bytes));
                props.layout = style::layout_style(
                    &theme,
                    LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(160.0)),
                );
                props.fit = fret_core::SvgFit::Contain;
                props.color = theme.color_token("foreground");
                ui::children![cx; cx.svg_icon_props(props)]
            } else {
                ui::children![cx; cx.spinner()]
            }
        })
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .rounded(Radius::Lg)
        .p(Space::N4);

        let svg_panel = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("SVG icon from file"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new(
                            "Loads an icon-style SVG from disk via `SvgFileSource` + `UiAssetsReloadEpoch` (ViewCache-safe dev reload).",
                        ),
                    );
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(
                        cx,
                        ui::v_flex(|cx| {
                            ui::children![
                                cx;
                                ui::h_flex(|cx| {
                                    ui::children![
                                        cx;
                                        shadcn::Label::new("SVG status:"),
                                        shadcn::Badge::new(svg_status)
                                            .variant(shadcn::BadgeVariant::Secondary)
                                            .test_id(TEST_ID_SVG_STATUS),
                                    ]
                                })
                                .gap(Space::N2)
                                .items_center(),
                                svg_box,
                            ]
                        })
                        .gap(Space::N3)
                        .w_full(),
                    );
                }),
            );
        })
        .ui()
        .w_full()
        .test_id(TEST_ID_PANEL_SVG);

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(cx, header);
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(
                        cx,
                        ui::v_flex(
                            |cx| ui::children![cx; actions, icons_panel, svg_panel, image_panel],
                        )
                        .gap(Space::N5)
                        .w_full(),
                    );
                }),
            );
        })
        .ui()
        .w_full()
        .max_w(Px(900.0));

        fret_cookbook::scaffold::centered_page_background_ui(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-icons-and-assets-basics")
        .window("cookbook-icons-and-assets-basics", (960.0, 860.0))
        // Register Lucide vendor icons during bootstrap so the icon SVG preload step (if enabled)
        // includes them.
        .register_icon_pack(fret_icons_lucide::register_vendor_icons)
        .setup(fret_cookbook::install_cookbook_defaults)
        .run_view::<IconsAndAssetsBasicsView>()
        .map_err(anyhow::Error::from)
}
