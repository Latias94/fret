//! Immediate-mode bullet-list helper.

use std::sync::Arc;

use fret_core::{Corners, Px};
use fret_ui::UiHost;
use fret_ui::element::{ContainerProps, Length, MarginEdges, SemanticsDecoration};
use fret_ui::{ElementContext, Theme};

use super::{BulletTextOptions, UiWriterImUiFacadeExt};

const BULLET_TRACK_WIDTH: Px = Px(14.0);
const BULLET_DIAMETER: Px = Px(6.0);
const BULLET_TOP_OFFSET: Px = Px(6.0);

pub(super) fn bullet_text_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    text: Arc<str>,
    options: BulletTextOptions,
) {
    let element = ui.with_cx_mut(|cx| bullet_text_element(cx, text, options));
    ui.add(element);
}

fn bullet_text_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    options: BulletTextOptions,
) -> fret_ui::element::AnyElement {
    let indicator_test_id = options
        .test_id
        .as_ref()
        .map(|base| Arc::from(format!("{base}.indicator")));
    let label_test_id = options
        .test_id
        .as_ref()
        .map(|base| Arc::from(format!("{base}.label")));

    let theme = Theme::global(&*cx.app);
    let color = theme
        .color_by_key("foreground")
        .unwrap_or_else(|| theme.color_token("foreground"));

    let mut bullet_props = ContainerProps::default();
    bullet_props.layout.size.width = Length::Px(BULLET_DIAMETER);
    bullet_props.layout.size.height = Length::Px(BULLET_DIAMETER);
    bullet_props.layout.flex.shrink = 0.0;
    bullet_props.layout.margin = MarginEdges {
        top: BULLET_TOP_OFFSET.into(),
        right: Px(0.0).into(),
        bottom: Px(0.0).into(),
        left: Px(0.0).into(),
    };
    bullet_props.background = Some(color);
    bullet_props.corner_radii = Corners::all(Px(999.0));

    let mut bullet = cx.container(bullet_props, |_cx| Vec::new());
    if let Some(test_id) = indicator_test_id {
        bullet = bullet.attach_semantics(SemanticsDecoration::default().test_id(test_id));
    }

    let mut track_props = ContainerProps::default();
    track_props.layout.size.width = Length::Px(BULLET_TRACK_WIDTH);
    track_props.layout.size.height = Length::Auto;
    track_props.layout.flex.shrink = 0.0;
    let bullet_track = cx.container(track_props, move |_cx| vec![bullet]);

    let mut label =
        crate::declarative::text::text_compact_paragraph(cx, text).inherit_foreground(color);
    if let Some(test_id) = label_test_id {
        label = label.attach_semantics(SemanticsDecoration::default().test_id(test_id));
    }

    let row = crate::ui::h_flex(move |_cx| vec![bullet_track, label])
        .gap_metric(Px(4.0).into())
        .items(crate::Items::Start)
        .no_wrap();

    if let Some(test_id) = options.test_id {
        row.test_id(test_id).into_element(cx)
    } else {
        row.into_element(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size, TextOverflow, TextWrap};
    use fret_ui::element::ElementKind;
    use fret_ui::elements;

    fn first_text<'a>(
        root: &'a fret_ui::element::AnyElement,
        expected: &str,
    ) -> Option<&'a fret_ui::element::AnyElement> {
        match &root.kind {
            ElementKind::Text(props) if props.text.as_ref() == expected => Some(root),
            _ => root
                .children
                .iter()
                .find_map(|child| first_text(child, expected)),
        }
    }

    fn test_bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(160.0)),
        )
    }

    #[test]
    fn bullet_text_uses_shared_compact_paragraph_role() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
            bullet_text_element(
                cx,
                Arc::from("Long bullet body that may wrap inside an editor panel"),
                BulletTextOptions::default(),
            )
        });
        let theme = Theme::global(&app);
        let expected_foreground = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"));

        let text = first_text(&el, "Long bullet body that may wrap inside an editor panel")
            .expect("expected bullet label text");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected bullet label to be text");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert!(text.inherited_text_style.is_some());
        assert_eq!(text.inherited_foreground, Some(expected_foreground));
    }
}
