use super::super::super::super::*;

pub(in crate::ui) fn preview_inspector_torture(
    cx: &mut ElementContext<'_, App>,
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

    let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

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
    let row = move |cx: &mut ElementContext<'_, App>, index: usize| {
        let zebra = (index % 2) == 0;
        let background = if zebra {
            theme.color_required("muted")
        } else {
            theme.color_required("background")
        };

        let depth = (index % 8) as f32;
        let indent_px = Px(depth * 12.0);

        let name = cx.text(format!("prop_{index}"));
        let value = cx.text(format!("value {index}"));

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
            vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .gap(Space::N2)
                    .items_center(),
                |_cx| vec![spacer, name, value],
            )]
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
        CachedSubtreeProps::default().contained_layout(true),
        |_cx| vec![list],
    )]
}
