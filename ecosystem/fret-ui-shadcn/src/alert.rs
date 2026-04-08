use std::marker::PhantomData;
use std::sync::Arc;

use fret_core::{Color, FontWeight, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, ElementKind, GridProps, GridTrackSizing, InsetEdge,
    LayoutStyle, Length, MainAlign, MarginEdge, PositionStyle, SemanticsDecoration, SpacingEdges,
    SpacingLength,
};
use fret_ui::{ElementContext, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::style as decl_style;

use crate::direction::LayoutDirection;

use fret_ui_kit::typography::scope_description_text;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, IntoUiElement, LayoutRefinement, MetricRef, PaddingRefinement,
    Radius, Space, UiPatch, UiPatchTarget, UiSupportsChrome, UiSupportsLayout, ui,
};

const ALERT_ACTION_MARKER_TEST_ID: &str = "__fret_shadcn.alert_action";

fn alert_padding_x(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.alert.padding_x")
        .unwrap_or(Px(16.0))
}

fn alert_padding_y(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.alert.padding_y")
        .unwrap_or(Px(12.0))
}

fn alert_gap_x(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.alert.gap_x")
        .unwrap_or(Px(12.0))
}

fn alert_gap_y(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.alert.gap_y")
        .unwrap_or(Px(2.0))
}

fn alert_icon_size(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.alert.icon_size")
        .unwrap_or(Px(16.0))
}

fn alert_icon_offset_y(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.alert.icon_offset_y")
        .unwrap_or(Px(2.0))
}

fn alert_action_padding_right(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.alert.action_padding_right")
        .unwrap_or(Px(72.0))
}

fn alert_action_top(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.alert.action_top")
        .unwrap_or(Px(8.0))
}

fn alert_action_right(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.alert.action_right")
        .unwrap_or(Px(8.0))
}

fn px_spacing_or_zero(length: SpacingLength) -> Px {
    match length {
        SpacingLength::Px(px) => px,
        SpacingLength::Fraction(_) | SpacingLength::Fill => Px(0.0),
    }
}

fn translate_alert_action_to_padding_box(
    action: &mut AnyElement,
    top_padding: SpacingLength,
    left_padding: SpacingLength,
    right_padding: SpacingLength,
) {
    let ElementKind::Container(props) = &mut action.kind else {
        return;
    };
    if props.layout.position != PositionStyle::Absolute {
        return;
    }

    let top_padding = px_spacing_or_zero(top_padding);
    let left_padding = px_spacing_or_zero(left_padding);
    let right_padding = px_spacing_or_zero(right_padding);

    if let InsetEdge::Px(top) = props.layout.inset.top {
        props.layout.inset.top = InsetEdge::Px(Px(top.0 - top_padding.0));
    }
    if let InsetEdge::Px(left) = props.layout.inset.left {
        props.layout.inset.left = InsetEdge::Px(Px(left.0 - left_padding.0));
    }
    if let InsetEdge::Px(right) = props.layout.inset.right {
        props.layout.inset.right = InsetEdge::Px(Px(right.0 - right_padding.0));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlertActionHorizontalAnchor {
    Left,
    Right,
}

fn default_alert_action_horizontal_anchor(dir: LayoutDirection) -> AlertActionHorizontalAnchor {
    match dir {
        LayoutDirection::Ltr => AlertActionHorizontalAnchor::Right,
        LayoutDirection::Rtl => AlertActionHorizontalAnchor::Left,
    }
}

fn has_explicit_alert_action_horizontal_inset(layout: &LayoutRefinement) -> bool {
    layout
        .inset
        .as_ref()
        .is_some_and(|inset| inset.left.is_some() || inset.right.is_some())
}

fn alert_action_horizontal_anchor(
    action: Option<&AnyElement>,
    fallback_dir: LayoutDirection,
) -> AlertActionHorizontalAnchor {
    let fallback = default_alert_action_horizontal_anchor(fallback_dir);
    let Some(action) = action else {
        return fallback;
    };
    let ElementKind::Container(props) = &action.kind else {
        return fallback;
    };
    let has_left = !matches!(props.layout.inset.left, InsetEdge::Auto);
    let has_right = !matches!(props.layout.inset.right, InsetEdge::Auto);
    match (has_left, has_right) {
        (true, false) => AlertActionHorizontalAnchor::Left,
        (false, true) => AlertActionHorizontalAnchor::Right,
        _ => fallback,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertVariant {
    #[default]
    Default,
    Destructive,
}

#[derive(Debug)]
pub struct Alert {
    children: Vec<AnyElement>,
    variant: AlertVariant,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

#[derive(Debug)]
pub struct AlertAction {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl AlertAction {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn build<H: UiHost, B>(build: B) -> AlertActionBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        AlertActionBuild {
            build: Some(build),
            layout: LayoutRefinement::default(),
            _phantom: PhantomData,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let theme_snapshot = theme.snapshot();
        let dir = crate::direction::use_direction(cx, None);
        let has_explicit_width = self
            .layout
            .size
            .as_ref()
            .and_then(|size| size.width.as_ref())
            .is_some();
        let has_explicit_height = self
            .layout
            .size
            .as_ref()
            .and_then(|size| size.height.as_ref())
            .is_some();
        let has_explicit_horizontal_inset =
            has_explicit_alert_action_horizontal_inset(&self.layout);
        let action_anchor = default_alert_action_horizontal_anchor(dir);
        let default_layout = {
            let layout = LayoutRefinement::default()
                .absolute()
                .top_px(alert_action_top(&theme_snapshot));
            if has_explicit_horizontal_inset {
                layout
            } else {
                match action_anchor {
                    AlertActionHorizontalAnchor::Left => {
                        layout.left_px(alert_action_right(&theme_snapshot))
                    }
                    AlertActionHorizontalAnchor::Right => {
                        layout.right_px(alert_action_right(&theme_snapshot))
                    }
                }
            }
        };
        let mut layout = decl_style::layout_style(theme, default_layout.merge(self.layout));
        // AlertAction is always an absolute slot inside Alert, even when caller refinements add
        // inset shorthands that would otherwise coerce the intermediate LayoutRefinement to
        // `Relative`.
        layout.position = PositionStyle::Absolute;
        if !has_explicit_width {
            layout.size.width = Length::Auto;
        }
        if !has_explicit_height {
            layout.size.height = Length::Auto;
        }

        cx.container(
            ContainerProps {
                layout,
                ..Default::default()
            },
            move |_cx| self.children,
        )
        .test_id(ALERT_ACTION_MARKER_TEST_ID)
    }
}

impl Alert {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            variant: AlertVariant::Default,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn build<H: UiHost, B>(build: B) -> AlertBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        AlertBuild {
            build: Some(build),
            variant: AlertVariant::Default,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            _phantom: PhantomData,
        }
    }

    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        alert_with_patch(cx, self.variant, self.children, self.chrome, self.layout)
    }
}

fn extend_landed_children<H: UiHost, I, T>(
    cx: &mut ElementContext<'_, H>,
    out: &mut Vec<AnyElement>,
    children: I,
) where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    for child in children {
        out.push(child.into_element(cx));
    }
}

pub fn alert<H: UiHost, I, F, T>(
    f: F,
) -> AlertBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    Alert::build(move |cx, out| {
        let children = f(cx);
        extend_landed_children(cx, out, children);
    })
}

pub struct AlertBuild<H, B> {
    build: Option<B>,
    variant: AlertVariant,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> AlertBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
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

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children =
            collect_built_alert_children(cx, self.build.expect("expected alert build closure"));
        Alert::new(children)
            .variant(self.variant)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for AlertBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsChrome for AlertBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> UiSupportsLayout for AlertBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for AlertBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        AlertBuild::into_element(self, cx)
    }
}

pub struct AlertActionBuild<H, B> {
    build: Option<B>,
    layout: LayoutRefinement,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> AlertActionBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = collect_built_alert_children(
            cx,
            self.build.expect("expected alert action build closure"),
        );
        AlertAction::new(children)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for AlertActionBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsLayout for AlertActionBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for AlertActionBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        AlertActionBuild::into_element(self, cx)
    }
}

