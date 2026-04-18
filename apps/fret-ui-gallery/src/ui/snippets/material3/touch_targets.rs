pub const SOURCE: &str = include_str!("touch_targets.rs");

// region: example
use std::sync::Arc;

use fret::{AppComponentCx, UiChild};
use fret_core::{Color, Corners, DrawOrder, Edges, Paint, Point, Px, Rect, SceneOp, Size};
use fret_ui::element::{CanvasProps, FlexProps, StackProps};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_material3 as material3;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    use fret_icons::ids;

    let checkbox_root = material3::Checkbox::uncontrolled(cx, false);
    let material3_checkbox = checkbox_root.checked_model();
    let switch_root = material3::Switch::uncontrolled(cx, false);
    let material3_switch = switch_root.selected_model();
    let radio_group_root = material3::RadioGroup::uncontrolled(cx, None::<Arc<str>>);
    let material3_radio_value = radio_group_root.value_model();
    let tabs_root = material3::Tabs::uncontrolled(cx, "overview");
    let material3_tabs_value = tabs_root.value_model();

    let min = cx.with_theme(|theme| {
        theme
            .metric_by_key("md.sys.layout.minimum-touch-target.size")
            .unwrap_or(Px(48.0))
    });

    let target_overlay =
        |cx: &mut AppComponentCx<'_>, label: &'static str, chrome: Option<Size>, child| {
            let min = min;

            let stack = cx.stack_props(
                StackProps {
                    layout: {
                        let mut l = fret_ui::element::LayoutStyle::default();
                        l.overflow = fret_ui::element::Overflow::Visible;
                        l
                    },
                },
                move |cx| {
                    let mut canvas = CanvasProps::default();
                    canvas.layout.position = fret_ui::element::PositionStyle::Absolute;
                    canvas.layout.inset.top = Some(Px(0.0)).into();
                    canvas.layout.inset.right = Some(Px(0.0)).into();
                    canvas.layout.inset.bottom = Some(Px(0.0)).into();
                    canvas.layout.inset.left = Some(Px(0.0)).into();

                    let overlay = cx.canvas(canvas, move |p| {
                        let bounds = p.bounds();
                        let center = Point::new(
                            Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
                            Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
                        );

                        let min_rect = Rect::new(
                            Point::new(Px(center.x.0 - min.0 * 0.5), Px(center.y.0 - min.0 * 0.5)),
                            Size::new(min, min),
                        );

                        let chrome_rect = chrome.map(|chrome| {
                            Rect::new(
                                Point::new(
                                    Px(center.x.0 - chrome.width.0 * 0.5),
                                    Px(center.y.0 - chrome.height.0 * 0.5),
                                ),
                                chrome,
                            )
                        });

                        fn outline(
                            p: &mut fret_ui::canvas::CanvasPainter<'_>,
                            order: u32,
                            rect: Rect,
                            color: Color,
                        ) {
                            p.scene().push(SceneOp::Quad {
                                order: DrawOrder(order),
                                rect,
                                background: Paint::TRANSPARENT.into(),

                                border: Edges::all(Px(1.0)),
                                border_paint: Paint::Solid(color).into(),

                                corner_radii: Corners::all(Px(0.0)),
                            });
                        }

                        fn srgb(hex: u32, a: f32) -> Color {
                            let mut c = Color::from_srgb_hex_rgb(hex);
                            c.a = a.clamp(0.0, 1.0);
                            c
                        }

                        outline(p, 0, bounds, srgb(0x1a_cc_33, 0.8));
                        outline(p, 1, min_rect, srgb(0xf2_bf_33, 0.9));
                        if let Some(chrome_rect) = chrome_rect {
                            outline(p, 2, chrome_rect, srgb(0x33_bf_f2, 0.9));
                        }
                    });

                    vec![child, overlay]
                },
            );

            shadcn::card(|cx| {
                ui::children![
                    cx;
                    shadcn::card_header(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_title(label),
                            shadcn::card_description(match chrome {
                                Some(chrome) => format!(
                                    "min={}px, chrome={}x{}px",
                                    min.0, chrome.width.0, chrome.height.0
                                ),
                                None => format!("min={}px", min.0),
                            }),
                        ]
                    }),
                    shadcn::card_content(|_cx| vec![stack]),
                ]
            })
            .refine_layout(LayoutRefinement::default().w_px(Px(280.0)).min_w_0())
            .into_element(cx)
        };

    let checkbox_chrome = {
        let size = cx.with_theme(|theme| {
            theme
                .metric_by_key("md.comp.checkbox.state-layer.size")
                .unwrap_or(Px(40.0))
        });
        Size::new(size, size)
    };
    let radio_chrome = {
        let size = cx.with_theme(|theme| {
            theme
                .metric_by_key("md.comp.radio-button.state-layer.size")
                .unwrap_or(Px(40.0))
        });
        Size::new(size, size)
    };
    let switch_chrome = {
        let (width, height) = cx.with_theme(|theme| {
            (
                theme
                    .metric_by_key("md.comp.switch.track.width")
                    .unwrap_or(Px(52.0)),
                theme
                    .metric_by_key("md.comp.switch.state-layer.size")
                    .unwrap_or(Px(40.0)),
            )
        });
        Size::new(width, height)
    };
    let icon_button_chrome = {
        let size = cx.with_theme(|theme| {
            theme
                .metric_by_key("md.comp.icon-button.small.container.height")
                .unwrap_or(Px(40.0))
        });
        Size::new(size, size)
    };

    let grid = {
        let mut props = FlexProps::default();
        props.layout = fret_ui::element::LayoutStyle::default();
        props.layout.size.width = fret_ui::element::Length::Fill;
        props.direction = fret_core::Axis::Horizontal;
        props.wrap = true;
        props.gap = Px(16.0).into();
        props.align = fret_ui::element::CrossAlign::Start;
        props.justify = fret_ui::element::MainAlign::Start;

        cx.flex(props, move |cx| {
            let checkbox = checkbox_root
                .clone()
                .a11y_label("Material3 checkbox")
                .test_id("ui-gallery-material3-touch-target-checkbox")
                .into_element(cx);
            let radio = material3::Radio::new_value("alpha", material3_radio_value.clone())
                .a11y_label("Material3 radio")
                .test_id("ui-gallery-material3-touch-target-radio")
                .into_element(cx);
            let switch = switch_root
                .clone()
                .a11y_label("Material3 switch")
                .test_id("ui-gallery-material3-touch-target-switch")
                .into_element(cx);
            let icon_button = material3::IconButton::new(ids::ui::SETTINGS)
                .a11y_label("Material3 icon button")
                .test_id("ui-gallery-material3-touch-target-icon-button")
                .into_element(cx);
            let tabs = material3::Tabs::new(material3_tabs_value.clone())
                .a11y_label("Material3 tabs (touch targets)")
                .test_id("ui-gallery-material3-touch-target-tabs")
                .scrollable(true)
                .items(vec![
                    material3::TabItem::new("overview", "A")
                        .a11y_label("Material3 tab")
                        .test_id("ui-gallery-material3-touch-target-tab"),
                ])
                .into_element(cx);

            vec![
                target_overlay(cx, "Checkbox", Some(checkbox_chrome), checkbox),
                target_overlay(cx, "Radio", Some(radio_chrome), radio),
                target_overlay(cx, "Switch", Some(switch_chrome), switch),
                target_overlay(cx, "Icon Button", Some(icon_button_chrome), icon_button),
                target_overlay(cx, "Tabs (scrollable, 1 item)", None, tabs),
            ]
        })
    };

    ui::v_flex(|cx| {
            vec![
                cx.text(
                    "Touch target overlay legend: green=bounds, yellow=min 48x48, cyan=token chrome (if shown).",
                ),
                grid,
            ]
        })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_start().into_element(cx)
}

// endregion: example
