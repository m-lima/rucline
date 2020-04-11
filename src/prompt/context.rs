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
    suggestions: Option<Suggestions>,
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
            suggestions: None,
        })
    }

    pub(super) fn buffer_as_string(&mut self) -> String {
        self.try_take_suggestion();
        self.buffer.to_string()
    }

    pub(super) fn print(&mut self) -> Result<(), crate::ErrorKind> {
        self.writer.print(&self.buffer, self.completion)
    }

    pub(super) fn write(&mut self, c: char) -> Result<(), crate::ErrorKind> {
        self.try_take_suggestion();
        self.buffer.write(c);
        self.update_completion();
        self.writer.print(&self.buffer, self.completion)
    }

    pub(super) fn delete(&mut self, scope: Scope) -> Result<(), crate::ErrorKind> {
        self.try_take_suggestion();
        self.buffer.delete(scope);
        self.update_completion();
        self.writer.print(&self.buffer, self.completion)
    }

    pub(super) fn move_cursor(
        &mut self,
        range: Range,
        direction: Direction,
    ) -> Result<(), crate::ErrorKind> {
        self.try_take_suggestion();
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

    fn update_completion(&mut self) {
        if let Some(completer) = self.completer {
            self.completion = completer
                .complete_for(&self.buffer)
                .map(std::convert::Into::into);
        }
    }

    pub(super) fn suggest(&mut self, direction: Direction) -> Result<(), crate::ErrorKind> {
        if let Some(suggester) = &self.suggester {
            if let Some(suggestions) = &mut self.suggestions {
                suggestions.cycle(direction);
                if let Some(index) = suggestions.index {
                    return self.writer.print_suggestions(index, &suggestions.options);
                }
            } else {
                let options = suggester.suggest_for(&self.buffer);
                if !options.is_empty() {
                    self.suggestions = Some(Suggestions::new(options, direction));
                    let suggestions = self.suggestions.as_ref().unwrap();
                    return self
                        .writer
                        .print_suggestions(suggestions.index.unwrap(), &suggestions.options);
                }
            }
        }

        self.writer.print(&self.buffer, self.completion)
    }

    pub(super) fn is_suggesting(&self) -> bool {
        self.suggestions.is_some()
    }

    pub(super) fn cancel_suggestion(&mut self) -> Result<(), crate::ErrorKind> {
        self.suggestions = None;
        self.writer.print(&self.buffer, self.completion)
    }

    fn try_take_suggestion(&mut self) {
        if let Some(suggestion) = self.suggestions.take().and_then(Suggestions::take) {
            self.buffer = suggestion;
        }
    }
}

struct Suggestions {
    index: Option<usize>,
    options: Vec<Buffer>,
}

impl Suggestions {
    fn new(options: &[String], direction: Direction) -> Self {
        let index = match direction {
            Direction::Forward => 0,
            Direction::Backward => options.len() - 1,
        };

        Self {
            options: options
                .iter()
                .map(|option| Buffer::from(option.as_str()))
                .collect(),
            index: Some(index),
        }
    }

    // Allowed because it is more readable
    #[allow(clippy::match_same_arms)]
    fn cycle(&mut self, direction: Direction) {
        use Direction::*;

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
        if let Some(index) = self.index {
            Some(self.options.swap_remove(index))
        } else {
            None
        }
    }
}