fn collect_built_alert_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build: impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
) -> Vec<AnyElement> {
    let mut out = Vec::new();
    build(cx, &mut out);
    out
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn patch_svg_icon_to_inherit_current_color(el: &mut AnyElement, fallback: Color, size: Px) {
    let ElementKind::SvgIcon(props) = &mut el.kind else {
        return;
    };

    props.color = fallback;
    props.inherit_color = true;
    props.layout.size.width = fret_ui::element::Length::Px(size);
    props.layout.size.height = fret_ui::element::Length::Px(size);
}

fn maybe_patch_text_color(el: &mut AnyElement, from: Color, to: Color) {
    match &mut el.kind {
        ElementKind::Text(props) if props.color == Some(from) => {
            props.color = Some(to);
        }
        ElementKind::StyledText(props) if props.color == Some(from) => {
            props.color = Some(to);
        }
        ElementKind::SelectableText(props) if props.color == Some(from) => {
            props.color = Some(to);
        }
        _ => {}
    }
}

fn patch_text_color_recursive(el: &mut AnyElement, from: Color, to: Color) {
    maybe_patch_text_color(el, from, to);
    for child in &mut el.children {
        patch_text_color_recursive(child, from, to);
    }
}

fn patch_inherited_foreground_recursive(el: &mut AnyElement, from: Color, to: Color) {
    if el.inherited_foreground == Some(from) {
        el.inherited_foreground = Some(to);
    }

    if let ElementKind::ForegroundScope(props) = &mut el.kind
        && props.foreground == Some(from)
    {
        props.foreground = Some(to);
    }

    for child in &mut el.children {
        patch_inherited_foreground_recursive(child, from, to);
    }
}

fn with_layout_style_mut(element: &mut AnyElement, mut apply: impl FnMut(&mut LayoutStyle)) {
    match &mut element.kind {
        ElementKind::Container(props) => apply(&mut props.layout),
        ElementKind::Pressable(props) => apply(&mut props.layout),
        ElementKind::Flex(props) => apply(&mut props.layout),
        ElementKind::Row(props) => apply(&mut props.layout),
        ElementKind::Column(props) => apply(&mut props.layout),
        ElementKind::Stack(props) => apply(&mut props.layout),
        ElementKind::SemanticFlex(props) => apply(&mut props.flex.layout),
        ElementKind::Grid(props) => apply(&mut props.layout),
        ElementKind::Text(props) => apply(&mut props.layout),
        ElementKind::StyledText(props) => apply(&mut props.layout),
        ElementKind::SelectableText(props) => apply(&mut props.layout),
        ElementKind::SvgIcon(props) => apply(&mut props.layout),
        _ => {}
    }
}

fn patch_alert_fill_width_layout(mut element: AnyElement) -> AnyElement {
    with_layout_style_mut(&mut element, |layout| {
        if matches!(layout.size.width, Length::Auto) {
            layout.size.width = Length::Fill;
        }
        if layout.size.min_width.is_none() {
            layout.size.min_width = Some(Length::Px(Px(0.0)));
        }
    });
    element
}

fn patch_alert_content_grid_lane(mut element: AnyElement) -> AnyElement {
    element = patch_alert_fill_width_layout(element);
    with_layout_style_mut(&mut element, |layout| {
        if layout.grid.column.start.is_none() {
            layout.grid.column.start = Some(2);
        }
    });
    element
}

fn patch_alert_icon_grid_slot(mut element: AnyElement, offset_y: Px) -> AnyElement {
    with_layout_style_mut(&mut element, |layout| {
        if layout.grid.column.start.is_none() {
            layout.grid.column.start = Some(1);
        }
        if layout.grid.row.start.is_none() {
            layout.grid.row.start = Some(1);
        }
        if layout.grid.align_self.is_none() {
            layout.grid.align_self = Some(CrossAlign::Start);
        }
        if layout.margin.top == MarginEdge::Px(Px(0.0)) {
            layout.margin.top = MarginEdge::Px(offset_y);
        }
    });
    element
}

fn alert_content_grid_columns(icon_size: Px, has_icon: bool) -> Vec<GridTrackSizing> {
    vec![
        GridTrackSizing::Px(if has_icon { icon_size } else { Px(0.0) }),
        GridTrackSizing::Fr(1.0),
    ]
}

fn alert_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: AlertVariant,
    mut children: Vec<AnyElement>,
    chrome_override: ChromeRefinement,
    layout_override: LayoutRefinement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let dir = crate::direction::use_direction(cx, None);
    let padding_x = alert_padding_x(&theme);
    let padding_y = alert_padding_y(&theme);
    let gap_x = alert_gap_x(&theme);
    let gap_y = alert_gap_y(&theme);
    let icon_size = alert_icon_size(&theme);
    let icon_offset_y = alert_icon_offset_y(&theme);
    let action_padding_inline_end = alert_action_padding_right(&theme);
    let has_action = children.iter().any(|child| {
        child
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
            == Some(ALERT_ACTION_MARKER_TEST_ID)
    });

    let bg = theme.color_token("card");
    let border = theme.color_token("border");
    let destructive = theme.color_token("destructive");
    let card_fg = theme.color_token("card-foreground");
    let muted_fg = theme.color_token("muted-foreground");

    let fg_default = match variant {
        AlertVariant::Default => card_fg,
        AlertVariant::Destructive => destructive,
    };
    let fg = chrome_override
        .text_color
        .as_ref()
        .map(|c| c.resolve(&theme))
        .unwrap_or(fg_default);
    let destructive_description = alpha_mul(destructive, 0.9);

    let icon = match children.first() {
        Some(first) if matches!(first.kind, ElementKind::SvgIcon(_)) => Some(children.remove(0)),
        _ => None,
    };
    let has_icon = icon.is_some();
    let mut body_children = children;

    let action_idx = body_children.iter().position(|child| {
        child
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
            == Some(ALERT_ACTION_MARKER_TEST_ID)
    });
    let mut action = action_idx.map(|idx| body_children.remove(idx));
    let action_anchor = alert_action_horizontal_anchor(action.as_ref(), dir);

    if variant == AlertVariant::Destructive
        && let Some(description) = body_children.get_mut(1)
    {
        patch_text_color_recursive(description, muted_fg, destructive_description);
        patch_inherited_foreground_recursive(description, muted_fg, destructive_description);
    }

    let (padding_left, padding_right) = if has_action {
        match action_anchor {
            AlertActionHorizontalAnchor::Left => (action_padding_inline_end, padding_x),
            AlertActionHorizontalAnchor::Right => (padding_x, action_padding_inline_end),
        }
    } else {
        (padding_x, padding_x)
    };

    let props = decl_style::container_props(
        &theme,
        ChromeRefinement::default()
            .merge(ChromeRefinement {
                padding: Some(PaddingRefinement {
                    top: Some(MetricRef::Px(padding_y)),
                    right: Some(MetricRef::Px(padding_right)),
                    bottom: Some(MetricRef::Px(padding_y)),
                    left: Some(MetricRef::Px(padding_left)),
                }),
                ..Default::default()
            })
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Color(bg))
            .border_color(ColorRef::Color(border))
            .merge(chrome_override),
        // shadcn/ui v4: Alert root uses `w-full` by default.
        LayoutRefinement::default().w_full().merge(layout_override),
    );

    let mut content_children: Vec<AnyElement> = body_children
        .into_iter()
        .map(patch_alert_content_grid_lane)
        .collect();
    if let Some(mut icon) = icon {
        patch_svg_icon_to_inherit_current_color(&mut icon, fg, icon_size);
        content_children.insert(0, patch_alert_icon_grid_slot(icon, icon_offset_y));
    }

    let main = cx.grid(
        GridProps {
            layout: decl_style::layout_style(
                &theme,
                LayoutRefinement::default().w_full().min_w_0(),
            ),
            cols: 1,
            rows: None,
            template_columns: Some(alert_content_grid_columns(icon_size, has_icon)),
            template_rows: None,
            gap: SpacingLength::Px(Px(0.0)),
            column_gap: Some(SpacingLength::Px(if has_icon { gap_x } else { Px(0.0) })),
            row_gap: Some(SpacingLength::Px(gap_y)),
            padding: SpacingEdges::all(SpacingLength::Px(Px(0.0))),
            justify: MainAlign::Start,
            align: CrossAlign::Start,
            justify_items: None,
        },
        move |_cx| content_children,
    );

    let mut props = props;
    props.layout.position = PositionStyle::Relative;
    if let Some(action) = action.as_mut() {
        translate_alert_action_to_padding_box(
            action,
            props.padding.top,
            props.padding.left,
            props.padding.right,
        );
    }

    cx.container(props, move |_cx| {
        let mut out: Vec<AnyElement> = vec![main.inherit_foreground(fg)];
        if let Some(action) = action {
            out.push(action);
        }
        out
    })
    .attach_semantics(SemanticsDecoration::default().role(SemanticsRole::Alert))
}

