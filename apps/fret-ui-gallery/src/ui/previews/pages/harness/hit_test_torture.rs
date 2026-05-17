use super::super::super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use fret::AppComponentCx;

fn ui_gallery_hit_test_torture_stripes() -> usize {
    static VALUE: OnceLock<usize> = OnceLock::new();
    *VALUE.get_or_init(|| {
        std::env::var("FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(256)
            .clamp(2, 4096)
    })
}

fn ui_gallery_hit_test_torture_noise() -> usize {
    static VALUE: OnceLock<usize> = OnceLock::new();
    *VALUE.get_or_init(|| {
        std::env::var("FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(50_000)
            .clamp(0, 200_000)
    })
}

pub(in crate::ui) fn preview_hit_test_torture(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui::action::PointerMoveCx;

    let stripes = ui_gallery_hit_test_torture_stripes();
    let noise = ui_gallery_hit_test_torture_noise();
    let area_w = Px(1024.0);
    let area_h = Px(480.0);
    let stripe_w = Px((area_w.0 / stripes as f32).max(1.0));
    let on_pointer_move: fret_ui::action::OnPointerMove =
        Arc::new(|_host, _action_cx, _mv: PointerMoveCx| false);

    let surface = preview_hit_test_torture_surface(
        cx,
        stripes,
        noise,
        area_w,
        area_h,
        stripe_w,
        on_pointer_move,
    );

    let header = ui::v_flex(|cx| {
        vec![
            doc_layout::paragraph_text(cx, "Goal: make hit-test a measurable hotspot so bounds-tree vs fallback traversal A/B is meaningful."),
            doc_layout::control_readout_text(cx, format!(
                "Shape: {stripes} stripes ({} px each) plus {noise} 1x1 noise regions.",
                stripe_w.0
            )),
            doc_layout::control_readout_text(cx,
                "Env: FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES / FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE",
            ),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N2)
    .into_element(cx);

    let content = ui::v_flex(|_cx| vec![header, surface])
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N3)
        .into_element(cx);

    let section = DocSection::build(cx, "Hit Test Torture", content)
        .description("Pointer move scripts sweep the root surface to measure hit-test traversal cost under dense pointer-region trees.")
        .no_shell()
        .max_w(Px(1100.0));

    let page = doc_layout::render_doc_page(
        cx,
        Some("Stress the hit-test hot path with dense, stable pointer regions."),
        vec![section],
    );

    vec![page.into_element(cx)]
}

fn preview_hit_test_torture_surface(
    cx: &mut AppComponentCx<'_>,
    stripes: usize,
    noise: usize,
    area_w: Px,
    area_h: Px,
    stripe_w: Px,
    on_pointer_move: fret_ui::action::OnPointerMove,
) -> AnyElement {
    use fret_ui::element::{
        LayoutStyle, Length, PointerRegionProps, PositionStyle, SemanticsProps, StackProps,
    };

    cx.cached_subtree_with(
        CachedSubtreeProps::default().contain_layout_when_bounds_known(true),
        move |cx| {
            let mut stack_layout = LayoutStyle::default();
            stack_layout.size.width = Length::Px(area_w);
            stack_layout.size.height = Length::Px(area_h);
            stack_layout.overflow = fret_ui::element::Overflow::Clip;

            let stack = cx.stack_props(
                StackProps {
                    layout: stack_layout,
                },
                move |cx| {
                    let mut out: Vec<AnyElement> = Vec::with_capacity(stripes + noise);

                    for i in 0..stripes {
                        out.push(cx.keyed(("stripe", i), |cx| {
                            let mut pointer = PointerRegionProps::default();
                            pointer.layout.position = PositionStyle::Absolute;
                            pointer.layout.overflow = fret_ui::element::Overflow::Clip;
                            pointer.layout.inset.top = Some(Px(0.0)).into();
                            pointer.layout.inset.bottom = Some(Px(0.0)).into();
                            pointer.layout.inset.left = Some(Px(stripe_w.0 * i as f32)).into();
                            pointer.layout.size.width = Length::Px(stripe_w);

                            let on_pointer_move = on_pointer_move.clone();
                            cx.pointer_region(pointer, move |cx| {
                                cx.pointer_region_on_pointer_move(on_pointer_move.clone());
                                Vec::new()
                            })
                        }));
                    }

                    let noise_cols = (area_w.0.max(1.0) as usize).max(1);
                    let noise_cell = Px(1.0);
                    for idx in 0..noise {
                        out.push(cx.keyed(("noise", idx), |cx| {
                            let x = (idx % noise_cols) as f32 * noise_cell.0;
                            let y = (idx / noise_cols) as f32 * noise_cell.0;

                            let mut pointer = PointerRegionProps::default();
                            pointer.layout.position = PositionStyle::Absolute;
                            pointer.layout.overflow = fret_ui::element::Overflow::Clip;
                            pointer.layout.inset.left = Some(Px(x)).into();
                            pointer.layout.inset.top = Some(Px(y)).into();
                            pointer.layout.size.width = Length::Px(noise_cell);
                            pointer.layout.size.height = Length::Px(noise_cell);

                            let on_pointer_move = on_pointer_move.clone();
                            cx.pointer_region(pointer, move |cx| {
                                cx.pointer_region_on_pointer_move(on_pointer_move.clone());
                                Vec::new()
                            })
                        }));
                    }

                    out
                },
            );

            vec![cx.semantics(
                SemanticsProps {
                    role: fret_core::SemanticsRole::Group,
                    test_id: Some(Arc::<str>::from("ui-gallery-hit-test-torture-root")),
                    ..Default::default()
                },
                move |_cx| vec![stack],
            )]
        },
    )
}
