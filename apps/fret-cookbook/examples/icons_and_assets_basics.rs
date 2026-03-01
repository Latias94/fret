use fret::prelude::*;
use fret_core::ImageColorSpace;
use fret_ui::element::{ImageProps, SvgIconProps};
use fret_ui_assets::ui::{ImageSourceElementContextExt as _, SvgFileElementContextExt as _};
use std::path::PathBuf;
use std::sync::Arc;

const TEST_ID_ROOT: &str = "cookbook.icons_and_assets_basics.root";
const TEST_ID_BUMP_RELOAD: &str = "cookbook.icons_and_assets_basics.bump_reload";
const TEST_ID_PANEL_ICONS: &str = "cookbook.icons_and_assets_basics.panel.icons";
const TEST_ID_PANEL_SVG: &str = "cookbook.icons_and_assets_basics.panel.svg";
const TEST_ID_PANEL_IMAGE: &str = "cookbook.icons_and_assets_basics.panel.image";
const TEST_ID_IMAGE_STATUS: &str = "cookbook.icons_and_assets_basics.image.status";
const TEST_ID_SVG_STATUS: &str = "cookbook.icons_and_assets_basics.svg.status";

fn file_path(s: &str) -> Arc<PathBuf> {
    Arc::new(PathBuf::from(s))
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

struct IconsAndAssetsBasicsState {
    window: AppWindowId,
    assets_reload_bumps: Model<u64>,
    applied_assets_reload_bumps: u64,
    file_image: fret_ui_assets::ImageSource,
    memory_image: fret_ui_assets::ImageSource,
    svg_file: fret_ui_assets::SvgFileSource,
}

struct IconsAndAssetsBasicsProgram;

impl MvuProgram for IconsAndAssetsBasicsProgram {
    type State = IconsAndAssetsBasicsState;
    type Message = ();

    fn init(app: &mut App, window: AppWindowId) -> Self::State {
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
            fret_ui_assets::ImageSource::from_path(file_path("assets/textures/test.jpg"));
        let memory_image = fret_ui_assets::ImageSource::rgba8(
            128,
            128,
            checkerboard_rgba8(128, 128, 16),
            ImageColorSpace::Srgb,
        );

        // `SvgIconProps` is an icon-style SVG element (monochrome + currentColor), not a full SVG
        // image renderer. Use an icon-like SVG (stroke=currentColor, fill=none) for this demo.
        let svg_file =
            fret_ui_assets::SvgFileSource::from_path(file_path("assets/demo/icon-search.svg"));

        Self::State {
            window,
            assets_reload_bumps: app.models_mut().insert(0),
            applied_assets_reload_bumps: 0,
            file_image,
            memory_image,
            svg_file,
        }
    }

    fn update(_app: &mut App, _state: &mut Self::State, _message: Self::Message) {}

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        _msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();

        let bumps_model = state.assets_reload_bumps.clone();
        let bump_reload: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&bumps_model, |v| {
                *v = v.wrapping_add(1);
            });
            host.request_redraw(action_cx.window);
            host.push_effect(Effect::RequestAnimationFrame(action_cx.window));
        });

        let bumps = state
            .assets_reload_bumps
            .read(&mut *cx.app, |_host, v| *v)
            .ok()
            .unwrap_or(0);
        if bumps != state.applied_assets_reload_bumps {
            fret_ui_assets::bump_ui_assets_reload_epoch(&mut *cx.app);
            state.applied_assets_reload_bumps = bumps;
            cx.app.request_redraw(state.window);
            cx.app
                .push_effect(Effect::RequestAnimationFrame(state.window));
        }

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("Icons + assets basics").into_element(cx),
            shadcn::CardDescription::new(
                "Icon packs (lucide/radix), semantic ui.* aliases, and file-based SVG/images via fret-ui-assets.",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let actions = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap_x(Space::N2)
                .items_center()
                .justify_center()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                [
                    shadcn::Button::new("Bump assets reload epoch")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .size(shadcn::ButtonSize::Sm)
                        .icon(IconId::new_static("ui.reset"))
                        .on_activate(bump_reload)
                        .into_element(cx)
                        .test_id(TEST_ID_BUMP_RELOAD),
                    shadcn::Badge::new("Tip: edit the files under `assets/` and click reload.")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                ]
            },
        );

        let icon_row =
            |cx: &mut ElementContext<'_, App>, title: &str, ids: [IconId; 3]| -> AnyElement {
                let buttons = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap_x(Space::N2)
                        .items_center()
                        .layout(LayoutRefinement::default().w_full()),
                    |cx| {
                        [
                            shadcn::Button::new("Search")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .leading_icon(ids[0].clone())
                                .into_element(cx),
                            shadcn::Button::new("Close")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .leading_icon(ids[1].clone())
                                .into_element(cx),
                            shadcn::Button::new("Copy")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .leading_icon(ids[2].clone())
                                .into_element(cx),
                        ]
                    },
                );
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap_y(Space::N2)
                        .layout(LayoutRefinement::default().w_full()),
                    |cx| [shadcn::Label::new(title).into_element(cx), buttons],
                )
            };

        let icons_panel = shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Icons").into_element(cx),
                shadcn::CardDescription::new(
                    "IconId is renderer-agnostic. Packs register data; components consume semantic ids (ui.*).",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap_y(Space::N4)
                        .layout(LayoutRefinement::default().w_full()),
                    |cx| {
                        [
                            icon_row(
                                cx,
                                "Semantic ids (ui.*)",
                                [
                                    IconId::new_static("ui.search"),
                                    IconId::new_static("ui.close"),
                                    IconId::new_static("ui.copy"),
                                ],
                            ),
                            icon_row(
                                cx,
                                "Vendor ids (lucide.*)",
                                [
                                    IconId::new_static("lucide.search"),
                                    IconId::new_static("lucide.x"),
                                    IconId::new_static("lucide.copy"),
                                ],
                            ),
                            icon_row(
                                cx,
                                "Vendor ids (radix.*)",
                                [
                                    IconId::new_static("radix.magnifying-glass"),
                                    IconId::new_static("radix.cross-2"),
                                    IconId::new_static("radix.copy"),
                                ],
                            ),
                        ]
                    },
                )
                ,
            ])
            .into_element(cx),
        ])
        .ui()
        .w_full()
        .into_element(cx)
        .test_id(TEST_ID_PANEL_ICONS);

        let file_image_state = cx.use_image_source_state(&state.file_image);
        let memory_image_state = cx.use_image_source_state(&state.memory_image);

        let image_status = match file_image_state.status {
            fret_ui_assets::image_asset_state::ImageLoadingStatus::Idle => "idle",
            fret_ui_assets::image_asset_state::ImageLoadingStatus::Loading => "loading",
            fret_ui_assets::image_asset_state::ImageLoadingStatus::Loaded => "ready",
            fret_ui_assets::image_asset_state::ImageLoadingStatus::Error => "error",
        };

        let render_image = |cx: &mut ElementContext<'_, App>,
                            title: &str,
                            st: &fret_ui_assets::ImageSourceState|
         -> AnyElement {
            let box_el = ui::container(cx, |cx| {
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
            .overflow_hidden()
            .into_element(cx);

            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap_y(Space::N2)
                    .layout(LayoutRefinement::default().w_full()),
                |cx| [shadcn::Label::new(title).into_element(cx), box_el],
            )
        };

        let image_panel = shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Images").into_element(cx),
                shadcn::CardDescription::new(
                    "File-based decode is async; in-memory RGBA8 is immediate and useful for deterministic demos.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap_y(Space::N3)
                        .layout(LayoutRefinement::default().w_full()),
                    |cx| {
                        [
                            ui::h_flex(cx, |cx| {
                                [
                                    shadcn::Label::new("File image status:").into_element(cx),
                                    shadcn::Badge::new(image_status)
                                        .variant(shadcn::BadgeVariant::Secondary)
                                        .into_element(cx)
                                        .test_id(TEST_ID_IMAGE_STATUS),
                                ]
                            })
                            .gap(Space::N2)
                            .items_center()
                            .into_element(cx),
                            stack::hstack(
                                cx,
                                stack::HStackProps::default()
                                    .gap_x(Space::N4)
                                    .items_center()
                                    .layout(LayoutRefinement::default().w_full()),
                                |cx| {
                                    [
                                        render_image(cx, "From path: `assets/textures/test.jpg`", &file_image_state),
                                        render_image(cx, "From RGBA8 buffer", &memory_image_state),
                                    ]
                                },
                            ),
                        ]
                    },
                )
                ,
            ])
            .into_element(cx),
        ])
        .ui()
        .w_full()
        .into_element(cx)
        .test_id(TEST_ID_PANEL_IMAGE);

        let svg_source = cx.svg_source_from_file(&state.svg_file);
        let svg_status = if svg_source.is_some() {
            "ready"
        } else {
            "loading"
        };

        let svg_box = ui::container(cx, |cx| {
            if let Some(svg) = svg_source {
                let mut props = SvgIconProps::new(svg);
                props.layout = style::layout_style(
                    &theme,
                    LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(160.0)),
                );
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
        .into_element(cx);

        let svg_panel = shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("SVG icon from file").into_element(cx),
                shadcn::CardDescription::new(
                    "Loads an icon-style SVG from disk via `SvgFileSource` + `UiAssetsReloadEpoch` (ViewCache-safe dev reload).",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap_y(Space::N3)
                    .layout(LayoutRefinement::default().w_full()),
                |cx| {
                    [
                        ui::h_flex(cx, |cx| {
                            [
                                shadcn::Label::new("SVG status:").into_element(cx),
                                shadcn::Badge::new(svg_status)
                                    .variant(shadcn::BadgeVariant::Secondary)
                                    .into_element(cx)
                                    .test_id(TEST_ID_SVG_STATUS),
                            ]
                        })
                        .gap(Space::N2)
                        .items_center()
                        .into_element(cx),
                        svg_box,
                    ]
                },
            )])
            .into_element(cx),
        ])
        .ui()
        .w_full()
        .into_element(cx)
        .test_id(TEST_ID_PANEL_SVG);

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap_y(Space::N5)
                .layout(LayoutRefinement::default().w_full()),
            |_cx| [actions, icons_panel, svg_panel, image_panel],
        );

        let card =
            shadcn::Card::new([header, shadcn::CardContent::new([content]).into_element(cx)])
                .ui()
                .w_full()
                .max_w(Px(900.0))
                .into_element(cx);

        ui::container(cx, |cx| {
            [ui::v_flex(cx, |_cx| [card])
                .gap(Space::N6)
                .items_center()
                .justify_center()
                .size_full()
                .into_element(cx)]
        })
        .bg(ColorRef::Color(theme.color_token("background")))
        .p(Space::N6)
        .into_element(cx)
        .test_id(TEST_ID_ROOT)
        .into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-icons-and-assets-basics")
        .window("cookbook-icons-and-assets-basics", (960.0, 860.0))
        // Register Radix vendor icons during bootstrap so the icon SVG preload step (if enabled)
        // includes them.
        .register_icon_pack(fret_icons_radix::register_vendor_icons)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_mvu::<IconsAndAssetsBasicsProgram>()
        .map_err(anyhow::Error::from)
}
