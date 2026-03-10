use std::sync::Arc;

use fret_core::{AttributedText, SemanticsRole, TextAlign, TextOverflow, TextSpan, TextWrap};
use fret_ui::element::{
    AnyElement, Length, PointerRegionProps, SelectableTextProps, SemanticsProps, SizeStyle,
    TextInkOverflow, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};

use super::control_registry::{ControlAction, ControlId, LabelEntry, control_registry_model};
use crate::declarative::text::label_text_refinement;
use crate::typography;

#[derive(Debug, Clone)]
pub struct Label {
    text: Arc<str>,
    for_control: Option<ControlId>,
    test_id: Option<Arc<str>>,
}

impl Label {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            for_control: None,
            test_id: None,
        }
    }

    /// Binds this label to a logical form control id (similar to HTML `label[for]` / `htmlFor`).
    ///
    /// When set, pointer activation on the label forwards to the registered control action and
    /// requests focus for the control. This also enables `aria-labelledby`-like semantics when
    /// the control uses the same `ControlId`.
    pub fn for_control(mut self, id: impl Into<ControlId>) -> Self {
        self.for_control = Some(id.into());
        self
    }

    /// Sets a stable `test_id` on the label root.
    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(for_control) = self.for_control else {
            let mut el = label(cx, self.text);
            if let Some(test_id) = self.test_id {
                el = el.test_id(test_id);
            }
            return el;
        };

        label_for_control(cx, self.text, for_control, self.test_id)
    }
}

