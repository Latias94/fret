use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, PressableState};
use fret_ui::{ElementContext, UiHost};

use super::super::{ButtonVariant, control_chrome};

mod a11y;
mod variant;

pub(super) use a11y::button_a11y;
pub(super) use variant::apply_button_variant_layout;
use variant::arrow_symbol;

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
