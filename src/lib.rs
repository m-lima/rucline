#![deny(warnings, missing_docs, clippy::pedantic, clippy::all)]
#![warn(rust_2018_idioms)]
// TDO: Remove when ready
#![allow(missing_docs, clippy::missing_errors_doc)]

//! Rucline (Rust CLI line /rɪˈklaɪn/) is a cross-platform, UTF-8 aware, advanced edtigin,
//! autocompletion capable, tab suggestion supporting line reader you can "recline" on.
//!
//! It provides advanced editing [`actions`] for user input and customization of the line
//! reader.
//!
//! It uses [`crossterm`] as a backend to provide cross-platform support, and provides advanced
//!
//! ### Basic usage:
//!
//! ```no_run
//! use rucline::completion;
//! use rucline::Prompt;
//!
//! if let Ok(Some(string)) = Prompt::new()
//!     // Create a bold prompt
//!     .prompt("What's you favorite website? ")
//!     // Add some likely values as completions
//!     .completer(completion::Basic::new(&[
//!         "https://www.rust-lang.org/",
//!         "https://docs.rs/",
//!         "https://crates.io/",
//!     ]))
//!     // Add some tab completions
//!     .suggester(completion::Basic::new(&[
//!         "https://www.startpage.com/",
//!         "https://www.google.com/",
//!     ]))
//!     //Block until value is ready
//!     .read_line()
//! {
//!     println!("'{}' seems to be your favorite website", string);
//! }
//! ```
//!
//! # Actions
//!
//! Rucline allow advanced [`actions`] for interacting with the [`Prompt`], but it
//! comes built-in with useful behavior. For example, a few of the build-ins:
//! * `Tab`: cycle through completions
//! * `Shift` + `Tab`: cycle through completions in reverse
//! * `CTRL` + `W`: delete the current work
//! * `CTRL` + `J`: delete the beginning of the word
//! * `CTRL` + `K`: delete the end of the word
//! * `CTRL` + `U`: delete the line
//! * `CTRL` + `H`: delete the beggining of the line
//! * `CTRL` + `L`: delete the end of the line
//!
//! **See [`Action`][`actions`] for the full default behavior**
//!
//! The behavior can be customized by overriding user [`events`] with [`actions`]. Which
//! in turn can be serialized, stored, and loaded at run-time.
//!
//!
//! ### Overriding key bindings
//!
//! ```no_run
//! use rucline::{completion, Prompt};
//! use rucline::key_bindings::{Action, Event, KeyBindings, Range};
//! use crossterm::event::KeyCode;
//!
//! let mut bindings = KeyBindings::new();
//!
//! // Accept the full suggestions if `right` is pressed
//! bindings.insert(Event::from(KeyCode::Right), Action::Complete(Range::Line));
//!
//! if let Ok(Some(string)) = Prompt::new()
//!     // Create a bold prompt
//!     .prompt("What's you favorite website? ")
//!     // Add some likely values as completions
//!     .completer(completion::Basic::new(&[
//!         "https://www.rust-lang.org/",
//!         "https://docs.rs/",
//!         "https://crates.io/",
//!     ]))
//!     // Override the `right` key to always fill the full suggestions line
//!     .bindings(bindings)
//!     //Block until value is ready
//!     .read_line()
//! {
//!     println!("'{}' seems to be your favorite website", string);
//! }
//! ```
//!
//! [`crossterm`]: https://docs.rs/crossterm/
//! [`KeyBindings`]: key_bindings/index.html
//! [`actions`]: key_bindings/enum.Action.html
//! [`events`]: key_bindings/type.Event.html
//! [`prompt`]: prompt/index.html
pub mod completion;
pub mod key_bindings;
pub mod prompt;

pub use prompt::Prompt;

pub use crossterm::ErrorKind;
