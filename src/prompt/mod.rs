//! Provides a prompt for user input that can be customized with [`actions`] and [`completions`].
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
//! TODO: Write about the builder here
//!
//! [`actions`]: ../actions/enum.Action.html
//! [`completions`]: ../completion/index.html

mod builder;
mod context;
mod writer;

use context::Context;
use writer::Writer;

use crate::actions::{action_for, Action, Direction, Overrider, Range, Scope};
use crate::buffer::Buffer;
use crate::completion::{Completer, Suggester};

pub use builder::{Builder, Prompt};

pub enum Outcome {
    Accepted(String),
    Canceled(Buffer),
}

impl Outcome {
    pub fn was_acceoted(&self) -> bool {
        if let Outcome::Accepted(_) = self {
            true
        } else {
            false
        }
    }

    pub fn unwrap(self) -> String {
        if let Outcome::Accepted(string) = self {
            string
        } else {
            panic!("called `Outcome::unwrap()` on a `Canceled` value")
        }
    }

    pub fn some(self) -> Option<String> {
        if let Outcome::Accepted(string) = self {
            Some(string)
        } else {
            None
        }
    }

    pub fn ok(self) -> Result<String, Buffer> {
        match self {
            Outcome::Accepted(string) => Ok(string),
            Outcome::Canceled(buffer) => Err(buffer),
        }
    }
}

// TODO: Support crossterm async
/// Blocks until an input is committed by the user.
///
///
/// Analogous to `std::io::stdin().read_line()`, however providing all the customization
/// configured in the [`Prompt`].
///
/// # Return
///
/// * `Outcome` - Either `Accepted(String)` containing the user input, or `Canceled(Buffer)`
/// containing the rejected buffer.
///
/// # Errors
/// * [`ErrorKind`] - If an error occurred while reading the user input.
///
/// [`Prompt`]: struct.Prompt.html
/// [`ErrorKind`]: ../enum.ErrorKind.html
pub fn read_line<'o, 'c, 's, O, C, S>(
    prompt: Option<&str>,
    buffer: Option<Buffer>,
    erase_after_read: bool,
    overrider: Option<&'o O>,
    completer: Option<&'c C>,
    suggester: Option<&'s S>,
) -> Result<Outcome, crate::ErrorKind>
where
    O: Overrider + ?Sized,
    C: Completer + ?Sized,
    S: Suggester + ?Sized,
{
    let mut context = Context::new(
        erase_after_read,
        prompt.as_deref(),
        buffer.clone(),
        completer,
        suggester,
    )?;

    context.print()?;
    loop {
        if let crossterm::event::Event::Key(e) = crossterm::event::read()? {
            match action_for(overrider, e, &context) {
                Action::Write(c) => context.write(c)?,
                Action::Delete(scope) => context.delete(scope)?,
                Action::Move(range, direction) => context.move_cursor(range, direction)?,
                Action::Complete(range) => context.complete(range)?,
                Action::Suggest(direction) => context.suggest(direction)?,
                Action::Noop => continue,
                Action::Cancel => {
                    if context.is_suggesting() {
                        context.cancel_suggestion()?;
                    } else {
                        return Ok(Outcome::Canceled(context.into()));
                    }
                }
                Action::Accept => return Ok(Outcome::Accepted(context.buffer_as_string())),
            }
        }
    }
}
