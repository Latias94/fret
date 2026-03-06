use std::sync::Arc;

use fret_core::{Corners, Edges, Px};
use fret_icons::IconId;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{Items, Justify, LayoutRefinement, Space};

use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

#[derive(Clone)]
/// A small “download/export transcript” button.
///
/// This component emits an intent via `on_activate`; performing the actual effect (clipboard/file IO)
/// is app-owned.
pub struct ConversationDownload {
    label: Arc<str>,
    disabled: bool,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    icon: IconId,
    show_label: bool,
}

impl std::fmt::Debug for ConversationDownload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConversationDownload")
            .field("label", &self.label.as_ref())
            .field("disabled", &self.disabled)
            .field("has_on_activate", &self.on_activate.is_some())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl ConversationDownload {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            on_activate: None,
            test_id: None,
            layout: LayoutRefinement::default(),
            icon: IconId::new_static("lucide.download"),
            show_label: false,
        }
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = icon;
        self
    }

    /// Show a text label (instead of the default icon-only affordance).
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let layout = self.layout;

        let mut btn = if self.show_label {
            Button::new(self.label.clone())
                .size(ButtonSize::Sm)
                .refine_layout(layout.clone())
        } else {
            Button::new("")
                .a11y_label(self.label)
                .size(ButtonSize::Icon)
                .leading_icon(self.icon)
                .corner_radii_override(Corners::all(Px(999.0)))
        }
        .variant(ButtonVariant::Outline)
        .disabled(self.disabled);

        if let Some(on_activate) = self.on_activate {
            btn = btn.on_activate(on_activate);
        }
        if let Some(test_id) = self.test_id {
            btn = btn.test_id(test_id);
        }

        let btn = btn.into_element(cx);

        if self.show_label {
            return btn;
        }

        let overlay_layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .absolute()
                .left(Space::N0)
                .right(Space::N0)
                .top(Space::N4)
                .merge(layout),
        );
        let pad = decl_style::space(&theme, Space::N4);

        cx.container(
            ContainerProps {
                layout: overlay_layout,
                padding: Edges::symmetric(pad, Px(0.0)).into(),
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::h_row(move |_cx| vec![btn])
                        .layout(LayoutRefinement::default().w_full())
                        .justify(Justify::End)
                        .items(Items::Center)
                        .into_element(cx),
                ]
            },
        )
    }
}
