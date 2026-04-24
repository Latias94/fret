use fret_core::{
    Color, Corners, Edges, Px, TextLineHeightPolicy, TextStyle, TextVerticalPlacement,
};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    ContainerProps, CrossAlign, LayoutStyle, Length, MainAlign, PressableState, RowProps,
    SizeStyle, TextInputProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{TextInputStyle, ThemeSnapshot, UiHost};

pub(super) const PORTAL_BUTTON_STACK_GAP: Px = Px(2.0);

#[derive(Debug, Clone)]
pub struct PortalSmallButtonUi {
    pub size: f32,
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub background: Color,
    pub background_hover: Color,
    pub background_pressed: Color,
    pub text_color: Color,
    pub text_style: TextStyle,
}

impl PortalSmallButtonUi {
    pub fn from_theme(theme: ThemeSnapshot) -> Self {
        fn alpha(mut c: Color, a: f32) -> Color {
            c.a = a;
            c
        }

        let font_size = theme.metric_token("metric.font.size").0;
        let radius_sm = theme.metric_token("metric.radius.sm").0;

        Self {
            size: 20.0,
            corner_radius: (radius_sm * 0.75).max(3.0),
            border_width: 1.0,
            border_color: theme.color_token("border"),
            background: theme.color_token("card"),
            background_hover: theme.color_token("accent"),
            background_pressed: alpha(theme.color_token("ring"), 0.22),
            text_color: theme.color_token("foreground"),
            text_style: TextStyle {
                size: Px((font_size - 1.0).max(10.0)),
                ..TextStyle::default()
            },
        }
    }

    pub fn background_for_state(&self, hovered: bool, pressed: bool) -> Color {
        if pressed {
            self.background_pressed
        } else if hovered {
            self.background_hover
        } else {
            self.background
        }
    }
}

impl Default for PortalSmallButtonUi {
    fn default() -> Self {
        Self {
            size: 20.0,
            corner_radius: 4.0,
            border_width: 1.0,
            border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.35,
            },
            background: Color {
                r: 0.16,
                g: 0.16,
                b: 0.18,
                a: 1.0,
            },
            background_hover: Color {
                r: 0.20,
                g: 0.21,
                b: 0.22,
                a: 1.0,
            },
            background_pressed: Color {
                r: 0.20,
                g: 0.55,
                b: 0.95,
                a: 0.22,
            },
            text_color: Color {
                r: 0.92,
                g: 0.93,
                b: 0.94,
                a: 1.0,
            },
            text_style: TextStyle::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct PortalTextInputUi {
    pub chrome: TextInputStyle,
    pub text_style: TextStyle,
    pub height: Px,
}

impl PortalTextInputUi {
    pub fn from_theme(theme: ThemeSnapshot) -> Self {
        let chrome = TextInputStyle::from_theme(theme.clone());
        let font_size = theme.metric_token("metric.font.size");
        let line_height = theme
            .metric_by_key("font.line_height")
            .unwrap_or_else(|| theme.metric_token("metric.font.line_height"));

        Self {
            height: portal_text_input_height(&chrome, line_height),
            chrome,
            text_style: portal_text_input_text_style(font_size, line_height),
        }
    }
}

pub(super) fn portal_button_stack_height(button: &PortalSmallButtonUi, count: usize) -> Px {
    if count == 0 {
        return Px(0.0);
    }

    let buttons = button.size.max(0.0) * count as f32;
    let gaps = PORTAL_BUTTON_STACK_GAP.0.max(0.0) * count.saturating_sub(1) as f32;
    Px(buttons + gaps)
}

pub(super) fn portal_text_input_props(
    model: Model<String>,
    ui: &PortalTextInputUi,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    height: Px,
) -> TextInputProps {
    let mut props = TextInputProps::new(model);
    props.chrome = ui.chrome.clone();
    props.text_style = ui.text_style.clone();
    props.submit_command = submit_command;
    props.cancel_command = cancel_command;
    props.layout.size = SizeStyle {
        width: Length::Fill,
        height: Length::Px(height),
        min_height: Some(Length::Px(height)),
        max_height: Some(Length::Px(height)),
        ..Default::default()
    };
    props
}

fn portal_text_input_text_style(font_size: Px, line_height: Px) -> TextStyle {
    TextStyle {
        size: font_size,
        line_height: Some(line_height),
        line_height_policy: TextLineHeightPolicy::FixedFromStyle,
        vertical_placement: TextVerticalPlacement::BoundsAsLineBox,
        ..Default::default()
    }
}

fn portal_text_input_height(chrome: &TextInputStyle, line_height: Px) -> Px {
    let pad_h = chrome.padding.top.0.max(0.0) + chrome.padding.bottom.0.max(0.0);
    let border_h = chrome.border.top.0.max(0.0) + chrome.border.bottom.0.max(0.0);
    Px((line_height.0.max(0.0) + pad_h + border_h).ceil())
}

pub fn render_small_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    ui: &PortalSmallButtonUi,
    hovered: bool,
    pressed: bool,
    label: &'static str,
) -> fret_ui::element::AnyElement {
    let mut chrome = ContainerProps::default();
    chrome.layout = LayoutStyle {
        size: fret_ui::element::SizeStyle {
            width: Length::Fill,
            height: Length::Fill,
            ..Default::default()
        },
        ..Default::default()
    };
    chrome.background = Some(ui.background_for_state(hovered, pressed));
    chrome.border = Edges::all(Px(ui.border_width));
    chrome.border_color = Some(ui.border_color);
    chrome.corner_radii = Corners::all(Px(ui.corner_radius));

    cx.container(chrome, |cx| {
        let mut row = RowProps::default();
        row.justify = MainAlign::Center;
        row.align = CrossAlign::Center;
        row.layout.size.width = Length::Fill;
        row.layout.size.height = Length::Fill;

        vec![cx.row(row, |cx| {
            let mut text = TextProps::new(label);
            text.color = Some(ui.text_color);
            text.style = Some(ui.text_style.clone());
            vec![cx.text_props(text)]
        })]
    })
}

pub fn render_pressable_small_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    ui: &PortalSmallButtonUi,
    state: PressableState,
    label: &'static str,
) -> fret_ui::element::AnyElement {
    render_small_button(cx, ui, state.hovered, state.pressed, label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portal_text_input_style_uses_fixed_control_line_box() {
        let style = portal_text_input_text_style(Px(13.0), Px(16.0));

        assert_eq!(style.size, Px(13.0));
        assert_eq!(style.line_height, Some(Px(16.0)));
        assert_eq!(
            style.line_height_policy,
            TextLineHeightPolicy::FixedFromStyle
        );
        assert_eq!(
            style.vertical_placement,
            TextVerticalPlacement::BoundsAsLineBox
        );
    }

    #[test]
    fn portal_text_input_height_includes_chrome_without_focus_variants() {
        let mut chrome = TextInputStyle::default();
        chrome.padding = Edges::symmetric(Px(8.0), Px(5.0));
        chrome.border = Edges::all(Px(1.0));

        assert_eq!(portal_text_input_height(&chrome, Px(16.0)), Px(28.0));
    }

    #[test]
    fn portal_button_stack_height_is_fixed_from_button_count() {
        let button = PortalSmallButtonUi {
            size: 20.0,
            ..PortalSmallButtonUi::default()
        };

        assert_eq!(portal_button_stack_height(&button, 0), Px(0.0));
        assert_eq!(portal_button_stack_height(&button, 1), Px(20.0));
        assert_eq!(portal_button_stack_height(&button, 3), Px(64.0));
    }
}
