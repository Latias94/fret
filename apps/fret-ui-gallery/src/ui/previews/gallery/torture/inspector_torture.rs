use super::super::super::super::*;
use fret::AppComponentCx;
use fret_ui::element::{LayoutStyle, Length, PressableA11y, PressableProps, SizeStyle};

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

fn inspector_row_test_id(index: usize) -> Arc<str> {
    Arc::<str>::from(format!("ui-gallery-inspector-row-{index}"))
}

fn inspector_row_label_test_id(index: usize) -> Arc<str> {
    Arc::<str>::from(format!("ui-gallery-inspector-row-{index}-label"))
}

fn inspector_row_value_test_id(index: usize) -> Arc<str> {
    Arc::<str>::from(format!("ui-gallery-inspector-row-{index}-value"))
}

fn inspector_row_semantics(index: usize, len: usize, selected: bool) -> PressableA11y {
    let mut a11y = PressableA11y {
        role: Some(fret_core::SemanticsRole::ListItem),
        label: Some(Arc::<str>::from(format!("prop_{index}"))),
        selected,
        test_id: Some(inspector_row_test_id(index)),
        ..Default::default()
    };

    if let (Ok(pos_in_set), Ok(set_size)) =
        (u32::try_from(index.saturating_add(1)), u32::try_from(len))
    {
        a11y.pos_in_set = Some(pos_in_set);
        a11y.set_size = Some(set_size);
    }

    a11y
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
    // Keep the retained window tight on the inspector torture surface so we do not pay for a
    // much wider offscreen row buffer than the direct-entry probe needs.
    let overscan = 8;
    let keep_alive: usize = std::env::var("FRET_UI_GALLERY_INSPECTOR_KEEP_ALIVE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0)
        .clamp(0, 4096);

    let scroll_handle = cx.slot_state(VirtualListScrollHandle::new, |h| h.clone());
    let selected_row = cx.local_model_keyed("inspector_selected_row", || None::<usize>);

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

    let row_gap_px = MetricRef::space(Space::N2).resolve(theme);
    let accent_color = theme.color_token("accent");
    let muted_color = theme.color_token("muted");
    let background_color = theme.color_token("background");
    let row = move |cx: &mut AppComponentCx<'_>, index: usize| {
        let zebra = (index % 2) == 0;
        let depth = (index % 8) as f32;
        let indent_px = Px(depth * 12.0);

        let selected_row_value = cx
            .get_model_copied(&selected_row, Invalidation::Paint)
            .flatten();
        let is_selected = selected_row_value == Some(index);
        let selected_row_for_activate = selected_row.clone();
        let on_select_row: fret_ui::action::OnActivate =
            Arc::new(move |host, action_cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&selected_row_for_activate, |value| *value = Some(index));
                host.request_redraw(action_cx.window);
            });

        let name = inspector_row_label_text(cx, format!("prop_{index}"))
            .test_id(inspector_row_label_test_id(index));
        let value = inspector_row_value_text(cx, format!("value {index}"))
            .test_id(inspector_row_value_test_id(index));
        let row_padding_left = Px(indent_px.0 + row_gap_px.0 * 2.0);

        let row = cx.pressable(
            PressableProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Px(row_height),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                a11y: inspector_row_semantics(index, len, is_selected),
                ..Default::default()
            },
            move |cx, st| {
                cx.pressable_add_on_activate(on_select_row.clone());

                let row_background = if is_selected {
                    accent_color
                } else if zebra {
                    muted_color
                } else {
                    background_color
                };

                let row_content = ui::h_flex(|_cx| vec![name, value])
                    .bg(ColorRef::Color(if st.pressed {
                        accent_color
                    } else {
                        row_background
                    }))
                    .overflow_hidden()
                    .paddings(Edges4::trbl(
                        MetricRef::space(Space::N2),
                        MetricRef::space(Space::N2),
                        MetricRef::space(Space::N2),
                        MetricRef::Px(row_padding_left),
                    ))
                    .layout(
                        LayoutRefinement::default()
                            .w_full()
                            .h_px(MetricRef::Px(row_height)),
                    )
                    .gap(Space::N2)
                    .items_center()
                    .into_element(cx);

                vec![row_content]
            },
        );

        row.test_id(inspector_row_test_id(index))
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
