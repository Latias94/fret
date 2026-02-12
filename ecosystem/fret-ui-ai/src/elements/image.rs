//! AI Elements-aligned generated image surface.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/image.tsx`.

use std::sync::Arc;

use fret_core::{ImageId, SemanticsRole, ViewportFit};
use fret_ui::element::{AnyElement, ImageProps, LayoutStyle, Length, SemanticsDecoration};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Radius};

#[derive(Clone)]
pub struct Image {
    image: ImageId,
    alt: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("image", &self.image)
            .field("alt", &self.alt.as_deref())
            .field("test_id", &self.test_id.as_deref())
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Image {
    pub fn new(image: ImageId) -> Self {
        Self {
            image,
            alt: None,
            test_id: None,
            chrome: ChromeRefinement::default().rounded(Radius::Md),
            layout: LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .overflow_hidden(),
        }
    }

    pub fn alt(mut self, alt: impl Into<Arc<str>>) -> Self {
        self.alt = Some(alt.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let mut wrapper = decl_style::container_props(&theme, self.chrome, self.layout);
        wrapper.snap_to_device_pixels = true;

        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Auto;

        let img = cx.image_props(ImageProps {
            layout,
            image: self.image,
            fit: ViewportFit::Contain,
            opacity: 1.0,
            uv: None,
        });

        let mut el = cx.container(wrapper, move |_cx| vec![img]);
        if let Some(test_id) = self.test_id {
            el = el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(test_id),
            );
        }
        el
    }
}
