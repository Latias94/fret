use super::super::super::super::*;
use fret::AppComponentCx;

fn inspector_row_label_text<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement
where
    T: Into<Arc<str>>,
{
    fret_ui_kit::declarative::text::text_list_row_label(cx, text)
}

fn inspector_row_value_text<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement
where
    T: Into<Arc<str>>,
{
    doc_layout::control_readout_text(cx, text)
}

pub(in crate::ui) fn preview_inspector_torture(
    cx: &mut AppComponentCx<'_>,
    theme: &Theme,
) -> Vec<AnyElement> {
    let len: usize = std::env::var("FRET_UI_GALLERY_INSPECTOR_LEN")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(50_000)
        .clamp(16, 200_000);
    let row_height = Px(28.0);
    let overscan = 12;
    let keep_alive: usize = std::env::var("FRET_UI_GALLERY_INSPECTOR_KEEP_ALIVE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0)
        .clamp(0, 4096);

    let scroll_handle = cx.slot_state(VirtualListScrollHandle::new, |h| h.clone());

    let list_layout = fret_ui::element::LayoutStyle {
        size: fret_ui::element::SizeStyle {
            width: fret_ui::element::Length::Fill,
            height: fret_ui::element::Length::Px(Px(460.0)),
            ..Default::default()
        },
        overflow: fret_ui::element::Overflow::Clip,
        ..Default::default()
    };

    let options =
        fret_ui::element::VirtualListOptions::known(row_height, overscan, move |_index| row_height)
            .keep_alive(keep_alive);

    let theme = theme.clone();
    let row = move |cx: &mut AppComponentCx<'_>, index: usize| {
        let zebra = (index % 2) == 0;
        let background = if zebra {
            theme.color_token("muted")
        } else {
            theme.color_token("background")
        };

        let depth = (index % 8) as f32;
        let indent_px = Px(depth * 12.0);

        let name = inspector_row_label_text(cx, format!("prop_{index}"));
        let value = inspector_row_value_text(cx, format!("value {index}"));

        let spacer = cx.container(
            fret_ui::element::ContainerProps {
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: fret_ui::element::Length::Px(indent_px),
                        height: fret_ui::element::Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        );

        let mut row_props = decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(background))
                .p(Space::N2),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(row_height)),
        );
        row_props.layout.overflow = fret_ui::element::Overflow::Clip;

        let row = cx.container(row_props, |cx| {
            vec![
                ui::h_flex(|_cx| vec![spacer, name, value])
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .gap(Space::N2)
                    .items_center()
                    .into_element(cx),
            ]
        });

        row.test_id(format!("ui-gallery-inspector-row-{index}-label"))
    };

    let list = cx.virtual_list_keyed_retained_with_layout_fn(
        list_layout,
        len,
        options,
        &scroll_handle,
        |i| i as fret_ui::ItemKey,
        row,
    );

    let list = list.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::List)
            .test_id("ui-gallery-inspector-root"),
    );

    vec![cx.cached_subtree_with(
        CachedSubtreeProps::default().contain_layout_when_bounds_known(true),
        |_cx| vec![list],
    )]
}
