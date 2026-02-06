use std::sync::Arc;

use fret_core::{Color, Corners, Edges, FontId, FontWeight, Px, TextStyle};
use fret_ui::element::PressableState;
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, PositionStyle, PressableProps,
    SemanticsDecoration, SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Size as ComponentSize, ui};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NativeSelectSize {
    Sm,
    #[default]
    Default,
}

#[derive(Clone)]
pub struct NativeSelect {
    label: Arc<str>,
    a11y_label: Option<Arc<str>>,
    aria_invalid: bool,
    disabled: bool,
    size: NativeSelectSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for NativeSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeSelect")
            .field("label", &self.label.as_ref())
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("aria_invalid", &self.aria_invalid)
            .field("disabled", &self.disabled)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl NativeSelect {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            a11y_label: None,
            aria_invalid: false,
            disabled: false,
            size: NativeSelectSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(mut self, size: NativeSelectSize) -> Self {
        self.size = size;
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
        native_select(
            cx,
            self.label,
            self.a11y_label,
            self.aria_invalid,
            self.disabled,
            self.size,
            self.chrome,
            self.layout,
        )
    }
}

pub fn native_select<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    a11y_label: Option<Arc<str>>,
    aria_invalid: bool,
    disabled: bool,
    size: NativeSelectSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let resolved = resolve_input_chrome(
        &theme,
        ComponentSize::default(),
        &chrome,
        InputTokenKeys::none(),
    );

    let (h, py) = match size {
        NativeSelectSize::Sm => (Px(32.0), Px(4.0)),
        NativeSelectSize::Default => (Px(36.0), Px(8.0)),
    };

    let text_style = TextStyle {
        font: FontId::default(),
        size: resolved.text_px,
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(theme.metric_required("font.line_height")),
        letter_spacing_em: None,
    };

    let mut border_color = resolved.border_color;
    let mut focus_ring = decl_style::focus_ring(&theme, resolved.radius);
    if aria_invalid {
        border_color = theme.color_required("destructive");
        let ring_key = if theme.name.contains("/dark") {
            "destructive/40"
        } else {
            "destructive/20"
        };
        focus_ring.color = theme
            .color_by_key(ring_key)
            .or_else(|| theme.color_by_key("destructive/20"))
            .unwrap_or(border_color);
    }

    let layout = decl_style::layout_style(&theme, layout.relative().min_w_0());
    let mut layout = layout;
    layout.size = SizeStyle {
        height: Length::Px(h),
        ..layout.size
    };

    let icon_size = Px(16.0);
    let icon_right = Px(14.0);
    let icon_top = Px((h.0 - icon_size.0) * 0.5);

    let mut content = ui::text(cx, label)
        .text_size_px(text_style.size)
        .line_height_px(text_style.line_height.unwrap_or(text_style.size))
        .font_normal()
        .nowrap()
        .text_color(ColorRef::Color(resolved.text_color));

    content = content.overflow(fret_core::TextOverflow::Clip);

    let content = content.into_element(cx);

    let icon_color = alpha_mul(theme.color_required("muted-foreground"), 0.5);
    let icon = decl_icon::icon_with(
        cx,
        fret_icons::ids::ui::CHEVRON_DOWN,
        Some(icon_size),
        Some(ColorRef::Color(icon_color)),
    );
    let icon = cx.container(
        ContainerProps {
            layout: LayoutStyle {
                position: PositionStyle::Absolute,
                inset: InsetStyle {
                    left: None,
                    top: Some(icon_top),
                    right: Some(icon_right),
                    bottom: None,
                },
                size: SizeStyle {
                    width: Length::Px(icon_size),
                    height: Length::Px(icon_size),
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: Edges::all(Px(0.0)),
            background: None,
            shadow: None,
            border: Edges::all(Px(0.0)),
            border_color: None,
            corner_radii: Corners::all(Px(0.0)),
            ..Default::default()
        },
        move |_cx| vec![icon],
    );

    let pressable = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: !disabled,
            focusable: !disabled,
            focus_ring: Some(focus_ring),
            ..Default::default()
        },
        |_cx, _state: PressableState| Vec::new(),
    );

    let props = ContainerProps {
        layout,
        padding: Edges {
            left: resolved.padding.left,
            right: Px(36.0),
            top: py,
            bottom: py,
        },
        background: Some(resolved.background),
        shadow: Some(decl_style::shadow_xs(&theme, resolved.radius)),
        border: Edges::all(resolved.border_width),
        border_color: Some(border_color),
        corner_radii: Corners::all(resolved.radius),
        ..Default::default()
    };

    let out = cx.container(props, move |_cx| vec![content, icon, pressable]);
    let out = out.attach_semantics(SemanticsDecoration {
        role: Some(fret_core::SemanticsRole::ComboBox),
        label: a11y_label.clone(),
        disabled: disabled.then_some(true),
        ..Default::default()
    });
    if disabled {
        cx.opacity(0.5, move |_cx| vec![out])
    } else {
        out
    }
}
