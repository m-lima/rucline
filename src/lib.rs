#![deny(warnings, missing_docs, clippy::pedantic, clippy::all)]
#![warn(rust_2018_idioms)]
// TODO: Remove these when ready
#![allow(clippy::missing_errors_doc)]
#![allow(missing_docs)]

pub mod completion;
pub mod key_bindings;
pub mod prompt;

pub use completion::*;
pub use prompt::Prompt;

pub use crossterm::ErrorKind;
