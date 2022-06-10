#![deny(warnings, missing_docs, clippy::pedantic, clippy::all)]
#![warn(rust_2018_idioms)]

//! Rucline, the Rust CLI Line reader, or simply "recline", is a cross-platform, UTF-8 compatible
//! line reader that provides hooks for autocompletion and drop-down suggestion. It supports advanced
//! editing [`actions`] and hooks for customizing the line reader behavior making it more flexible
//! than simply reading from `stdin`.
//!
//! ### Basic usage:
//!
//! ```no_run
//! use rucline::Outcome::Accepted;
//! use rucline::prompt::{Builder, Prompt};
//!
//! if let Ok(Accepted(string)) = Prompt::from("What's you favorite website? ")
//!     // Add some tab completions (Optional)
//!     .suggester(vec![
//!         "https://www.rust-lang.org/",
//!         "https://docs.rs/",
//!         "https://crates.io/",
//!     ])
//!     //Block until value is ready
//!     .read_line()
//! {
//!     println!("'{}' seems to be your favorite website", string);
//! }
//! ```
//!
//! # Actions
//!
//! Rucline's behavior can be customized and composed with use of [`actions`].
//!
//! There is a built-in set of default [`actions`] that will be executed upon user interaction.
//! These are meant to feel natural when coming from the default terminal, while also adding further
//! functionality and editing commands. For example, a few of the built-ins:
//! * `Tab`: cycle through completions
//! * `Shift` + `Tab`: cycle through completions in reverse
//! * `CTRL` + `W`: delete the current word
//! * `CTRL` + `J`: delete until the beginning of the word
//! * `CTRL` + `K`: delete until the end of the word
//! * `CTRL` + `U`: delete the whole line
//! * `CTRL` + `H`: delete until the beggining of the line
//! * `CTRL` + `L`: delete until the end of the line
//!
//! > See [`Action`] for the full default behavior specification
//!
//! The default behavior can be customized by overriding user [`events`] with [`actions`]. Which
//! in turn can be serialized, stored, and loaded at run-time with the `config-serde` feature flag.
//!
//!
//! ### Overriding key bindings
//!
//! ```no_run
//! use rucline::Outcome::Accepted;
//! use rucline::actions::{Action, Event, KeyBindings, KeyCode, Range};
//! use rucline::prompt::{Builder, Prompt};
//!
//! let mut bindings = KeyBindings::new();
//!
//! // Accept the full suggestions if `right` is pressed
//! bindings.insert(Event::from(KeyCode::Right), Action::Complete(Range::Line));
//!
//! if let Ok(Accepted(string)) = Prompt::from("What's you favorite website? ")
//!     // Add some likely values as completions
//!     .completer(vec![
//!         "https://www.rust-lang.org/",
//!         "https://docs.rs/",
//!         "https://crates.io/",
//!     ])
//!     // Set the new key bindings as an override
//!     .overrider(bindings)
//!     //Block until value is ready
//!     .read_line()
//! {
//!     println!("'{}' seems to be your favorite website", string);
//! }
//! ```
//!
//! [`actions`]: actions/enum.Action.html
//! [`Action`]: actions/index.html#default-behavior
//! [`events`]: actions/type.Event.html
pub mod actions;
mod buffer;
pub mod completion;
pub mod prompt;

pub use buffer::Buffer;
pub use prompt::Outcome;
pub use prompt::Prompt;

pub use crossterm;

/// Alias to `crossterm::ErrorKind` from [`crossterm`](https://docs.rs/crossterm/).
pub use crossterm::ErrorKind as Error;
