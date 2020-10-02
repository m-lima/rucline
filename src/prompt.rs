//! Provides a prompt for user input that can be customized with [`actions`] and [`completions`].
//!
//! ### Basic usage:
//!
//! ```no_run
//! use rucline::completion;
//! use rucline::Prompt;
//!
//! if let Ok(Some(string)) = Prompt::from("What's you favorite website? ")
//!     // Add some tab completions (Optional)
//!     .suggester(&completion::Basic::new(&[
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

mod buffer;
mod char_string;
mod context;
mod navigation;
mod writer;

use buffer::Buffer;
use char_string::{CharString, CharStringView};
use context::ContextImpl;
use writer::Writer;

use crate::actions::{action_for, Action, Direction, Overrider, Range, Scope};
use crate::completion::{Completer, Suggester};

/// Represents and stores a prompt that shall be presented to the user for input.
///
/// When built, the prompt will have no customization or completions. Also the default
/// [`erase_after_read`] is `true`.
///
/// [`erase_after_read`]: struct.Prompt.html#method.erase_after_read
pub struct Prompt<'o, 'c, 's> {
    erase_after_read: bool,
    text: Option<CharString>,
    overrider: Option<&'o dyn Overrider>,
    completer: Option<&'c dyn Completer>,
    suggester: Option<&'s dyn Suggester>,
}

impl<'o, 'c, 's> Prompt<'o, 'c, 's> {
    /// Creates a new empty prompt, with no prompt text.
    #[must_use]
    pub fn new() -> Self {
        Prompt::default()
    }

    /// Modifies the prompt text
    ///
    /// # Arguments
    ///
    /// * `string` - The new prompt text
    #[must_use]
    // Allowed because `impl ToString` doesn't necessarily need to consume `string`
    #[allow(clippy::needless_pass_by_value)]
    pub fn text(mut self, string: impl ToString) -> Self {
        self.text = Some(string.to_string().into());
        self
    }

    /// Removes the current prompt text, leaving it empty;
    #[must_use]
    pub fn remove_text(mut self) -> Self {
        self.text = None;
        self
    }

    /// Controls if the prompt shall be erased after user input.
    ///
    /// If set to `false` (default), after user input, the terminal will receive a new line
    /// after the prompt text and the user input. Any drop-down completions will be removed,
    /// however.
    ///
    /// If set to `true`, the whole prompt and input will be erased. The cursor returns to the
    /// original position as if nothing happened.
    #[must_use]
    pub fn erase_after_read(mut self, erase_after_read: bool) -> Self {
        self.erase_after_read = erase_after_read;
        self
    }

    /// Modifies the behavior of the prompt by setting a [`Overrider`].
    ///
    /// # Arguments
    ///
    /// * [`overrider`] - The new overrider
    ///
    /// [`Overrider`]: ../actions/trait.Overrider.html
    #[must_use]
    pub fn overrider(mut self, overrider: &'o dyn Overrider) -> Self {
        self.overrider = Some(overrider);
        self
    }

    /// Removes the current overrider, returning to the default prompt behavior.
    #[must_use]
    pub fn remove_overrider(mut self) -> Self {
        self.overrider = None;
        self
    }

    /// Sets the in-line completion provider.
    ///
    /// # Arguments
    ///
    /// * [`completer`] - The new completer
    ///
    /// [`Completer`]: ../completion/trait.Completer.html
    #[must_use]
    pub fn completer(mut self, completer: &'c dyn Completer) -> Self {
        self.completer = Some(completer);
        self
    }

    /// Removes the current in-line completion provider. No in-line completions will be presented.
    #[must_use]
    pub fn remove_completer(mut self) -> Self {
        self.completer = None;
        self
    }

    /// Sets the drop-down suggestion provider.
    ///
    /// # Arguments
    ///
    /// * [`suggester`] - The new suggester
    ///
    /// [`Suggester`]: ../completion/trait.Suggester.html
    #[must_use]
    pub fn suggester(mut self, suggester: &'s dyn Suggester) -> Self {
        self.suggester = Some(suggester);
        self
    }

    /// Removes the current drop-down suggestion provider. No drop-down suggestions will be
    /// presented.
    #[must_use]
    pub fn remove_suggester(mut self) -> Self {
        self.suggester = None;
        self
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
    /// * `Option<String>` - A string containing the user input, or `None` if the user has
    /// cancelled the input.
    ///
    /// # Errors
    /// * [`ErrorKind`] - If an error occurred while reading the user input.
    ///
    /// [`Prompt`]: struct.Prompt.html
    /// [`ErrorKind`]: ../enum.ErrorKind.html
    pub fn read_line(&self) -> Result<Option<String>, crate::ErrorKind> {
        let mut context = ContextImpl::new(
            self.erase_after_read,
            self.text.as_ref(),
            self.completer,
            self.suggester,
        )?;

        context.print()?;
        loop {
            if let crossterm::event::Event::Key(e) = crossterm::event::read()? {
                match action_for(self.overrider, e, &context) {
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
                            return Ok(None);
                        }
                    }
                    Action::Accept => return Ok(Some(context.buffer_as_string())),
                }
            }
        }
    }
}

impl Default for Prompt<'_, '_, '_> {
    fn default() -> Self {
        Self {
            erase_after_read: false,
            text: None,
            overrider: None,
            completer: None,
            suggester: None,
        }
    }
}

impl<S: ToString> std::convert::From<S> for Prompt<'_, '_, '_> {
    fn from(string: S) -> Self {
        Self {
            erase_after_read: false,
            text: Some(string.to_string().into()),
            overrider: None,
            completer: None,
            suggester: None,
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
