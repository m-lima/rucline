mod buffer;
mod char_string;
mod navigation;
mod writer;

use buffer::Buffer;
use char_string::{CharString, CharStringView};
use context::ContextImpl;
use writer::Writer;

pub mod context;
pub use context::Context;

use crate::actions::{action_for, Action, Direction, Overrider, Range, Scope};
use crate::completion::{Completer, Suggester};

pub struct Prompt {
    erase_after_read: bool,
    prompt: Option<CharString>,
    overrider: Option<Box<dyn Overrider>>,
    completer: Option<Box<dyn Completer>>,
    suggester: Option<Box<dyn Suggester>>,
}

impl Prompt {
    #[must_use]
    pub fn new() -> Self {
        Prompt::default()
    }

    #[must_use]
    pub fn erase_after_read(mut self, erase_after_read: bool) -> Self {
        self.erase_after_read = erase_after_read;
        self
    }

    #[must_use]
    pub fn overrider(mut self, overrider: impl Overrider + 'static) -> Self {
        self.overrider = Some(Box::new(overrider));
        self
    }

    #[must_use]
    pub fn completer(mut self, completer: impl Completer + 'static) -> Self {
        self.completer = Some(Box::new(completer));
        self
    }

    #[must_use]
    pub fn suggester(mut self, suggester: impl Suggester + 'static) -> Self {
        self.suggester = Some(Box::new(suggester));
        self
    }

    // TODO: Support crossterm async
    pub fn read_line(&self) -> Result<Option<String>, crate::ErrorKind> {
        let mut context = ContextImpl::new(
            self.erase_after_read,
            self.prompt.as_ref(),
            &self.completer,
            &self.suggester,
        )?;

        context.print()?;
        loop {
            if let crossterm::event::Event::Key(e) = crossterm::event::read()? {
                match action_for(&self.overrider, e, &context) {
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

impl Default for Prompt {
    fn default() -> Self {
        Self {
            erase_after_read: false,
            prompt: None,
            overrider: None,
            completer: None,
            suggester: None,
        }
    }
}

// TODO: Avoid the `to_string()` and incorporate Colorize into char_string
impl<S: ToString> std::convert::From<S> for Prompt {
    fn from(string: S) -> Self {
        Self {
            erase_after_read: false,
            prompt: Some(string.to_string().into()),
            overrider: None,
            completer: None,
            suggester: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

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

        let prompt = Prompt::from("My prompt".green());

        assert_eq!(
            prompt.prompt.unwrap().len(),
            format!("{}", "My prompt".green()).len()
        )
    }
}
