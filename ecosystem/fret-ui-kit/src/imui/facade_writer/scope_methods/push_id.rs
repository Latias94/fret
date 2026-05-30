use super::super::*;

pub(in crate::imui::facade_writer) fn push_id<H, W, K, R>(
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
