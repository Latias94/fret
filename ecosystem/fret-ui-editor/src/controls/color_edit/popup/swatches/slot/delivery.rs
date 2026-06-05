use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost, UiActionHostAdapter};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::super::drag_drop::{ColorDragDropStore, take_delivered_color_drop};
use super::super::super::super::{
    ColorEditDragDropOptions, ColorEditPaletteEntry, ColorEditPaletteSlotDrop,
    OnColorEditPaletteSlotDrop,
};

pub(super) struct PresetSwatchDropDeliveryArgs {
    pub(super) index: usize,
    pub(super) entry: ColorEditPaletteEntry,
    pub(super) enabled: bool,
    pub(super) drag_drop_options: ColorEditDragDropOptions,
    pub(super) drag_drop_store: Model<ColorDragDropStore>,
    pub(super) swatch_id: GlobalElementId,
    pub(super) on_palette_slot_drop: Option<OnColorEditPaletteSlotDrop>,
}

pub(super) fn deliver_preset_swatch_drop<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: PresetSwatchDropDeliveryArgs,
) {
    if !args.enabled || !args.drag_drop_options.enabled {
        return;
    }

    let Some(on_palette_slot_drop) = args.on_palette_slot_drop else {
        return;
    };

    let Some(payload) = take_delivered_color_drop(cx, &args.drag_drop_store, args.swatch_id) else {
        return;
    };

    let action_cx = ActionCx {
        window: cx.window,
        target: args.swatch_id,
    };
    let event = ColorEditPaletteSlotDrop::new(args.index, args.entry, payload);
    let mut host = UiActionHostAdapter { app: cx.app };
    on_palette_slot_drop(&mut host, action_cx, event);
    host.request_redraw(action_cx.window);
}
