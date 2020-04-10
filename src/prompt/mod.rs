mod buffer;
mod char_string;
mod navigation;
mod writer;

use crate::completer::Completer;

use buffer::Buffer;
use char_string::{CharString, CharStringView};
use writer::Writer;

use crate::key_bindings::{action_for, Action, Direction, KeyBindings, Range, Scope};

pub struct Prompt {
    erase_after_read: bool,
    prompt: Option<CharString>,
    bindings: Option<KeyBindings>,
    completer: Option<Box<dyn Completer>>,
}

impl Prompt {
    #[must_use]
    pub fn new() -> Self {
        Prompt::default()
    }

    pub fn erase_after_read(&mut self, erase_after_read: bool) -> &mut Self {
        self.erase_after_read = erase_after_read;
        self
    }

    pub fn prompt(&mut self, prompt: &str) -> &mut Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn bindings(&mut self, bindings: KeyBindings) -> &mut Self {
        self.bindings = Some(bindings);
        self
    }

    pub fn completer(&mut self, completer: impl Completer + 'static) -> &mut Self {
        self.completer = Some(Box::new(completer));
        self
    }

    pub fn read_line(&self) -> Result<Option<String>, crate::ErrorKind> {
        let mut context =
            Context::new(self.erase_after_read, self.prompt.as_ref(), &self.completer)?;

        context.print()?;
        loop {
            if let crossterm::event::Event::Key(e) = crossterm::event::read()? {
                match action_for(self.bindings.as_ref(), e) {
                    Action::Write(c) => context.write(c)?,
                    Action::Delete(scope) => context.delete(scope)?,
                    Action::Move(range, direction) => context.move_cursor(range, direction)?,
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
            erase_after_read: false,
            prompt: None,
            bindings: None,
            completer: None,
        }
    }
}

struct Context<'a> {
    writer: Writer,
    buffer: Buffer,
    completer: &'a Option<Box<dyn Completer>>,
    completion: Option<CharStringView<'a>>,
}

impl<'a> Context<'a> {
    fn new(
        erase_on_drop: bool,
        prompt: Option<&CharString>,
        completer: &'a Option<Box<dyn Completer>>,
    ) -> Result<Self, crate::ErrorKind> {
        Ok(Self {
            writer: Writer::new(erase_on_drop, prompt)?,
            buffer: Buffer::new(),
            completer,
            completion: None,
        })
    }

    fn buffer_as_string(&self) -> String {
        self.buffer.to_string()
    }

    fn print(&mut self) -> Result<(), crate::ErrorKind> {
        self.writer.print(&self.buffer, self.completion)
    }

    fn write(&mut self, c: char) -> Result<(), crate::ErrorKind> {
        self.buffer.write(c);
        self.update_completion();
        self.writer.print(&self.buffer, self.completion)
    }

    fn delete(&mut self, scope: Scope) -> Result<(), crate::ErrorKind> {
        self.buffer.delete(scope);
        self.update_completion();
        self.writer.print(&self.buffer, self.completion)
    }

    fn move_cursor(&mut self, range: Range, direction: Direction) -> Result<(), crate::ErrorKind> {
        if self.buffer.at_end() && direction == Direction::Forward {
            self.complete(range)
        } else {
            self.buffer.move_cursor(range, direction);
            self.writer.print(&self.buffer, self.completion)
        }
    }

    fn complete(&mut self, range: Range) -> Result<(), crate::ErrorKind> {
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
}