#[track_caller]
pub fn label<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let text = text.into();
    let (fg, refinement, line_height) = {
        let theme = Theme::global(&*cx.app);

        let fg = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"));
        let (refinement, line_height) = label_text_refinement(theme);

        (fg, refinement, line_height)
    };

    typography::scope_text_style_with_color(
        cx.text_props(TextProps {
            layout: fret_ui::element::LayoutStyle {
                size: SizeStyle {
                    height: Length::Px(line_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            text,
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
        fg,
    )
}

#[track_caller]
fn label_for_control<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    for_control: ControlId,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let control_registry = control_registry_model(cx);
    let control_snapshot = cx
        .app
        .models()
        .read(&control_registry, |reg| {
            reg.control_for(cx.window, &for_control).cloned()
        })
        .ok()
        .flatten();
    let enabled = control_snapshot.as_ref().map(|c| c.enabled).unwrap_or(true);
    let controls_element = control_snapshot.as_ref().map(|c| c.element.0);

    let props = SemanticsProps {
        role: SemanticsRole::Text,
        label: Some(text.clone()),
        test_id,
        controls_element,
        disabled: !enabled,
        ..Default::default()
    };

    let for_control_outer = for_control.clone();
    let control_registry_outer = control_registry.clone();
    cx.semantics(props, move |cx| {
        let label_element = cx.root_id();

        let _ = cx.app.models_mut().update(&control_registry_outer, |reg| {
            reg.register_label(
                cx.window,
                cx.frame_id,
                for_control_outer.clone(),
                LabelEntry {
                    element: label_element,
                },
            );
        });

        let for_control_inner = for_control_outer.clone();
        let control_registry_inner = control_registry_outer.clone();
        let control_snapshot_inner = control_snapshot.clone();

        vec![cx.pointer_region(PointerRegionProps::default(), move |cx| {
            let control_registry_on_down = control_registry_inner.clone();
            let for_control_on_down = for_control_inner.clone();
            let control_snapshot_on_down = control_snapshot_inner.clone();
            cx.pointer_region_add_on_pointer_down(Arc::new(move |host, acx, down| {
                // If the pointer-down hit-test chain includes a pressable (e.g. an embedded
                // button), let that descendant own the interaction rather than capturing.
                if down.hit_pressable_target.is_some() {
                    return false;
                }

                let target = host
                    .models_mut()
                    .read(&control_registry_on_down, |reg| {
                        reg.control_for(acx.window, &for_control_on_down).map(|c| {
                            (
                                c.enabled,
                                c.element,
                                matches!(c.action, ControlAction::FocusOnly),
                            )
                        })
                    })
                    .ok()
                    .flatten()
                    .or_else(|| {
                        control_snapshot_on_down.as_ref().map(|c| {
                            (
                                c.enabled,
                                c.element,
                                matches!(c.action, ControlAction::FocusOnly),
                            )
                        })
                    });
                if let Some((true, element, focus_on_pointer_down)) = target {
                    if focus_on_pointer_down {
                        host.request_focus(element);
                        return false;
                    }
                    host.capture_pointer();
                }
                true
            }));

            let control_registry_on_up = control_registry_inner.clone();
            let for_control_on_up = for_control_inner.clone();
            let control_snapshot_on_up = control_snapshot_inner.clone();
            cx.pointer_region_add_on_pointer_up(Arc::new(move |host, acx, up| {
                host.release_pointer_capture();
                if !up.is_click {
                    return true;
                }
                if up.down_hit_pressable_target.is_some() {
                    return false;
                }

                let control = host
                    .models_mut()
                    .read(&control_registry_on_up, |reg| {
                        reg.control_for(acx.window, &for_control_on_up).cloned()
                    })
                    .ok()
                    .flatten();
                let Some(control) = control.or_else(|| control_snapshot_on_up.clone()) else {
                    return true;
                };
                if !control.enabled {
                    return true;
                }
                if matches!(control.action, ControlAction::FocusOnly) {
                    return true;
                }

                host.request_focus(control.element);
                control.action.invoke(host, acx);
                host.request_redraw(acx.window);
                true
            }));

            let enabled = control_snapshot_inner
                .as_ref()
                .map(|c| c.enabled)
                .unwrap_or(true);
            let child = label(cx, text.clone());
            if enabled {
                vec![child]
            } else {
                vec![cx.opacity(0.7, move |_cx| vec![child])]
            }
        })]
    })
}

#[derive(Debug, Clone)]
pub struct SelectableLabel {
    text: Arc<str>,
}

impl SelectableLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        selectable_label(cx, self.text)
    }
}

/// A non-editable label that supports text selection (drag-to-select + `edit.copy`).
///
/// Recommended usage:
/// - Use this for "information" labels (IDs, paths, log snippets, read-only values).
/// - Avoid using it inside pressable/clickable rows because it intentionally captures left-drag
///   selection gestures and stops propagation (use a dedicated copy button instead).
#[track_caller]
pub fn selectable_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let text: Arc<str> = text.into();
    let (fg, refinement, line_height) = {
        let theme = Theme::global(&*cx.app);

        let fg = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"));
        let (refinement, line_height) = label_text_refinement(theme);

        (fg, refinement, line_height)
    };

    let spans: Arc<[TextSpan]> = Arc::from([TextSpan::new(text.len())]);
    let rich = AttributedText::new(Arc::clone(&text), spans);

    typography::scope_text_style_with_color(
        cx.selectable_text_props(SelectableTextProps {
            layout: fret_ui::element::LayoutStyle {
                size: SizeStyle {
                    height: Length::Px(line_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            rich,
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
            interactive_spans: Arc::from([]),
        }),
        refinement,
        fg,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::ElementKind;
    use fret_ui::elements;
    use fret_ui::{Theme, ThemeConfig};

    fn test_app() -> App {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Label Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("font.size".to_string(), 13.0),
                    ("font.line_height".to_string(), 20.0),
                    ("component.label.text_px".to_string(), 13.0),
                    ("component.label.line_height".to_string(), 18.0),
                ]),
                colors: std::collections::HashMap::from([(
                    "foreground".to_string(),
                    "#112233".to_string(),
                )]),
                ..ThemeConfig::default()
            });
        });
        app
    }

    fn test_bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        )
    }

    #[test]
    fn label_defaults_match_shadcn_expectations() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            label(cx, "Email")
        });
        let theme = Theme::global(&app);
        let (expected_refinement, line_height) = label_text_refinement(&theme);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected label(...) to build a Text element");
        };

        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(props.layout.size.height, Length::Px(line_height));
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(
            el.inherited_foreground,
            Some(theme.color_token("foreground"))
        );
        assert_eq!(el.inherited_text_style, Some(expected_refinement));
        assert_eq!(
            el.inherited_text_style
                .as_ref()
                .and_then(|style| style.font.clone()),
            Some(fret_core::FontId::ui())
        );
        assert_eq!(
            el.inherited_text_style
                .as_ref()
                .and_then(|style| style.weight),
            Some(fret_core::FontWeight::MEDIUM)
        );
    }

    #[test]
    fn selectable_label_scopes_inherited_refinement_without_leaf_style() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            selectable_label(cx, "Order #42")
        });
        let theme = Theme::global(&app);

        let ElementKind::SelectableText(props) = &el.kind else {
            panic!("expected selectable_label(...) to build a SelectableText element");
        };

        assert_eq!(props.layout.size.height, Length::Px(Px(18.0)));
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(
            el.inherited_foreground,
            Some(theme.color_token("foreground"))
        );
        assert_eq!(
            el.inherited_text_style,
            Some(label_text_refinement(&theme).0)
        );
    }

    #[test]
    fn label_for_control_registers_in_control_registry() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let control_id = ControlId::from("email");
        let mut reg_model: Option<
            fret_runtime::Model<crate::primitives::control_registry::ControlRegistry>,
        > = None;

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            reg_model = Some(control_registry_model(cx));
            Label::new("Email")
                .for_control(control_id.clone())
                .into_element(cx)
        });

        let ElementKind::Semantics(_props) = &el.kind else {
            panic!("expected Label::for_control(...) to build a Semantics root");
        };

        let reg_model = reg_model.expect("control registry model");
        let entry = app
            .models()
            .read(&reg_model, |reg| {
                reg.label_for(window, &control_id).cloned()
            })
            .ok()
            .flatten()
            .expect("expected label to register itself in the control registry");

        assert_eq!(entry.element, el.id);
    }
}
