//! wasm-targeted compile smoke for the `imui` authoring surface.
//!
//! This crate exists to ensure `fret-authoring` + `fret-imui` remain portable and can be
//! type-checked for `wasm32-unknown-unknown` without pulling in platform backends.
//!
//! Run:
//! - `cargo check -p fret-imui-wasm-smoke --target wasm32-unknown-unknown`

#![deny(deprecated)]

use fret_authoring::UiWriter;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

#[allow(dead_code)]
fn writer_surface_smoke<H: UiHost>(ui: &mut impl UiWriter<H>) {
    ui.with_cx_mut(|_cx| ());
    ui.extend(Vec::<AnyElement>::new());
    ui.mount(|_cx| Vec::<AnyElement>::new());
    ui.keyed("key", |_cx| ());
    ui.keyed(123_u64, |_cx| ());
}

#[allow(dead_code)]
fn imui_entry_point_smoke<'cx, H: UiHost>(cx: &mut ElementContext<'cx, H>) {
    let _ = fret_imui::imui(cx, |_ui| {});
}
