#![deny(warnings, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![warn(rust_2018_idioms)]

pub mod key_bindings;
pub mod completer;
pub mod prompt;

pub use prompt::Prompt;
pub use completer::Completer;

pub use crossterm::ErrorKind;
