use super::super::super::super::*;
use fret::AppComponentCx;
use fret_core::{AttributedText, Edges, TextSpan};
use fret_ui::element::{
    ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps, SemanticsProps, SizeStyle,
};
use fret_ui_kit::ColorRef;
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::typography::{UiTextSize, control_text_style, muted_foreground_color};

fn inspector_row_test_id(index: usize) -> Arc<str> {
    Arc::<str>::from(format!("ui-gallery-inspector-row-{index}"))
}

fn inspector_row_label_test_id(index: usize) -> Arc<str> {
    Arc::<str>::from(format!("ui-gallery-inspector-row-{index}-label"))
}

fn inspector_row_label_value_text(
    cx: &mut AppComponentCx<'_>,
    text_style: fret_core::TextStyle,
    label_color: CoreColor,
    value_color: CoreColor,
    label: Arc<str>,
    value: Arc<str>,
) -> AnyElement {
    let text = Arc::<str>::from(format!("{label} {value}"));

    let mut value_span = TextSpan::new(value.len());
    value_span.paint.fg = Some(value_color);

    let rich = AttributedText::new(
        text,
        Arc::<[TextSpan]>::from([TextSpan::new(label.len()), TextSpan::new(1), value_span]),
    );

    ui::rich_text(rich)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .text_style(text_style)
        .text_color(ColorRef::Color(label_color))
        .truncate()
        .overflow_hidden()
        .into_element(cx)
}

fn inspector_row_semantics(
    index: usize,
    len: usize,
    selected: bool,
    label: Arc<str>,
    test_id: Arc<str>,
) -> PressableA11y {
    let mut a11y = PressableA11y {
        role: Some(fret_core::SemanticsRole::ListItem),
        label: Some(label),
        selected,
        test_id: Some(test_id),
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

    // The inspector torture rows are fixed-height, so we can use the fixed virtual-list path and
    // avoid a full known-height rebuild on every retained-list refresh.
    let mut options =
        fret_ui::element::VirtualListOptions::fixed(row_height, overscan).keep_alive(keep_alive);
    options.key_cache = fret_ui::element::VirtualListKeyCacheMode::VisibleOnly;

    let row_gap_px = MetricRef::space(Space::N2).resolve(theme);
    let accent_color = theme.color_token("accent");
    let muted_color = theme.color_token("muted");
    let background_color = theme.color_token("background");
    let label_color = theme.color_token("foreground");
    let value_color = muted_foreground_color(theme);
    let text_style = control_text_style(theme, UiTextSize::Sm);
    let selected_row_value = cx
        .get_model_copied(&selected_row, Invalidation::Paint)
        .flatten();

    let root = cx.semantics_with_id(
        SemanticsProps {
            role: fret_core::SemanticsRole::List,
            test_id: Some(Arc::from("ui-gallery-inspector-root")),
            ..Default::default()
        },
        move |cx, root_id| {
            let row = move |cx: &mut AppComponentCx<'_>, index: usize| {
                let zebra = (index % 2) == 0;
                let depth = (index % 8) as f32;
                let indent_px = Px(depth * 12.0);

                let is_selected = selected_row_value == Some(index);
                let selected_row_for_activate = selected_row.clone();
                let on_select_row: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        let _ = host
                            .models_mut()
                            .update(&selected_row_for_activate, |value| *value = Some(index));
                        host.request_redraw(action_cx.window);
                    });

                let row_padding_left = Px(indent_px.0 + row_gap_px.0 * 2.0);
                let row_background = if is_selected {
                    accent_color
                } else if zebra {
                    muted_color
                } else {
                    background_color
                };

                let row_test_id = inspector_row_test_id(index);
                let row_label_test_id = inspector_row_label_test_id(index);
                let label = Arc::<str>::from(format!("prop_{index}"));
                let value = Arc::<str>::from(format!("value {index}"));
                let row_content = inspector_row_label_value_text(
                    cx,
                    text_style.clone(),
                    label_color,
                    value_color,
                    label.clone(),
                    value,
                )
                .attach_semantics(SemanticsDecoration::default().test_id(row_label_test_id));

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
                        a11y: inspector_row_semantics(
                            index,
                            len,
                            is_selected,
                            label.clone(),
                            row_test_id.clone(),
                        ),
                        ..Default::default()
                    },
                    move |cx, st| {
                        cx.pressable_add_on_activate(on_select_row.clone());
                        cx.pressable_add_on_activate_focus(Arc::new(
                            move |host, _action_cx, _reason| {
                                host.request_focus(root_id);
                            },
                        ));

                        let mut chrome = ContainerProps::default();
                        chrome.layout.size.width = Length::Fill;
                        chrome.layout.size.height = Length::Fill;
                        chrome.background = Some(if st.pressed {
                            accent_color
                        } else {
                            row_background
                        });
                        chrome.padding = Edges {
                            top: row_gap_px,
                            right: row_gap_px,
                            bottom: row_gap_px,
                            left: row_padding_left,
                        }
                        .into();

                        [ui::container_props(chrome, move |_cx| [row_content]).into_element(cx)]
                    },
                );

                row.test_id(row_test_id)
            };

            let list = cx.virtual_list_keyed_retained_with_layout_fn(
                list_layout,
                len,
                options,
                &scroll_handle,
                |i| i as fret_ui::ItemKey,
                row,
            );

            vec![list]
        },
    );

    vec![root]
}
