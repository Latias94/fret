//! AI Elements-aligned `ChainOfThought` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/chain-of-thought.tsx`.

use std::sync::Arc;

use fret_core::{
    Color, FontWeight, Point, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap, Transform2D,
};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, InteractivityGateProps, LayoutStyle, Length,
    PressableA11y, PressableProps, SemanticsDecoration, TextProps, VisualTransformProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, MetricRef, Radius, Space,
    typography,
};
use fret_ui_shadcn::{Badge, BadgeVariant, Collapsible};

pub type OnChainOfThoughtOpenChange = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, bool) + 'static>;

fn alpha(color: Color, a: f32) -> Color {
    Color {
        a: a.clamp(0.0, 1.0),
        ..color
    }
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

fn text_sm(theme: &Theme, weight: FontWeight) -> TextStyle {
    let mut style =
        typography::TypographyPreset::control_ui(typography::UiTextSize::Sm).resolve(theme);
    style.weight = weight;
    style
}

fn text_xs(theme: &Theme, weight: FontWeight) -> TextStyle {
    let mut style =
        typography::TypographyPreset::control_ui(typography::UiTextSize::Xs).resolve(theme);
    style.weight = weight;
    style
}

#[derive(Debug, Default, Clone)]
struct ChainOfThoughtProviderState {
    controller: Option<ChainOfThoughtController>,
}

#[derive(Clone)]
pub struct ChainOfThoughtController {
    pub open: Model<bool>,
    pub is_open: bool,
    pub on_open_change: Option<OnChainOfThoughtOpenChange>,
}

impl std::fmt::Debug for ChainOfThoughtController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChainOfThoughtController")
            .field("open", &"<model>")
            .field("is_open", &self.is_open)
            .field("has_on_open_change", &self.on_open_change.is_some())
            .finish()
    }
}

pub fn use_chain_of_thought_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<ChainOfThoughtController> {
    cx.inherited_state::<ChainOfThoughtProviderState>()
        .and_then(|st| st.controller.clone())
}

#[derive(Clone)]
/// Collapsible container aligned with AI Elements `ChainOfThought`.
pub struct ChainOfThought {
    open: Option<Model<bool>>,
    default_open: bool,
    on_open_change: Option<OnChainOfThoughtOpenChange>,
    test_id_root: Option<Arc<str>>,
    layout: LayoutRefinement,
    gap: Space,
}

impl std::fmt::Debug for ChainOfThought {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChainOfThought")
            .field("open", &"<model>")
            .field("default_open", &self.default_open)
            .field("has_on_open_change", &self.on_open_change.is_some())
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("layout", &self.layout)
            .field("gap", &self.gap)
            .finish()
    }
}

