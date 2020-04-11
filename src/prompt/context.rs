use super::{
    navigation, Buffer, CharString, CharStringView, Completer, Direction, Range, Scope, Suggester,
    Writer,
};

pub(super) struct Context<'a> {
    writer: Writer,
    buffer: Buffer,
    completer: &'a Option<Box<dyn Completer>>,
    completion: Option<CharStringView<'a>>,
    suggester: &'a Option<Box<dyn Suggester>>,
    suggestion: Option<usize>,
}

impl<'a> Context<'a> {
    pub(super) fn new(
        erase_on_drop: bool,
        prompt: Option<&CharString>,
        completer: &'a Option<Box<dyn Completer>>,
        suggester: &'a Option<Box<dyn Suggester>>,
    ) -> Result<Self, crate::ErrorKind> {
        Ok(Self {
            writer: Writer::new(erase_on_drop, prompt)?,
            buffer: Buffer::new(),
            completer,
            completion: None,
            suggester,
            suggestion: None,
        })
    }

    pub(super) fn buffer_as_string(&self) -> String {
        self.buffer.to_string()
    }

    pub(super) fn print(&mut self) -> Result<(), crate::ErrorKind> {
        self.writer.print(&self.buffer, self.completion)
    }

    pub(super) fn write(&mut self, c: char) -> Result<(), crate::ErrorKind> {
        self.buffer.write(c);
        self.update_completion();
        self.writer.print(&self.buffer, self.completion)
    }

    pub(super) fn delete(&mut self, scope: Scope) -> Result<(), crate::ErrorKind> {
        self.buffer.delete(scope);
        self.update_completion();
        self.writer.print(&self.buffer, self.completion)
    }

    pub(super) fn move_cursor(
        &mut self,
        range: Range,
        direction: Direction,
    ) -> Result<(), crate::ErrorKind> {
        if self.buffer.at_end() && direction == Direction::Forward {
            self.complete(range)
        } else {
            self.buffer.move_cursor(range, direction);
            self.writer.print(&self.buffer, self.completion)
        }
    }

    pub(super) fn complete(&mut self, range: Range) -> Result<(), crate::ErrorKind> {
        self.buffer.go_to_end();
        if let Some(completion) = &self.completion {
            match range {
                Range::Line => {
                    self.buffer.write_str(completion);
                    self.update_completion();
                    self.writer.print(&self.buffer, self.completion)
                }
                Range::Word => {
                    let index = navigation::next_word(0, &completion);
                    self.buffer.write_str(&completion[0..index]);
                    self.update_completion();
                    self.writer.print(&self.buffer, self.completion)
                }
                Range::Single => {
                    self.buffer.write(completion[0]);
                    self.update_completion();
                    self.writer.print(&self.buffer, self.completion)
                }
            }
        } else {
            Ok(())
        }
    }

    pub(super) fn suggest(&mut self, direction: Direction) -> Result<(), crate::ErrorKind> {
        if let Some(suggester) = self.suggester {
            let suggestions = suggester.suggest_for(&self.buffer);

            if suggestions.is_empty() {
                Ok(())
            } else {
                use Direction::*;

                let last_suggestion = suggestions.len() - 1;

                // Allowed because it is more readable
                #[allow(clippy::match_same_arms)]
                {
                    self.suggestion = match (direction, self.suggestion) {
                        (Forward, None) => Some(0),
                        (Forward, Some(index)) if index < last_suggestion => Some(index + 1),
                        (Forward, Some(_)) => None,
                        (Backward, None) => Some(last_suggestion),
                        (Backward, Some(0)) => None,
                        (Backward, Some(index)) => Some(index - 1),
                    };
                }

                self.writer
                    .print_suggestions(&self.buffer, suggestions, self.suggestion)
            }
        } else {
            Ok(())
        }
    }

    fn update_completion(&mut self) {
        if let Some(completer) = self.completer {
            self.completion = completer
                .complete_for(&self.buffer)
                .map(std::convert::Into::into);
        }
    }
}