#[derive(Debug)]
pub struct AlertTitle {
    content: AlertTitleContent,
}

#[derive(Debug)]
enum AlertTitleContent {
    Text(Arc<str>),
    Children(Vec<AnyElement>),
}

impl AlertTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            content: AlertTitleContent::Text(text.into()),
        }
    }

    pub fn new_children(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            content: AlertTitleContent::Children(children.into_iter().collect()),
        }
    }

    /// Builder-first variant that collects children at `into_element(cx)` time.
    pub fn build<H: UiHost, B>(build: B) -> AlertTitleBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        AlertTitleBuild {
            build: Some(build),
            _phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let px = theme
            .metric_by_key("component.alert.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let line_height = theme
            .metric_by_key("component.alert.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));
        let mut title = match self.content {
            AlertTitleContent::Text(text) => ui::text(text)
                .text_size_px(px)
                .line_height_px(line_height)
                .font_weight(FontWeight::MEDIUM)
                // Tailwind: `tracking-tight` ~= `-0.025em`.
                .letter_spacing_em(-0.025)
                .truncate()
                .into_element(cx),
            AlertTitleContent::Children(mut children) => {
                for child in &mut children {
                    patch_alert_title_text_style_recursive(child, px, line_height);
                }

                match children.len() {
                    0 => ui::text("")
                        .text_size_px(px)
                        .line_height_px(line_height)
                        .font_weight(FontWeight::MEDIUM)
                        .letter_spacing_em(-0.025)
                        .truncate()
                        .into_element(cx),
                    1 => children.pop().expect("children.len() == 1"),
                    _ => ui::v_flex(move |_cx| children)
                        .gap(Space::N0)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx),
                }
            }
        };

        let icon_min_height = alert_icon_size(&theme);
        with_layout_style_mut(&mut title, |layout| {
            if layout.size.min_height.is_none() {
                layout.size.min_height = Some(Length::Px(icon_min_height));
            }
        });
        title
    }
}

