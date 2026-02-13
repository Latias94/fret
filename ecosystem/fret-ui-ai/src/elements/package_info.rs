//! AI Elements-aligned `PackageInfo` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/package-info.tsx`.

use std::sync::Arc;

use fret_core::{
    Color, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_icons::IconId;
use fret_ui::element::{
    AnyElement, LayoutStyle, Length, SemanticsDecoration, SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Justify, LayoutRefinement, Radius, Space,
};
use fret_ui_shadcn::{Badge, BadgeVariant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageInfoChangeKind {
    Major,
    Minor,
    Patch,
    Added,
    Removed,
}

impl PackageInfoChangeKind {
    fn label(self) -> &'static str {
        match self {
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Patch => "patch",
            Self::Added => "added",
            Self::Removed => "removed",
        }
    }

    fn icon(self) -> IconId {
        match self {
            Self::Added => fret_icons::ids::ui::PLUS,
            Self::Removed => fret_icons::ids::ui::MINUS,
            Self::Major | Self::Minor | Self::Patch => fret_icons::ids::ui::ARROW_RIGHT,
        }
    }
}

#[derive(Clone)]
pub struct PackageInfoController {
    pub name: Arc<str>,
    pub current_version: Option<Arc<str>>,
    pub new_version: Option<Arc<str>>,
    pub change_type: Option<PackageInfoChangeKind>,
}

impl std::fmt::Debug for PackageInfoController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackageInfoController")
            .field("name", &self.name)
            .field("current_version", &self.current_version.as_deref())
            .field("new_version", &self.new_version.as_deref())
            .field("change_type", &self.change_type)
            .finish()
    }
}

#[derive(Debug, Default, Clone)]
struct PackageInfoProviderState {
    controller: Option<PackageInfoController>,
}

pub fn use_package_info_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<PackageInfoController> {
    cx.inherited_state::<PackageInfoProviderState>()
        .and_then(|st| st.controller.clone())
}

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn monospace_style(theme: &Theme, size: Px, weight: FontWeight) -> TextStyle {
    TextStyle {
        font: FontId::monospace(),
        size,
        weight,
        slant: Default::default(),
        line_height: Some(theme.metric_required("metric.font.mono_line_height")),
        letter_spacing_em: None,
    }
}

fn change_type_bg(change_type: PackageInfoChangeKind) -> ColorRef {
    // These are tailwind-ish sRGB values (light-theme) used as a safe default. Apps can override
    // via theme tokens.
    let (key, fallback) = match change_type {
        PackageInfoChangeKind::Added => (
            "color.ai.package_info.change.added.bg",
            Color {
                r: 219.0 / 255.0,
                g: 234.0 / 255.0,
                b: 254.0 / 255.0,
                a: 1.0,
            },
        ),
        PackageInfoChangeKind::Major => (
            "color.ai.package_info.change.major.bg",
            Color {
                r: 254.0 / 255.0,
                g: 226.0 / 255.0,
                b: 226.0 / 255.0,
                a: 1.0,
            },
        ),
        PackageInfoChangeKind::Minor => (
            "color.ai.package_info.change.minor.bg",
            Color {
                r: 254.0 / 255.0,
                g: 249.0 / 255.0,
                b: 195.0 / 255.0,
                a: 1.0,
            },
        ),
        PackageInfoChangeKind::Patch => (
            "color.ai.package_info.change.patch.bg",
            Color {
                r: 220.0 / 255.0,
                g: 252.0 / 255.0,
                b: 231.0 / 255.0,
                a: 1.0,
            },
        ),
        PackageInfoChangeKind::Removed => (
            "color.ai.package_info.change.removed.bg",
            Color {
                r: 243.0 / 255.0,
                g: 244.0 / 255.0,
                b: 246.0 / 255.0,
                a: 1.0,
            },
        ),
    };
    ColorRef::Token {
        key,
        fallback: ColorFallback::Color(fallback),
    }
}

