use std::sync::Arc;

use fret_core::{
    Color, ColorScheme, FontWeight, Px, SemanticsRole, TextAlign, TextOverflow, TextWrap,
};
use fret_icons::IconId;
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsDecoration, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::facade::Spinner;

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn with_alpha(mut color: Color, alpha: f32) -> Color {
    color.a = alpha.clamp(0.0, 1.0);
    color
}

fn hex_rgb(rgb: u32) -> Color {
    fret_ui_kit::colors::linear_from_hex_rgb(rgb)
}

fn scheme_is_dark(theme: &Theme) -> bool {
    theme.color_scheme == Some(ColorScheme::Dark)
}

#[derive(Debug, Clone, Copy)]
struct PersonaPalette {
    shell_bg: Color,
    inner_bg: Color,
    accent_primary: Color,
    accent_secondary: Color,
    state_ring: Color,
    core_bg: Color,
    core_border: Color,
    indicator_fg: Color,
    label_fg: Color,
}

fn persona_palette(theme: &Theme, variant: PersonaVariant, state: PersonaState) -> PersonaPalette {
    let dark = scheme_is_dark(theme);
    let (base, glow) = match variant {
        PersonaVariant::Obsidian => (hex_rgb(0x38_BD_F8), hex_rgb(0x81_8C_F8)),
        PersonaVariant::Mana => (hex_rgb(0x8B_5C_F6), hex_rgb(0xC0_84_FC)),
        PersonaVariant::Opal => (hex_rgb(0x94_A3_B8), hex_rgb(0xE2_E8_F0)),
        PersonaVariant::Halo => (hex_rgb(0xF5_9E_0B), hex_rgb(0xFB_71_8C)),
        PersonaVariant::Glint => (hex_rgb(0x06_B6_D4), hex_rgb(0x2D_D4_BF)),
        PersonaVariant::Command => (hex_rgb(0x10_B9_81), hex_rgb(0x22_C5_5E)),
    };
    let ring_mul = match state {
        PersonaState::Idle => 0.28,
        PersonaState::Listening => 0.56,
        PersonaState::Thinking => 0.40,
        PersonaState::Speaking => 0.62,
        PersonaState::Asleep => 0.18,
    };
    let foreground = theme
        .color_by_key("foreground")
        .unwrap_or_else(|| theme.color_required("foreground"));
    let background = theme
        .color_by_key("background")
        .unwrap_or_else(|| theme.color_required("background"));

    PersonaPalette {
        shell_bg: with_alpha(base, if dark { 0.22 } else { 0.12 }),
        inner_bg: with_alpha(glow, if dark { 0.34 } else { 0.20 }),
        accent_primary: with_alpha(base, if dark { 0.82 } else { 0.56 }),
        accent_secondary: with_alpha(glow, if dark { 0.68 } else { 0.42 }),
        state_ring: with_alpha(base, ring_mul),
        core_bg: with_alpha(background, if dark { 0.72 } else { 0.84 }),
        core_border: with_alpha(base, if dark { 0.56 } else { 0.30 }),
        indicator_fg: foreground,
        label_fg: muted_fg(theme),
    }
}

fn circle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    color: Color,
    layout: LayoutRefinement,
) -> AnyElement {
    let props = decl_style::container_props(
        theme,
        ChromeRefinement::default()
            .rounded(Radius::Full)
            .bg(ColorRef::Color(color)),
        layout,
    );
    cx.container(props, |_cx| std::iter::empty::<AnyElement>())
}

fn circle_border<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    color: Color,
    layout: LayoutRefinement,
) -> AnyElement {
    let props = decl_style::container_props(
        theme,
        ChromeRefinement::default()
            .rounded(Radius::Full)
            .border_1()
            .border_color(ColorRef::Color(color)),
        layout,
    );
    cx.container(props, |_cx| std::iter::empty::<AnyElement>())
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

    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Listening => "Listening",
            Self::Thinking => "Thinking",
            Self::Speaking => "Speaking",
            Self::Asleep => "Asleep",
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

    pub fn label(self) -> &'static str {
        match self {
            Self::Command => "Command",
            Self::Glint => "Glint",
            Self::Halo => "Halo",
            Self::Mana => "Mana",
            Self::Obsidian => "Obsidian",
            Self::Opal => "Opal",
        }
    }
}