fn patch_alert_text_style_recursive(
    el: &mut AnyElement,
    px: Px,
    line_height: Px,
    weight: FontWeight,
    wrap: TextWrap,
    overflow: TextOverflow,
) {
    fn patch_text_style(
        style: &mut Option<fret_core::TextStyle>,
        px: Px,
        line_height: Px,
        weight: FontWeight,
    ) {
        let mut style_value = style.take().unwrap_or_default();
        style_value.size = px;
        style_value.weight = weight;
        style_value.line_height = Some(line_height);
        style_value.line_height_em = None;
        style_value.letter_spacing_em = Some(if weight == FontWeight::MEDIUM {
            -0.025
        } else {
            0.0
        });
        *style = Some(style_value);
    }

    match &mut el.kind {
        ElementKind::Text(props) => {
            patch_text_style(&mut props.style, px, line_height, weight);
            props.wrap = wrap;
            props.overflow = overflow;
        }
        ElementKind::StyledText(props) => {
            patch_text_style(&mut props.style, px, line_height, weight);
            props.wrap = wrap;
            props.overflow = overflow;
        }
        ElementKind::SelectableText(props) => {
            patch_text_style(&mut props.style, px, line_height, weight);
            props.wrap = wrap;
            props.overflow = overflow;
        }
        _ => {}
    }

    for child in &mut el.children {
        patch_alert_text_style_recursive(child, px, line_height, weight, wrap, overflow);
    }
}

fn patch_alert_title_text_style_recursive(el: &mut AnyElement, px: Px, line_height: Px) {
    patch_alert_text_style_recursive(
        el,
        px,
        line_height,
        FontWeight::MEDIUM,
        TextWrap::None,
        TextOverflow::Ellipsis,
    );
}

pub struct AlertTitleBuild<H, B> {
    build: Option<B>,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> AlertTitleBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = collect_built_alert_children(
            cx,
            self.build.expect("expected alert title build closure"),
        );
        AlertTitle::new_children(children).into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for AlertTitleBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, _patch: UiPatch) -> Self {
        self
    }
}

impl<H: UiHost, B> IntoUiElement<H> for AlertTitleBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        AlertTitleBuild::into_element(self, cx)
    }
}

#[derive(Debug)]
pub struct AlertDescription {
    content: AlertDescriptionContent,
}

#[derive(Debug)]
enum AlertDescriptionContent {
    Text(Arc<str>),
    Children(Vec<AnyElement>),
}

impl AlertDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            content: AlertDescriptionContent::Text(text.into()),
        }
    }

    pub fn new_children(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            content: AlertDescriptionContent::Children(children.into_iter().collect()),
        }
    }

    /// Builder-first variant that collects children at `into_element(cx)` time.
    pub fn build<H: UiHost, B>(build: B) -> AlertDescriptionBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        AlertDescriptionBuild {
            build: Some(build),
            _phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let gap = MetricRef::space(Space::N1).resolve(&theme);
        let layout =
            decl_style::layout_style(&theme, LayoutRefinement::default().w_full().min_w_0());
        let props = GridProps {
            layout,
            cols: 1,
            rows: None,
            template_columns: Some(vec![GridTrackSizing::Fr(1.0)]),
            template_rows: None,
            gap: SpacingLength::Px(gap),
            column_gap: None,
            row_gap: None,
            padding: SpacingEdges::all(SpacingLength::Px(Px(0.0))),
            justify: MainAlign::Start,
            align: CrossAlign::Start,
            justify_items: Some(CrossAlign::Start),
        };
        let children = match self.content {
            AlertDescriptionContent::Text(text) => vec![patch_alert_fill_width_layout(
                ui::raw_text(text)
                    .wrap(TextWrap::Word)
                    .overflow(TextOverflow::Clip)
                    .into_element(cx),
            )],
            AlertDescriptionContent::Children(children) => children
                .into_iter()
                .map(patch_alert_fill_width_layout)
                .collect(),
        };

        scope_description_text(
            cx.grid(props, move |_cx| children),
            &theme,
            "component.alert.description",
        )
    }
}

pub struct AlertDescriptionBuild<H, B> {
    build: Option<B>,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> AlertDescriptionBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = collect_built_alert_children(
            cx,
            self.build
                .expect("expected alert description build closure"),
        );
        AlertDescription::new_children(children).into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for AlertDescriptionBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, _patch: UiPatch) -> Self {
        self
    }
}