fn change_type_fg(change_type: PackageInfoChangeKind) -> ColorRef {
    let (key, fallback) = match change_type {
        PackageInfoChangeKind::Added => (
            "color.ai.package_info.change.added.fg",
            Color {
                r: 29.0 / 255.0,
                g: 78.0 / 255.0,
                b: 216.0 / 255.0,
                a: 1.0,
            },
        ),
        PackageInfoChangeKind::Major => (
            "color.ai.package_info.change.major.fg",
            Color {
                r: 185.0 / 255.0,
                g: 28.0 / 255.0,
                b: 28.0 / 255.0,
                a: 1.0,
            },
        ),
        PackageInfoChangeKind::Minor => (
            "color.ai.package_info.change.minor.fg",
            Color {
                r: 161.0 / 255.0,
                g: 98.0 / 255.0,
                b: 7.0 / 255.0,
                a: 1.0,
            },
        ),
        PackageInfoChangeKind::Patch => (
            "color.ai.package_info.change.patch.fg",
            Color {
                r: 21.0 / 255.0,
                g: 128.0 / 255.0,
                b: 61.0 / 255.0,
                a: 1.0,
            },
        ),
        PackageInfoChangeKind::Removed => (
            "color.ai.package_info.change.removed.fg",
            Color {
                r: 55.0 / 255.0,
                g: 65.0 / 255.0,
                b: 81.0 / 255.0,
                a: 1.0,
            },
        ),
    };
    ColorRef::Token {
        key,
        fallback: ColorFallback::Color(fallback),
    }
}

/// Root surface aligned with AI Elements `PackageInfo`.
#[derive(Clone)]
pub struct PackageInfo {
    name: Arc<str>,
    current_version: Option<Arc<str>>,
    new_version: Option<Arc<str>>,
    change_type: Option<PackageInfoChangeKind>,
    test_id_root: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for PackageInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackageInfo")
            .field("name", &self.name)
            .field("current_version", &self.current_version.as_deref())
            .field("new_version", &self.new_version.as_deref())
            .field("change_type", &self.change_type)
            .field("test_id_root", &self.test_id_root.as_deref())
            .finish()
    }
}

