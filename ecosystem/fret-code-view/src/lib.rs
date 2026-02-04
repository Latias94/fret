//! Code view component(s) for Fret.

mod code_block;
mod copy_button;
#[cfg(feature = "imui")]
pub mod imui;
mod prepare;
mod syntax;

pub use code_block::{
    CodeBlock, CodeBlockCopyButtonPlacement, CodeBlockHeaderBackground, CodeBlockHeaderSlots,
    CodeBlockUiOptions, CodeBlockWrap, code_block, code_block_with, code_block_with_header_slots,
};
