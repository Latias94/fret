use std::sync::Arc;

use fret_core::{Px, SemanticsRole};
use fret_ui::element::{AnyElement, FlexProps, MainAlign, SemanticsDecoration};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

use crate::avatar::Avatar as ShadcnAvatar;
use crate::avatar::AvatarFallback as ShadcnAvatarFallback;
use crate::test_id::attach_test_id;

/// Input item for [`AvatarStack`].
#[derive(Debug, Clone)]
pub enum AvatarStackItem {
    Avatar(ShadcnAvatar),
    Element(AnyElement),
}

impl From<ShadcnAvatar> for AvatarStackItem {
    fn from(value: ShadcnAvatar) -> Self {
        Self::Avatar(value)
    }
}

impl From<AnyElement> for AvatarStackItem {
    fn from(value: AnyElement) -> Self {
        Self::Element(value)
    }
}

/// A small avatar stack block inspired by Kibo's shadcn blocks.
///
/// This intentionally avoids CSS mask tricks; it uses overlap + clipping for a stable GPU-first
/// outcome.
///
/// Upstream inspiration (MIT):
/// - `repo-ref/kibo/packages/avatar-stack`
#[derive(Debug, Clone)]
pub struct AvatarStack {
    items: Vec<AvatarStackItem>,
    size: Px,
    overlap: Space,
    max_visible: Option<usize>,
    test_id: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl AvatarStack {
    pub fn new<I, T>(items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<AvatarStackItem>,
    {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            size: Px(40.0),
            overlap: Space::N1,
            max_visible: None,
            test_id: None,
            a11y_label: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn items<I, T>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<AvatarStackItem>,
    {
        self.items = items.into_iter().map(Into::into).collect();
        self
    }

    /// Fixed square size for each avatar.
    pub fn size_px(mut self, size: Px) -> Self {
        self.size = size;
        self
    }

    /// Negative overlap between adjacent avatars.
    pub fn overlap(mut self, overlap: Space) -> Self {
        self.overlap = overlap;
        self
    }

    /// Maximum number of visible items in the stack.
    ///
    /// If the stack overflows, the last visible item becomes a `+N` overflow indicator (so the
    /// overflow indicator counts toward `max_visible`).
    pub fn max_visible(mut self, max_visible: usize) -> Self {
        self.max_visible = Some(max_visible);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let size = self.size;
            let overlap = self.overlap;
            let stack_layout = decl_style::layout_style(&theme, self.layout);

            // Common web outcome: `ring-2 ring-background`.
            let ring = theme.color_required("background");
            let item_chrome = ChromeRefinement::default()
                .rounded(Radius::Full)
                .border_width(Px(2.0))
                .border_color(ColorRef::Color(ring));
            let item_layout_base = LayoutRefinement::default()
                .w_px(size)
                .h_px(size)
                .flex_shrink_0()
                .overflow_hidden();

            let mut items = self.items;
            if let Some(max_visible) = self.max_visible {
                let max_visible = max_visible.max(1);
                if items.len() > max_visible {
                    let visible = max_visible.saturating_sub(1);
                    let overflow_count = items.len().saturating_sub(visible);
                    items.truncate(visible);
                    let overflow = ShadcnAvatar::new([ShadcnAvatarFallback::new(format!(
                        "+{overflow_count}"
                    ))
                    .into_element(cx)]);
                    items.push(AvatarStackItem::Avatar(overflow));
                }
            }

            let mut out = Vec::with_capacity(items.len());
            for (idx, item) in items.into_iter().enumerate() {
                let mut layout = item_layout_base.clone();
                if idx > 0 {
                    layout = layout.ml_neg(overlap);
                }

                let child = match item {
                    AvatarStackItem::Avatar(avatar) => avatar
                        .refine_layout(LayoutRefinement::default().w_px(size).h_px(size))
                        .into_element(cx),
                    AvatarStackItem::Element(el) => el,
                };

                let wrapper = cx.container(
                    decl_style::container_props(&theme, item_chrome.clone(), layout),
                    move |_cx| vec![child],
                );
                out.push(wrapper);
            }

            let root = cx.flex(
                FlexProps {
                    layout: stack_layout,
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    padding: fret_core::Edges::all(Px(0.0)),
                    justify: MainAlign::Start,
                    align: fret_ui::element::CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| out,
            );

            let test_id = self
                .test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.avatar-stack"));
            let root = attach_test_id(root, test_id.clone());

            root.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .label(
                        self.a11y_label
                            .unwrap_or_else(|| Arc::<str>::from("Avatar stack")),
                    ),
            )
        })
    }
}
