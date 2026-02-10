//! Material 3 dividers.

use std::sync::Arc;

use fret_core::{Axis, Color, Px, SemanticsRole};
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, ContainerProps, Length, SemanticsProps};
use fret_ui::elements::ElementContext;
use fret_ui_kit::declarative::ElementContextThemeExt as _;

use crate::tokens::divider as divider_tokens;

#[derive(Debug, Clone)]
pub struct Divider {
    orientation: Axis,
    thickness: Option<Px>,
    color: Option<Color>,
    test_id: Option<Arc<str>>,
}

impl Divider {
    pub fn horizontal() -> Self {
        Self {
            orientation: Axis::Horizontal,
            thickness: None,
            color: None,
            test_id: None,
        }
    }

    pub fn vertical() -> Self {
        Self {
            orientation: Axis::Vertical,
            thickness: None,
            color: None,
            test_id: None,
        }
    }

    pub fn thickness(mut self, thickness: Px) -> Self {
        self.thickness = Some(thickness);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (thickness, color) = cx.with_theme(|theme| {
            let thickness = self
                .thickness
                .unwrap_or_else(|| divider_tokens::thickness(theme));
            let color = self.color.unwrap_or_else(|| divider_tokens::color(theme));
            (thickness, color)
        });

        let mut props = ContainerProps::default();
        props.background = Some(color);

        match self.orientation {
            Axis::Horizontal => {
                props.layout.size.width = Length::Fill;
                props.layout.size.height = Length::Px(thickness);
            }
            Axis::Vertical => {
                props.layout.size.width = Length::Px(thickness);
                props.layout.size.height = Length::Fill;
            }
        }

        let divider = cx.container(props, |_cx| Vec::new());

        let Some(test_id) = self.test_id else {
            return divider;
        };

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Generic,
                test_id: Some(test_id),
                focusable: false,
                ..Default::default()
            },
            |_cx| vec![divider],
        )
    }
}
