//! AI Elements-aligned `Plan` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/plan.tsx`.

use std::sync::Arc;
use std::time::Duration;

use fret_core::{Px, SemanticsRole, TextWrap};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, ElementKind, InteractivityGateProps, LayoutStyle, OpacityProps,
    SemanticsDecoration,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::overlay_motion;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;
use fret_ui_kit::primitives::collapsible as radix_collapsible;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, Justify, LayoutRefinement, Space};
use fret_ui_shadcn::facade::{
    Button, ButtonSize, ButtonVariant, CardAction, CardContent, CardDescription, CardFooter,
    CardHeader, CardTitle,
};

const CARD_ACTION_MARKER_PREFIX: &str = "fret-ui-shadcn.card-action:";

#[derive(Debug, Clone)]
pub struct PlanController {
    pub open: Model<bool>,
    pub is_streaming: bool,
    pub disabled: bool,
    pub content_id: GlobalElementId,
}

pub fn use_plan_controller<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<PlanController> {
    cx.provided::<PlanController>().cloned()
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

impl Default for Plan {
    fn default() -> Self {
        Self::new()
    }
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

    /// Composable children surface aligned with upstream `Plan` usage:
    /// callers compose `PlanHeader`/`PlanTrigger`/`PlanContent`/`PlanFooter` as direct children.
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.into_element_with_children(cx, |cx, _controller| children(cx))
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

    pub fn into_element_with_children<H: UiHost>(
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

        cx.scope(move |cx| {
            let open_model =
                controllable_state::use_controllable_model(cx, controlled_open.clone(), || {
                    default_open
                })
                .model();
            let open = cx
                .get_model_copied(&open_model, Invalidation::Layout)
                .unwrap_or(default_open);

            // Stable target id for trigger `controls` relationships (Radix `aria-controls`).
            let content_id = cx.keyed("plan-content", |cx| cx.root_id());

            let controller = PlanController {
                open: open_model,
                is_streaming,
                disabled,
                content_id,
            };

            let mut root = cx.container(
                decl_style::container_props(&theme, chrome, layout),
                move |cx| {
                    cx.provide(controller.clone(), |cx| {
                        let body = ui::v_stack(move |cx| children(cx, controller.clone()))
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .gap(Space::N6)
                            .into_element(cx);

                        vec![body]
                    })
                },
            );

            if let Some(test_id) = test_id_root {
                root = root.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Group)
                        .expanded(open)
                        .test_id(test_id),
                );
            }

            root
        })
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
        if !has_action_marker && children.len() >= 2
            && let Some(action) = children.pop() {
                let action = CardAction::new([action]).into_element(cx);
                children.push(action);
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

        let open = cx
            .get_model_copied(&controller.open, Invalidation::Layout)
            .unwrap_or(false);

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

        let el = button.into_element(cx);
        radix_collapsible::apply_collapsible_trigger_controls_expanded(
            el,
            controller.content_id,
            open,
        )
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

        cx.keyed("plan-content-motion", move |cx| {
            let open = cx
                .get_model_copied(&controller.open, Invalidation::Layout)
                .unwrap_or(false);

            let toggle_duration = {
                let theme_full = Theme::global(&*cx.app);
                theme_full
                    .duration_ms_by_key("duration.shadcn.motion.collapsible.toggle")
                    .or_else(|| theme_full.duration_ms_by_key("duration.motion.collapsible.toggle"))
                    .or_else(|| theme_full.duration_ms_by_key("duration.shadcn.motion.200"))
            }
            .map(|ms| Duration::from_millis(ms as u64))
            .unwrap_or(Duration::from_millis(200));
            let toggle_ticks = ticks_60hz_for_duration(toggle_duration);

            let toggle_easing = {
                let theme_full = Theme::global(&*cx.app);
                theme_full
                    .easing_by_key("easing.shadcn.motion.collapsible.toggle")
                    .or_else(|| theme_full.easing_by_key("easing.motion.collapsible.toggle"))
            }
            .unwrap_or_else(|| overlay_motion::shadcn_motion_ease_bezier(cx));

            let motion = radix_collapsible::measured_height_motion_for_root_with_cubic_bezier(
                cx,
                open,
                false,
                true,
                toggle_ticks,
                toggle_ticks,
                toggle_easing,
            );
            if !motion.should_render {
                return hidden(cx);
            }

            let content = CardContent::new(self.children)
                .refine_style(self.chrome)
                .refine_layout(self.layout)
                .into_element(cx);

            let wrapper_layout = decl_style::layout_style(
                &Theme::global(&*cx.app).snapshot(),
                motion.wrapper_refinement.clone(),
            );
            let wrapper_child = cx.opacity_props(
                OpacityProps {
                    layout: LayoutStyle::default(),
                    opacity: motion.wrapper_opacity,
                },
                move |_cx| vec![content],
            );

            let wrapper_kind = if motion.wants_measurement {
                ElementKind::InteractivityGate(InteractivityGateProps {
                    layout: wrapper_layout,
                    present: true,
                    interactive: false,
                })
            } else {
                ElementKind::Container(ContainerProps {
                    layout: wrapper_layout,
                    ..Default::default()
                })
            };

            let mut wrapper_el =
                AnyElement::new(controller.content_id, wrapper_kind, vec![wrapper_child]);
            let _ = radix_collapsible::update_measured_for_motion(cx, motion, wrapper_el.id);

            if let Some(test_id) = self.test_id {
                wrapper_el = wrapper_el.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Group)
                        .test_id(test_id),
                );
            }

            wrapper_el
        })
    }
}