impl PackageInfo {
    pub fn new(name: impl Into<Arc<str>>) -> Self {
        Self {
            name: name.into(),
            current_version: None,
            new_version: None,
            change_type: None,
            test_id_root: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn current_version(mut self, version: impl Into<Arc<str>>) -> Self {
        self.current_version = Some(version.into());
        self
    }

    pub fn new_version(mut self, version: impl Into<Arc<str>>) -> Self {
        self.new_version = Some(version.into());
        self
    }

    pub fn change_type(mut self, change_type: PackageInfoChangeKind) -> Self {
        self.change_type = Some(change_type);
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
        children: impl FnOnce(&mut ElementContext<'_, H>, PackageInfoController) -> Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let controller = PackageInfoController {
            name: self.name.clone(),
            current_version: self.current_version.clone(),
            new_version: self.new_version.clone(),
            change_type: self.change_type,
        };

        let chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Token {
                key: "background",
                fallback: ColorFallback::Color(theme.color_required("background")),
            })
            .p(Space::N4)
            .merge(self.chrome);
        let layout = LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .merge(self.layout);

        let test_id_root = self.test_id_root.clone();

        let root = cx.container(
            decl_style::container_props(&theme, chrome, layout),
            move |cx| {
                cx.with_state(PackageInfoProviderState::default, |st| {
                    st.controller = Some(controller.clone());
                });
                children(cx, controller)
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_with_children(cx, move |cx, controller| {
            let header = PackageInfoHeader::new()
                .children([
                    PackageInfoName::new().into_element(cx),
                    PackageInfoChangeType::new().into_element(cx),
                ])
                .into_element(cx);

            let mut out = vec![header];
            if controller.current_version.is_some() || controller.new_version.is_some() {
                out.push(PackageInfoVersion::new().into_element(cx));
            }
            out
        })
    }
}

#[derive(Clone, Default)]
pub struct PackageInfoHeader {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl PackageInfoHeader {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .items_center()
                .justify(Justify::Between)
                .gap(Space::N2),
            move |_cx| self.children,
        );

        let theme = Theme::global(&*cx.app).clone();
        cx.container(
            decl_style::container_props(&theme, self.chrome, self.layout),
            move |_cx| vec![row],
        )
    }
}

#[derive(Clone)]
pub struct PackageInfoName {
    label: Option<Arc<str>>,
    icon: IconId,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl Default for PackageInfoName {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageInfoName {
    pub fn new() -> Self {
        Self {
            label: None,
            icon: IconId::new_static("lucide.package"),
            layout: LayoutRefinement::default().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = icon;
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = muted_fg(&theme);

        let label = self
            .label
            .or_else(|| use_package_info_controller(cx).map(|c| c.name));
        let Some(label) = label else {
            return cx.text("");
        };

        let icon = decl_icon::icon_with(cx, self.icon, Some(Px(16.0)), Some(ColorRef::Color(fg)));
        let text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: label,
            style: Some(monospace_style(
                &theme,
                theme.metric_required("component.text.sm_px"),
                FontWeight::MEDIUM,
            )),
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        let row = stack::hstack(
            cx,
            stack::HStackProps::default().items_center().gap(Space::N2),
            move |_cx| vec![icon, text],
        );

        cx.container(
            decl_style::container_props(&theme, self.chrome, self.layout),
            move |_cx| vec![row],
        )
    }
}

#[derive(Clone, Default)]
pub struct PackageInfoChangeType {
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl PackageInfoChangeType {
    pub fn new() -> Self {
        Self {
            test_id: None,
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
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

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_package_info_controller(cx) else {
            return cx.text("");
        };
        let Some(change_type) = controller.change_type else {
            return cx.text("");
        };

        let icon = decl_icon::icon_with(
            cx,
            change_type.icon(),
            Some(Px(12.0)),
            Some(change_type_fg(change_type)),
        );
        let text = cx.text(change_type.label());

        let badge = Badge::new(change_type.label())
            .variant(BadgeVariant::Secondary)
            .children([icon, text])
            .refine_style(
                ChromeRefinement::default()
                    .bg(change_type_bg(change_type))
                    .text_color(change_type_fg(change_type))
                    .merge(self.chrome),
            )
            .refine_layout(self.layout)
            .into_element(cx);

        let Some(test_id) = self.test_id else {
            return badge;
        };
        badge.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

#[derive(Clone, Default)]
pub struct PackageInfoVersion {
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl PackageInfoVersion {
    pub fn new() -> Self {
        Self {
            test_id: None,
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
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

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_package_info_controller(cx) else {
            return cx.text("");
        };
        let current = controller.current_version;
        let new = controller.new_version;
        if current.is_none() && new.is_none() {
            return cx.text("");
        }

        let theme = Theme::global(&*cx.app).clone();

        let has_current = current.is_some();
        let has_new = new.is_some();

        let mut parts: Vec<AnyElement> = Vec::new();
        if let Some(current) = current.as_ref() {
            parts.push(cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: current.clone(),
                style: Some(monospace_style(
                    &theme,
                    theme.metric_required("component.text.sm_px"),
                    FontWeight::NORMAL,
                )),
                color: Some(muted_fg(&theme)),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            }));
        }
        if has_current && has_new {
            parts.push(decl_icon::icon_with(
                cx,
                fret_icons::ids::ui::ARROW_RIGHT,
                Some(Px(12.0)),
                Some(ColorRef::Color(muted_fg(&theme))),
            ));
        }
        if let Some(new) = new.as_ref() {
            parts.push(cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: new.clone(),
                style: Some(monospace_style(
                    &theme,
                    theme.metric_required("component.text.sm_px"),
                    FontWeight::MEDIUM,
                )),
                color: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            }));
        }

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .items_center()
                .gap(Space::N2),
            move |_cx| parts,
        );

        let chrome = ChromeRefinement::default()
            .text_color(ColorRef::Color(muted_fg(&theme)))
            .merge(self.chrome);
        let layout = LayoutRefinement::default().w_full().min_w_0().mt(Space::N2);

        let container = cx.container(
            decl_style::container_props(&theme, chrome, layout),
            move |_cx| vec![row],
        );

        let Some(test_id) = self.test_id else {
            return container;
        };
        container.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

#[derive(Clone, Default)]
pub struct PackageInfoDescription {
    text: Arc<str>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl PackageInfoDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: theme.metric_required("component.text.sm_px"),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(theme.metric_required("component.text.sm_line_height")),
                letter_spacing_em: None,
            }),
            color: Some(muted_fg(&theme)),
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
        });

        let chrome = ChromeRefinement::default().merge(self.chrome);
        let layout = LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .mt(Space::N2)
            .merge(self.layout);
        cx.container(
            decl_style::container_props(&theme, chrome, layout),
            move |_cx| vec![text],
        )
    }
}

#[derive(Clone, Default)]
pub struct PackageInfoContent {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl PackageInfoContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let chrome = ChromeRefinement::default().pt(Space::N3).merge(self.chrome);
        let layout = LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .mt(Space::N3)
            .merge(self.layout);
        let mut props = decl_style::container_props(&theme, chrome, layout);
        props.background = None;
        props.border = Edges {
            left: Px(0.0),
            right: Px(0.0),
            top: Px(1.0),
            bottom: Px(0.0),
        };
        props.border_color = Some(theme.color_required("border"));

        cx.container(props, move |_cx| self.children)
    }
}

#[derive(Clone, Default)]
pub struct PackageInfoDependencies {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl PackageInfoDependencies {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let label = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: Arc::<str>::from("Dependencies"),
            style: Some(TextStyle {
                font: FontId::default(),
                size: theme.metric_required("component.text.xs_px"),
                weight: FontWeight::MEDIUM,
                slant: Default::default(),
                line_height: Some(theme.metric_required("component.text.xs_line_height")),
                letter_spacing_em: Some(0.08),
            }),
            color: Some(muted_fg(&theme)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        let list = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N1),
            move |_cx| self.children,
        );

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(self.layout)
                .gap(Space::N2),
            move |_cx| vec![label, list],
        );

        let chrome = ChromeRefinement::default().merge(self.chrome);
        let layout = LayoutRefinement::default().w_full().min_w_0();
        cx.container(
            decl_style::container_props(&theme, chrome, layout),
            move |_cx| vec![body],
        )
    }
}

#[derive(Clone)]
pub struct PackageInfoDependency {
    name: Arc<str>,
    version: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl PackageInfoDependency {
    pub fn new(name: impl Into<Arc<str>>) -> Self {
        Self {
            name: name.into(),
            version: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn version(mut self, version: impl Into<Arc<str>>) -> Self {
        self.version = Some(version.into());
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let name = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.name,
            style: Some(monospace_style(
                &theme,
                theme.metric_required("component.text.sm_px"),
                FontWeight::NORMAL,
            )),
            color: Some(muted_fg(&theme)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        let mut items = vec![name];
        if let Some(version) = self.version {
            items.push(cx.text_props(TextProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Auto,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: version,
                style: Some(monospace_style(
                    &theme,
                    theme.metric_required("component.text.xs_px"),
                    FontWeight::NORMAL,
                )),
                color: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            }));
        }

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .items_center()
                .justify(Justify::Between),
            move |_cx| items,
        );

        cx.container(
            decl_style::container_props(&theme, self.chrome, self.layout),
            move |_cx| vec![row],
        )
    }
}
