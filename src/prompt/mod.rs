mod buffer;
mod char_string;
mod completer;
mod navigation;
mod writer;

use buffer::Buffer;
use char_string::CharString;
use completer::Completer;
use writer::Writer;

use crate::key_bindings::{action_for, Action, Direction, KeyBindings, Range, Scope};

pub struct Prompt {
    prompt: Option<CharString>,
    bindings: Option<KeyBindings>,
    completer: Option<Completer>,
}

impl Prompt {
    #[must_use]
    pub fn new() -> Self {
        Prompt::default()
    }

    pub fn prompt(&mut self, prompt: Option<&str>) -> &mut Self {
        self.prompt = prompt.map(std::convert::Into::into);
        self
    }

    pub fn bindings(&mut self, bindings: Option<KeyBindings>) -> &mut Self {
        self.bindings = bindings;
        self
    }

    pub fn completions(&mut self, completions: Option<&[&str]>) -> &mut Self {
        self.completer = completions.map(std::convert::Into::into);
        self
    }

    pub fn read_line(&self) -> Result<Option<String>, crate::ErrorKind> {
        let mut context = Context::new(self.prompt.as_ref(), self.completer.as_ref())?;

        context.print()?;
        loop {
            if let crossterm::event::Event::Key(e) = crossterm::event::read()? {
                match action_for(self.bindings.as_ref(), e) {
                    Action::Write(c) => context.write(c)?,
                    Action::Delete(scope) => context.delete(scope)?,
                    Action::Move(range) => context.move_cursor(range)?,
                    Action::Complete(range) => context.complete(range)?,
                    Action::Suggest(_) => {}
                    Action::Noop => continue,
                    Action::Cancel => return Ok(None),
                    Action::Accept => return Ok(Some(context.buffer_as_string())),
                }
            }
        }
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            prompt: None,
            bindings: None,
            completer: None,
        }
    }
}

struct Context<'a> {
    writer: Writer,
    buffer: Buffer,
    completer: Option<&'a Completer>,
    completion: Option<CharString>,
}

impl<'a> Context<'a> {
    fn new(
        prompt: Option<&CharString>,
        completer: Option<&'a Completer>,
    ) -> Result<Self, crate::ErrorKind> {
        Ok(Self {
            writer: Writer::new(prompt)?,
            buffer: Buffer::new(),
            completer,
            completion: None,
        })
    }

    fn buffer_as_string(&self) -> String {
        self.buffer.to_string()
    }

    fn print(&mut self) -> Result<(), crate::ErrorKind> {
        self.writer.print(&self.buffer, &self.completion)
    }

    fn write(&mut self, c: char) -> Result<(), crate::ErrorKind> {
        self.buffer.write(c);
        self.update_completion();
        self.writer.print(&self.buffer, &self.completion)
    }

    fn delete(&mut self, scope: Scope) -> Result<(), crate::ErrorKind> {
        self.buffer.delete(scope);
        self.update_completion();
        self.writer.print(&self.buffer, &self.completion)
    }

    fn move_cursor(&mut self, range: Range) -> Result<(), crate::ErrorKind> {
        // TODO handle completiong when buffer is `at_end`
        self.buffer.move_cursor(range);
        self.writer.print(&self.buffer, &self.completion)
    }

    fn complete(&mut self, range: Range) -> Result<(), crate::ErrorKind> {
        // TODO handle when buffer is not `at_end`
        if self.buffer.at_end() {
            if let Some(completion) = &self.completion {
                use Direction::Forward;
                use Range::*;

                match range {
                    Line(Forward) | Single(Forward) => {
                        self.buffer.write_str(completion);
                        self.update_completion();
                        self.writer.print(&self.buffer, &self.completion)?;
                    }
                    Word(Forward) => {
                        let index = navigation::next_word(0, &completion);
                        self.buffer.write_str(&completion[0..index]);
                        self.update_completion();
                        self.writer.print(&self.buffer, &self.completion)?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn update_completion(&mut self) {
        if let Some(completer) = self.completer {
            self.completion = completer.complete_for(&self.buffer);
        }
    }
}

impl std::ops::Drop for Context<'_> {
    // Allowed because this is a drop and the previous construction already managed the get through
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        // Flush the user written buffer before dropping the writer
        self.buffer.move_cursor(Range::Line(Direction::Forward));
        self.writer.print(&self.buffer, &None);
    }
}
