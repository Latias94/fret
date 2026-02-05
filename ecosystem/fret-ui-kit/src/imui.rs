//! Optional integration helpers for immediate-mode authoring frontends.
//!
//! This module lives in `fret-ui-kit` (not `fret-imui`) to preserve dependency direction:
//!
//! - `fret-imui` stays policy-light and depends only on `fret-ui` (+ `fret-authoring` contract).
//! - `fret-ui-kit` can optionally provide bridges that allow `UiBuilder<T>` patch vocabulary to be
//!   used from immediate-style control flow.

use std::sync::Arc;

use fret_authoring::Response;
use fret_authoring::UiWriter;
use fret_core::SemanticsRole;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::UiBuilder;
use crate::{UiIntoElement, UiPatchTarget};

/// A value that can be rendered into a declarative element within an `ElementContext`.
///
/// This is used to bridge the `UiBuilder<T>` ecosystem authoring surface (ADR 0175) into
/// immediate-mode frontends (`UiWriter`).
pub trait UiKitIntoElement<H: UiHost> {
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement;
}

impl<H: UiHost, T> UiKitIntoElement<H> for UiBuilder<T>
where
    T: UiPatchTarget + UiIntoElement,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, F, I> UiKitIntoElement<H> for UiBuilder<crate::ui::FlexBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, B> UiKitIntoElement<H> for UiBuilder<crate::ui::FlexBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, B> UiKitIntoElement<H> for UiBuilder<crate::ui::ContainerBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, F, I> UiKitIntoElement<H> for UiBuilder<crate::ui::ContainerBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, F, I> UiKitIntoElement<H> for UiBuilder<crate::ui::StackBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, F, I> UiKitIntoElement<H> for UiBuilder<crate::ui::ScrollAreaBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

impl<H: UiHost, B> UiKitIntoElement<H> for UiBuilder<crate::ui::ScrollAreaBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn into_any_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element(cx)
    }
}

/// Extension trait bridging `fret-ui-kit` authoring (`UiBuilder<T>`) into an immediate-mode output.
pub trait UiWriterUiKitExt<H: UiHost>: UiWriter<H> {
    /// Render a `UiBuilder<T>` (or other supported authoring value) into the current output list.
    #[track_caller]
    fn add_ui<B>(&mut self, value: B)
    where
        B: UiKitIntoElement<H>,
    {
        let element = self.with_cx_mut(|cx| value.into_any_element(cx));
        self.add(element);
    }
}

impl<H: UiHost, W: UiWriter<H> + ?Sized> UiWriterUiKitExt<H> for W {}

/// A richer interaction result intended for immediate-mode facade helpers.
///
/// This is a ui-kit-level convenience wrapper: it extends the minimal `fret-authoring::Response`
/// contract with additional commonly requested signals.
#[derive(Debug, Clone, Copy, Default)]
pub struct ResponseExt {
    pub core: Response,
    pub secondary_clicked: bool,
    pub double_clicked: bool,
}

impl ResponseExt {
    pub fn clicked(self) -> bool {
        self.core.clicked()
    }

    pub fn changed(self) -> bool {
        self.core.changed()
    }
}

const fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut i = 0usize;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
        i += 1;
    }
    hash
}

const KEY_CLICKED: u64 = fnv1a64(b"fret-ui-kit.imui.clicked.v1");
const KEY_CHANGED: u64 = fnv1a64(b"fret-ui-kit.imui.changed.v1");
const KEY_SECONDARY_CLICKED: u64 = fnv1a64(b"fret-ui-kit.imui.secondary_clicked.v1");
const KEY_DOUBLE_CLICKED: u64 = fnv1a64(b"fret-ui-kit.imui.double_clicked.v1");

/// Immediate-mode facade helpers for any authoring frontend that implements `UiWriter`.
///
/// This is intentionally a small convenience layer. It aims to feel closer to egui/imgui while
/// still compiling down to Fret's declarative element tree and delegating complex policy to
/// higher-level components.
pub trait UiWriterImUiFacadeExt<H: UiHost>: UiWriter<H> {
    fn text(&mut self, text: impl Into<Arc<str>>) {
        let element = self.with_cx_mut(|cx| cx.text(text));
        self.add(element);
    }