impl ChainOfThought {
    pub fn new() -> Self {
        Self {
            open: None,
            default_open: false,
            on_open_change: None,
            test_id_root: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            gap: Space::N4,
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

    pub fn on_open_change(mut self, on_open_change: OnChainOfThoughtOpenChange) -> Self {
        self.on_open_change = Some(on_open_change);
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element_with_children<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement> + 'static,
    ) -> AnyElement {
        let controlled_open = self.open;
        let default_open = self.default_open;
        let on_open_change = self.on_open_change;
        let test_id_root = self.test_id_root;
        let layout = self.layout;
        let gap = self.gap;

        cx.scope(move |cx| {
            let open =
                controllable_state::use_controllable_model(cx, controlled_open, || default_open)
                    .model();
            let is_open = cx
                .get_model_copied(&open, Invalidation::Layout)
                .unwrap_or(false);

            let controller = ChainOfThoughtController {
                open,
                is_open,
                on_open_change,
            };

            cx.with_state(ChainOfThoughtProviderState::default, |st| {
                st.controller = Some(controller.clone());
            });

            let theme = Theme::global(&*cx.app).clone();
            let body = ui::v_stack(move |cx| children(cx))
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(gap)
                .into_element(cx);

            let mut root = cx.container(
                ContainerProps {
                    layout: decl_style::layout_style(&theme, layout),
                    ..Default::default()
                },
                move |_cx| vec![body],
            );

            if let Some(test_id) = test_id_root {
                root = root.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Group)
                        .test_id(test_id),
                );
            }
            root
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainOfThoughtStepStatus {
    Complete,
    Active,
    Pending,
}

impl Default for ChainOfThoughtStepStatus {
    fn default() -> Self {
        Self::Complete
    }
}

pub struct ChainOfThoughtHeader {
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ChainOfThoughtHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChainOfThoughtHeader")
            .field("children_len", &self.children.as_ref().map(|v| v.len()))
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl ChainOfThoughtHeader {
    pub fn new() -> Self {
        Self {
            children: None,
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_chain_of_thought_controller(cx) else {
            debug_assert!(
                false,
                "ChainOfThoughtHeader must be rendered within a ChainOfThought scope"
            );
            return cx.container(Default::default(), |_| Vec::new());
        };

        let theme = Theme::global(&*cx.app).clone();
        let muted_fg = theme.color_token("muted-foreground");
        let fg_hover = theme.color_token("foreground");

        let open = controller.open;
        let is_open = controller.is_open;
        let on_open_change = controller.on_open_change;

        let children = self.children;
        let test_id = self.test_id;
        let layout = self.layout;

        cx.pressable(
            PressableProps {
                layout: decl_style::layout_style(&theme, layout),
                enabled: true,
                focusable: true,
                a11y: PressableA11y {
                    role: Some(SemanticsRole::Button),
                    label: Some(Arc::<str>::from("Toggle chain of thought")),
                    test_id,
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, st| {
                let fg = if st.hovered { fg_hover } else { muted_fg };

                cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
                    let current = host.models_mut().get_cloned(&open).unwrap_or(false);
                    let next = !current;
                    let _ = host.models_mut().update(&open, |v| *v = next);
                    if let Some(cb) = on_open_change.clone() {
                        cb(host, action_cx, next);
                    }
                    host.request_redraw(action_cx.window);
                }));

                let icon_size = Px(16.0);
                let brain = decl_icon::icon_with(
                    cx,
                    fret_icons::IconId::new_static("lucide.brain"),
                    Some(icon_size),
                    Some(ColorRef::Color(fg)),
                );

                let label = if let Some(children) = children {
                    cx.stack_props(
                        fret_ui::element::StackProps {
                            layout: decl_style::layout_style(
                                &theme,
                                LayoutRefinement::default().min_w_0().flex_1(),
                            ),
                        },
                        move |_cx| children,
                    )
                } else {
                    cx.text_props(TextProps {
                        layout: decl_style::layout_style(
                            &theme,
                            LayoutRefinement::default().min_w_0().flex_1(),
                        ),
                        text: Arc::from("Chain of Thought"),
                        style: Some(text_sm(&theme, FontWeight::NORMAL)),
                        color: Some(fg),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        align: fret_core::TextAlign::Start,
                        ink_overflow: Default::default(),
                    })
                };

                let chevron_rotation = if is_open { 180.0 } else { 0.0 };
                let center = Point::new(Px(8.0), Px(8.0));
                let chevron_transform =
                    Transform2D::rotation_about_degrees(chevron_rotation, center);
                let chevron = cx.visual_transform_props(
                    VisualTransformProps {
                        layout: decl_style::layout_style(
                            &theme,
                            LayoutRefinement::default()
                                .w_px(MetricRef::Px(icon_size))
                                .h_px(MetricRef::Px(icon_size))
                                .flex_shrink_0(),
                        ),
                        transform: chevron_transform,
                    },
                    move |cx| {
                        vec![decl_icon::icon_with(
                            cx,
                            fret_icons::ids::ui::CHEVRON_DOWN,
                            Some(icon_size),
                            Some(ColorRef::Color(fg)),
                        )]
                    },
                );

                let row = ui::h_row(move |_cx| vec![brain, label, chevron])
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .items(Items::Center)
                    .gap(Space::N2)
                    .into_element(cx);

                vec![row]
            },
        )
    }
}

pub struct ChainOfThoughtContent {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ChainOfThoughtContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChainOfThoughtContent")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl ChainOfThoughtContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().mt(Space::N2).w_full().min_w_0(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_chain_of_thought_controller(cx) else {
            debug_assert!(
                false,
                "ChainOfThoughtContent must be rendered within a ChainOfThought scope"
            );
            return cx.container(Default::default(), |_| Vec::new());
        };

        let open = controller.open;
        let children = self.children;
        let layout = self.layout;
        let test_id = self.test_id;

        let inner = Collapsible::new(open).into_element(
            cx,
            move |cx, _is_open| hidden(cx),
            move |cx| {
                let theme = Theme::global(&*cx.app).clone();
                let body = ui::v_stack(move |_cx| children)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N3)
                    .into_element(cx);

                let mut content = cx.container(
                    ContainerProps {
                        layout: decl_style::layout_style(&theme, layout),
                        ..Default::default()
                    },
                    move |_cx| vec![body],
                );

                if let Some(test_id) = test_id.clone() {
                    content = content.attach_semantics(
                        SemanticsDecoration::default()
                            .role(SemanticsRole::Group)
                            .test_id(test_id),
                    );
                }

                content
            },
        );

        inner
    }
}

pub struct ChainOfThoughtStep {
    label: Arc<str>,
    description: Option<Arc<str>>,
    status: ChainOfThoughtStepStatus,
    children: Vec<AnyElement>,
    icon: fret_icons::IconId,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ChainOfThoughtStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChainOfThoughtStep")
            .field("label", &self.label.as_ref())
            .field(
                "description",
                &self.description.as_ref().map(|s| s.as_ref()),
            )
            .field("status", &self.status)
            .field("children_len", &self.children.len())
            .field("icon", &self.icon)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl ChainOfThoughtStep {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            description: None,
            status: ChainOfThoughtStepStatus::Complete,
            children: Vec::new(),
            icon: fret_icons::IconId::new_static("lucide.dot"),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn status(mut self, status: ChainOfThoughtStepStatus) -> Self {
        self.status = status;
        self
    }

    pub fn icon(mut self, icon: fret_icons::IconId) -> Self {
        self.icon = icon;
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
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
        let theme = Theme::global(&*cx.app).clone();

        let base_fg = theme.color_token("muted-foreground");
        let fg = match self.status {
            ChainOfThoughtStepStatus::Active => theme.color_token("foreground"),
            ChainOfThoughtStepStatus::Complete => base_fg,
            ChainOfThoughtStepStatus::Pending => alpha(base_fg, 0.5),
        };

        let icon_size = Px(16.0);
        let icon = decl_icon::icon_with(cx, self.icon, Some(icon_size), Some(ColorRef::Color(fg)));

        let line = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    position: fret_ui::element::PositionStyle::Absolute,
                    inset: fret_ui::element::InsetStyle {
                        top: Some(Px(28.0)).into(),
                        bottom: Some(Px(0.0)).into(),
                        left: Some(Px(7.5)).into(),
                        ..Default::default()
                    },
                    size: fret_ui::element::SizeStyle {
                        width: Length::Px(Px(1.0)),
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                background: Some(theme.color_token("border")),
                ..Default::default()
            },
            |_cx| Vec::new(),
        );

        let icon_col = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .mt(Space::N0p5)
                        .min_w_0()
                        .flex_shrink_0(),
                ),
                ..Default::default()
            },
            move |_cx| vec![icon, line],
        );

