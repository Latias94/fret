use std::sync::Arc;

use fret_components_ui::LayoutRefinement;
use fret_components_ui::declarative::style as decl_style;
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, Length, SizeStyle, SliderProps};
use fret_ui::{ElementCx, SliderStyle, Theme, UiHost};

#[derive(Clone)]
pub struct Slider {
    model: Model<Vec<f32>>,
    min: f32,
    max: f32,
    step: f32,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    layout: LayoutRefinement,
    style: Option<SliderStyle>,
}

impl Slider {
    pub fn new(model: Model<Vec<f32>>) -> Self {
        Self {
            model,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            disabled: false,
            a11y_label: None,
            layout: LayoutRefinement::default(),
            style: None,
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn style(mut self, style: SliderStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        slider(
            cx,
            self.model,
            self.min,
            self.max,
            self.step,
            self.disabled,
            self.a11y_label,
            self.layout,
            self.style,
        )
    }
}

pub fn slider<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    model: Model<Vec<f32>>,
    min: f32,
    max: f32,
    step: f32,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    layout: LayoutRefinement,
    style: Option<SliderStyle>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let style = style.unwrap_or_else(|| {
        let mut style = SliderStyle::from_theme(&theme);
        let radius = Px((style.thumb_size.0 * 0.5).max(0.0));
        style.focus_ring = Some(decl_style::focus_ring(&theme, radius));
        style
    });

    let root_layout = decl_style::layout_style(&theme, layout.relative().w_full());

    let mut props = SliderProps::new(model);
    props.a11y_label = a11y_label;
    props.min = min;
    props.max = max;
    props.step = step;
    props.enabled = !disabled;
    props.chrome = style;
    props.layout = root_layout;
    props.layout.size = SizeStyle {
        width: Length::Fill,
        height: Length::Auto,
        ..Default::default()
    };

    cx.slider(props)
}
