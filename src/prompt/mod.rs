mod buffer;
mod char_string;
mod navigation;

use char_string::CharString;

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
        let mut writer = Writer::new()?;
        let mut buffer = buffer::Buffer::new();

        loop {
            writer.print(&self.prompt, &buffer)?;

            match crossterm::event::read()? {
                crossterm::event::Event::Resize(width, heigth) => writer.resize(width, heigth),
                crossterm::event::Event::Key(e) => match action_for(self.bindings.as_ref(), e) {
                    Action::Write(c) => buffer.write(c),
                    Action::Delete(scope) => buffer.delete(scope),
                    Action::Move(movement) => buffer.move_cursor(movement),
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

struct Writer {
    width: u16,
    height: u16,
    prev_length: usize,
}

impl Writer {
    fn new() -> Result<Self, crate::ErrorKind> {
        crossterm::terminal::enable_raw_mode()?;
        let (width, height) = crossterm::terminal::size()?;
        Ok(Self {
            width,
            height,
            prev_length: 0,
        })
    }

    fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    // TODO: Allow long strings
    #[allow(clippy::cast_possible_truncation)]
    fn print(
        &mut self,
        prompt: &Option<CharString>,
        buffer: &buffer::Buffer,
    ) -> Result<(), crate::ErrorKind> {
        use std::io::Write;
        let mut stdout = std::io::stdout();

        crossterm::queue!(
            stdout,
            crossterm::cursor::MoveTo(0, 20),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown),
        )?;

        let start = if let Some(prompt) = prompt {
            crossterm::queue!(stdout, crossterm::style::Print(&prompt),)?;
            prompt.len()
        } else {
            0
        };

        self.prev_length = start + buffer.len();

        crossterm::execute!(stdout, crossterm::style::Print(buffer),)
        // crossterm::cursor::MoveToColumn((start + buffer.position() + 1) as u16)
    }
}

impl std::ops::Drop for Writer {
    // Allowed because this is a drop and the previous construction already managed the get through
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        use std::io::Write;
        crossterm::terminal::disable_raw_mode();
        crossterm::execute!(
            std::io::stdout(),
            crossterm::style::ResetColor,
            crossterm::style::Print('\n')
        );
    }
}