        let label = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.label,
            style: Some(text_sm(&theme, FontWeight::NORMAL)),
            color: Some(fg),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        });

        let mut body_children: Vec<AnyElement> = Vec::new();
        body_children.push(label);

        if let Some(description) = self.description {
            body_children.push(cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: description,
                style: Some(text_xs(&theme, FontWeight::NORMAL)),
                color: Some(base_fg),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            }));
        }
        body_children.extend(self.children);

        let body = ui::v_stack(move |_cx| body_children)
            .layout(
                LayoutRefinement::default()
                    .w_full()
                    .min_w_0()
                    .overflow_hidden(),
            )
            .gap(Space::N2)
            .into_element(cx);

        let mut row = ui::h_row(move |_cx| vec![icon_col, body])
            .layout(self.layout)
            .gap(Space::N2)
            .items(Items::Start)
            .into_element(cx);

        if let Some(test_id) = self.test_id {
            row = row.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            );
        }
        row
    }
}

pub struct ChainOfThoughtSearchResults {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    gap: Space,
}

impl std::fmt::Debug for ChainOfThoughtSearchResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChainOfThoughtSearchResults")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("gap", &self.gap)
            .finish()
    }
}

impl ChainOfThoughtSearchResults {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            gap: Space::N2,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap_px = decl_style::space(&theme, self.gap);