/// Footer wrapper aligned with AI Elements `PlanFooter`.
#[derive(Debug)]
pub struct PlanFooter {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    justify: Justify,
}

impl PlanFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            justify: Justify::Start,
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

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub fn justify_start(self) -> Self {
        self.justify(Justify::Start)
    }

    pub fn justify_center(self) -> Self {
        self.justify(Justify::Center)
    }

    pub fn justify_end(self) -> Self {
        self.justify(Justify::End)
    }

    pub fn justify_between(self) -> Self {
        self.justify(Justify::Between)
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let el = CardFooter::new(self.children)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .justify(self.justify)
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
    use fret_ui::elements::GlobalElementId;

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

    fn find_element_by_test_id<'a>(
        element: &'a AnyElement,
        test_id: &str,
    ) -> Option<&'a AnyElement> {
        let matches = element
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
            .is_some_and(|id| id == test_id)
            || matches!(
                &element.kind,
                ElementKind::Semantics(props) if props.test_id.as_deref().is_some_and(|id| id == test_id)
            )
            || matches!(
                &element.kind,
                ElementKind::Pressable(props) if props.a11y.test_id.as_deref().is_some_and(|id| id == test_id)
            );
        if matches {
            return Some(element);
        }

        element
            .children
            .iter()
            .find_map(|child| find_element_by_test_id(child, test_id))
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
            cx.provide(
                PlanController {
                    open: open.clone(),
                    is_streaming: true,
                    disabled: false,
                    content_id: GlobalElementId(1),
                },
                |cx| PlanTitle::new("Title").into_element(cx),
            )
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
            cx.provide(
                PlanController {
                    open: open.clone(),
                    is_streaming: true,
                    disabled: false,
                    content_id: GlobalElementId(1),
                },
                |cx| PlanDescription::new("Description").into_element(cx),
            )
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

    #[test]
    fn plan_trigger_stamps_controls_and_expanded_for_collapsible_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );

        let content_id = GlobalElementId(42);

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            cx.provide(
                PlanController {
                    open: open.clone(),
                    is_streaming: false,
                    disabled: false,
                    content_id,
                },
                |cx| PlanTrigger::default().test_id("trigger").into_element(cx),
            )
        });

        let trigger = find_element_by_test_id(&element, "trigger").expect("expected trigger node");
        let ElementKind::Pressable(props) = &trigger.kind else {
            panic!("expected trigger to resolve to a pressable element");
        };
        assert_eq!(props.a11y.expanded, Some(false));
        assert_eq!(props.a11y.controls_element, Some(content_id.0));
    }

    #[test]
    fn plan_content_uses_controller_content_id_as_root_element_id() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(true);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );

        let content_id = GlobalElementId(77);

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            cx.provide(
                PlanController {
                    open: open.clone(),
                    is_streaming: false,
                    disabled: false,
                    content_id,
                },
                |cx| {
                    PlanContent::new([cx.text("Hello")])
                        .test_id("content")
                        .into_element(cx)
                },
            )
        });

        assert_eq!(element.id, content_id);
        assert!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|d| d.test_id.as_deref())
                .is_some_and(|id| id == "content")
        );
    }
}
