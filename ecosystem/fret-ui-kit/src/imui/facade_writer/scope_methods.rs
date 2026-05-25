use super::*;

pub(super) fn push_id<H, W, K, R>(
    ui: &mut W,
    key: K,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>) -> R,
) -> R
where
    H: UiHost,
    W: UiWriter<H> + ?Sized,
    K: Hash,
{
    let mut result = None;
    let elements = ui.with_cx_mut(|cx| {
        cx.keyed(key, |cx| {
            prepare_imui_runtime_for_frame(cx);
            let mut out = Vec::new();
            let mut child_ui = ImUiFacade {
                cx,
                out: &mut out,
                build_focus: None,
            };
            result = Some(f(&mut child_ui));
            out
        })
    });
    ui.extend(elements);
    result.expect("imui push_id closure should produce a result")
}

pub(super) fn disabled_scope<H, W>(
    ui: &mut W,
    disabled: bool,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    H: UiHost,
    W: UiWriter<H> + ?Sized,
{
    if !disabled {
        let elements = ui.with_cx_mut(|cx| {
            prepare_imui_runtime_for_frame(cx);
            let mut out = Vec::new();
            let mut child_ui = ImUiFacade {
                cx,
                out: &mut out,
                build_focus: None,
            };
            f(&mut child_ui);
            out
        });
        ui.extend(elements);
        return;
    }

    enum Built {
        Inline(Vec<AnyElement>),
        Wrapped(Box<AnyElement>),
    }

    let built = ui.with_cx_mut(|cx| {
        let depth = disabled_scope_depth_for(cx);
        let was_disabled = depth.get() > 0;
        let _guard = DisabledScopeGuard::push(depth);

        let build_children = |cx: &mut ElementContext<'_, H>| {
            prepare_imui_runtime_for_frame(cx);
            let mut out = Vec::new();
            let mut child_ui = ImUiFacade {
                cx,
                out: &mut out,
                build_focus: None,
            };
            f(&mut child_ui);
            out
        };

        if was_disabled {
            Built::Inline(build_children(cx))
        } else {
            let alpha = disabled_alpha_for(cx);
            Built::Wrapped(Box::new(cx.pointer_region(
                PointerRegionProps::default(),
                |cx| {
                    cx.pointer_region_on_pointer_down(Arc::new(|_host, _acx, _down| true));
                    cx.pointer_region_on_pointer_up(Arc::new(|_host, _acx, _up| true));
                    vec![cx.opacity(alpha, |cx| {
                        vec![cx.focus_traversal_gate(false, |cx| build_children(cx))]
                    })]
                },
            )))
        }
    });

    match built {
        Built::Inline(elements) => ui.extend(elements),
        Built::Wrapped(element) => ui.add(*element),
    }
}
