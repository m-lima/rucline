mod buffer;
mod char_string;
mod completer;
mod navigation;
mod writer;

use buffer::Buffer;
use char_string::CharString;
use completer::Completer;
use writer::Writer;

use crate::key_bindings::{action_for, Action, Direction::*, KeyBindings, Range::*};

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
        let mut writer = Writer::new(self.prompt.as_ref())?;
        let mut buffer = Buffer::new();

        let mut completion = None;

        writer.print(&buffer, &completion)?;
        loop {
            if let crossterm::event::Event::Key(e) = crossterm::event::read()? {
                match action_for(self.bindings.as_ref(), e) {
                    Action::Write(c) => buffer.write(c),
                    Action::Delete(scope) => buffer.delete(scope),
                    Action::Move(range) => {
                        if buffer.at_end() {
                            if let Some(completion) = &completion {
                                match range {
                                    Line(Forward) | Single(Forward) => buffer.write_str(completion),
                                    Word(Forward) => {
                                        let index = navigation::next_word(0, &completion);
                                        buffer.write_str(&completion[0..index]);
                                    }
                                    _ => {},
                                }
                            }
                        }
                        buffer.move_cursor(range);
                    }
                    Action::Complete(_) | Action::Suggest(_) => {}
                    Action::Noop => continue,
                    Action::Accept => return Ok(Some(buffer.to_string())),
                    Action::Cancel => return Ok(None),
                }

                // TODO: Avoid this step if there was no change to the buffer's content
                if let Some(completer) = &self.completer {
                    completion = completer.complete_for(&buffer);
                }
                writer.print(&buffer, &completion)?;
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
