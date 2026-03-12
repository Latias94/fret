//! AI Elements-aligned `PackageInfo` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/package-info.tsx`.

use std::sync::Arc;

use fret_core::{
    Color, ColorScheme, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle,
    TextWrap,
};
use fret_icons::IconId;
use fret_ui::element::{
    AnyElement, LayoutStyle, Length, SemanticsDecoration, SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, Radius, Space,
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
    fn display_label(self) -> &'static str {
        match self {
            Self::Major => "Major",
            Self::Minor => "Minor",
            Self::Patch => "Patch",
            Self::Added => "Added",
            Self::Removed => "Removed",
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

pub fn use_package_info_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<PackageInfoController> {
    cx.provided::<PackageInfoController>().cloned()
}

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"))
}

fn monospace_style(theme: &Theme, size: Px, weight: FontWeight) -> TextStyle {
    typography::as_control_text(TextStyle {
        font: FontId::monospace(),
        size,
        weight,
        slant: Default::default(),
        line_height: Some(theme.metric_token("metric.font.mono_line_height")),
        letter_spacing_em: None,
        ..Default::default()
    })
}

fn with_alpha(mut c: Color, alpha: f32) -> Color {
    c.a = alpha.clamp(0.0, 1.0);
    c
}

fn is_dark(theme: &Theme) -> bool {
    matches!(theme.color_scheme, Some(ColorScheme::Dark))
}

fn change_type_bg(theme: &Theme, change_type: PackageInfoChangeKind) -> ColorRef {
    // Apps can override via theme tokens. Defaults mirror AI Elements:
    // - light: `bg-*-100`
    // - dark:  `bg-*-900/30`
    let dark = is_dark(theme);
    let (key, fallback) = match change_type {
        PackageInfoChangeKind::Added => (
            "color.ai.package_info.change.added.bg",
            if dark {
                with_alpha(
                    // Tailwind blue-900 (#1e3a8a) / 30%.
                    fret_ui_kit::colors::linear_from_hex_rgb(0x1E_3A_8A),
                    0.30,
                )
            } else {
                // Tailwind blue-100 (#dbeafe).
                fret_ui_kit::colors::linear_from_hex_rgb(0xDB_EA_FE)
            },
        ),
        PackageInfoChangeKind::Major => (
            "color.ai.package_info.change.major.bg",
            if dark {
                with_alpha(
                    // Tailwind red-900 (#7f1d1d) / 30%.
                    fret_ui_kit::colors::linear_from_hex_rgb(0x7F_1D_1D),
                    0.30,
                )
            } else {
                // Tailwind red-100 (#fee2e2).
                fret_ui_kit::colors::linear_from_hex_rgb(0xFE_E2_E2)
            },
        ),
        PackageInfoChangeKind::Minor => (
            "color.ai.package_info.change.minor.bg",
            if dark {
                with_alpha(
                    // Tailwind yellow-900 (#713f12) / 30%.
                    fret_ui_kit::colors::linear_from_hex_rgb(0x71_3F_12),
                    0.30,
                )
            } else {
                // Tailwind yellow-100 (#fef9c3).
                fret_ui_kit::colors::linear_from_hex_rgb(0xFE_F9_C3)
            },
        ),
        PackageInfoChangeKind::Patch => (
            "color.ai.package_info.change.patch.bg",
            if dark {
                with_alpha(
                    // Tailwind green-900 (#14532d) / 30%.
                    fret_ui_kit::colors::linear_from_hex_rgb(0x14_53_2D),
                    0.30,
                )
            } else {
                // Tailwind green-100 (#dcfce7).
                fret_ui_kit::colors::linear_from_hex_rgb(0xDC_FC_E7)
            },
        ),
        PackageInfoChangeKind::Removed => (
            "color.ai.package_info.change.removed.bg",
            if dark {
                with_alpha(
                    // Tailwind gray-900 (#111827) / 30%.
                    fret_ui_kit::colors::linear_from_hex_rgb(0x11_18_27),
                    0.30,
                )
            } else {
                // Tailwind gray-100 (#f3f4f6).
                fret_ui_kit::colors::linear_from_hex_rgb(0xF3_F4_F6)
            },
        ),
    };
    ColorRef::Token {
        key,
        fallback: ColorFallback::Color(fallback),
    }
}

fn change_type_fg(theme: &Theme, change_type: PackageInfoChangeKind) -> ColorRef {
    // Defaults mirror AI Elements:
    // - light: `text-*-700`
    // - dark:  `text-*-400`
    let dark = is_dark(theme);
    let (key, fallback) = match change_type {
        PackageInfoChangeKind::Added => (
            "color.ai.package_info.change.added.fg",
            if dark {
                // Tailwind blue-400 (#60a5fa).
                fret_ui_kit::colors::linear_from_hex_rgb(0x60_A5_FA)
            } else {
                // Tailwind blue-700 (#1d4ed8).
                fret_ui_kit::colors::linear_from_hex_rgb(0x1D_4E_D8)
            },
        ),
        PackageInfoChangeKind::Major => (
            "color.ai.package_info.change.major.fg",
            if dark {
                // Tailwind red-400 (#f87171).
                fret_ui_kit::colors::linear_from_hex_rgb(0xF8_71_71)
            } else {
                // Tailwind red-700 (#b91c1c).
                fret_ui_kit::colors::linear_from_hex_rgb(0xB9_1C_1C)
            },
        ),
        PackageInfoChangeKind::Minor => (
            "color.ai.package_info.change.minor.fg",
            if dark {
                // Tailwind yellow-400 (#facc15).
                fret_ui_kit::colors::linear_from_hex_rgb(0xFA_CC_15)
            } else {
                // Tailwind yellow-700 (#a16207).
                fret_ui_kit::colors::linear_from_hex_rgb(0xA1_62_07)
            },
        ),
        PackageInfoChangeKind::Patch => (
            "color.ai.package_info.change.patch.fg",
            if dark {
                // Tailwind green-400 (#4ade80).
                fret_ui_kit::colors::linear_from_hex_rgb(0x4A_DE_80)
            } else {
                // Tailwind green-700 (#15803d).
                fret_ui_kit::colors::linear_from_hex_rgb(0x15_80_3D)
            },
        ),
        PackageInfoChangeKind::Removed => (
            "color.ai.package_info.change.removed.fg",
            if dark {
                // Tailwind gray-400 (#9ca3af).
                fret_ui_kit::colors::linear_from_hex_rgb(0x9C_A3_AF)
            } else {
                // Tailwind gray-700 (#374151).
                fret_ui_kit::colors::linear_from_hex_rgb(0x37_41_51)
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
                fallback: ColorFallback::Color(theme.color_token("background")),
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
            move |cx| cx.provide(controller.clone(), |cx| children(cx, controller.clone())),
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
            let mut header_children = vec![PackageInfoName::new().into_element(cx)];
            if controller.change_type.is_some() {
                header_children.push(PackageInfoChangeType::new().into_element(cx));
            }
            let header = PackageInfoHeader::new()
                .children(header_children)
                .into_element(cx);

            let mut out = vec![header];
            if controller.current_version.is_some() || controller.new_version.is_some() {
                out.push(PackageInfoVersion::new().into_element(cx));
            }
            out
        })
    }
}

#[derive(Default)]
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
        let row = ui::h_row(move |_cx| self.children)
            .items(Items::Center)
            .justify(Justify::Between)
            .gap(Space::N2)
            .into_element(cx);

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
                theme.metric_token("component.text.sm_px"),
                FontWeight::MEDIUM,
            )),
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        });

        let row = ui::h_row(move |_cx| vec![icon, text])
            .items(Items::Center)
            .gap(Space::N2)
            .into_element(cx);

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

        let theme = Theme::global(&*cx.app).clone();
        let badge = Badge::new(change_type.display_label())
            .variant(BadgeVariant::Secondary)
            .leading_icon(change_type.icon())
            .refine_style(
                ChromeRefinement::default()
                    .bg(change_type_bg(&theme, change_type))
                    .text_color(change_type_fg(&theme, change_type))
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
                    theme.metric_token("component.text.sm_px"),
                    FontWeight::NORMAL,
                )),
                color: Some(muted_fg(&theme)),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
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
                    theme.metric_token("component.text.sm_px"),
                    FontWeight::MEDIUM,
                )),
                color: Some(theme.color_token("foreground")),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            }));
        }

        let row = ui::h_row(move |_cx| parts)
            .items(Items::Center)
            .gap(Space::N2)
            .into_element(cx);

        let chrome = ChromeRefinement::default().merge(self.chrome);
        let layout = LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .mt(Space::N2)
            .merge(self.layout);

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
            style: None,
            color: None,
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        });

        let chrome = ChromeRefinement::default().merge(self.chrome);
        let layout = LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .mt(Space::N2)
            .merge(self.layout);
        typography::scope_description_text_with_fallbacks(
            cx.container(
                decl_style::container_props(&theme, chrome, layout),
                move |_cx| vec![text],
            ),
            &theme,
            "component.package_info.description",
            Some("component.text.sm_px"),
            Some("component.text.sm_line_height"),
        )
    }
}

