use super::{Buffer, CharString};

pub(super) struct Writer<'a> {
    prompt: Option<&'a CharString>,
    start: usize,
    width: usize,
    printed_length: usize,
    cursor_offset: (usize, usize),
}

impl<'a> Writer<'a> {
    pub(super) fn new(prompt: Option<&'a CharString>) -> Result<Self, crate::ErrorKind> {
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
            printed_length: 0,
            cursor_offset: (0, 0),
        })
    }

    pub(super) fn resize(&mut self, width: u16) {
        self.width = usize::from(width);
    }

    fn update_cursor_offset(&mut self, buffer: &Buffer) {
        self.cursor_offset = if buffer.at_end() {
            (0, 0)
        } else {
            let offset = buffer.len() - buffer.position();
            if offset > self.width {
                let lines = offset / self.width;
                (offset - lines * self.width, lines)
            } else {
                (offset, 0)
            }
        };
        eprintln!("offset: ({}, {})", self.cursor_offset.0, self.cursor_offset.1);
    }

    pub(super) fn print(&mut self, buffer: &Buffer) -> Result<(), crate::ErrorKind> {
        use std::io::Write;
        let mut stdout = std::io::stdout();

        // Safe to cast because values are always =< `self.width`
        #[allow(clippy::cast_possible_truncation)]
        {
            let mut lines = self.printed_length / self.width;
            if lines > 0 {
                // Take away edge case where the cursor is out of bounds to the left
                if self.printed_length - lines * self.width == 0 {
                    lines -= 1;
                }

                lines -= self.cursor_offset.1;

                crossterm::queue!(stdout, crossterm::cursor::MoveUp(lines as u16))?;
            }
        }

        crossterm::queue!(
            stdout,
            crossterm::cursor::MoveToColumn(0),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown),
        )?;

        if let Some(prompt) = self.prompt {
            crossterm::queue!(stdout, crossterm::style::Print(prompt))?;
        }

        self.update_cursor_offset(&buffer);
        self.printed_length = self.start + buffer.len();

        crossterm::execute!(
            stdout,
            crossterm::style::Print(&buffer),
            crossterm::cursor::MoveLeft(self.cursor_offset.0 as u16),
            crossterm::cursor::MoveUp(self.cursor_offset.1 as u16),
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
