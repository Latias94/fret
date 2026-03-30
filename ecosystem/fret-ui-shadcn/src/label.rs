use std::sync::Arc;

use fret_core::{Axis, Edges, Px};
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, LayoutStyle, MainAlign};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::current_color;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_kit::primitives::label as primitive_label;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, IntoUiElement, LayoutRefinement, UiPatch, UiPatchTarget,
    UiSupportsChrome, UiSupportsLayout,
};

pub use fret_ui_kit::primitives::label::{SelectableLabel, label, selectable_label};

#[derive(Debug)]
pub struct Label {
    text: Arc<str>,
    children: Vec<AnyElement>,
    wrapped_children: Option<Vec<AnyElement>>,
    for_control: Option<ControlId>,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Label {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            children: Vec::new(),
            wrapped_children: None,
            for_control: None,
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Adds inline leading children before the label text.
    ///
    /// This is the ergonomic Fret mapping for shadcn's generic `children` slot when the label
    /// still owns the visible text content.
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self.wrapped_children = None;
        self
    }

    /// Replaces the default text content with a custom inline subtree.
    ///
    /// The original `text` is still kept as the label's accessible name and association label.
    pub fn wrap(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.wrapped_children = Some(children.into_iter().collect());
        self
    }

    pub fn for_control(mut self, id: impl Into<ControlId>) -> Self {
        self.for_control = Some(id.into());
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let text = self.text.clone();
        let root = label_root(
            cx,
            text.clone(),
            self.children,
            self.wrapped_children,
            self.chrome,
            self.layout,
        );

        let mut label = primitive_label::Label::new(text).wrap_root(root);
        if let Some(for_control) = self.for_control {
            label = label.for_control(for_control);
        }
        if let Some(test_id) = self.test_id {
            label = label.test_id(test_id);
        }
        label.into_element(cx)
    }
}

impl UiPatchTarget for Label {
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl UiSupportsChrome for Label {}
impl UiSupportsLayout for Label {}

impl<H: UiHost> IntoUiElement<H> for Label {
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        Label::into_element(self, cx)
    }
}

fn label_root<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    leading_children: Vec<AnyElement>,
    wrapped_children: Option<Vec<AnyElement>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let base_fg = theme
        .color_by_key("foreground")
        .unwrap_or_else(|| theme.color_token("foreground"));
    let resolved_fg_ref = chrome
        .text_color
        .clone()
        .unwrap_or(ColorRef::Color(base_fg));
    let props = decl_style::container_props(&theme, chrome, layout);

    let children = match wrapped_children {
        Some(children) if !children.is_empty() => children,
        _ => {
            let mut content = leading_children;
            content.push(primitive_label::label(cx, text));
            content
        }
    };

    let row = cx.flex(
        FlexProps {
            layout: LayoutStyle::default(),
            direction: Axis::Horizontal,
            gap: Px(8.0).into(),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |cx| current_color::scope_children(cx, resolved_fg_ref.clone(), |_cx| children),
    );

    cx.container(props, move |_cx| vec![row])
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Size};
    use fret_runtime::Model;
    use fret_ui::element::{ColumnProps, ContainerProps, ElementKind, Length, SizeStyle};
    use fret_ui::elements;

    #[test]
    fn label_children_surface_renders_inline_children_before_text() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let window = AppWindowId::default();
        let bounds = fret_core::Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );

        let root =
            elements::with_element_cx(&mut app, window, bounds, "label-children-inline", |cx| {
                let icon = cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Px(Px(12.0)),
                                height: Length::Px(Px(12.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                );

                Label::new("Email").children([icon]).into_element(cx)
            });

        let row = root.children.first().expect("expected inner flex row");
        let ElementKind::Flex(props) = &row.kind else {
            panic!(
                "expected label root to contain a Flex row, got {:?}",
                row.kind
            );
        };
        assert_eq!(props.direction, Axis::Horizontal);
        assert_eq!(props.align, CrossAlign::Center);
        assert_eq!(props.gap, Px(8.0).into());
        assert_eq!(row.children.len(), 2, "expected icon + text children");
        assert!(
            matches!(row.children[0].kind, ElementKind::Container(_)),
            "expected first child to keep the custom inline element"
        );
        assert!(
            matches!(row.children[1].kind, ElementKind::Text(_)),
            "expected second child to be the label text"
        );
    }

    #[test]
    fn label_children_for_control_keeps_control_registry_labelled_by_link() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let window = AppWindowId::default();
        let bounds = fret_core::Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let model: Model<String> = app.models_mut().insert(String::new());

        let root = elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "label-children-labelled-by",
            |cx| {
                let control_id = ControlId::from("email");

                cx.column(ColumnProps::default(), move |cx| {
                    let icon = cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Px(Px(12.0)),
                                    height: Length::Px(Px(12.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    );

                    vec![
                        Label::new("Email")
                            .children([icon])
                            .for_control(control_id.clone())
                            .into_element(cx),
                        crate::input::Input::new(model.clone())
                            .control_id(control_id)
                            .into_element(cx),
                    ]
                })
            },
        );

        let label_id = root.children[0].id;

        fn find_text_input(el: &AnyElement) -> Option<&AnyElement> {
            if matches!(el.kind, ElementKind::TextInput(_)) {
                return Some(el);
            }
            for child in &el.children {
                if let Some(found) = find_text_input(child) {
                    return Some(found);
                }
            }
            None
        }

        let input = find_text_input(&root).expect("expected a TextInput node");
        let decoration = input
            .semantics_decoration
            .as_ref()
            .expect("expected semantics decoration on TextInput");
        assert_eq!(decoration.labelled_by_element, Some(label_id.0));
    }
}