        let mut row = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(&theme, self.layout),
                direction: fret_core::Axis::Horizontal,
                gap: gap_px.into(),
                justify: fret_ui::element::MainAlign::Start,
                align: fret_ui::element::CrossAlign::Center,
                wrap: true,
                ..Default::default()
            },
            move |_cx| self.children,
        );

        if let Some(test_id) = self.test_id {
            row = row.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        row
    }
}

pub struct ChainOfThoughtSearchResult {
    label: Arc<str>,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for ChainOfThoughtSearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChainOfThoughtSearchResult")
            .field("label", &self.label.as_ref())
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl ChainOfThoughtSearchResult {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
            test_id: None,
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut el = Badge::new(self.label)
            .variant(BadgeVariant::Secondary)
            .label_weight(FontWeight::NORMAL)
            .children(self.children)
            .into_element(cx);

        if let Some(test_id) = self.test_id {
            el = el.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        el
    }
}

pub struct ChainOfThoughtImage {
    children: Vec<AnyElement>,
    caption: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ChainOfThoughtImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChainOfThoughtImage")
            .field("children_len", &self.children.len())
            .field("caption", &self.caption.as_ref().map(|s| s.as_ref()))
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl ChainOfThoughtImage {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            caption: None,
            test_id: None,
            layout: LayoutRefinement::default().mt(Space::N2).w_full().min_w_0(),
        }
    }

    pub fn caption(mut self, caption: impl Into<Arc<str>>) -> Self {
        self.caption = Some(caption.into());
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
        let theme = Theme::global(&*cx.app).clone();
        let muted = theme
            .color_by_key("muted")
            .unwrap_or_else(|| theme.color_token("muted.background"));
        let caption_fg = theme.color_token("muted-foreground");

        let max_h = Px(352.0); // 22rem

        let image_container = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .bg(ColorRef::Color(muted))
                    .p(Space::N3),
                LayoutRefinement::default()
                    .w_full()
                    .min_w_0()
                    .max_h(max_h)
                    .overflow_hidden(),
            ),
            move |cx| {
                vec![
                    ui::h_row(move |_cx| self.children)
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .justify(Justify::Center)
                        .items(Items::Center)
                        .into_element(cx),
                ]
            },
        );

        let mut out: Vec<AnyElement> = vec![image_container];
        if let Some(caption) = self.caption {
            out.push(cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: caption,
                style: Some(text_xs(&theme, FontWeight::NORMAL)),
                color: Some(caption_fg),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            }));
        }

        let mut wrapper = ui::v_stack(move |_cx| out)
            .layout(self.layout)
            .gap(Space::N2)
            .into_element(cx);

        if let Some(test_id) = self.test_id {
            wrapper = wrapper.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        wrapper
    }
}

pub fn set_chain_of_thought_open(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    open: &Model<bool>,
    value: bool,
) {
    let _ = host.models_mut().update(open, |v| *v = value);
    host.request_redraw(action_cx.window);
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::ElementKind;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(600.0), Px(400.0)),
        )
    }

    #[test]
    fn chain_of_thought_search_results_wraps() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let el =
                ChainOfThoughtSearchResults::new([cx.text("a"), cx.text("b")]).into_element(cx);
            let ElementKind::Flex(props) = &el.kind else {
                panic!(
                    "expected ChainOfThoughtSearchResults root to be Flex, got {:?}",
                    el.kind
                );
            };
            assert!(
                props.wrap,
                "expected ChainOfThoughtSearchResults to enable flex wrap"
            );
        });
    }
}
