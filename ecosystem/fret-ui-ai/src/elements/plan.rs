//! AI Elements-aligned `Plan` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/plan.tsx`.

use std::sync::Arc;

use fret_core::{Px, SemanticsRole, TextWrap};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, InteractivityGateProps, LayoutStyle, SemanticsDecoration};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Space};
use fret_ui_shadcn::{
    Button, ButtonSize, ButtonVariant, CardAction, CardContent, CardDescription, CardFooter,
    CardHeader, CardTitle,
};

const CARD_ACTION_MARKER_PREFIX: &str = "fret-ui-shadcn.card-action:";

#[derive(Debug, Default, Clone)]
struct PlanProviderState {
    controller: Option<PlanController>,
}

#[derive(Debug, Clone)]
pub struct PlanController {
    pub open: Model<bool>,
    pub is_streaming: bool,
    pub disabled: bool,
}

pub fn use_plan_controller<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<PlanController> {
    cx.inherited_state::<PlanProviderState>()
        .and_then(|st| st.controller.clone())
}

fn hidden<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.interactivity_gate_props(
        InteractivityGateProps {
            layout: LayoutStyle::default(),
            present: false,
            interactive: false,
        },
        |_cx| Vec::new(),
    )
}

fn plan_base_chrome(theme: &Theme) -> ChromeRefinement {
    let bg = theme.color_token("card");
    let border = theme.color_token("border");

    // shadcn/ui v4 Card uses `rounded-xl`, which is derived from the base `--radius`.
    let base_radius = theme.metric_token("metric.radius.lg");
    let rounded_xl = Px(base_radius.0 + 4.0);

    ChromeRefinement::default()
        .radius(rounded_xl)
        .border_1()
        .bg(ColorRef::Color(bg))
        .border_color(ColorRef::Color(border))
        .py(Space::N6)
}

/// Collapsible plan container aligned with AI Elements `Plan`.
#[derive(Debug, Clone)]
pub struct Plan {
    open: Option<Model<bool>>,
    default_open: bool,
    disabled: bool,
    is_streaming: bool,
    test_id_root: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl Plan {
    pub fn new() -> Self {
        Self {
            open: None,
            default_open: false,
            disabled: false,
            is_streaming: false,
            test_id_root: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    /// Controlled open model (Radix `open`).
    pub fn open_model(mut self, open: Model<bool>) -> Self {
        self.open = Some(open);
        self
    }

    /// Uncontrolled initial open value (Radix `defaultOpen`).
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn is_streaming(mut self, is_streaming: bool) -> Self {
        self.is_streaming = is_streaming;
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element_with_children<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>, PlanController) -> Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let chrome = plan_base_chrome(&theme).merge(self.chrome);
        let layout = self.layout;
        let default_open = self.default_open;
        let controlled_open = self.open.clone();
        let is_streaming = self.is_streaming;
        let disabled = self.disabled;
        let test_id_root = self.test_id_root.clone();

        let root = cx.container(
            decl_style::container_props(&theme, chrome, layout),
            move |cx| {
                let open_model =
                    controllable_state::use_controllable_model(cx, controlled_open.clone(), || {
                        default_open
                    })
                    .model();

                let controller = PlanController {
                    open: open_model,
                    is_streaming,
                    disabled,
                };

                cx.with_state(PlanProviderState::default, |st| {
                    st.controller = Some(controller.clone());
                });

                let body = ui::v_stack(move |cx| children(cx, controller))
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N6)
                    .into_element(cx);

                vec![body]
            },
        );

        let Some(test_id) = test_id_root else {
            return root;
        };
        root.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Header wrapper aligned with AI Elements `PlanHeader`.
#[derive(Debug)]
pub struct PlanHeader {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl PlanHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut children = self.children;
        let has_action_marker = children.iter().any(|child| {
            child
                .semantics_decoration
                .as_ref()
                .and_then(|d| d.test_id.as_deref())
                .is_some_and(|id| id.starts_with(CARD_ACTION_MARKER_PREFIX))
        });
        if !has_action_marker && children.len() >= 2 {
            if let Some(action) = children.pop() {
                let action = CardAction::new([action]).into_element(cx);
                children.push(action);
            }
        }

        let el = CardHeader::new(children)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .into_element(cx);
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Title text aligned with AI Elements `PlanTitle`.
#[derive(Debug, Clone)]
pub struct PlanTitle {
    text: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl PlanTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let is_streaming = use_plan_controller(cx)
            .map(|c| c.is_streaming)
            .unwrap_or(false);

        let el = if is_streaming {
            let theme = Theme::global(&*cx.app).snapshot();
            fret_ui_kit::typography::scope_text_style_with_color(
                super::Shimmer::new(self.text.clone())
                    .use_resolved_passive_text()
                    .role(SemanticsRole::Text)
                    .wrap(TextWrap::Word)
                    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
                fret_ui_kit::typography::title_text_refinement(&theme, "component.card.title"),
                theme.color_token("card-foreground"),
            )
        } else {
            CardTitle::new(self.text.clone()).into_element(cx)
        };

        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Text)
                .test_id(test_id),
        )
    }
}

/// Description text aligned with AI Elements `PlanDescription`.
#[derive(Debug, Clone)]
pub struct PlanDescription {
    text: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl PlanDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let is_streaming = use_plan_controller(cx)
            .map(|c| c.is_streaming)
            .unwrap_or(false);

