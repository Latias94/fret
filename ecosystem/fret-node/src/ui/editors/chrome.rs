use fret_core::{Color, Corners, Edges, Px, TextStyle};
use fret_ui::element::{
    ContainerProps, CrossAlign, LayoutStyle, Length, MainAlign, PressableState, RowProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{ThemeSnapshot, UiHost};

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