impl<H: UiHost, B> IntoUiElement<H> for AlertDescriptionBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        AlertDescriptionBuild::into_element(self, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, AttributedText, Color, Point, Px, Rect, Size, TextOverflow, TextSpan,
    };
    use fret_icons::IconId;
    use fret_ui::element::{
        CrossAlign, ElementKind, GridProps, InsetEdge, LayoutStyle, Length, SpacingLength,
    };
    use fret_ui_kit::declarative::icon as decl_icon;

    fn contains_foreground_scope(el: &AnyElement) -> bool {
        matches!(el.kind, ElementKind::ForegroundScope(_))
            || el.children.iter().any(contains_foreground_scope)
    }

    fn find_first_inherited_foreground_node(el: &AnyElement) -> Option<&AnyElement> {
        if el.inherited_foreground.is_some() {
            return Some(el);
        }
        el.children
            .iter()
            .find_map(find_first_inherited_foreground_node)
    }

    fn find_text_element<'a>(el: &'a AnyElement, needle: &str) -> Option<&'a AnyElement> {
        match &el.kind {
            ElementKind::Text(props) if props.text.as_ref() == needle => Some(el),
            _ => el
                .children
                .iter()
                .find_map(|child| find_text_element(child, needle)),
        }
    }

    fn find_first_styled_text(el: &AnyElement) -> Option<&fret_ui::element::StyledTextProps> {
        if let ElementKind::StyledText(props) = &el.kind {
            return Some(props);
        }
        el.children.iter().find_map(find_first_styled_text)
    }

    fn find_first_selectable_text(
        el: &AnyElement,
    ) -> Option<&fret_ui::element::SelectableTextProps> {
        if let ElementKind::SelectableText(props) = &el.kind {
            return Some(props);
        }
        el.children.iter().find_map(find_first_selectable_text)
    }

    fn find_element_by_test_id<'a>(el: &'a AnyElement, needle: &str) -> Option<&'a AnyElement> {
        if el
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
            == Some(needle)
        {
            return Some(el);
        }
        el.children
            .iter()
            .find_map(|child| find_element_by_test_id(child, needle))
    }

    fn find_first_grid(el: &AnyElement) -> Option<&AnyElement> {
        if matches!(el.kind, ElementKind::Grid(_)) {
            return Some(el);
        }
        el.children.iter().find_map(find_first_grid)
    }

    fn layout_style_for(element: &AnyElement) -> Option<&LayoutStyle> {
        match &element.kind {
            ElementKind::Container(props) => Some(&props.layout),
            ElementKind::Pressable(props) => Some(&props.layout),
            ElementKind::Flex(props) => Some(&props.layout),
            ElementKind::Row(props) => Some(&props.layout),
            ElementKind::Column(props) => Some(&props.layout),
            ElementKind::Stack(props) => Some(&props.layout),
            ElementKind::SemanticFlex(props) => Some(&props.flex.layout),
            ElementKind::Grid(props) => Some(&props.layout),
            ElementKind::Text(props) => Some(&props.layout),
            ElementKind::StyledText(props) => Some(&props.layout),
            ElementKind::SelectableText(props) => Some(&props.layout),
            ElementKind::SvgIcon(props) => Some(&props.layout),
            _ => None,
        }
    }

    #[test]
    fn alert_description_children_scope_inherited_text_style() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            AlertDescription::new_children([cx.text("Nested body")]).into_element(cx)
        });

        let text = find_text_element(&element, "Nested body").expect("expected nested text node");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected nested alert description child to be text");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());

        let theme = fret_ui::Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::description_text_refinement(
                &theme,
                "component.alert.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );
    }

    #[test]
    fn alert_description_uses_source_aligned_grid_stack() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            AlertDescription::new_children([cx.text("Line one"), cx.text("Line two")])
                .into_element(cx)
        });

        let ElementKind::Grid(GridProps {
            layout,
            gap,
            justify_items,
            ..
        }) = &element.kind
        else {
            panic!("expected AlertDescription root to be a grid element");
        };

        assert_eq!(layout.size.width, Length::Fill);
        assert_eq!(layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(
            *gap,
            MetricRef::space(Space::N1)
                .resolve(Theme::global(&app))
                .into()
        );
        assert_eq!(*justify_items, Some(CrossAlign::Start));

        for child in &element.children {
            let layout =
                layout_style_for(child).expect("alert description child should carry layout");
            assert_eq!(layout.size.width, Length::Fill);
            assert_eq!(layout.size.min_width, Some(Length::Px(Px(0.0))));
        }
    }

    #[test]
    fn alert_stamps_role_without_layout_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Alert::new([cx.text("Hello")]).into_element(cx)
        });

        assert!(
            !matches!(element.kind, ElementKind::Semantics(_)),
            "expected Alert to avoid `Semantics` wrappers; use `attach_semantics` instead"
        );
        assert_eq!(
            element.semantics_decoration.as_ref().and_then(|d| d.role),
            Some(SemanticsRole::Alert)
        );
    }

    #[test]
    fn alert_title_truncates_by_default_like_current_shadcn() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            AlertTitle::new("A very long alert title that should wrap").into_element(cx)
        });

        let ElementKind::Text(props) = &element.kind else {
            panic!(
                "expected AlertTitle to be a Text element, got {:?}",
                element.kind
            );
        };

        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.layout.size.min_height, Some(Length::Px(Px(16.0))));
    }

    #[test]
    fn alert_title_children_patch_rich_text_with_title_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(100.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let rich = AttributedText::new(
                Arc::<str>::from("Alert title rendered from a rich text child"),
                Arc::<[TextSpan]>::from([TextSpan::new(
                    "Alert title rendered from a rich text child".len(),
                )]),
            );

            AlertTitle::new_children([cx.styled_text(rich)]).into_element(cx)
        });

        let ElementKind::StyledText(props) = &element.kind else {
            panic!(
                "expected AlertTitle::new_children(single child) to keep the rich text node, got {:?}",
                element.kind
            );
        };

        let style = props
            .style
            .as_ref()
            .expect("expected AlertTitle children to receive explicit title text style");
        let theme = Theme::global(&app).snapshot();
        let expected_px = theme
            .metric_by_key("component.alert.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let expected_line_height = theme
            .metric_by_key("component.alert.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        assert_eq!(style.size, expected_px);
        assert_eq!(style.weight, FontWeight::MEDIUM);
        assert_eq!(style.line_height, Some(expected_line_height));
        assert_eq!(style.letter_spacing_em, Some(-0.025));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
    }

    #[test]
    fn alert_description_children_scope_rich_text_with_description_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let rich = AttributedText::new(
                Arc::<str>::from("Alert description rendered from a rich text child"),
                Arc::<[TextSpan]>::from([TextSpan::new(
                    "Alert description rendered from a rich text child".len(),
                )]),
            );

            AlertDescription::new_children([cx.styled_text(rich)]).into_element(cx)
        });

        let props = find_first_styled_text(&element)
            .expect("expected AlertDescription children to keep the rich text node");
        assert!(props.style.is_none());
        assert!(props.color.is_none());

        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::description_text_refinement(
                &theme,
                "component.alert.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
    }

    #[test]
    fn alert_title_children_preserve_interactive_spans_under_title_scope() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let rich = AttributedText::new(
                Arc::<str>::from("Open support article"),
                Arc::<[TextSpan]>::from([TextSpan::new("Open support article".len())]),
            );

            let mut props = fret_ui::element::SelectableTextProps::new(rich);
            props.interactive_spans =
                Arc::from([fret_ui::element::SelectableTextInteractiveSpan {
                    range: 0.."Open support article".len(),
                    tag: Arc::<str>::from("support-article"),
                }]);

            AlertTitle::new_children([cx.selectable_text_props(props)]).into_element(cx)
        });

        let props = find_first_selectable_text(&element)
            .expect("expected AlertTitle children to keep selectable text nodes");
        let style = props
            .style
            .as_ref()
            .expect("expected AlertTitle to patch selectable text with title typography");
        let theme = Theme::global(&app).snapshot();
        let expected_px = theme
            .metric_by_key("component.alert.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let expected_line_height = theme
            .metric_by_key("component.alert.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        assert_eq!(props.interactive_spans.len(), 1);
        assert_eq!(props.interactive_spans[0].tag.as_ref(), "support-article");
        assert_eq!(style.size, expected_px);
        assert_eq!(style.weight, FontWeight::MEDIUM);
        assert_eq!(style.line_height, Some(expected_line_height));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
    }

    #[test]
    fn alert_description_children_preserve_interactive_spans_under_description_scope() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let rich = AttributedText::new(
                Arc::<str>::from("Open support article"),
                Arc::<[TextSpan]>::from([TextSpan::new("Open support article".len())]),
            );

            let mut props = fret_ui::element::SelectableTextProps::new(rich);
            props.interactive_spans =
                Arc::from([fret_ui::element::SelectableTextInteractiveSpan {
                    range: 0.."Open support article".len(),
                    tag: Arc::<str>::from("support-article"),
                }]);

            AlertDescription::new_children([cx.selectable_text_props(props)]).into_element(cx)
        });

        let props = find_first_selectable_text(&element)
            .expect("expected AlertDescription children to keep selectable text nodes");
        assert!(props.style.is_none());
        assert!(props.color.is_none());

        let theme = Theme::global(&app).snapshot();
        assert_eq!(props.interactive_spans.len(), 1);
        assert_eq!(props.interactive_spans[0].tag.as_ref(), "support-article");
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::description_text_refinement(
                &theme,
                "component.alert.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );
    }

    #[test]
    fn alert_title_build_collects_children_on_builder_path() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            AlertTitle::build(|cx, out| {
                out.push(cx.text("Builder title content"));
            })
            .into_element(cx)
        });

        let text = find_text_element(&element, "Builder title content")
            .expect("expected AlertTitle::build to retain nested text content");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected AlertTitle::build child to resolve to a text node");
        };

        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
    }

    #[test]
    fn alert_description_build_collects_children_on_builder_path() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            AlertDescription::build(|cx, out| {
                out.push(cx.text("Builder description content"));
            })
            .into_element(cx)
        });

        let text = find_text_element(&element, "Builder description content")
            .expect("expected AlertDescription::build to retain nested text content");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected AlertDescription::build child to resolve to a text node");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
    }

    #[test]
    fn alert_with_action_reserves_right_padding_like_shadcn_in_ltr() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Alert::new([
                AlertTitle::new("Heads up!").into_element(cx),
                AlertDescription::new("You can add components to your app.").into_element(cx),
                AlertAction::new([cx.text("Undo")]).into_element(cx),
            ])
            .into_element(cx)
        });

        let ElementKind::Container(props) = &element.kind else {
            panic!(
                "expected Alert root to be a Container, got {:?}",
                element.kind
            );
        };

        assert_eq!(props.padding.right, SpacingLength::Px(Px(72.0)));
    }

    #[test]
    fn alert_with_action_reserves_left_padding_like_shadcn_in_rtl() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            crate::direction::with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
                Alert::new([
                    AlertTitle::new("Heads up!").into_element(cx),
                    AlertDescription::new("You can add components to your app.").into_element(cx),
                    AlertAction::new([cx.text("Undo")]).into_element(cx),
                ])
                .into_element(cx)
            })
        });

        let ElementKind::Container(props) = &element.kind else {
            panic!(
                "expected Alert root to be a Container, got {:?}",
                element.kind
            );
        };

        assert_eq!(props.padding.left, SpacingLength::Px(Px(72.0)));
        assert_eq!(props.padding.right, SpacingLength::Px(Px(16.0)));
    }

    #[test]
    fn alert_with_action_translates_absolute_action_offsets_to_padding_box_in_ltr() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Alert::new([
                AlertTitle::new("Heads up!").into_element(cx),
                AlertAction::new([cx.text("Undo")]).into_element(cx),
            ])
            .into_element(cx)
        });

        let action = find_element_by_test_id(&element, ALERT_ACTION_MARKER_TEST_ID)
            .expect("expected Alert to keep the action child under the action marker test id");
        let ElementKind::Container(props) = &action.kind else {
            panic!(
                "expected Alert action marker to resolve to a Container, got {:?}",
                action.kind
            );
        };

        assert_eq!(props.layout.position, PositionStyle::Absolute);
        assert_eq!(props.layout.inset.top, InsetEdge::Px(Px(-4.0)));
        assert_eq!(props.layout.inset.right, InsetEdge::Px(Px(-64.0)));
        assert_eq!(props.layout.inset.left, InsetEdge::Auto);
    }

    #[test]
    fn alert_with_action_translates_absolute_action_offsets_to_padding_box_in_rtl() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            crate::direction::with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
                Alert::new([
                    AlertTitle::new("Heads up!").into_element(cx),
                    AlertAction::new([cx.text("Undo")]).into_element(cx),
                ])
                .into_element(cx)
            })
        });

        let action = find_element_by_test_id(&element, ALERT_ACTION_MARKER_TEST_ID)
            .expect("expected Alert to keep the action child under the action marker test id");
        let ElementKind::Container(props) = &action.kind else {
            panic!(
                "expected Alert action marker to resolve to a Container, got {:?}",
                action.kind
            );
        };

        assert_eq!(props.layout.position, PositionStyle::Absolute);
        assert_eq!(props.layout.inset.top, InsetEdge::Px(Px(-4.0)));
        assert_eq!(props.layout.inset.left, InsetEdge::Px(Px(-64.0)));
        assert_eq!(props.layout.inset.right, InsetEdge::Auto);
    }

    #[test]
    fn alert_root_content_grid_tracks_and_gaps_match_current_shadcn_source() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Alert::new([AlertTitle::new("Heads up!").into_element(cx)]).into_element(cx)
        });

        let ElementKind::Container(props) = &element.kind else {
            panic!(
                "expected Alert root to be a Container, got {:?}",
                element.kind
            );
        };

        assert_eq!(props.padding.left, SpacingLength::Px(Px(16.0)));
        assert_eq!(props.padding.right, SpacingLength::Px(Px(16.0)));
        assert_eq!(props.padding.top, SpacingLength::Px(Px(12.0)));
        assert_eq!(props.padding.bottom, SpacingLength::Px(Px(12.0)));

        let grid = find_first_grid(&element).expect("expected Alert to build an inner grid");
        let ElementKind::Grid(GridProps {
            layout,
            template_columns,
            column_gap,
            row_gap,
            align,
            ..
        }) = &grid.kind
        else {
            panic!("expected Alert content root to be a grid element");
        };

        assert_eq!(layout.size.width, Length::Fill);
        assert_eq!(layout.size.min_width, Some(Length::Px(Px(0.0))));
        let expected_columns = alert_content_grid_columns(Px(16.0), false);
        assert_eq!(
            template_columns.as_deref(),
            Some(expected_columns.as_slice())
        );
        assert_eq!(*column_gap, Some(SpacingLength::Px(Px(0.0))));
        assert_eq!(*row_gap, Some(SpacingLength::Px(Px(2.0))));
        assert_eq!(*align, CrossAlign::Start);

        let title = find_text_element(grid, "Heads up!").expect("expected Alert title text");
        let layout = layout_style_for(title).expect("alert title should carry layout");
        assert_eq!(layout.grid.column.start, Some(2));
        assert_eq!(layout.size.width, Length::Fill);
        assert_eq!(layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(layout.size.min_height, Some(Length::Px(Px(16.0))));
    }

    #[test]
    fn alert_with_icon_uses_source_aligned_grid_columns_and_icon_slot() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Alert::new([
                decl_icon::icon(cx, IconId::new_static("lucide.terminal")),
                AlertTitle::new("Heads up!").into_element(cx),
                AlertDescription::new("You can add components to your app.").into_element(cx),
            ])
            .into_element(cx)
        });

        let grid = find_first_grid(&element).expect("expected Alert to build an inner grid");
        let ElementKind::Grid(GridProps {
            template_columns,
            column_gap,
            row_gap,
            ..
        }) = &grid.kind
        else {
            panic!("expected Alert content root to be a grid element");
        };

        let expected_columns = alert_content_grid_columns(Px(16.0), true);
        assert_eq!(
            template_columns.as_deref(),
            Some(expected_columns.as_slice())
        );
        assert_eq!(*column_gap, Some(SpacingLength::Px(Px(12.0))));
        assert_eq!(*row_gap, Some(SpacingLength::Px(Px(2.0))));

        let icon = grid
            .children
            .iter()
            .find(|child| matches!(child.kind, ElementKind::SvgIcon(_)))
            .expect("expected Alert grid to keep the leading icon as a direct child");
        let icon_layout = layout_style_for(icon).expect("alert icon should carry layout");
        assert_eq!(icon_layout.grid.column.start, Some(1));
        assert_eq!(icon_layout.grid.row.start, Some(1));
        assert_eq!(icon_layout.grid.align_self, Some(CrossAlign::Start));
        assert_eq!(icon_layout.margin.top, MarginEdge::Px(Px(2.0)));

        let description = grid
            .children
            .iter()
            .find(|child| matches!(child.kind, ElementKind::Grid(_)))
            .expect("expected Alert grid to include the description grid child");
        let description_layout =
            layout_style_for(description).expect("alert description should carry layout");
        assert_eq!(description_layout.grid.column.start, Some(2));
        assert_eq!(description_layout.size.width, Length::Fill);
    }

    #[test]
    fn alert_build_collects_children_on_builder_path() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Alert::build(|cx, out| {
                use fret_ui_kit::ui::UiElementSinkExt as _;

                out.push_ui(cx, AlertTitle::new("Heads up!"));
                out.push_ui(cx, AlertDescription::new("Built via Alert::build"));
                out.push_ui(
                    cx,
                    AlertAction::build(|cx, out| {
                        out.push(cx.text("Undo"));
                    }),
                );
            })
            .into_element(cx)
        });

        let ElementKind::Container(props) = &element.kind else {
            panic!(
                "expected Alert::build root to be a Container, got {:?}",
                element.kind
            );
        };

        assert_eq!(props.padding.right, SpacingLength::Px(Px(72.0)));
        assert!(
            find_text_element(&element, "Built via Alert::build").is_some(),
            "expected Alert::build to retain nested description content"
        );
    }

    #[test]
    fn alert_build_reserves_left_padding_like_shadcn_in_rtl() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            crate::direction::with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
                Alert::build(|cx, out| {
                    use fret_ui_kit::ui::UiElementSinkExt as _;

                    out.push_ui(cx, AlertTitle::new("Heads up!"));
                    out.push_ui(cx, AlertDescription::new("Built via Alert::build"));
                    out.push_ui(
                        cx,
                        AlertAction::build(|cx, out| {
                            out.push(cx.text("Undo"));
                        }),
                    );
                })
                .into_element(cx)
            })
        });

        let ElementKind::Container(props) = &element.kind else {
            panic!(
                "expected Alert::build root to be a Container, got {:?}",
                element.kind
            );
        };

        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            props.padding.left,
            SpacingLength::Px(alert_action_padding_right(&theme))
        );
        assert_eq!(
            props.padding.right,
            SpacingLength::Px(alert_padding_x(&theme))
        );
        assert!(
            find_text_element(&element, "Built via Alert::build").is_some(),
            "expected Alert::build to retain nested description content in RTL"
        );
    }

    #[test]
    fn alert_action_explicit_right_override_wins_over_rtl_fallback() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );
        let custom_right = Px(24.0);

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            crate::direction::with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
                Alert::new([
                    AlertTitle::new("Heads up!").into_element(cx),
                    AlertAction::new([cx.text("Undo")])
                        .refine_layout(
                            LayoutRefinement::default()
                                .right_px(custom_right)
                                .w_px(Px(88.0)),
                        )
                        .into_element(cx),
                ])
                .into_element(cx)
            })
        });

        let ElementKind::Container(root_props) = &element.kind else {
            panic!(
                "expected Alert root to be a Container, got {:?}",
                element.kind
            );
        };

        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            root_props.padding.left,
            SpacingLength::Px(alert_padding_x(&theme))
        );
        assert_eq!(
            root_props.padding.right,
            SpacingLength::Px(alert_action_padding_right(&theme))
        );

        let action = find_element_by_test_id(&element, ALERT_ACTION_MARKER_TEST_ID)
            .expect("expected Alert to keep the action child under the action marker test id");
        let ElementKind::Container(action_props) = &action.kind else {
            panic!(
                "expected Alert action marker to resolve to a Container, got {:?}",
                action.kind
            );
        };

        assert_eq!(action_props.layout.position, PositionStyle::Absolute);
        assert_eq!(
            action_props.layout.inset.top,
            InsetEdge::Px(Px(alert_action_top(&theme).0 - alert_padding_y(&theme).0))
        );
        assert_eq!(
            action_props.layout.inset.right,
            InsetEdge::Px(Px(custom_right.0 - alert_action_padding_right(&theme).0,))
        );
        assert_eq!(action_props.layout.inset.left, InsetEdge::Auto);
    }

    #[test]
    fn alert_action_uses_upstream_offsets_and_merges_layout_refinement() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            AlertAction::new([cx.text("Undo")])
                .refine_layout(LayoutRefinement::default().w_px(Px(88.0)))
                .into_element(cx)
        });

        let ElementKind::Container(props) = &element.kind else {
            panic!(
                "expected AlertAction root to be a Container, got {:?}",
                element.kind
            );
        };

        assert_eq!(props.layout.position, PositionStyle::Absolute);
        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            props.layout.inset.top,
            InsetEdge::Px(alert_action_top(&theme))
        );
        assert_eq!(
            props.layout.inset.right,
            InsetEdge::Px(alert_action_right(&theme))
        );
        assert_eq!(props.layout.inset.left, InsetEdge::Auto);
        assert_eq!(props.layout.size.width, Length::Px(Px(88.0)));
    }

    #[test]
    fn alert_action_uses_logical_end_offsets_in_rtl() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            crate::direction::with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
                AlertAction::new([cx.text("Undo")])
                    .refine_layout(LayoutRefinement::default().w_px(Px(88.0)))
                    .into_element(cx)
            })
        });

        let ElementKind::Container(props) = &element.kind else {
            panic!(
                "expected AlertAction root to be a Container, got {:?}",
                element.kind
            );
        };

        assert_eq!(props.layout.position, PositionStyle::Absolute);
        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            props.layout.inset.top,
            InsetEdge::Px(alert_action_top(&theme))
        );
        assert_eq!(props.layout.inset.right, InsetEdge::Auto);
        assert_eq!(
            props.layout.inset.left,
            InsetEdge::Px(alert_action_right(&theme))
        );
        assert_eq!(props.layout.size.width, Length::Px(Px(88.0)));
    }

    #[test]
    fn alert_action_build_preserves_upstream_offsets() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            AlertAction::build(|cx, out| {
                out.push(cx.text("Undo"));
            })
            .refine_layout(LayoutRefinement::default().w_px(Px(88.0)))
            .into_element(cx)
        });

        let ElementKind::Container(props) = &element.kind else {
            panic!(
                "expected AlertAction::build root to be a Container, got {:?}",
                element.kind
            );
        };

        assert_eq!(props.layout.position, PositionStyle::Absolute);
        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            props.layout.inset.top,
            InsetEdge::Px(alert_action_top(&theme))
        );
        assert_eq!(
            props.layout.inset.right,
            InsetEdge::Px(alert_action_right(&theme))
        );
        assert_eq!(props.layout.inset.left, InsetEdge::Auto);
        assert_eq!(props.layout.size.width, Length::Px(Px(88.0)));
    }

    #[test]
    fn alert_action_build_uses_logical_end_offsets_in_rtl() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            crate::direction::with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
                AlertAction::build(|cx, out| {
                    out.push(cx.text("Undo"));
                })
                .refine_layout(LayoutRefinement::default().w_px(Px(88.0)))
                .into_element(cx)
            })
        });

        let ElementKind::Container(props) = &element.kind else {
            panic!(
                "expected AlertAction::build root to be a Container, got {:?}",
                element.kind
            );
        };

        assert_eq!(props.layout.position, PositionStyle::Absolute);
        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            props.layout.inset.top,
            InsetEdge::Px(alert_action_top(&theme))
        );
        assert_eq!(props.layout.inset.right, InsetEdge::Auto);
        assert_eq!(
            props.layout.inset.left,
            InsetEdge::Px(alert_action_right(&theme))
        );
        assert_eq!(props.layout.size.width, Length::Px(Px(88.0)));
    }

    #[test]
    fn alert_forces_icon_to_inherit_current_color() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let icon = decl_icon::icon_with(
                cx,
                IconId::new_static("lucide.terminal"),
                None,
                Some(ColorRef::Color(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                })),
            );

            Alert::new([
                icon,
                AlertTitle::new("Heads up!").into_element(cx),
                AlertDescription::new("You can add components to your app.").into_element(cx),
            ])
            .into_element(cx)
        });

        fn find_first_svg_icon(el: &AnyElement) -> Option<&fret_ui::element::SvgIconProps> {
            if let ElementKind::SvgIcon(props) = &el.kind {
                return Some(props);
            }
            for child in &el.children {
                if let Some(found) = find_first_svg_icon(child) {
                    return Some(found);
                }
            }
            None
        }

        let icon = find_first_svg_icon(&element).expect("expected an svg icon under Alert");
        assert!(
            icon.inherit_color,
            "expected Alert icon to inherit currentColor via ForegroundScope"
        );
    }

    #[test]
    fn alert_attaches_foreground_to_main_content_without_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let expected_fg = Theme::global(&*cx.app).color_token("foreground");
            let el = Alert::new([
                decl_icon::icon_with(cx, IconId::new_static("lucide.terminal"), None, None),
                AlertTitle::new("Heads up!").into_element(cx),
                AlertDescription::new("You can add components to your app.").into_element(cx),
            ])
            .into_element(cx);

            let inherited = find_first_inherited_foreground_node(&el)
                .expect("expected alert subtree to carry inherited foreground");
            assert_eq!(inherited.inherited_foreground, Some(expected_fg));
            assert!(
                !contains_foreground_scope(&el),
                "expected Alert to attach inherited foreground without inserting a ForegroundScope"
            );
        });
    }
}
