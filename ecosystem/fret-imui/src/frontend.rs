use std::hash::Hash;

use fret_authoring::{UiWriter, mark_immediate_render_frame};
use fret_ui::element::{AnyElement, ColumnProps, Elements, Length, RowProps};
use fret_ui::{ElementContext, ElementContextAccess, UiHost};

/// Default IMUI mount: wraps produced siblings in a fill-sized `Column` so they stack safely.
///
/// This is the pit-of-success entry point for view roots and non-layout parents. Use
/// [`imui_raw`] only when the surrounding host already owns sibling flow and you explicitly want
/// bare element emission.
pub fn imui<'a, H: UiHost>(
    cx: &mut ElementContext<'a, H>,
    f: impl for<'cx> FnOnce(&mut ImUi<'cx, 'a, H>),
) -> Elements {
    let mut props = ColumnProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;

    let element = cx.column(props, |cx| imui_raw(cx, f));
    element.into()
}

/// Capability-first adapter for [`imui`] when the caller only owns `ElementContextAccess`.
pub fn imui_in<'a, H: UiHost + 'a, Cx>(
    cx: &mut Cx,
    f: impl for<'cx> FnOnce(&mut ImUi<'cx, 'a, H>),
) -> Elements
where
    Cx: ElementContextAccess<'a, H>,
{
    imui(cx.elements(), f)
}

/// Advanced IMUI mount: emits produced siblings directly without adding a host layout node.
///
/// Use this only when an explicit `Column`/`Row`/flex host already owns sibling flow. Calling this
/// under a non-layout parent reintroduces the common "all children overlap at `(0,0)`" footgun.
pub fn imui_raw<'a, H: UiHost>(
    cx: &mut ElementContext<'a, H>,
    f: impl for<'cx> FnOnce(&mut ImUi<'cx, 'a, H>),
) -> Elements {
    let mut out = Vec::new();
    imui_build(cx, &mut out, f);
    out.into()
}

/// Capability-first adapter for [`imui_raw`] when the caller only owns `ElementContextAccess`.
pub fn imui_raw_in<'a, H: UiHost + 'a, Cx>(
    cx: &mut Cx,
    f: impl for<'cx> FnOnce(&mut ImUi<'cx, 'a, H>),
) -> Elements
where
    Cx: ElementContextAccess<'a, H>,
{
    imui_raw(cx.elements(), f)
}

pub fn imui_build<'a, H: UiHost>(
    cx: &mut ElementContext<'a, H>,
    out: &mut Vec<AnyElement>,
    f: impl for<'cx> FnOnce(&mut ImUi<'cx, 'a, H>),
) {
    let _ = mark_immediate_render_frame(cx);
    let mut ui = ImUi { cx, out };
    f(&mut ui);
}

/// Capability-first adapter for [`imui_build`] when the caller only owns `ElementContextAccess`.
pub fn imui_build_in<'a, H: UiHost + 'a, Cx>(
    cx: &mut Cx,
    out: &mut Vec<AnyElement>,
    f: impl for<'cx> FnOnce(&mut ImUi<'cx, 'a, H>),
) where
    Cx: ElementContextAccess<'a, H>,
{
    imui_build(cx.elements(), out, f);
}

pub struct ImUi<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    out: &'cx mut Vec<AnyElement>,
}

impl<'cx, 'a, H: UiHost> ImUi<'cx, 'a, H> {
    pub fn cx_mut(&mut self) -> &mut ElementContext<'a, H> {
        self.cx
    }

    pub fn add(&mut self, element: AnyElement) {
        self.out.push(element);
    }

    pub fn mount<I>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> I)
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.out.extend(f(self.cx));
    }

    #[track_caller]
    pub fn id<K: Hash>(&mut self, key: K, f: impl for<'cx2, 'a2> FnOnce(&mut ImUi<'cx2, 'a2, H>)) {
        let out = &mut *self.out;
        self.cx.keyed(key, |cx| {
            let mut ui = ImUi { cx, out };
            f(&mut ui);
        });
    }

    #[track_caller]
    pub fn push_id<K: Hash>(
        &mut self,
        key: K,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUi<'cx2, 'a2, H>),
    ) {
        self.id(key, f);
    }

    #[track_caller]
    pub fn for_each_keyed<I, K, T>(
        &mut self,
        items: I,
        mut f: impl FnMut(&mut ImUi<'_, '_, H>, &K, T),
    ) where
        I: IntoIterator<Item = (K, T)>,
        K: Hash,
    {
        let f = &mut f;
        for (key, item) in items {
            self.id(&key, |ui| f(ui, &key, item));
        }
    }

    /// Iterates over a slice using callsite-based (unkeyed) identity.
    ///
    /// This is convenient for static lists where order never changes. For dynamic collections
    /// (insert/remove/reorder), prefer `for_each_keyed(...)` or wrap each item in `ui.id(key, ...)`
    /// to preserve per-element state.
    ///
    /// In debug builds, the underlying runtime emits a warning if the list order changes between
    /// frames (aligning with the existing `ElementContext::for_each_unkeyed` diagnostics).
    #[track_caller]
    pub fn for_each_unkeyed<T: Hash>(
        &mut self,
        items: &[T],
        mut f: impl FnMut(&mut ImUi<'_, '_, H>, usize, &T),
    ) {
        let f = &mut f;
        let out = &mut *self.out;
        self.cx.for_each_unkeyed(items, |cx, index, item| {
            let mut ui = ImUi { cx, out };
            f(&mut ui, index, item);
        });
    }

    pub fn row(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUi<'cx2, 'a2, H>)) {
        let element = self.cx.row(RowProps::default(), |cx| imui_raw(cx, f));
        self.out.push(element);
    }

    pub fn column(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUi<'cx2, 'a2, H>)) {
        let element = self.cx.column(ColumnProps::default(), |cx| imui_raw(cx, f));
        self.out.push(element);
    }
}

impl<'cx, 'a, H: UiHost> UiWriter<H> for ImUi<'cx, 'a, H> {
    fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {
        f(self.cx)
    }

    fn add(&mut self, element: AnyElement) {
        self.out.push(element);
    }
}
