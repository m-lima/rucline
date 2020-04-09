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
        let mut writer = Writer::new(self.prompt.as_ref())?;
        let mut buffer = buffer::Buffer::new();

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

struct Writer<'a> {
    prompt: Option<&'a CharString>,
    start: usize,
    width: usize,
}

impl<'a> Writer<'a> {
    fn new(prompt: Option<&'a CharString>) -> Result<Self, crate::ErrorKind> {
        crossterm::terminal::enable_raw_mode()?;
        let width = crossterm::terminal::size().map(|size| usize::from(size.0))?;
        let start = if let Some(prompt) = prompt {
            prompt.len()
        } else {
            0
        };
        Ok(Self {
            prompt,
            start,
            width,
        })
    }

    fn resize(&mut self, width: u16) {
        self.width = usize::from(width);
    }

    // Allowed because we catch overflows
    #[allow(clippy::cast_possible_truncation)]
    fn calculate_cursor_position(&self, buffer: &buffer::Buffer) -> Result<(u16, u16), crate::ErrorKind> {
        let position = self.start + buffer.position();
        let lines = (position - 1) / self.width;
        let columns = position + 1 - lines * self.width;

        if lines > usize::from(u16::max_value()) || columns > usize::from(u16::max_value()){
            return Err(crate::ErrorKind::ResizingTerminalFailure(String::from(
                "terminal width is too narrow",
            )));
        }
        Ok((columns as u16, lines as u16))
    }

    fn print(&mut self, buffer: &buffer::Buffer) -> Result<(), crate::ErrorKind> {
        use std::io::Write;
        let mut stdout = std::io::stdout();

        let cursor = self.calculate_cursor_position(&buffer)?;

        crossterm::queue!(
            stdout,
            crossterm::cursor::MoveUp(cursor.1),
            crossterm::cursor::MoveToColumn(0),
            crossterm::cursor::SavePosition,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown),
        )?;

        if let Some(prompt) = self.prompt {
            crossterm::queue!(stdout, crossterm::style::Print(prompt))?;
        }

        crossterm::execute!(
            stdout,
            crossterm::style::Print(&buffer),
            crossterm::cursor::RestorePosition,
            crossterm::cursor::MoveToColumn(cursor.0),
            crossterm::cursor::MoveDown(cursor.1),
        )
    }
}

impl std::ops::Drop for Writer<'_> {
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
