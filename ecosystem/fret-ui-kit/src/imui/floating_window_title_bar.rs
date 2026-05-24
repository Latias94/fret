use std::sync::Arc;

use fret_authoring::UiWriter as _;
use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::UiHost;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::AnyElement;

pub(super) fn floating_window_title_bar_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    area: super::FloatingAreaContext,
    title: Arc<str>,
    open_model: Option<Model<bool>>,
    title_bar_test_id: Arc<str>,
    close_button_test_id: Arc<str>,
    resizable_layout: bool,
    options: super::FloatingWindowOptions,
) -> AnyElement {
    let row = super::floating_window_title_bar_props::title_bar_row_props(resizable_layout);

    let title = title.clone();
    let title_bar_test_id = title_bar_test_id.clone();
    let open_for_key = open_model.clone();
    let can_interact = options.inputs_enabled;
    let can_close = can_interact && options.closable && open_for_key.is_some();
    let can_collapse = can_interact && options.collapsible;
    let can_move = can_interact && options.movable;
    let on_left_double_click: Option<super::OnFloatingAreaLeftDoubleClick> = if can_collapse {
        Some(Arc::new(
            move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                  acx: fret_ui::action::ActionCx| {
                host.record_transient_event(acx, super::KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED);
                host.notify(acx);
            },
        ))
    } else {
        None
    };

    let drag_surface = super::floating_area_drag_surface_element(
        cx,
        area,
        super::floating_window_title_bar_props::title_bar_drag_surface_props(
            resizable_layout,
            can_interact,
        ),
        on_left_double_click,
        can_move,
        options.activate_on_click,
        move |cx, region_id| {
            cx.key_clear_on_key_down_for(region_id);
            if can_close && let Some(open) = open_for_key.clone() {
                cx.key_on_key_down_for(
                    region_id,
                    Arc::new(move |host, acx, down| {
                        if down.key != KeyCode::Escape || down.repeat {
                            return false;
                        }
                        let _ = host.update_model(&open, |v: &mut bool| {
                            *v = false;
                        });
                        host.notify(acx);
                        true
                    }),
                );
            }
        },
        move |ui| {
            let element = ui.with_cx_mut(|cx| {
                let title = if resizable_layout {
                    crate::declarative::text::text_chrome_title(cx, title.clone())
                } else {
                    crate::declarative::text::text_section_chrome_label(cx, title.clone())
                };
                title.attach_semantics(
                    fret_ui::element::SemanticsDecoration::default()
                        .test_id(title_bar_test_id.clone()),
                )
            });
            ui.add(element);
        },
    );

    let close = (options.inputs_enabled && options.closable)
        .then(|| open_model.clone())
        .flatten()
        .map(|open| {
            let props = super::floating_window_title_bar_props::title_bar_close_button_props(
                close_button_test_id.clone(),
            );
            cx.pressable(props, move |cx, _state| {
                cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                    let _ = host.update_model(&open, |v: &mut bool| {
                        *v = false;
                    });
                    host.notify(acx);
                }));
                vec![floating_window_close_glyph_text(cx)]
            })
        });

    cx.row(row, move |_cx| {
        let mut out = vec![drag_surface];
        if let Some(close) = close {
            out.push(close);
        }
        out
    })
}

pub(super) fn floating_window_close_glyph_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> AnyElement {
    crate::declarative::text::text_chrome_glyph(cx, Arc::<str>::from("\u{00D7}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size, TextOverflow, TextWrap};
    use fret_ui::element::{ElementKind, Length};
    use fret_ui::elements;

    fn test_bounds() -> Rect {
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(48.0)))
    }

    #[test]
    fn floating_window_close_glyph_uses_shared_chrome_glyph_text_role() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
            floating_window_close_glyph_text(cx)
        });

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected floating window close glyph to be text");
        };

        assert_eq!(props.text.as_ref(), "\u{00D7}");
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert!(el.inherited_text_style.is_some());
    }
}
