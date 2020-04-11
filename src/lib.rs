#![deny(warnings, missing_docs, clippy::pedantic, clippy::all)]
#![warn(rust_2018_idioms)]
// TODO: Remove these when ready
#![allow(clippy::missing_errors_doc)]
#![allow(missing_docs)]

// TODO: Support crossterm async

pub mod completer;
pub mod key_bindings;
pub mod prompt;
pub mod suggester;

pub use completer::Completer;
pub use prompt::Prompt;
pub use suggester::Suggester;

pub use crossterm::ErrorKind;
