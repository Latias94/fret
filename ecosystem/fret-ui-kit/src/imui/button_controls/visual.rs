use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_ui::element::{
    AnyElement, ContainerProps, Length, PressableA11y, PressableProps, PressableState,
};
use fret_ui::{ElementContext, UiHost};

use super::super::{ButtonArrowDirection, ButtonOptions, ButtonVariant, control_chrome};

pub(super) struct ButtonVisual {
    chrome: ContainerProps,
    content: ButtonVisualContent,
}

impl ButtonVisual {
    pub(super) fn into_parts(self) -> (ContainerProps, ButtonVisualContent) {
        (self.chrome, self.content)
    }
}

pub(super) struct ButtonVisualContent {
    label: Arc<str>,
    foreground: Option<Color>,
}

impl ButtonVisualContent {
    pub(super) fn children<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> Vec<AnyElement> {
        let Some(foreground) = self.foreground else {
            return Vec::new();
        };
        let label = self.label;
        vec![cx.flex(control_chrome::centered_row_props(), move |cx| {
            vec![control_chrome::control_text(cx, label.clone(), foreground)]
        })]
    }
}

pub(super) fn button_a11y(
    label: &Arc<str>,
    options: &ButtonOptions,
    variant: ButtonVariant,
) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::Button),
        label: button_a11y_label(label, options, variant),
        test_id: options.test_id.clone(),
        ..Default::default()
    }
}

pub(super) fn apply_button_variant_layout(props: &mut PressableProps, variant: ButtonVariant) {
    match variant {
        ButtonVariant::Default => {
            props.layout.size.min_height = Some(Length::Px(control_chrome::BUTTON_MIN_HEIGHT));
        }
        ButtonVariant::Small => {
            props.layout.size.min_height =
                Some(Length::Px(control_chrome::SMALL_BUTTON_MIN_HEIGHT));
        }
        ButtonVariant::Arrow(_) => {
            props.layout.size.width = Length::Px(control_chrome::ARROW_BUTTON_SIZE);
            props.layout.size.height = Length::Px(control_chrome::ARROW_BUTTON_SIZE);
        }
        ButtonVariant::Invisible { size } => {
            props.layout.size.width = Length::Px(size.width);
            props.layout.size.height = Length::Px(size.height);
        }
    }
}

pub(super) fn resolve_button_visual<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
    state: PressableState,
    variant: ButtonVariant,
    label: Arc<str>,
) -> ButtonVisual {
    match variant {
        ButtonVariant::Default => {
            let (palette, chrome) = control_chrome::button_chrome(cx, enabled, state);
            ButtonVisual::visible(chrome, label, palette.foreground)
        }
        ButtonVariant::Small => {
            let (palette, mut chrome) = control_chrome::button_chrome(cx, enabled, state);
            chrome.padding = Edges {
                left: Px(8.0),
                right: Px(8.0),
                top: Px(2.0),
                bottom: Px(2.0),
            }
            .into();
            chrome.corner_radii = Corners::all(control_chrome::CONTROL_RADIUS);
            ButtonVisual::visible(chrome, label, palette.foreground)
        }
        ButtonVariant::Arrow(direction) => {
            let (palette, mut chrome) = control_chrome::button_chrome(cx, enabled, state);
            chrome.padding = Edges::all(Px(0.0)).into();
            ButtonVisual::visible(chrome, arrow_symbol(direction), palette.foreground)
        }
        ButtonVariant::Invisible { .. } => ButtonVisual::invisible(),
    }
}

impl ButtonVisual {
    fn visible(chrome: ContainerProps, label: Arc<str>, foreground: Color) -> Self {
        Self {
            chrome,
            content: ButtonVisualContent {
                label,
                foreground: Some(foreground),
            },
        }
    }

    fn invisible() -> Self {
        Self {
            chrome: ContainerProps::default(),
            content: ButtonVisualContent {
                label: Arc::from(""),
                foreground: None,
            },
        }
    }
}

fn arrow_symbol(direction: ButtonArrowDirection) -> Arc<str> {
    Arc::from(match direction {
        ButtonArrowDirection::Left => "<",
        ButtonArrowDirection::Right => ">",
        ButtonArrowDirection::Up => "^",
        ButtonArrowDirection::Down => "v",
    })
}

fn arrow_a11y_label(direction: ButtonArrowDirection) -> Arc<str> {
    Arc::from(match direction {
        ButtonArrowDirection::Left => "Left arrow button",
        ButtonArrowDirection::Right => "Right arrow button",
        ButtonArrowDirection::Up => "Up arrow button",
        ButtonArrowDirection::Down => "Down arrow button",
    })
}

fn button_a11y_label(
    label: &Arc<str>,
    options: &ButtonOptions,
    variant: ButtonVariant,
) -> Option<Arc<str>> {
    options.a11y_label.clone().or_else(|| match variant {
        ButtonVariant::Arrow(direction) => Some(arrow_a11y_label(direction)),
        ButtonVariant::Invisible { .. } if label.is_empty() => None,
        _ => Some(label.clone()),
    })
}
