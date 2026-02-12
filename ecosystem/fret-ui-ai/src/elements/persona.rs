use std::sync::Arc;

use fret_core::{Color, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_icons::IconId;
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, LayoutRefinement, Radius, Space,
};
use fret_ui_shadcn::Spinner;

fn text_xs_style(theme: &Theme, weight: FontWeight) -> TextStyle {
    let size = theme
        .metric_by_key("component.text.xs_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.text.xs_line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));
    TextStyle {
        font: Default::default(),
        size,
        weight,
        slant: Default::default(),
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn token_color_with_alpha(
    theme: &Theme,
    key: &'static str,
    fallback_key: &'static str,
    alpha: f32,
) -> Color {
    let base = theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key(fallback_key))
        .unwrap_or_else(|| theme.color_required("foreground"));
    let alpha = alpha.clamp(0.0, 1.0);
    Color {
        r: base.r,
        g: base.g,
        b: base.b,
        a: base.a * alpha,
    }
}

/// Persona animation state taxonomy aligned with AI Elements `persona.tsx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersonaState {
    Idle,
    Listening,
    Thinking,
    Speaking,
    Asleep,
}

impl PersonaState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Listening => "listening",
            Self::Thinking => "thinking",
            Self::Speaking => "speaking",
            Self::Asleep => "asleep",
        }
    }
}

/// Persona visual variant keys aligned with AI Elements `persona.tsx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersonaVariant {
    Command,
    Glint,
    Halo,
    Mana,
    Obsidian,
    Opal,
}

impl PersonaVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Glint => "glint",
            Self::Halo => "halo",
            Self::Mana => "mana",
            Self::Obsidian => "obsidian",
            Self::Opal => "opal",
        }
    }
}

/// Persona surface (UI-only placeholder).
///
/// Upstream AI Elements renders a Rive animation (`@rive-app/react-webgl2`) loaded from a remote
/// `.riv` asset. In Fret we keep IO and platform backends **app-owned**, so this element renders
/// a lightweight placeholder that preserves:
///
/// - state taxonomy (`PersonaState`)
/// - variant taxonomy (`PersonaVariant`)
/// - stable selectors for diag scripts (`test_id`)
#[derive(Clone)]
pub struct Persona {
    state: PersonaState,
    variant: PersonaVariant,
    size: Px,
    show_label: bool,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for Persona {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Persona")
            .field("state", &self.state)
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("show_label", &self.show_label)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl Persona {
    pub fn new(state: PersonaState) -> Self {
        Self {
            state,
            variant: PersonaVariant::Obsidian,
            size: Px(96.0),
            show_label: false,
            test_id: None,
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn variant(mut self, variant: PersonaVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: Px) -> Self {
        self.size = Px(size.0.max(0.0));
        self
    }

    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
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

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let muted = muted_fg(&theme);
        let bg = token_color_with_alpha(&theme, "muted", "accent", 0.35);

        let icon = match self.state {
            PersonaState::Idle => decl_icon::icon_with(
                cx,
                IconId::new_static("lucide.sparkles"),
                Some(Px(20.0)),
                Some(ColorRef::Color(muted)),
            ),
            PersonaState::Listening => decl_icon::icon_with(
                cx,
                IconId::new_static("lucide.mic"),
                Some(Px(20.0)),
                Some(ColorRef::Color(muted)),
            ),
            PersonaState::Thinking => Spinner::new().into_element(cx),
            PersonaState::Speaking => decl_icon::icon_with(
                cx,
                IconId::new_static("lucide.volume-2"),
                Some(Px(20.0)),
                Some(ColorRef::Color(muted)),
            ),
            PersonaState::Asleep => decl_icon::icon_with(
                cx,
                IconId::new_static("lucide.moon"),
                Some(Px(20.0)),
                Some(ColorRef::Color(muted)),
            ),
        };

        let label = if self.show_label {
            Some(cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: Arc::from(format!(
                    "{} • {}",
                    self.variant.as_str(),
                    self.state.as_str()
                )),
                style: Some(text_xs_style(&theme, FontWeight::MEDIUM)),
                color: Some(muted),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
            }))
        } else {
            None
        };

        let size = self.size;
        let orb = {
            let centered = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .items(Items::Center)
                    .justify_center(),
                move |_cx| [icon],
            );

            let props = decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .rounded(Radius::Full)
                    .bg(ColorRef::Color(bg))
                    .border_1()
                    .border_color(ColorRef::Token {
                        key: "border",
                        fallback: ColorFallback::ThemePanelBorder,
                    })
                    .merge(self.chrome),
                LayoutRefinement::default().w_px(size).h_px(size),
            );
            cx.container(props, move |_cx| [centered])
        };

        let body: AnyElement = if let Some(label) = label {
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N2)
                    .items(Items::Center)
                    .layout(LayoutRefinement::default().min_w_0().merge(self.layout)),
                move |_cx| vec![orb, label],
            )
        } else {
            let props = decl_style::container_props(
                &theme,
                ChromeRefinement::default(),
                LayoutRefinement::default().merge(self.layout),
            );
            cx.container(props, move |_cx| [orb])
        };

        let Some(test_id) = self.test_id else {
            return body;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Generic,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| [body],
        )
    }
}