/// Controller passed to the optional custom visual slot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PersonaController {
    pub state: PersonaState,
    pub variant: PersonaVariant,
    pub size: Px,
}

/// Persona surface.
///
/// Upstream AI Elements renders a Rive animation (`@rive-app/react-webgl2`) loaded from a remote
/// `.riv` asset. In Fret we keep IO and backend ownership app-side, so this element provides a
/// docs-aligned, variant-aware placeholder surface with an optional custom visual slot.
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
            size: Px(64.0),
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_with_children(cx, |_cx, _controller| Vec::new())
    }

    pub fn into_element_with_children<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>, PersonaController) -> Vec<AnyElement>,
    ) -> AnyElement {
        let controller = PersonaController {
            state: self.state,
            variant: self.variant,
            size: self.size,
        };
        let custom_children = children(cx, controller);
        self.render_with_children(cx, custom_children)
    }

    fn render_with_children<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        custom_children: Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let palette = persona_palette(&theme, self.variant, self.state);

        let indicator_size = Px((self.size.0 * 0.24).clamp(16.0, 24.0));
        let core_size = Px((self.size.0 * 0.46).clamp(32.0, 56.0));
        let core_inset = Px(((self.size.0 - core_size.0) * 0.5).max(0.0));

        let default_indicator = match self.state {
            PersonaState::Idle => decl_icon::icon_with(
                cx,
                IconId::new_static("lucide.sparkles"),
                Some(indicator_size),
                Some(ColorRef::Color(palette.indicator_fg)),
            ),
            PersonaState::Listening => decl_icon::icon_with(
                cx,
                IconId::new_static("lucide.mic"),
                Some(indicator_size),
                Some(ColorRef::Color(palette.indicator_fg)),
            ),
            PersonaState::Thinking => Spinner::new()
                .color(ColorRef::Color(palette.indicator_fg))
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(indicator_size)
                        .h_px(indicator_size),
                )
                .into_element(cx),
            PersonaState::Speaking => decl_icon::icon_with(
                cx,
                IconId::new_static("lucide.volume-2"),
                Some(indicator_size),
                Some(ColorRef::Color(palette.indicator_fg)),
            ),
            PersonaState::Asleep => decl_icon::icon_with(
                cx,
                IconId::new_static("lucide.moon-star"),
                Some(indicator_size),
                Some(ColorRef::Color(palette.indicator_fg)),
            ),
        };

        let center_content = if custom_children.is_empty() {
            default_indicator
        } else if custom_children.len() == 1 {
            custom_children.into_iter().next().expect("checked length")
        } else {
            ui::v_stack(move |_cx| custom_children)
                .gap(Space::N1)
                .items(Items::Center)
                .into_element(cx)
        };

        let center_wrap = {
            let centered = ui::h_row(move |_cx| vec![center_content])
                .layout(LayoutRefinement::default().w_full().h_full())
                .items(Items::Center)
                .justify(Justify::Center)
                .into_element(cx);
            let props = decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .rounded(Radius::Full)
                    .bg(ColorRef::Color(palette.core_bg))
                    .border_1()
                    .border_color(ColorRef::Color(palette.core_border)),
                LayoutRefinement::default()
                    .absolute()
                    .top_px(core_inset)
                    .left_px(core_inset)
                    .w_px(core_size)
                    .h_px(core_size),
            );
            cx.container(props, move |_cx| [centered])
        };

        let inner = circle(
            cx,
            &theme,
            palette.inner_bg,
            LayoutRefinement::default()
                .absolute()
                .inset_px(Px(self.size.0 * 0.12)),
        );
        let accent_top = circle(
            cx,
            &theme,
            palette.accent_primary,
            LayoutRefinement::default()
                .absolute()
                .top_px(Px(self.size.0 * 0.14))
                .right_px(Px(self.size.0 * 0.12))
                .w_px(Px(self.size.0 * 0.18))
                .h_px(Px(self.size.0 * 0.18)),
        );
        let accent_bottom = circle(
            cx,
            &theme,
            palette.accent_secondary,
            LayoutRefinement::default()
                .absolute()
                .bottom_px(Px(self.size.0 * 0.14))
                .left_px(Px(self.size.0 * 0.12))
                .w_px(Px(self.size.0 * 0.14))
                .h_px(Px(self.size.0 * 0.14)),
        );
        let state_ring = match self.state {
            PersonaState::Listening | PersonaState::Speaking | PersonaState::Thinking => {
                Some(circle_border(
                    cx,
                    &theme,
                    palette.state_ring,
                    LayoutRefinement::default()
                        .absolute()
                        .inset_px(Px(self.size.0 * 0.06)),
                ))
            }
            PersonaState::Idle | PersonaState::Asleep => None,
        };

        let orb = {
            let mut children = vec![inner, accent_top, accent_bottom];
            if let Some(state_ring) = state_ring {
                children.push(state_ring);
            }
            children.push(center_wrap);
            let props = decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .rounded(Radius::Full)
                    .bg(ColorRef::Color(palette.shell_bg))
                    .border_1()
                    .border_color(ColorRef::Color(palette.core_border))
                    .merge(self.chrome),
                LayoutRefinement::default()
                    .w_px(self.size)
                    .h_px(self.size)
                    .flex_shrink_0(),
            );
            cx.container(props, move |_cx| children)
        };

        let body = if self.show_label {
            let label = cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: Arc::from(format!("{} / {}", self.variant.label(), self.state.label())),
                style: Some(typography::preset_text_style_with_overrides(
                    &theme,
                    typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                    Some(FontWeight::MEDIUM),
                    None,
                )),
                color: Some(palette.label_fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
                align: TextAlign::Center,
                ink_overflow: fret_ui::element::TextInkOverflow::None,
            });
            ui::v_stack(move |_cx| vec![orb, label])
                .gap(Space::N2)
                .items(Items::Center)
                .layout(LayoutRefinement::default().min_w_0().merge(self.layout))
                .into_element(cx)
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
        body.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Generic)
                .test_id(test_id),
        )
    }
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
            Size::new(Px(400.0), Px(240.0)),
        )
    }

    fn contains_text(element: &AnyElement, needle: &str) -> bool {
        match &element.kind {
            ElementKind::Text(props) if props.text.as_ref() == needle => true,
            _ => element
                .children
                .iter()
                .any(|child| contains_text(child, needle)),
        }
    }

    fn find_text_by_content<'a>(element: &'a AnyElement, needle: &str) -> Option<&'a TextProps> {
        match &element.kind {
            ElementKind::Text(props) if props.text.as_ref() == needle => Some(props),
            _ => element
                .children
                .iter()
                .find_map(|child| find_text_by_content(child, needle)),
        }
    }

    #[test]
    fn persona_stamps_test_id_without_extra_semantics_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Persona::new(PersonaState::Listening)
                    .test_id("ui-ai-persona")
                    .into_element(cx)
            });

        assert_eq!(
            element.semantics_decoration.as_ref().and_then(|d| d.role),
            Some(SemanticsRole::Generic)
        );
        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|d| d.test_id.as_deref()),
            Some("ui-ai-persona")
        );
    }

    #[test]
    fn persona_custom_visual_slot_replaces_default_indicator() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Persona::new(PersonaState::Thinking)
                    .show_label(true)
                    .into_element_with_children(cx, |cx, _controller| vec![cx.text("Custom orb")])
            });

        assert!(contains_text(&element, "Custom orb"));
        assert!(contains_text(&element, "Obsidian / Thinking"));
    }

    #[test]
    fn persona_label_uses_shared_xs_typography_preset() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Persona::new(PersonaState::Speaking)
                    .variant(PersonaVariant::Halo)
                    .show_label(true)
                    .into_element(cx)
            });

        let theme = Theme::global(&app).clone();
        let label = find_text_by_content(&element, "Halo / Speaking").expect("persona label text");
        assert_eq!(
            label.style,
            Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                Some(FontWeight::MEDIUM),
                None,
            ))
        );
    }
}
