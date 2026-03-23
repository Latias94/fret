//! Code view component(s) for Fret.
//!
//! This crate currently focuses on high-quality code fences:
//! - monospaced text layout + optional syntax highlighting,
//! - copy-to-clipboard affordances,
//! - configurable wrapping and header slots for actions.
//!
//! It is used by higher-level content renderers like `fret-markdown`.

mod code_block;
mod copy_button;
#[cfg(feature = "imui")]
pub mod imui;
mod prepare;
mod syntax;

pub use code_block::{
    CodeBlock, CodeBlockCopyButtonPlacement, CodeBlockHeaderBackground, CodeBlockHeaderSlots,
    CodeBlockUiOptions, CodeBlockWindowedOptions, CodeBlockWrap, code_block,
    code_block_non_windowed, code_block_windowed, code_block_with, code_block_with_header_slots,
    code_block_with_header_slots_non_windowed, code_block_with_header_slots_windowed,
    code_block_with_non_windowed, code_block_with_windowed,
};
