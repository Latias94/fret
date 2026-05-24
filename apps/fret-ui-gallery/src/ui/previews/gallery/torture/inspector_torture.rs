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
    let overscan = 12;
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

    let theme = theme.clone();
    let row = move |cx: &mut AppComponentCx<'_>, index: usize| {
        let zebra = (index % 2) == 0;
        let depth = (index % 8) as f32;
        let indent_px = Px(depth * 12.0);

        let selected_row_value = cx
            .get_model_copied(&selected_row, Invalidation::Layout)
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
        let row_theme = theme.clone();

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

                let background = if is_selected {
                    row_theme.color_token("accent")
                } else if zebra {
                    row_theme.color_token("muted")
                } else {
                    row_theme.color_token("background")
                };

                let mut row_props = decl_style::container_props(
                    &row_theme,
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(if st.pressed {
                            row_theme.color_token("accent")
                        } else {
                            background
                        }))
                        .p(Space::N2),
                    LayoutRefinement::default()
                        .w_full()
                        .h_px(MetricRef::Px(row_height)),
                );
                row_props.layout.overflow = fret_ui::element::Overflow::Clip;

                vec![cx.container(row_props, |cx| {
                    vec![
                        ui::h_flex(|_cx| vec![spacer, name, value])
                            .layout(LayoutRefinement::default().w_full().h_full())
                            .gap(Space::N2)
                            .items_center()
                            .into_element(cx),
                    ]
                })]
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
