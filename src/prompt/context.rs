use super::{Buffer, Completer, Direction, Range, Scope, Suggester, Writer};

use crate::Error;

pub(super) struct Context<'c, 's, C, S>
where
    C: Completer + ?Sized,
    S: Suggester + ?Sized,
{
    writer: Writer,
    buffer: Buffer,
    completer: Option<&'c C>,
    completion: Option<std::borrow::Cow<'c, str>>,
    suggester: Option<&'s S>,
    suggestions: Option<Suggestions<'s>>,
}

impl<'c, 's, C, S> Context<'c, 's, C, S>
where
    C: Completer + ?Sized,
    S: Suggester + ?Sized,
{
    pub(super) fn new(
        erase_on_drop: bool,
        prompt: Option<&str>,
        buffer: Option<Buffer>,
        completer: Option<&'c C>,
        suggester: Option<&'s S>,
    ) -> Result<Self, Error> {
        Ok(Self {
            writer: Writer::new(erase_on_drop, prompt)?,
            buffer: buffer.unwrap_or_else(Buffer::new),
            completer,
            completion: None,
            suggester,
            suggestions: None,
        })
    }

    pub(super) fn buffer_as_string(&mut self) -> String {
        self.try_take_suggestion();
        self.buffer.to_string()
    }

    pub(super) fn print(&mut self) -> Result<(), Error> {
        self.writer.print(&self.buffer, self.completion.as_deref())
    }

    pub(super) fn write(&mut self, c: char) -> Result<(), Error> {
        self.try_take_suggestion();
        self.buffer.write(c);
        self.update_completion();
        self.writer.print(&self.buffer, self.completion.as_deref())
    }

    pub(super) fn delete(&mut self, scope: Scope) -> Result<(), Error> {
        self.try_take_suggestion();
        self.buffer.delete(scope);
        self.update_completion();
        self.writer.print(&self.buffer, self.completion.as_deref())
    }

    pub(super) fn move_cursor(&mut self, range: Range, direction: Direction) -> Result<(), Error> {
        self.try_take_suggestion();
        self.buffer.move_cursor(range, direction);
        self.writer.print(&self.buffer, self.completion.as_deref())
    }

    // Allowed because using map requires a `self` borrow
    #[allow(clippy::option_if_let_else)]
    pub(super) fn complete(&mut self, range: Range) -> Result<(), Error> {
        self.buffer.go_to_end();
        if let Some(completion) = &self.completion {
            if completion.is_empty() {
                Ok(())
            } else {
                self.buffer.write_range(&completion, range);
                self.update_completion();
                self.writer.print(&self.buffer, self.completion.as_deref())
            }
        } else {
            Ok(())
        }
    }

    fn update_completion(&mut self) {
        if let Some(completer) = self.completer {
            self.completion = completer.complete_for(self);
        }
    }

    pub(super) fn suggest(&mut self, direction: Direction) -> Result<(), Error> {
        if let Some(suggester) = &self.suggester {
            if let Some(suggestions) = &mut self.suggestions {
                suggestions.cycle(direction);
                if let Some(index) = suggestions.index {
                    return self
                        .writer
                        .print_suggestions(index, suggestions.options.as_ref());
                }
            } else {
                let options = suggester.suggest_for(self);
                if !options.is_empty() {
                    self.suggestions = Some(Suggestions::new(options, direction));
                    let suggestions = self.suggestions.as_ref().unwrap();
                    return self
                        .writer
                        .print_suggestions(suggestions.index.unwrap(), &suggestions.options);
                }
            }
        }

        self.writer.print(&self.buffer, self.completion.as_deref())
    }

    pub(super) fn is_suggesting(&self) -> bool {
        self.suggestions.is_some()
    }

    pub(super) fn cancel_suggestion(&mut self) -> Result<(), Error> {
        self.suggestions = None;
        self.writer.print(&self.buffer, self.completion.as_deref())
    }

    fn try_take_suggestion(&mut self) {
        if let Some(suggestion) = self.suggestions.take().and_then(Suggestions::take) {
            self.buffer = suggestion;
        }
    }
}

impl<C, S> std::convert::Into<Buffer> for Context<'_, '_, C, S>
where
    C: Completer + ?Sized,
    S: Suggester + ?Sized,
{
    fn into(self) -> Buffer {
        self.buffer
    }
}

impl<C, S> std::ops::Deref for Context<'_, '_, C, S>
where
    C: Completer + ?Sized,
    S: Suggester + ?Sized,
{
    type Target = Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

struct Suggestions<'a> {
    index: Option<usize>,
    options: Vec<std::borrow::Cow<'a, str>>,
}

impl<'a> Suggestions<'a> {
    fn new(options: Vec<std::borrow::Cow<'a, str>>, direction: Direction) -> Self {
        let index = match direction {
            Direction::Forward => 0,
            Direction::Backward => options.len() - 1,
        };

        Self {
            options,
            index: Some(index),
        }
    }

    // Allowed because it is more readable
    #[allow(clippy::match_same_arms)]
    fn cycle(&mut self, direction: Direction) {
        use Direction::{Backward, Forward};

        let last_index = self.options.len() - 1;

        self.index = match (direction, self.index) {
            (Forward, None) => Some(0),
            (Forward, Some(index)) if index < last_index => Some(index + 1),
            (Forward, _) => None,
            (Backward, None) => Some(last_index),
            (Backward, Some(0)) => None,
            (Backward, Some(index)) => Some(index - 1),
        };
    }

    fn take(mut self) -> Option<Buffer> {
        self.index
            .map(|index| Buffer::from(self.options.swap_remove(index)))
    }
}