    fn separator(&mut self) {
        let element = self.with_cx_mut(|cx| {
            let mut props = fret_ui::element::ContainerProps::default();
            let theme = fret_ui::Theme::global(&*cx.app);
            props.background = Some(theme.color_required("border"));
            props.layout.size.width = fret_ui::element::Length::Fill;
            props.layout.size.height = fret_ui::element::Length::Px(fret_core::Px(1.0));
            cx.container(props, |_| Vec::new())
        });
        self.add(element);
    }

    fn button(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
        let label = label.into();
        let mut response = ResponseExt::default();

        let element = self.with_cx_mut(|cx| {
            let mut props = fret_ui::element::PressableProps::default();
            props.a11y = fret_ui::element::PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(label.clone()),
                ..Default::default()
            };

            cx.pressable_with_id(props, |cx, state, id| {
                cx.pressable_clear_on_pointer_down();
                cx.pressable_clear_on_pointer_move();
                cx.pressable_clear_on_pointer_up();

                cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                    host.record_transient_event(acx, KEY_CLICKED);
                    host.notify(acx);
                }));

                cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
                    if up.is_click && up.button == fret_core::MouseButton::Right {
                        host.record_transient_event(acx, KEY_SECONDARY_CLICKED);
                        host.notify(acx);
                        return fret_ui::action::PressablePointerUpResult::SkipActivate;
                    }

                    if up.is_click
                        && up.button == fret_core::MouseButton::Left
                        && up.click_count == 2
                    {
                        host.record_transient_event(acx, KEY_DOUBLE_CLICKED);
                        host.notify(acx);
                    }

                    fret_ui::action::PressablePointerUpResult::Continue
                }));

                response.core.hovered = state.hovered;
                response.core.pressed = state.pressed;
                response.core.focused = state.focused;
                response.core.clicked = cx.take_transient_for(id, KEY_CLICKED);
                response.secondary_clicked = cx.take_transient_for(id, KEY_SECONDARY_CLICKED);
                response.double_clicked = cx.take_transient_for(id, KEY_DOUBLE_CLICKED);
                response.core.rect = cx.last_bounds_for_element(id);

                vec![cx.text(label.clone())]
            })
        });

        self.add(element);
        response
    }

    fn checkbox_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
    ) -> ResponseExt {
        let label = label.into();
        let model = model.clone();
        let mut response = ResponseExt::default();

        let element = self.with_cx_mut(|cx| {
            let value = cx
                .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| *v)
                .unwrap_or(false);

            let mut props = fret_ui::element::PressableProps::default();
            props.a11y = fret_ui::element::PressableA11y {
                role: Some(SemanticsRole::Checkbox),
                label: Some(label.clone()),
                checked: Some(value),
                ..Default::default()
            };

            cx.pressable_with_id(props, |cx, state, id| {
                cx.pressable_clear_on_pointer_down();
                cx.pressable_clear_on_pointer_move();
                cx.pressable_clear_on_pointer_up();

                let model = model.clone();
                cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                    let _ = host.update_model(&model, |v: &mut bool| *v = !*v);
                    host.record_transient_event(acx, KEY_CHANGED);
                    host.notify(acx);
                }));

                response.core.hovered = state.hovered;
                response.core.pressed = state.pressed;
                response.core.focused = state.focused;
                response.core.changed = cx.take_transient_for(id, KEY_CHANGED);
                response.core.rect = cx.last_bounds_for_element(id);

                let prefix: Arc<str> = if value {
                    Arc::from("[x] ")
                } else {
                    Arc::from("[ ] ")
                };
                vec![cx.text(Arc::from(format!("{prefix}{label}")))]
            })
        });

        self.add(element);
        response
    }
}

impl<H: UiHost, W: UiWriter<H> + ?Sized> UiWriterImUiFacadeExt<H> for W {}
