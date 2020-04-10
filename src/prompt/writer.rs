use super::{Buffer, CharString};

pub(super) struct Writer {
    printed_length: usize,
    cursor_offset: usize,
}

impl Writer {
    pub(super) fn new(prompt: Option<&CharString>) -> Result<Self, crate::ErrorKind> {
        crossterm::terminal::enable_raw_mode()?;
        if let Some(prompt) = prompt {
            use std::io::Write;

            crossterm::queue!(std::io::stdout(), crossterm::style::Print(prompt))?;
        }

        Ok(Self {
            printed_length: 0,
            cursor_offset: 0,
        })
    }

    pub(super) fn print(
        &mut self,
        buffer: &Buffer,
        completion: &Option<CharString>,
    ) -> Result<(), crate::ErrorKind> {
        use std::io::Write;
        let mut stdout = std::io::stdout();

        rewind_cursor(&mut stdout, self.printed_length - self.cursor_offset)?;

        crossterm::queue!(
            stdout,
            crossterm::style::ResetColor,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown),
        )?;

        self.cursor_offset = buffer.len() - buffer.cursor();
        self.printed_length = buffer.len();

        crossterm::queue!(stdout, crossterm::style::Print(&buffer),)?;

        if let Some(completion) = completion {
            crossterm::queue!(
                stdout,
                crossterm::style::SetForegroundColor(crossterm::style::Color::Blue),
                crossterm::style::Print(completion),
                crossterm::style::ResetColor,
            )?;
            rewind_cursor(&mut stdout, completion.len())?;
        }

        rewind_cursor(&mut stdout, self.cursor_offset)?;

        crossterm::execute!(stdout)
    }
}

// Allowed because we slice `usize` into `u16` chunks
#[allow(clippy::cast_possible_truncation)]
fn rewind_cursor(stdout: &mut std::io::Stdout, amount: usize) -> Result<(), crate::ErrorKind> {
    use std::io::Write;

    if amount == 0 {
        return Ok(());
    }

    let mut remaining = amount;
    while remaining > usize::from(u16::max_value()) {
        crossterm::queue!(stdout, crossterm::cursor::MoveLeft(u16::max_value()),)?;
        remaining -= usize::from(u16::max_value());
    }

    crossterm::queue!(stdout, crossterm::cursor::MoveLeft(remaining as u16),)
}

// TODO: Fix showing characters upon desctruction
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