#[derive(Default)]
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
        props.border_color = Some(theme.color_token("border"));

        cx.container(props, move |_cx| self.children)
    }
}

#[derive(Default)]
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
            text: Arc::<str>::from("DEPENDENCIES"),
            style: Some(typography::as_control_text(TextStyle {
                font: FontId::default(),
                size: theme.metric_token("component.text.xs_px"),
                weight: FontWeight::MEDIUM,
                slant: Default::default(),
                line_height: Some(theme.metric_token("component.text.xs_line_height")),
                letter_spacing_em: Some(0.08),
                ..Default::default()
            })),
            color: Some(muted_fg(&theme)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        });

        let list = ui::v_flex(move |_cx| self.children)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N1)
            .into_element(cx);

        let body = ui::v_flex(move |_cx| vec![label, list])
            .layout(self.layout)
            .gap(Space::N2)
            .into_element(cx);

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
                theme.metric_token("component.text.sm_px"),
                FontWeight::NORMAL,
            )),
            color: Some(muted_fg(&theme)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
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
                    theme.metric_token("component.text.xs_px"),
                    FontWeight::NORMAL,
                )),
                color: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            }));
        }

        let row = ui::h_row(move |_cx| items)
            .items(Items::Center)
            .justify(Justify::Between)
            .into_element(cx);

        cx.container(
            decl_style::container_props(&theme, self.chrome, self.layout),
            move |_cx| vec![row],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::{AnyElement, ElementKind};
    use fret_ui::{Theme, ThemeConfig};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(720.0), Px(480.0)),
        )
    }

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
    fn package_info_root_provides_controller_to_default_parts() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                PackageInfo::new("serde")
                    .current_version("1.0.0")
                    .new_version("2.0.0")
                    .change_type(PackageInfoChangeKind::Major)
                    .into_element(cx)
            });

        assert!(find_text_by_content(&element, "serde").is_some());
        assert!(find_text_by_content(&element, "1.0.0").is_some());
        assert!(find_text_by_content(&element, "2.0.0").is_some());
        assert!(find_text_by_content(&element, "Major").is_some());
    }

    #[test]
    fn package_info_description_scopes_inherited_description_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "PackageInfo Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("component.text.sm_px".to_string(), 13.0),
                    ("component.text.sm_line_height".to_string(), 18.0),
                ]),
                colors: std::collections::HashMap::from([(
                    "muted-foreground".to_string(),
                    "#667788".to_string(),
                )]),
                ..ThemeConfig::default()
            });
        });

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                PackageInfoDescription::new("Package summary").into_element(cx)
            });

        let ElementKind::Container(_) = &element.kind else {
            panic!("expected PackageInfoDescription to build a Container root");
        };
        let child = element.children.first().expect("expected text child");
        let ElementKind::Text(props) = &child.kind else {
            panic!("expected PackageInfoDescription child to be Text");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.wrap, TextWrap::Grapheme);

        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_foreground,
            Some(typography::muted_foreground_color(&theme))
        );
        assert_eq!(
            element.inherited_text_style,
            Some(typography::description_text_refinement_with_fallbacks(
                &theme,
                "component.package_info.description",
                Some("component.text.sm_px"),
                Some("component.text.sm_line_height"),
            ))
        );
    }
}