        let el = if is_streaming {
            let theme = Theme::global(&*cx.app).snapshot();
            fret_ui_kit::typography::scope_description_text(
                super::Shimmer::new(self.text.clone())
                    .use_resolved_passive_text()
                    .role(SemanticsRole::Text)
                    .wrap(TextWrap::Word)
                    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
                &theme,
                "component.card.description",
            )
        } else {
            CardDescription::new(self.text.clone()).into_element(cx)
        };

        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Text)
                .test_id(test_id),
        )
    }
}

/// Action slot aligned with AI Elements `PlanAction`.
#[derive(Debug)]
pub struct PlanAction {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl PlanAction {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let el = CardAction::new(self.children)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .into_element(cx);
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Collapsible trigger aligned with AI Elements `PlanTrigger`.
#[derive(Debug, Clone)]
pub struct PlanTrigger {
    a11y_label: Arc<str>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl Default for PlanTrigger {
    fn default() -> Self {
        Self {
            a11y_label: Arc::<str>::from("Toggle plan"),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }
}

impl PlanTrigger {
    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = label.into();
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_plan_controller(cx) else {
            return hidden(cx);
        };

        let theme = Theme::global(&*cx.app).clone();
        let icon_size = Px(16.0);

        let icon = decl_icon::icon_with(
            cx,
            ids::ui::CHEVRONS_UP_DOWN,
            Some(icon_size),
            Some(ColorRef::Color(theme.color_token("muted-foreground"))),
        );

        let mut button = Button::new(self.a11y_label)
            .children([icon])
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .refine_layout(self.layout)
            .disabled(controller.disabled)
            .toggle_model(controller.open);
        if let Some(test_id) = self.test_id {
            button = button.test_id(test_id);
        }
        button.into_element(cx)
    }
}

/// Collapsible content wrapper aligned with AI Elements `PlanContent`.
#[derive(Debug)]
pub struct PlanContent {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl PlanContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_plan_controller(cx) else {
            return hidden(cx);
        };

        let open = cx
            .get_model_copied(&controller.open, Invalidation::Layout)
            .unwrap_or(false);
        if !open {
            return hidden(cx);
        }

        let el = CardContent::new(self.children)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .into_element(cx);
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Footer wrapper aligned with AI Elements `PlanFooter`.
#[derive(Debug)]
pub struct PlanFooter {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl PlanFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let el = CardFooter::new(self.children)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .into_element(cx);
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::{AnyElement, ElementKind};

    fn find_text_by_content<'a>(element: &'a AnyElement, content: &str) -> Option<&'a AnyElement> {
        if let ElementKind::Text(props) = &element.kind
            && props.text.as_ref() == content
        {
            return Some(element);
        }

        element
            .children
            .iter()
            .find_map(|child| find_text_by_content(child, content))
    }

    #[test]
    fn plan_title_streaming_scopes_inherited_title_typography_for_shimmer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            cx.with_state(PlanProviderState::default, |st| {
                st.controller = Some(PlanController {
                    open: open.clone(),
                    is_streaming: true,
                    disabled: false,
                });
            });
            PlanTitle::new("Title").into_element(cx)
        });

        let theme = fret_ui::Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::title_text_refinement(
                &theme,
                "component.card.title",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(theme.color_token("card-foreground"))
        );

        let text = find_text_by_content(&element, "Title")
            .expect("expected shimmer base text child under the scoped root");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected shimmer title branch to render a text leaf");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.wrap, TextWrap::Word);
    }

    #[test]
    fn plan_description_streaming_scopes_inherited_description_typography_for_shimmer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            cx.with_state(PlanProviderState::default, |st| {
                st.controller = Some(PlanController {
                    open: open.clone(),
                    is_streaming: true,
                    disabled: false,
                });
            });
            PlanDescription::new("Description").into_element(cx)
        });

        let theme = fret_ui::Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::description_text_refinement(
                &theme,
                "component.card.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );

        let text = find_text_by_content(&element, "Description")
            .expect("expected shimmer base text child under the scoped root");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected shimmer description branch to render a text leaf");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.wrap, TextWrap::Word);
    }
}
