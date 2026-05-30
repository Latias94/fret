use std::sync::Arc;

use fret_core::Color;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::imui::control_chrome;

pub(in crate::imui::button_controls) struct ButtonVisualContent {
    label: Arc<str>,
    foreground: Option<Color>,
}

impl ButtonVisualContent {
    pub(in crate::imui::button_controls) fn visible(label: Arc<str>, foreground: Color) -> Self {
        Self {
            label,
            foreground: Some(foreground),
        }
    }

    pub(in crate::imui::button_controls) fn empty() -> Self {
        Self {
            label: Arc::from(""),
            foreground: None,
        }
    }

    pub(in crate::imui::button_controls) fn children<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> Vec<AnyElement> {
        let Some(foreground) = self.foreground else {
            return Vec::new();
        };
        let label = self.label;
        vec![cx.flex(control_chrome::centered_row_props(), move |cx| {
            vec![control_chrome::control_text(cx, label.clone(), foreground)]
        })]
    }
}
