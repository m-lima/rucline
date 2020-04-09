mod buffer;
mod char_string;
mod navigation;
mod writer;

use buffer::Buffer;
use char_string::CharString;
use writer::Writer;

use crate::key_bindings::{action_for, Action, KeyBindings};

pub struct Prompt {
    prompt: Option<CharString>,
    bindings: Option<KeyBindings>,
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

    pub fn read_line(&self) -> Result<Option<String>, crate::ErrorKind> {
        let mut writer = Writer::new(self.prompt.as_ref())?;
        let mut buffer = Buffer::new();

        writer.print(&buffer)?;
        loop {
            match crossterm::event::read()? {
                crossterm::event::Event::Resize(width, _) => writer.resize(width),
                crossterm::event::Event::Key(e) => match action_for(self.bindings.as_ref(), e) {
                    Action::Write(c) => {
                        buffer.write(c);
                        writer.print(&buffer)?;
                    }
                    Action::Delete(scope) => {
                        buffer.delete(scope);
                        writer.print(&buffer)?;
                    }
                    Action::Move(movement) => {
                        buffer.move_cursor(movement);
                        writer.print(&buffer)?;
                    }
                    Action::Complete(_) | Action::Suggest(_) => {}
                    Action::Noop => continue,
                    Action::Accept => return Ok(Some(buffer.to_string())),
                    Action::Cancel => return Ok(None),
                },
                _ => continue,
            }
        }
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            prompt: None,
            bindings: None,
        }
    }
}
