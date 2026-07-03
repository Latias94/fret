use std::sync::Arc;

use fret::app::prelude::*;
use fret::imui::{
    kit::{
        DebugDrawCommandKind, DebugDrawInteractionOptions, DebugDrawOptions, DebugDrawStrokeStyle,
        DebugDrawVertex,
    },
    prelude::*,
};
use fret::style::Space;
use fret_core::{Color, ImageId, Point, Px, Rect, Size, UvPoint};

const TEST_ID_ROOT: &str = "cookbook.imui_debug_draw.root";
const TEST_ID_CANVAS: &str = "cookbook.imui_debug_draw.canvas";
const TEST_ID_LIST_SUMMARY: &str = "cookbook.imui_debug_draw.summary.list";
const TEST_ID_COMMAND_SUMMARY: &str = "cookbook.imui_debug_draw.summary.commands";
const TEST_ID_RESPONSE_SUMMARY: &str = "cookbook.imui_debug_draw.summary.response";

struct ImUiDebugDrawBasicsView;

impl View for ImUiDebugDrawBasicsView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        ui::v_flex(|cx| {
            let debug_draw = ui::v_flex(|cx| {
                imui_in(cx, |ui| {
                    ui.text("Debug draw");

                    let response = ui.debug_draw_with_options(
                        "cookbook.imui_debug_draw.canvas",
                        DebugDrawOptions {
                            test_id: Some(Arc::from(TEST_ID_CANVAS)),
                            interaction: DebugDrawInteractionOptions::enabled()
                                .with_a11y_label("IMUI debug draw canvas"),
                            ..Default::default()
                        },
                        |draw| {
                            draw.push_clip_rect(rect(8.0, 8.0, 236.0, 92.0));
                            draw.add_rect_filled_multi_color(
                                rect(12.0, 12.0, 88.0, 72.0),
                                color(0x22_c5_5e),
                                color(0x3b_82_f6),
                                color(0xf9_73_16),
                                color(0xe1_1d_48),
                            );
                            draw.add_line_with_style(
                                point(16.0, 94.0),
                                point(98.0, 16.0),
                                color(0x11_18_27),
                                DebugDrawStrokeStyle::new(Px(2.0)),
                            );

                            draw.channels_split(3);
                            draw.channels_set_current(1);
                            draw.add_triangle_mesh(
                                [
                                    DebugDrawVertex::colored(point(116.0, 24.0), color(0xef_44_44)),
                                    DebugDrawVertex::colored(point(176.0, 24.0), color(0x22_c5_5e)),
                                    DebugDrawVertex::colored(point(146.0, 82.0), color(0x3b_82_f6)),
                                ],
                                [0, 1, 2],
                            );

                            draw.channels_set_current(2);
                            draw.add_image_triangle_mesh(
                                ImageId::default(),
                                [
                                    DebugDrawVertex::new(
                                        point(184.0, 24.0),
                                        UvPoint::new(0.0, 0.0),
                                        color(0xff_ff_ff),
                                    ),
                                    DebugDrawVertex::new(
                                        point(236.0, 24.0),
                                        UvPoint::new(1.0, 0.0),
                                        color(0xff_ff_ff),
                                    ),
                                    DebugDrawVertex::new(
                                        point(210.0, 82.0),
                                        UvPoint::new(0.5, 1.0),
                                        color(0xff_ff_ff),
                                    ),
                                ],
                                [0, 1, 2],
                            );

                            draw.channels_set_current(0);
                            draw.add_triangle_list([
                                DebugDrawVertex::colored(point(116.0, 90.0), color(0xf5_f5_f4)),
                                DebugDrawVertex::colored(point(136.0, 94.0), color(0xa8_a2_9e)),
                                DebugDrawVertex::colored(point(126.0, 106.0), color(0x44_40_3c)),
                            ]);
                            draw.channels_merge();
                            draw.pop_clip_rect();
                        },
                    );

                    let command_summaries = response.command_summaries();
                    let list_summary = response.list_summary();
                    let image_meshes = command_summaries
                        .iter()
                        .filter(|summary| summary.kind() == DebugDrawCommandKind::ImageTriangleMesh)
                        .count();
                    let max_channel = command_summaries
                        .iter()
                        .filter_map(|summary| summary.channel())
                        .max()
                        .unwrap_or(0);
                    let list_summary_text = format!(
                        "List: commands={}, triangles={}, max_clip_depth={}, final_clip_depth={}",
                        list_summary.command_count(),
                        list_summary.triangle_count(),
                        list_summary.max_clip_depth(),
                        list_summary.final_clip_depth()
                    );
                    let command_summary_text = format!(
                        "Commands: image_meshes={}, max_channel={}, first={:?}",
                        image_meshes,
                        max_channel,
                        command_summaries.first().map(|summary| summary.kind())
                    );
                    let response_summary_text = format!(
                        "Response: enabled={}, hovered={}, clicked={}, rect_ready={}",
                        response.response().enabled(),
                        response.hovered_like_imgui(),
                        response.clicked(),
                        response.rect().is_some()
                    );
                    ui.text(list_summary_text);
                    ui.text(command_summary_text);
                    ui.text(response_summary_text);
                })
            });

            ui::children![
                cx;
                shadcn::Label::new("Immediate-mode debug draw"),
                debug_draw,
                cx.text("List metadata").test_id(TEST_ID_LIST_SUMMARY),
                cx.text("Command metadata").test_id(TEST_ID_COMMAND_SUMMARY),
                cx.text("Response metadata").test_id(TEST_ID_RESPONSE_SUMMARY),
            ]
        })
        .size_full()
        .gap(Space::N4)
        .test_id(TEST_ID_ROOT)
        .into_element_in(cx)
        .into()
    }
}

fn point(x: f32, y: f32) -> Point {
    Point::new(Px(x), Px(y))
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(point(x, y), Size::new(Px(width), Px(height)))
}

fn color(hex: u32) -> Color {
    Color::from_srgb_hex_rgb(hex)
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-imui-debug-draw-basics")
        .window("cookbook-imui-debug-draw-basics", (720.0, 420.0))
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<ImUiDebugDrawBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
