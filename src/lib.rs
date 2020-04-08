#![deny(warnings, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![warn(rust_2018_idioms)]

pub mod key_bindings;
pub mod prompt;

pub use crossterm::ErrorKind;
