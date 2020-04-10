#![deny(warnings, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::find_map)]
#![warn(rust_2018_idioms)]

pub mod completer;
pub mod key_bindings;
pub mod prompt;
pub mod suggester;

pub use completer::Completer;
pub use prompt::Prompt;
pub use suggester::Suggester;

pub use crossterm::ErrorKind;
