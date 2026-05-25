use super::*;

/// A minimal `UiWriter` implementation used by facade container helpers (e.g. floating windows).
///
/// This mirrors the `fret-imui::ImUi` pattern without depending on the `fret-imui` crate.
pub struct ImUiFacade<'cx, 'a, H: UiHost> {
    pub(in crate::imui) cx: &'cx mut ElementContext<'a, H>,
    pub(in crate::imui) out: &'cx mut Vec<AnyElement>,
    pub(in crate::imui) build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
}

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub(super) fn record_focusable(&mut self, id: Option<GlobalElementId>, enabled: bool) {
        if !enabled {
            return;
        }
        let Some(id) = id else {
            return;
        };
        let Some(st) = self.build_focus.as_ref() else {
            return;
        };
        if st.get().is_none() {
            st.set(Some(id));
        }
    }

    pub fn cx_mut(&mut self) -> &mut ElementContext<'a, H> {
        self.cx
    }

    pub fn add(&mut self, element: AnyElement) {
        self.out.push(element);
    }

    pub fn id<K: Hash>(
        &mut self,
        key: K,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let out = &mut *self.out;
        let build_focus = self.build_focus.clone();
        self.cx.keyed(key, |cx| {
            prepare_imui_runtime_for_frame(cx);
            let mut ui = ImUiFacade {
                cx,
                out,
                build_focus,
            };
            f(&mut ui);
        });
    }

    pub fn push_id<K: Hash>(
        &mut self,
        key: K,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        self.id(key, f);
    }

    pub fn for_each_keyed<I, K, T>(
        &mut self,
        items: I,
        mut f: impl FnMut(&mut ImUiFacade<'_, '_, H>, &K, T),
    ) where
        I: IntoIterator<Item = (K, T)>,
        K: Hash,
    {
        let f = &mut f;
        for (key, item) in items {
            self.id(&key, |ui| f(ui, &key, item));
        }
    }

    /// Disable all `imui`-facade interactions within the closure and dim visuals (ImGui-style
    /// `BeginDisabled/EndDisabled`).
    ///
    /// Notes:
    /// - This is scoped to the closure (Rust-friendly) rather than a manual begin/end pair.
    /// - The disabled alpha multiplier is controlled by theme number
    ///   `component.imui.disabled_alpha` (default `0.60`).
    pub fn disabled_scope(
        &mut self,
        disabled: bool,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        if !disabled {
            f(self);
            return;
        }

        let was_disabled = self.with_cx_mut(|cx| imui_is_disabled(cx));
        if was_disabled {
            f(self);
            return;
        }

        let build_focus = self.build_focus.clone();
        let element = self.with_cx_mut(|cx| {
            let depth = disabled_scope_depth_for(cx);
            let _guard = DisabledScopeGuard::push(depth);
            let alpha = disabled_alpha_for(cx);
            cx.pointer_region(PointerRegionProps::default(), |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|_host, _acx, _down| true));
                cx.pointer_region_on_pointer_up(Arc::new(|_host, _acx, _up| true));
                vec![cx.opacity(alpha, |cx| {
                    vec![cx.focus_traversal_gate(false, |cx| {
                        prepare_imui_runtime_for_frame(cx);
                        let mut out = Vec::new();
                        let mut ui = ImUiFacade {
                            cx,
                            out: &mut out,
                            build_focus,
                        };
                        f(&mut ui);
                        out
                    })]
                })]
            })
        });
        self.add(element);
    }
}

impl<'cx, 'a, H: UiHost> UiWriter<H> for ImUiFacade<'cx, 'a, H> {
    fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {
        f(self.cx)
    }

    fn add(&mut self, element: AnyElement) {
        self.out.push(element);
    }
}
