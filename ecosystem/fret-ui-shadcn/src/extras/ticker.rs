use std::any::Any;
use std::sync::Arc;

use fret_core::ImageId;
use fret_runtime::ActionId;
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Space, ui};

use crate::avatar::{Avatar, AvatarFallback, AvatarImage};
use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::test_id::attach_test_id;

type ActionPayloadFactory = Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>;

/// A compact stock/asset ticker block inspired by Kibo's shadcn blocks.
///
/// Upstream inspiration (MIT):
/// - `repo-ref/kibo/packages/ticker`
///
/// Note: the upstream component formats currency/locale via `Intl.NumberFormat`. To keep extras
/// dependency-light and deterministic, this component takes pre-formatted strings by default.
#[derive(Clone)]
pub struct Ticker {
    symbol: Arc<str>,
    icon_image: Option<ImageId>,
    icon_fallback: Option<Arc<str>>,
    price: Arc<str>,
    change: Arc<str>,
    change_kind: TickerChangeKind,
    action: Option<ActionId>,
    action_payload: Option<ActionPayloadFactory>,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Ticker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ticker")
            .field("symbol", &self.symbol)
            .field("icon_image", &self.icon_image.is_some())
            .field("icon_fallback", &self.icon_fallback)
            .field("price", &self.price)
            .field("change", &self.change)
            .field("change_kind", &self.change_kind)
            .field("action", &self.action)
            .field("action_payload", &self.action_payload.is_some())
            .field("on_activate", &self.on_activate.is_some())
            .field("test_id", &self.test_id)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TickerChangeKind {
    #[default]
    Up,
    Down,
    Flat,
}

impl Ticker {
    pub fn new(symbol: impl Into<Arc<str>>) -> Self {
        let symbol = symbol.into();
        Self {
            symbol: symbol.clone(),
            icon_image: None,
            icon_fallback: None,
            price: Arc::<str>::from(""),
            change: Arc::<str>::from(""),
            change_kind: TickerChangeKind::default(),
            action: None,
            action_payload: None,
            on_activate: None,
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn icon_image(mut self, image: Option<ImageId>) -> Self {
        self.icon_image = image;
        self
    }

    /// Sets a fallback label used when there is no icon image.
    pub fn icon_fallback(mut self, fallback: impl Into<Arc<str>>) -> Self {
        self.icon_fallback = Some(fallback.into());
        self
    }

    pub fn price(mut self, price: impl Into<Arc<str>>) -> Self {
        self.price = price.into();
        self
    }

    pub fn change(mut self, change: impl Into<Arc<str>>) -> Self {
        self.change = change.into();
        self
    }

    pub fn change_kind(mut self, kind: TickerChangeKind) -> Self {
        self.change_kind = kind;
        self
    }

    /// Bind a stable action ID to this ticker (action-first authoring).
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Attach a payload for parameterized ticker actions (ADR 0312).
    pub fn action_payload<T>(mut self, payload: T) -> Self
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        let payload = Arc::new(payload);
        self.action_payload = Some(Arc::new(move || Box::new(payload.as_ref().clone())));
        self
    }

    /// Like [`Ticker::action_payload`], but computes the payload lazily on activation.
    pub fn action_payload_factory(mut self, payload: ActionPayloadFactory) -> Self {
        self.action_payload = Some(payload);
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

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).snapshot();

            let symbol = self.symbol.clone();
            let label = Arc::<str>::from(format!("Ticker {symbol}"));

            let icon_image = self.icon_image;
            let icon_fallback = self
                .icon_fallback
                .unwrap_or_else(|| symbol.chars().take(2).collect::<String>().into());

            let icon = Avatar::new([
                AvatarImage::maybe(icon_image).into_element(cx),
                AvatarFallback::new(icon_fallback)
                    .when_image_missing(icon_image)
                    .into_element(cx),
            ])
            .into_element(cx);

            let symbol_text = ui::text(symbol.clone())
                .font_medium()
                .nowrap()
                .into_element(cx);

            let muted = theme.color_token("muted-foreground");
            let price_text = ui::text(self.price)
                .text_color(ColorRef::Color(muted))
                .nowrap()
                .into_element(cx);

            let change_color = match self.change_kind {
                TickerChangeKind::Up => theme.color_token("primary"),
                TickerChangeKind::Down => theme.color_token("destructive"),
                TickerChangeKind::Flat => muted,
            };
            let change_text = ui::text(self.change)
                .text_color(ColorRef::Color(change_color))
                .nowrap()
                .into_element(cx);

            let mut button = Button::new(label)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .refine_style(
                    ChromeRefinement::default()
                        .px(Space::N0)
                        .py(Space::N0)
                        .merge(self.chrome),
                )
                .refine_layout(LayoutRefinement::default().merge(self.layout))
                .children([icon, symbol_text, price_text, change_text]);
            if let Some(action) = self.action {
                button = button.action(action);
            }
            if let Some(payload) = self.action_payload {
                button = button.action_payload_factory(payload);
            }
            if let Some(on_activate) = self.on_activate {
                button = button.on_activate(on_activate);
            }
            let button = button.into_element(cx);

            let test_id = self
                .test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.ticker"));
            attach_test_id(button, test_id)
        })
    }
}
