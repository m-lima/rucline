//! Provides a prompt for user input that can be customized with [`actions`] and [`completions`].
//!
//! ### Basic usage:
//!
//! ```no_run
//! use rucline::completion::Basic;
//! use rucline::Prompt;
//!
//! if let Ok(Some(string)) = Prompt::from("What's you favorite website? ")
//!     // Add some tab completions (Optional)
//!     .suggester(&Basic::new(&[
//!         "https://www.rust-lang.org/",
//!         "https://docs.rs/",
//!         "https://crates.io/",
//!     ]))
//!     //Block until value is ready
//!     .read_line()
//! {
//!     println!("'{}' seems to be your favorite website", string);
//! }
//! ```
//!
//! ## Reusing existing prompt:
//! The prompt customization allows chaining to make one-line usages more convenient.
//! This doesn't mean that a new prompt must be re-built everytime, but does mean that the
//! reference must be updated to the new version.
//!
//! ```no_run
//! use rucline::prompt::Prompt;
//!
//! let mut prompt = Prompt::from("First name: ").erase_after_read(true);
//! let first_name = prompt.read_line().unwrap();
//!
//! // Reassign the modified prompt
//! prompt = prompt.text("Last name: ").erase_after_read(false);
//! let last_name = prompt.read_line().unwrap();
//! ```
//!
//! [`crossterm`]: https://docs.rs/crossterm/
//! [`actions`]: ../actions/enum.Action.html
//! [`completions`]: ../completion/index.html
//! [`events`]: actions/type.Event.html
//! [`prompt`]: prompt/index.html

mod builder;
mod context;
mod writer;

use context::ContextImpl;
use writer::Writer;

use crate::actions::{action_for, Action, Direction, Overrider, Range, Scope};
use crate::buffer::Buffer;
use crate::completion::{Completer, Suggester};

pub use builder::{from, new, Builder};

/// The final outcome from reading the line.
/// TODO: Document more
pub enum Outcome {
    /// TODO
    Accepted(String),
    /// TODO
    Canceled(Buffer),
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
/// * `Outcome` - `Accepted(String)` containing the user input, or `Canceled(Buffer)` if the
/// user has cancelled the input, containing the current buffer.
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
    let mut context = ContextImpl::new(
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

#[cfg(test)]
mod test {
    use super::Prompt;

    #[test]
    fn can_reuse_prompt() {
        let mut prompt = Prompt::new().erase_after_read(true);
        assert!(prompt.erase_after_read);

        prompt = prompt.erase_after_read(false);
        assert!(!prompt.erase_after_read);
    }

    #[test]
    fn accept_decorated_prompt() {
        use colored::Colorize;

        let mut prompt = Prompt::from("My prompt".green());

        assert_eq!(
            prompt.text.take().unwrap().len(),
            format!("{}", "My prompt".green()).len()
        );

        prompt = prompt.text("My prompt".blue());

        assert_eq!(
            prompt.text.unwrap().len(),
            format!("{}", "My prompt".blue()).len()
        );
    }

    #[test]
    fn remove_text() {
        let mut prompt = Prompt::new();
        assert!(prompt.text.is_none());

        prompt = prompt.text("Bla");
        match &prompt.text {
            Some(text) => assert_eq!(text.to_string(), "Bla"),
            None => panic!(),
        };

        prompt = prompt.remove_text();
        assert!(prompt.text.is_none());
    }
}
