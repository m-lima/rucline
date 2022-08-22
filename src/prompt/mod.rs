//! Provides a method for presenting a prompt for user input that can be customized with [`actions`]
//! and [`completions`].
//!
//! The core functionality of this module is [`read_line`]. Its invocation can be cumbersome due
//! to required type annotations, therefore this module also provider a [`Builder`] which helps to
//! craft the invocation to [`read_line`].
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
//! [`actions`]: ../actions/enum.Action.html
//! [`completions`]: ../completion/index.html
//! [`read_line`]: fn.read_line.html
//! [`Builder`]: trait.Builder.html

mod builder;
mod context;
mod writer;

use context::Context;
use writer::Writer;

use crate::actions::{action_for, Action, Direction, Overrider, Range, Scope};
use crate::completion::{Completer, Suggester};
use crate::Buffer;

pub use builder::{Builder, Prompt};

/// The outcome of [`read_line`], being either accepted or canceled by the user.
///
/// [`read_line`]: fn.read_line.html
pub enum Outcome {
    /// If the user accepts the prompt input, i.e. an [`Accept`] event was emitted. this variant will
    /// contain the accepted text.
    ///
    /// [`Accept`]: ../actions/enum.Action.html#variant.Accept
    Accepted(String),
    /// If the user cancels the prompt input, i.e. a [`Cancel`] event was emitted. this variant will
    /// contain the rejected buffer, with text and cursor position intact from the moment of
    /// rejection.
    ///
    /// [`Cancel`]: ../actions/enum.Action.html#variant.Cancel
    Canceled(Buffer),
}

impl Outcome {
    /// Returns true if the outcome was accepted.
    #[must_use]
    pub fn was_accepted(&self) -> bool {
        matches!(self, Outcome::Accepted(_))
    }

    /// Returns the accepted text.
    ///
    /// # Panics
    ///
    /// Panics if the [`Outcome`] is [`Canceled`]
    ///
    /// [`Outcome`]: enum.Outcome.html
    /// [`Canceled`]: enum.Outcome.html#variant.Canceled
    #[must_use]
    pub fn unwrap(self) -> String {
        if let Outcome::Accepted(string) = self {
            string
        } else {
            panic!("called `Outcome::unwrap()` on a `Canceled` value")
        }
    }

    /// Converts this [`Outcome`] into an [`Option`] of the accepted text.
    ///
    /// # Return
    /// * `Some(String)` - If the [`Outcome`] is [`accepted`].
    /// * `None` - If the [`Outcome`] is [`canceled`].
    ///
    /// [`Outcome`]: enum.Outcome.html
    /// [`Option`]: std::option::Option
    /// [`accepted`]: enum.Outcome.html#variant.Accepted
    /// [`canceled`]: enum.Outcome.html#variant.Canceled
    #[must_use]
    pub fn some(self) -> Option<String> {
        match self {
            Outcome::Accepted(string) => Some(string),
            Outcome::Canceled(_) => None,
        }
    }

    /// Converts this [`Outcome`] into a [`Result`] containing the accepted text or the canceled buffer.
    ///
    /// # Return
    /// * `Ok(String)` - If the [`Outcome`] is [`accepted`].
    /// * `Err(Buffer)` - If the [`Outcome`] is [`canceled`].
    ///
    /// # Errors
    /// * [`Buffer`] - If the user canceled the input.
    ///
    /// [`Outcome`]: enum.Outcome.html
    /// [`Result`]: std::result::Result
    /// [`Buffer`]: ../buffer/struct.Buffer.html
    /// [`accepted`]: enum.Outcome.html#variant.Accepted
    /// [`canceled`]: enum.Outcome.html#variant.Canceled
    pub fn ok(self) -> Result<String, Buffer> {
        match self {
            Outcome::Accepted(string) => Ok(string),
            Outcome::Canceled(buffer) => Err(buffer),
        }
    }
}

// TODO: Support crossterm async
/// Analogous to `std::io::stdin().read_line()`, however providing all the customization
/// configured in the passed parameters.
///
/// This method will block until an input is committed by the user.
///
/// Calling this method directly can be cumbersome, therefore it is recommended to use the helper
/// [`Prompt`] or [`Builder`] to craft the call.
///
/// # Return
/// * [`Outcome`] - Either [`Accepted`] containing the user input, or [`Canceled`]
/// containing the rejected [`buffer`].
///
/// # Errors
/// * [`Error`] - If an error occurred while reading the user input.
///
/// [`Accepted`]: enum.Outcome.html#variant.Accepted
/// [`Builder`]: trait.Builder.html
/// [`Canceled`]: enum.Outcome.html#variant.Canceled
/// [`Error`]: ../enum.Error.html
/// [`Outcome`]: enum.Outcome.html
/// [`Prompt`]: struct.Prompt.html
/// [`buffer`]: ../buffer/struct.Buffer.html
pub fn read_line<O, C, S>(
    prompt: Option<&str>,
    buffer: Option<Buffer>,
    erase_after_read: bool,
    display_suggestion_options: bool,
    overrider: Option<&O>,
    completer: Option<&C>,
    suggester: Option<&S>,
) -> Result<Outcome, crate::Error>
where
    O: Overrider + ?Sized,
    C: Completer + ?Sized,
    S: Suggester + ?Sized,
{
    let mut context = Context::new(
        erase_after_read,
        display_suggestion_options,
        prompt,
        buffer,
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
