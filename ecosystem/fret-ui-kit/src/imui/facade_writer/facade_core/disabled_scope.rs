use super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
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
