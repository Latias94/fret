use fret_ui::ElementCx;
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, ContainerProps, Overflow, PressableProps, PressableState};
use fret_ui::elements::GlobalElementId;

/// Composes the recommended "control chrome" structure:
///
/// - outer `Pressable` remains `Overflow::Visible` so focus rings can extend outward
/// - inner chrome `Container` is forced to `Overflow::Clip` so rounded corners/borders mask content
///
/// This matches the common shadcn/Radix mental model of:
/// `Pressable (focus ring) -> SurfaceChrome (overflow-hidden) -> content`.
pub fn control_chrome_pressable_with_id_props<'a, H, F, C>(
    cx: &mut ElementCx<'a, H>,
    f: F,
) -> AnyElement
where
    H: UiHost + 'a,
    F: FnOnce(
        &mut ElementCx<'a, H>,
        PressableState,
        GlobalElementId,
    ) -> (PressableProps, ContainerProps, C),
    C: FnOnce(&mut ElementCx<'a, H>) -> Vec<AnyElement>,
{
    cx.pressable_with_id_props(|cx, st, id| {
        let (mut pressable_props, mut chrome_props, children) = f(cx, st, id);

        pressable_props.layout.overflow = Overflow::Visible;
        chrome_props.layout.overflow = Overflow::Clip;

        let content = cx.container(chrome_props, children);
        (pressable_props, vec![content])
    })
}
