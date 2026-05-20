use std::sync::Arc;

use fret_core::{FontWeight, Px};
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign};
use fret_ui::{ElementContext, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};
use fret_ui_kit::{MetricRef, ui};

/// A small, keyboard-first hint row: a keycap (`Kbd`) followed by a text label.
///
/// This intentionally keeps both parts on the same sizing baseline (height, padding, typography)
/// to avoid mixed-script (e.g. Latin + CJK) alignment drift in compact footers/toolbars.
#[derive(Debug, Clone)]
pub struct ShortcutHint {
    keys: Arc<str>,
    label: Arc<str>,
    layout: LayoutRefinement,
}

impl ShortcutHint {
    pub fn new(keys: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            keys: keys.into(),
            label: label.into(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        shortcut_hint_with_patch(cx, self.keys, self.label, self.layout)
    }
}

fn shortcut_hint_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    keys: Arc<str>,
    label: Arc<str>,
    layout_override: LayoutRefinement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    let kbd = crate::kbd::Kbd::new(keys).into_element(cx);
    let label = shortcut_hint_label(cx, &theme, label);

    let base_h = Px(20.0);
    let layout_override = LayoutRefinement::default()
        .h_px(base_h)
        .min_h(base_h)
        .merge(layout_override);
    let mut layout = decl_style::layout_style(&theme, layout_override);
    // Default to `flex-none` so hint blocks wrap instead of squishing unpredictably.
    layout.flex.grow = 0.0;
    layout.flex.shrink = 0.0;

    cx.flex(
        FlexProps {
            layout,
            direction: fret_core::Axis::Horizontal,
            gap: MetricRef::space(Space::N1).resolve(&theme).into(),
            padding: fret_core::Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |_cx| vec![kbd, label],
    )
}

fn shortcut_hint_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &ThemeSnapshot,
    label: Arc<str>,
) -> AnyElement {
    let fg = theme.color_token("muted-foreground");

    let px = theme
        .metric_by_key("component.kbd.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.kbd.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    let mut label_style =
        typography::fixed_line_box_style(fret_core::FontId::ui(), px, line_height);
    label_style.weight = FontWeight::MEDIUM;
    let mut label_refinement = typography::composable_refinement_from_style(&label_style);
    label_refinement.weight = Some(FontWeight::MEDIUM);

    let chrome = ChromeRefinement::default().px(Space::N1).py(Space::N0p5);
    let layout = LayoutRefinement::default().h_px(Px(20.0)).min_h(Px(20.0));
    let props = decl_style::container_props(theme, chrome, layout);

    cx.container(props, |cx| {
        vec![
            ui::h_flex(|cx| {
                vec![
                    decl_text::text_keycap_label(cx, label)
                        .inherit_text_style(label_refinement.clone())
                        .inherit_foreground(fg),
                ]
            })
            .w_full()
            .h_full()
            .justify_center()
            .items_center()
            .into_element(cx),
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size, TextOverflow, TextWrap};
    use fret_ui::element::{ElementKind, Length};

    fn find_text<'a>(node: &'a AnyElement, needle: &str) -> Option<&'a AnyElement> {
        if let ElementKind::Text(props) = &node.kind
            && props.text.as_ref() == needle
        {
            return Some(node);
        }

        node.children
            .iter()
            .find_map(|child| find_text(child, needle))
    }

    #[test]
    fn shortcut_hint_label_uses_shared_keycap_role() {
        let window = AppWindowId::default();
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            ShortcutHint::new("Ctrl", "Open command palette").into_element(cx)
        });

        let label = find_text(&element, "Open command palette").expect("expected hint label text");
        let ElementKind::Text(props) = &label.kind else {
            panic!("expected hint label text leaf");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);

        let mut expected_style =
            typography::fixed_line_box_style(fret_core::FontId::ui(), Px(12.0), Px(16.0));
        expected_style.weight = FontWeight::MEDIUM;
        let mut expected = typography::composable_refinement_from_style(&expected_style);
        expected.weight = Some(FontWeight::MEDIUM);
        assert_eq!(label.inherited_text_style.as_ref(), Some(&expected));
        assert_eq!(
            label.inherited_foreground,
            Some(Theme::global(&app).color_token("muted-foreground"))
        );
    }
}
