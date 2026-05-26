use std::sync::Arc;

use fret_core::{Color, TextStyle};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::typography;

pub(crate) fn menu_item_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    style: &TextStyle,
    foreground: Color,
) -> AnyElement {
    let mut refinement = typography::composable_refinement_from_style(style);
    refinement.weight = Some(style.weight);

    decl_text::text_list_row_label(cx, label)
        .inherit_text_style(refinement)
        .inherit_foreground(foreground)
}

pub(crate) fn menu_section_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    style: &TextStyle,
    foreground: Color,
) -> AnyElement {
    let mut refinement = typography::composable_refinement_from_style(style);
    refinement.weight = Some(fret_core::FontWeight::MEDIUM);

    decl_text::text_list_row_label(cx, label)
        .inherit_text_style(refinement)
        .inherit_foreground(foreground)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, FontId, FontWeight, Point, Px, Rect, Size, TextOverflow, TextWrap,
    };
    use fret_ui::element::{ElementKind, Length};

    #[test]
    fn menu_item_label_uses_shared_list_row_role_with_menu_refinement() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(40.0)));

        let foreground = fret_core::Color {
            r: 0.2,
            g: 0.4,
            b: 0.8,
            a: 1.0,
        };
        let mut style = typography::fixed_line_box_style(FontId::ui(), Px(14.0), Px(20.0));
        style.weight = FontWeight::NORMAL;
        style.letter_spacing_em = Some(0.02);

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            menu_item_label(cx, Arc::from("Open recent project"), &style, foreground)
        });

        let ElementKind::Text(props) = &element.kind else {
            panic!("expected menu item label text leaf");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        let mut expected = typography::composable_refinement_from_style(&style);
        expected.weight = Some(FontWeight::NORMAL);
        assert_eq!(element.inherited_text_style.as_ref(), Some(&expected));
        assert_eq!(element.inherited_foreground, Some(foreground));
    }

    #[test]
    fn menu_section_label_uses_sm_medium_foreground_refinement() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(40.0)));

        let foreground = fret_core::Color {
            r: 0.3,
            g: 0.3,
            b: 0.3,
            a: 1.0,
        };
        let mut style = typography::fixed_line_box_style(FontId::ui(), Px(14.0), Px(20.0));
        style.weight = FontWeight::NORMAL;

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            menu_section_label(cx, Arc::from("My Account"), &style, foreground)
        });

        let ElementKind::Text(props) = &element.kind else {
            panic!("expected menu section label text leaf");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        let mut expected = typography::composable_refinement_from_style(&style);
        expected.weight = Some(FontWeight::MEDIUM);
        assert_eq!(element.inherited_text_style.as_ref(), Some(&expected));
        assert_eq!(element.inherited_foreground, Some(foreground));
    }
}
