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
        let stdout = std::io::stdout();
        let mut writer = Writer::new(&stdout)?;
        let mut buffer = buffer::Buffer::new();

        loop {
            writer.print(&self.prompt, &buffer)?;
            match self.next_event()? {
                Action::Write(c) => buffer.write(c),
                Action::Delete(scope) => buffer.delete(scope),
                Action::Move(movement) => buffer.move_cursor(movement),
                Action::Complete(_) | Action::Suggest(_) => {}
                Action::Noop => continue,
                Action::Accept => return Ok(Some(buffer.to_string())),
                Action::Cancel => return Ok(None),
            }
        }
    }

    fn next_event(&self) -> Result<Action, crate::ErrorKind> {
        match crossterm::event::read()? {
            crossterm::event::Event::Key(e) => Ok(action_for(self.bindings.as_ref(), e)),
            _ => Ok(Action::Noop),
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
    lock: std::io::StdoutLock<'a>,
    _row: u16,
}

impl<'a> Writer<'a> {
    fn new(stdout: &'a std::io::Stdout) -> Result<Self, crate::ErrorKind> {
        crossterm::terminal::enable_raw_mode()?;
        Ok(Self {
            lock: stdout.lock(),
            _row: crossterm::cursor::position().map(|pos| pos.1)?,
        })
    }

    // TODO: Allow long strings
    #[allow(clippy::cast_possible_truncation)]
    fn print(
        &mut self,
        prompt: &Option<CharString>,
        buffer: &buffer::Buffer,
    ) -> Result<(), crate::ErrorKind> {
        use std::io::Write;

        crossterm::queue!(
            self.lock,
            crossterm::cursor::MoveToColumn(0),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
        )?;

        let start = if let Some(prompt) = prompt {
            crossterm::queue!(self.lock, crossterm::style::Print(prompt))?;
            prompt.len()
        } else {
            0
        };

        crossterm::execute!(
            self.lock,
            crossterm::style::Print(buffer),
            crossterm::cursor::MoveToColumn((start + buffer.position() + 1) as u16)
        )
    }
}

impl<'a> std::ops::Drop for Writer<'a> {
    // Allowed because this is a drop and the previous construction already managed the get through
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        use std::io::Write;
        crossterm::terminal::disable_raw_mode();
        crossterm::execute!(
            self.lock,
            crossterm::style::ResetColor,
            crossterm::style::Print('\n')
        );
    }
}
