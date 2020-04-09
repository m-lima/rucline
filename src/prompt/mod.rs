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
                crossterm::event::Event::Resize(width, heigth) => writer.resize(width, heigth)?,
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
    prev_lines: u16,
}

impl Writer {
    fn new() -> Result<Self, crate::ErrorKind> {
        crossterm::terminal::enable_raw_mode()?;
        let (width, height) = crossterm::terminal::size()?;
        Ok(Self {
            width,
            height,
            prev_lines: 0,
        })
    }

    fn resize(&mut self, width: u16, height: u16) -> Result<(), crate::ErrorKind> {
        self.width = width;
        self.height = height;

        if self.prev_lines > 0 {
            use std::io::Write;
            crossterm::queue!(
                std::io::stdout(),
                crossterm::cursor::MoveUp(self.prev_lines),
            )?;
            self.prev_lines = 0;
        }

        Ok(())
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

        if self.prev_lines > 0 {
            crossterm::queue!(stdout, crossterm::cursor::MoveUp(self.prev_lines),)?;
        }

        crossterm::queue!(
            stdout,
            crossterm::cursor::MoveToColumn(0),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown),
        )?;

        // let (prompt_length, start_position) = prompt.map_or((0,0), |prompt|{
        //     crossterm::queue!(stdout, crossterm::style::Print(&prompt),)?;
        //     (prompt.len(), 0)
        // });

        let mut current_column = 0;
        let mut lines = 0;
        if let Some(prompt) = prompt {
            lines += self.print_multiline(&mut stdout, &prompt, &mut current_column)?;
        }

        lines += self.print_multiline(&mut stdout, buffer.chars(), &mut current_column)?;
        self.prev_lines = lines;

        crossterm::execute!(stdout, crossterm::cursor::MoveToColumn(0))
        // crossterm::cursor::MoveToColumn((start + buffer.position() + 1) as u16)
    }

    // Allowed because we always cap on width size, which is never larger than u16::max_value
    // #[allow(clippy::cast_possible_truncation)]
    fn print_multiline(
        &self,
        stdout: &mut impl std::io::Write,
        string: &[char],
        current_column: &mut u16,
    ) -> Result<u16, crate::ErrorKind> {
        let mut lines = 0;
        for c in string {
            if *current_column == self.width {
            //     let line = crossterm::cursor::position().map(|pos| pos.1)?;
            //     if line == self.height {
            //         crossterm::queue!(stdout, crossterm::terminal::ScrollUp(1))?;
            //     }
            //     crossterm::queue!(
            //         stdout,
            //         crossterm::cursor::MoveDown(1),
            //         crossterm::cursor::MoveToColumn(0)
            //     )?;
                *current_column = 0;
                lines += 1;
            }

            crossterm::queue!(stdout, crossterm::style::Print(*c))?;
            *current_column += 1;
        }
        Ok(lines)
        // let available_width = usize::from(width - *current_column);
        // let mut lines = 0;
        // if string.len() < available_width {
        //     crossterm::queue!(stdout, crossterm::style::Print(&string))?;
        //     current_column += string.len() as u16;
        //     Ok(lines)
        // } else {
        //     let chunk = &string[0..available_width];
        //     crossterm::queue!(stdout, crossterm::style::Print(&chunk), crossterm::style::Print('\n'))?;
        //     lines += 1;
        //     let chunks = &string[available_width..].chunks_exact(usize::from(width));
        //     for chunk in chunks {
        //         crossterm::queue!(stdout, crossterm::style::Print(&chunk), crossterm::style::Print('\n'))?;
        //         lines += 1;
        //     }
        //     let chunk = chunks.remainder();
        //     current_column = chunk.len();
        //     crossterm::queue!(stdout, crossterm::style::Print(&chunk))?;
        //     Ok(lines)
        // }
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
